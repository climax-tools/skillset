use std::path::PathBuf;

use async_trait::async_trait;
use git2::Repository;

use super::{SkillSource, SourceType};
use crate::error::{Result, SkillsetError};
use crate::skill::types::{FetchedSkill, SkillMetadata};

pub struct GitSource {
    cache_dir: PathBuf,
}

impl GitSource {
    pub fn new(project_path: &std::path::Path) -> Self {
        let cache_dir = project_path.join(".skillset").join("cache");
        Self { cache_dir }
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

    fn clone_repository(&self, git_url: &str, skill_name: &str) -> Result<PathBuf> {
        Self::clone_repository_static(&self.cache_dir, git_url, skill_name)
    }

    fn clone_repository_static(
        cache_dir: &PathBuf,
        git_url: &str,
        skill_name: &str,
    ) -> Result<PathBuf> {
        // Create cache directory if it doesn't exist
        std::fs::create_dir_all(cache_dir)?;

        // Clone to cache directory
        let clone_path = cache_dir.join(skill_name);

        // Remove existing directory if it exists
        if clone_path.exists() {
            std::fs::remove_dir_all(&clone_path)?;
        }

        // Clone the repository
        let _repo = Repository::clone(git_url, &clone_path).map_err(|e| SkillsetError::Git(e))?;

        Ok(clone_path)
    }
}

#[async_trait]
impl SkillSource for GitSource {
    async fn fetch(&self, reference: &str) -> Result<FetchedSkill> {
        let git_url = reference
            .strip_prefix("git:")
            .unwrap_or(reference)
            .to_string();
        let skill_name = self.extract_skill_name_from_url(&git_url)?;
        let cache_dir = self.cache_dir.clone();
        let skill_name_for_closure = skill_name.clone();

        // Clone repository in blocking task
        let source_path = tokio::task::spawn_blocking(move || {
            Self::clone_repository_static(&cache_dir, &git_url, &skill_name_for_closure)
        })
        .await
        .map_err(|e| SkillsetError::Source(format!("Task execution failed: {}", e)))??;

        // Create basic metadata
        let metadata = SkillMetadata {
            installed_at: chrono::Utc::now().to_rfc3339(),
            repo_path: source_path.clone(),
            convention: "unknown".to_string(), // Will be detected later
            checksum: None,
            description: None,
            author: None,
            dependencies: Vec::new(),
        };

        Ok(FetchedSkill {
            name: skill_name,
            version: "latest".to_string(), // Use default branch for minimal version
            source_path,
            metadata,
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
