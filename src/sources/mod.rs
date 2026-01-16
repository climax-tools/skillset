use async_trait::async_trait;
use std::collections::HashMap;

use crate::error::Result;
use crate::skill::types::FetchedSkill;

pub mod git;

#[async_trait]
pub trait SkillSource: Send + Sync {
    async fn fetch(&self, reference: &str) -> Result<FetchedSkill>;
    async fn get_metadata(&self, reference: &str) -> Result<crate::skill::types::SkillMetadata>;
    fn source_type(&self) -> SourceType;
}

#[derive(Debug, Clone)]
pub enum SourceType {
    Git,
    Oci,
    Local,
}

pub struct SourceRegistry {
    sources: HashMap<String, Box<dyn SkillSource>>,
}

impl SourceRegistry {
    pub fn new() -> Self {
        Self {
            sources: HashMap::new(),
        }
    }

    pub fn register(&mut self, source: Box<dyn SkillSource>) {
        let source_type = source.source_type();
        let type_name = match source_type {
            SourceType::Git => "git",
            SourceType::Oci => "oci",
            SourceType::Local => "local",
        };
        self.sources.insert(type_name.to_string(), source);
    }

    pub fn get(&self, source_type: &str) -> Option<&dyn SkillSource> {
        self.sources.get(source_type).map(|s| s.as_ref())
    }
}
