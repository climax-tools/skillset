use std::path::PathBuf;

use async_trait::async_trait;
use git2::Repository;

use super::{SkillSource, SourceType};
use crate::cache::{CacheMetadata, CachePaths};
use crate::error::{Result, SkillsetError};
use crate::skill::types::{FetchedSkill, SkillMetadata};

pub struct GitSource {
    cache: CachePaths,
}

impl GitSource {
    pub fn new() -> Result<Self> {
        let cache = CachePaths::new()?;
        cache.ensure_directories()?;
        Ok(Self { cache })
    }

    fn parse_reference(&self, reference: &str) -> Result<(String, Option<String>)> {
        let git_url = reference
            .strip_prefix("git:")
            .unwrap_or(reference)
            .to_string();

        // For now, we don't support explicit references in git URL
        // This can be extended later to handle URL#branch syntax
        Ok((git_url, None))
    }

    fn extract_skill_name_from_url(&self, git_url: &str) -> Result<String> {
        // Extract repo name from git URL
        // Example: https://github.com/user/skill-name.git -> skill-name
        let parts: Vec<&str> = git_url.split('/').collect();
        if parts.len() >= 2 {
            let last_part = parts.last().unwrap();
            let name = last_part.trim_end_matches(".git");
            Ok(name.to_string())
        } else {
            Err(SkillsetError::Source(format!(
                "Unable to extract skill name from git URL: {}",
                git_url
            )))
        }
    }

    async fn get_or_clone(
        &self,
        url: &str,
        reference: Option<&str>,
        skill_name: &str,
    ) -> Result<PathBuf> {
        let cache_key = self.cache.git_cache_key(url, reference);
        let checkout_path = self.cache.git_checkout_path(skill_name);
        let url_clone = url.to_string();

        // For now, we'll just clone directly to checkout location
        // The bare repository caching can be added later if needed
        let checkout_path_clone = checkout_path.clone();
        tokio::task::spawn_blocking(move || {
            // Remove existing checkout if it exists
            if checkout_path_clone.exists() {
                std::fs::remove_dir_all(&checkout_path_clone)?;
            }

            // Clone repository directly to checkout location
            // Remove any existing directory first to avoid lock conflicts
            if checkout_path_clone.exists() {
                std::fs::remove_dir_all(&checkout_path_clone).map_err(|e| SkillsetError::Io(e))?;
            }
            Repository::clone(&url_clone, &checkout_path_clone)
                .map_err(|e| SkillsetError::Git(e))?;

            Ok::<PathBuf, SkillsetError>(checkout_path_clone)
        })
        .await
        .map_err(|e| SkillsetError::Source(format!("Task execution failed: {}", e)))??;

        // Save metadata asynchronously
        let metadata = CacheMetadata {
            url: url.to_string(),
            reference: reference.map(|r| r.to_string()),
            skill_name: skill_name.to_string(),
            source_type: "git".to_string(),
        };

        let metadata_path = self.cache.metadata_path(&cache_key);
        metadata.save(&metadata_path).await?;

        Ok(checkout_path)
    }
}

#[async_trait]
impl SkillSource for GitSource {
    async fn fetch(&self, reference: &str) -> Result<FetchedSkill> {
        let (url, ref_spec) = self.parse_reference(reference)?;
        let skill_name = self.extract_skill_name_from_url(&url)?;
        let checkout_path = self
            .get_or_clone(&url, ref_spec.as_deref(), &skill_name)
            .await?;

        Ok(FetchedSkill {
            name: skill_name,
            version: ref_spec.unwrap_or_else(|| "latest".to_string()),
            source_path: checkout_path.clone(),
            metadata: SkillMetadata {
                installed_at: chrono::Utc::now().to_rfc3339(),
                repo_path: checkout_path,
                convention: "unknown".to_string(), // Will be detected later
                checksum: None,
                description: None,
                author: None,
                dependencies: Vec::new(),
            },
        })
    }

    async fn get_metadata(&self, reference: &str) -> Result<SkillMetadata> {
        // For minimal version, just fetch and return metadata
        let fetched_skill = self.fetch(reference).await?;
        Ok(fetched_skill.metadata)
    }

    fn source_type(&self) -> SourceType {
        SourceType::Git
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_parse_reference() {
        let source = GitSource::new().unwrap();

        // Test basic URL
        let (url, reference) = source
            .parse_reference("https://github.com/user/repo.git")
            .unwrap();
        assert_eq!(url, "https://github.com/user/repo.git");
        assert_eq!(reference, None);

        // Test with git: prefix
        let (url, reference) = source
            .parse_reference("git:https://github.com/user/repo.git")
            .unwrap();
        assert_eq!(url, "https://github.com/user/repo.git");
        assert_eq!(reference, None);
    }

    #[test]
    fn test_extract_skill_name_from_url() {
        let source = GitSource::new().unwrap();

        // Test HTTPS URL
        let name = source
            .extract_skill_name_from_url("https://github.com/user/skill-name.git")
            .unwrap();
        assert_eq!(name, "skill-name");

        // Test without .git extension
        let name = source
            .extract_skill_name_from_url("https://github.com/user/skill-name")
            .unwrap();
        assert_eq!(name, "skill-name");

        // Test invalid URL
        let result = source.extract_skill_name_from_url("invalid-url");
        assert!(result.is_err());
    }
}
