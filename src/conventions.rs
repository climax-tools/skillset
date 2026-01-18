use crate::error::Result;
use async_trait::async_trait;
use std::collections::HashMap;

#[async_trait]
pub trait Convention: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn description(&self) -> &str;
    async fn detect(&self, path: &std::path::Path) -> Result<bool>;
    async fn organize(
        &self,
        skill_name: &str,
        source_path: &std::path::Path,
        target_path: &std::path::Path,
    ) -> Result<()>;
}

pub struct ConventionRegistry {
    conventions: HashMap<String, Box<dyn Convention>>,
}

impl ConventionRegistry {
    pub fn new() -> Self {
        Self {
            conventions: HashMap::new(),
        }
    }

    pub fn register(&mut self, convention: Box<dyn Convention>) {
        let name = convention.name().to_string();
        self.conventions.insert(name, convention);
    }

    pub fn get(&self, name: &str) -> Option<&dyn Convention> {
        self.conventions.get(name).map(|c| c.as_ref())
    }

    pub fn list(&self) -> Vec<&str> {
        self.conventions.keys().map(|k| k.as_str()).collect()
    }

    pub async fn detect_convention(&self, path: &std::path::Path) -> Result<Option<String>> {
        for (name, convention) in &self.conventions {
            if convention.detect(path).await? {
                return Ok(Some(name.clone()));
            }
        }
        Ok(None)
    }
}

pub struct AutoGptConvention;

impl AutoGptConvention {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Convention for AutoGptConvention {
    fn name(&self) -> &str {
        "autogpt"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn description(&self) -> &str {
        "Auto-GPT agent framework convention"
    }

    async fn detect(&self, path: &std::path::Path) -> Result<bool> {
        let skill_py = path.join("skill.py");
        let requirements_txt = path.join("requirements.txt");
        Ok(skill_py.exists() && requirements_txt.exists())
    }

    async fn organize(
        &self,
        skill_name: &str,
        source_path: &std::path::Path,
        target_path: &std::path::Path,
    ) -> Result<()> {
        let final_path = target_path.join("skills").join("autogpt").join(skill_name);
        std::fs::create_dir_all(&final_path)?;

        // Copy skill files
        copy_dir_all(source_path, &final_path)?;

        Ok(())
    }
}

pub struct LangchainConvention;

impl LangchainConvention {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Convention for LangchainConvention {
    fn name(&self) -> &str {
        "langchain"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn description(&self) -> &str {
        "LangChain agent framework convention"
    }

    async fn detect(&self, path: &std::path::Path) -> Result<bool> {
        let tool_yaml = path.join("tool.yaml");
        let py_files = path.read_dir()?.any(|entry| {
            if let Ok(entry) = entry {
                if let Some(name) = entry.file_name().to_str() {
                    return name.ends_with(".py");
                }
            }
            false
        });
        Ok(tool_yaml.exists() || py_files)
    }

    async fn organize(
        &self,
        skill_name: &str,
        source_path: &std::path::Path,
        target_path: &std::path::Path,
    ) -> Result<()> {
        let final_path = target_path
            .join("skills")
            .join("langchain")
            .join(skill_name);
        std::fs::create_dir_all(&final_path)?;

        copy_dir_all(source_path, &final_path)?;

        Ok(())
    }
}

pub struct AgentSkillsConvention;

impl AgentSkillsConvention {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Convention for AgentSkillsConvention {
    fn name(&self) -> &str {
        "agent-skills"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn description(&self) -> &str {
        "Vercel Agent Skills convention (SKILL.md format)"
    }

    async fn detect(&self, path: &std::path::Path) -> Result<bool> {
        let skill_md = path.join("SKILL.md");
        let skill_yaml = path.join("skill.yaml");
        let scripts_dir = path.join("scripts");
        let references_dir = path.join("references");

        Ok(skill_md.exists()
            || skill_yaml.exists()
            || scripts_dir.exists()
            || references_dir.exists())
    }

    async fn organize(
        &self,
        skill_name: &str,
        source_path: &std::path::Path,
        target_path: &std::path::Path,
    ) -> Result<()> {
        let final_path = target_path
            .join("skills")
            .join("agent-skills")
            .join(skill_name);
        std::fs::create_dir_all(&final_path)?;

        copy_dir_all(source_path, &final_path)?;

        Ok(())
    }
}

fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else if file_type.is_file() {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
