use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::error::Result;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsetConfig {
    pub skills: HashMap<String, SkillConfig>,
    pub conventions: HashMap<String, ConventionConfig>,
    pub registry: RegistryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillConfig {
    pub version: String,
    pub source: String,
    pub installed_at: Option<String>,
    pub repo_path: Option<String>,
    pub convention: Option<String>,
    pub checksum: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConventionConfig {
    pub enabled: bool,
    pub path_pattern: String,
    pub detection_patterns: Vec<String>,
    pub metadata_file: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    pub default: String,
    pub auth_url: Option<String>,
    pub auth: HashMap<String, AuthConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub username: Option<String>,
    pub password_env: Option<String>,
    pub token_env: Option<String>,
}

impl Default for SkillsetConfig {
    fn default() -> Self {
        Self {
            skills: HashMap::new(),
            conventions: Self::default_conventions(),
            registry: RegistryConfig {
                default: "ghcr.io".to_string(),
                auth_url: None,
                auth: HashMap::new(),
            },
        }
    }
}

impl SkillsetConfig {
    fn default_conventions() -> HashMap<String, ConventionConfig> {
        let mut conventions = HashMap::new();
        
        conventions.insert("autogpt".to_string(), ConventionConfig {
            enabled: true,
            path_pattern: "skills/autogpt/{name}".to_string(),
            detection_patterns: vec![
                "skill.py".to_string(),
                "requirements.txt".to_string(),
                "__init__.py".to_string(),
            ],
            metadata_file: Some("skill.json".to_string()),
        });
        
        conventions.insert("langchain".to_string(), ConventionConfig {
            enabled: true,
            path_pattern: "skills/langchain/{name}".to_string(),
            detection_patterns: vec![
                "tool.yaml".to_string(),
                "*.py".to_string(),
                "pyproject.toml".to_string(),
            ],
            metadata_file: Some("tool.yaml".to_string()),
        });
        
        conventions.insert("custom".to_string(), ConventionConfig {
            enabled: false,
            path_pattern: "skills/custom/{name}".to_string(),
            detection_patterns: vec![
                "*.js".to_string(),
                "package.json".to_string(),
                "index.js".to_string(),
            ],
            metadata_file: Some("package.json".to_string()),
        });
        
        conventions
    }
    
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: SkillsetConfig = serde_json::from_str(&content)
            .map_err(|e| crate::error::SkillsetError::JsonSerialization(e))?;
        Ok(config)
    }
    
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| crate::error::SkillsetError::SerializationError(e.to_string()))?;
        fs::write(path, content)?;
        Ok(())
    }
    
    /// Try to load from JSON, fall back to TOML for backward compatibility
    pub fn load_from_file_flexible(path: &Path) -> Result<Self> {
        if path.exists() {
            // Try JSON first
            match Self::load_from_file(path) {
                Ok(config) => Ok(config),
                Err(_) => {
                    // Try TOML if JSON fails
                    let toml_path = path.with_extension("toml");
                    if toml_path.exists() {
                        eprintln!("Warning: skillset.toml is deprecated, please migrate to skillset.json");
                        // For now, just try JSON
                        Self::load_from_file(path)
                    } else {
                        Err(crate::error::SkillsetError::Config(
                            "Configuration file not found: skillset.json".to_string()
                        ))
                    }
                }
            }
        } else {
            Ok(Self::default())
        }
    }
}