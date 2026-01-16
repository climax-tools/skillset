use anyhow::Result;
use dirs::cache_dir;
use std::path::PathBuf;

pub use metadata::CacheMetadata;

mod metadata;

#[derive(Clone)]
pub struct CachePaths {
    base_dir: PathBuf,
    git_dir: PathBuf,
    metadata_dir: PathBuf,
}

impl CachePaths {
    pub fn new() -> Result<Self> {
        let base_dir = cache_dir()
            .ok_or_else(|| anyhow::anyhow!("No cache directory found"))?
            .join("skillset");

        Ok(Self {
            git_dir: base_dir.join("git"),
            metadata_dir: base_dir.join("metadata"),
            base_dir,
        })
    }

    pub fn ensure_directories(&self) -> Result<()> {
        std::fs::create_dir_all(&self.git_dir.join("db"))?;
        std::fs::create_dir_all(&self.git_dir.join("checkouts"))?;
        std::fs::create_dir_all(&self.metadata_dir)?;
        Ok(())
    }

    pub fn git_cache_key(&self, url: &str, reference: Option<&str>) -> String {
        use sha2::{Digest, Sha256};
        let input = format!("{}#{}", url, reference.unwrap_or("latest"));
        format!("{:x}", Sha256::digest(input.as_bytes()))
    }

    pub fn git_bare_path(&self, cache_key: &str) -> PathBuf {
        self.git_dir.join("db").join(cache_key)
    }

    pub fn git_checkout_path(&self, skill_name: &str) -> PathBuf {
        self.git_dir.join("checkouts").join(skill_name)
    }

    pub fn metadata_path(&self, cache_key: &str) -> PathBuf {
        self.metadata_dir.join(format!("{}.json", cache_key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_cache_paths_creation() {
        let paths = CachePaths::new();
        assert!(paths.is_ok());

        let paths = paths.unwrap();
        assert!(paths.base_dir.ends_with("skillset"));
        assert!(paths.git_dir.ends_with("skillset/git"));
        assert!(paths.metadata_dir.ends_with("skillset/metadata"));
    }

    #[test]
    fn test_cache_key_generation() {
        let paths = CachePaths::new().unwrap();

        let key1 = paths.git_cache_key("https://github.com/user/repo.git", None);
        let key2 = paths.git_cache_key("https://github.com/user/repo.git", Some("main"));
        let key3 = paths.git_cache_key("https://github.com/user/repo.git", None);

        assert_eq!(key1, key3); // Same URL + no reference = same key
        assert_ne!(key1, key2); // Different reference = different key
        assert_eq!(key1.len(), 64); // SHA256 hex length

        // Different URLs should have different keys
        let key4 = paths.git_cache_key("https://github.com/user/other.git", None);
        assert_ne!(key1, key4);
    }

    #[test]
    fn test_path_construction() {
        let paths = CachePaths::new().unwrap();
        let cache_key = "abcd1234";

        let bare_path = paths.git_bare_path(cache_key);
        assert!(bare_path.ends_with("skillset/git/db/abcd1234"));

        let checkout_path = paths.git_checkout_path("my-skill");
        assert!(checkout_path.ends_with("skillset/git/checkouts/my-skill"));

        let metadata_path = paths.metadata_path(cache_key);
        assert!(metadata_path.ends_with("skillset/metadata/abcd1234.json"));
    }
}
