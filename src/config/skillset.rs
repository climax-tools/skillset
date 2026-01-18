use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsetConfig {
    pub skills: HashMap<String, SkillConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registry: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conventions: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SkillConfig {
    Simple(String), // Just version
    Detailed {
        version: String,
        source: Option<String>,     // Override auto-resolution
        convention: Option<String>, // Override auto-detection
    },
}

impl SkillConfig {
    pub fn get_version(&self) -> &str {
        match self {
            Self::Simple(version) => version,
            Self::Detailed { version, .. } => version,
        }
    }

    pub fn has_explicit_source(&self) -> bool {
        matches!(
            self,
            Self::Detailed {
                source: Some(_),
                ..
            }
        )
    }

    pub fn get_explicit_source(&self) -> Option<&String> {
        match self {
            Self::Detailed {
                source: Some(s), ..
            } => Some(s),
            _ => None,
        }
    }

    pub fn get_explicit_convention(&self) -> Option<&String> {
        match self {
            Self::Detailed {
                convention: Some(c),
                ..
            } => Some(c),
            _ => None,
        }
    }
}

impl Default for SkillsetConfig {
    fn default() -> Self {
        Self {
            skills: HashMap::new(),
            registry: None,
            conventions: None,
        }
    }
}

impl SkillsetConfig {
    /// Get the registry URL with runtime default fallback
    pub fn get_registry(&self) -> &str {
        self.registry.as_deref().unwrap_or("ghcr.io/skillset")
    }

    /// Get the list of enabled conventions with runtime default fallback
    pub fn get_conventions(&self) -> Vec<String> {
        self.conventions.clone().unwrap_or_else(|| {
            vec![
                "autogpt".to_string(),
                "langchain".to_string(),
                "agent-skills".to_string(),
            ]
        })
    }

    pub fn resolve_skill_reference(
        &self,
        skill_name: &str,
        skill_config: &SkillConfig,
    ) -> Result<String> {
        // If explicit source is provided, use it
        if let Some(source) = skill_config.get_explicit_source() {
            return Ok(source.clone());
        }

        // Otherwise resolve from name and version
        let version = skill_config.get_version();
        self.resolve_name_to_oci_reference(skill_name, version)
    }

    fn resolve_name_to_oci_reference(&self, skill_name: &str, version: &str) -> Result<String> {
        // Parse the registry to get domain and namespace
        let parts: Vec<&str> = self.get_registry().split('/').collect();
        let (domain, default_namespace) = if parts.len() >= 2 {
            (parts[0], parts[1])
        } else {
            (parts[0], "skillset") // Default namespace if not specified
        };

        let prefixed_version = if version.starts_with('v') {
            version.to_string()
        } else {
            format!("v{}", version)
        };

        if skill_name.starts_with('@') {
            // Handle scoped names: @user/skill
            let scoped_part = &skill_name[1..]; // Remove '@'
            let parts: Vec<&str> = scoped_part.splitn(2, '/').collect();
            if parts.len() != 2 {
                return Err(crate::error::SkillsetError::InvalidSkillName(format!(
                    "Invalid scoped skill name: {}. Expected format: @user/skill",
                    skill_name
                )));
            }
            let (user, name) = (parts[0], parts[1]);
            Ok(format!(
                "oci:{}/{}/{}:{}",
                domain, user, name, prefixed_version
            ))
        } else {
            // Handle simple names: skill-name
            Ok(format!(
                "oci:{}/{}/{}:{}",
                domain, default_namespace, skill_name, prefixed_version
            ))
        }
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
                Err(_) => Err(crate::error::SkillsetError::Config(
                    "Configuration file not found: skillset.json".to_string(),
                )),
            }
        } else {
            Ok(Self::default())
        }
    }
}
