use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetadata {
    pub url: String,
    pub reference: Option<String>,
    pub skill_name: String,
    pub source_type: String,
}

impl CacheMetadata {
    pub async fn load(path: &Path) -> Result<Option<Self>> {
        if !path.exists() {
            return Ok(None);
        }

        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to read metadata from {:?}: {}", path, e))?;

        let metadata = serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse metadata from {:?}: {}", path, e))?;

        Ok(Some(metadata))
    }

    pub async fn save(&self, path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| anyhow::anyhow!("Failed to serialize metadata: {}", e))?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create directory {:?}: {}", parent, e))?;
        }

        tokio::fs::write(path, content)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to write metadata to {:?}: {}", path, e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;

    #[tokio::test]
    async fn test_save_and_load_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let metadata_path = temp_dir.path().join("test.json");

        let original = CacheMetadata {
            url: "https://github.com/user/repo.git".to_string(),
            reference: Some("main".to_string()),
            skill_name: "repo".to_string(),
            source_type: "git".to_string(),
        };

        // Save metadata
        original.save(&metadata_path).await.unwrap();
        assert!(metadata_path.exists());

        // Load metadata
        let loaded = CacheMetadata::load(&metadata_path).await.unwrap();
        assert!(loaded.is_some());

        let loaded = loaded.unwrap();
        assert_eq!(loaded.url, original.url);
        assert_eq!(loaded.reference, original.reference);
        assert_eq!(loaded.skill_name, original.skill_name);
        assert_eq!(loaded.source_type, original.source_type);
    }

    #[tokio::test]
    async fn test_load_nonexistent_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let metadata_path = temp_dir.path().join("nonexistent.json");

        let loaded = CacheMetadata::load(&metadata_path).await.unwrap();
        assert!(loaded.is_none());
    }

    #[tokio::test]
    async fn test_save_creates_parent_directory() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir
            .path()
            .join("nested")
            .join("dir")
            .join("metadata.json");

        let metadata = CacheMetadata {
            url: "https://github.com/user/repo.git".to_string(),
            reference: None,
            skill_name: "repo".to_string(),
            source_type: "git".to_string(),
        };

        // Save to nested path - should create parent directories
        metadata.save(&nested_path).await.unwrap();
        assert!(nested_path.exists());
        assert!(nested_path.parent().unwrap().exists());
    }

    #[tokio::test]
    async fn test_handle_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        let metadata_path = temp_dir.path().join("invalid.json");

        // Write invalid JSON
        fs::write(&metadata_path, "{ invalid json }").await.unwrap();

        let loaded = CacheMetadata::load(&metadata_path).await;
        assert!(loaded.is_err());
        assert!(loaded
            .unwrap_err()
            .to_string()
            .contains("Failed to parse metadata"));
    }
}
