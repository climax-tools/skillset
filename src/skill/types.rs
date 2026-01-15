use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub version: String,
    pub source: SkillSource,
    pub metadata: SkillMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SkillSource {
    Git {
        url: String,
        branch: Option<String>,
        tag: Option<String>,
    },
    Oci {
        reference: String,
        registry: Option<String>,
    },
    Local {
        path: PathBuf,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetadata {
    pub installed_at: String,
    pub repo_path: PathBuf,
    pub convention: String,
    pub checksum: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FetchedSkill {
    pub name: String,
    pub version: String,
    pub source_path: PathBuf,
    pub metadata: SkillMetadata,
}
