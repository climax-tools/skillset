use std::path::PathBuf;

use crate::config::skillset::SkillsetConfig;
use crate::conventions::ConventionRegistry;
use crate::error::Result;
use crate::skill::FetchedSkill;

pub struct SkillManager {
    convention_registry: ConventionRegistry,
    config: SkillsetConfig,
    project_path: PathBuf,
}

impl SkillManager {
    pub fn new(project_path: PathBuf) -> Result<Self> {
        let config = Self::load_config(&project_path)?;
        let mut convention_registry = ConventionRegistry::new();

        // Register built-in conventions
        convention_registry.register(Box::new(crate::conventions::AutoGptConvention::new()));
        convention_registry.register(Box::new(crate::conventions::LangchainConvention::new()));

        Ok(Self {
            convention_registry,
            config,
            project_path,
        })
    }

    fn load_config(project_path: &PathBuf) -> Result<SkillsetConfig> {
        let config_path = project_path.join("skillset.json");
        if config_path.exists() {
            SkillsetConfig::load_from_file(&config_path)
        } else {
            Ok(SkillsetConfig::default())
        }
    }

    pub fn config_mut(&mut self) -> &mut SkillsetConfig {
        &mut self.config
    }

    pub fn config(&self) -> &SkillsetConfig {
        &self.config
    }

    pub fn save_config(&self) -> Result<()> {
        let config_path = self.project_path.join("skillset.json");
        self.config.save_to_file(&config_path)
    }

    pub async fn add_skill(
        &mut self,
        reference: &str,
        convention: Option<String>,
        version: Option<String>,
    ) -> Result<()> {
        // Parse reference to determine source type
        let (source_type, source_ref, skill_name) = self.parse_reference(reference)?;

        // Fetch skill from source
        let fetched_skill = self.fetch_skill(&source_type, &source_ref, version).await?;

        // Detect or use specified convention
        let convention_name = if let Some(conv) = convention {
            conv
        } else {
            self.detect_convention(&fetched_skill.source_path).await?
        };

        // Organize skill according to convention
        self.organize_skill(&fetched_skill, &convention_name)
            .await?;

        // Update configuration
        self.update_config(&fetched_skill, &convention_name)?;

        println!("Successfully added skill: {}", fetched_skill.name);
        Ok(())
    }

    pub async fn add_skill_by_name(
        &mut self,
        skill_name: &str,
        skill_config: &crate::config::skillset::SkillConfig,
    ) -> Result<()> {
        // Resolve skill name to OCI reference
        let resolved_reference = self
            .config
            .resolve_skill_reference(skill_name, skill_config)?;

        // Parse the resolved reference
        let (source_type, source_ref, actual_name) = self.parse_reference(&resolved_reference)?;

        // Fetch skill from source
        let version = Some(skill_config.get_version().to_string());
        let fetched_skill = self.fetch_skill(&source_type, &source_ref, version).await?;

        // Use convention from config if specified, otherwise auto-detect
        let convention_name = if let Some(conv) = skill_config.get_explicit_convention() {
            conv.clone()
        } else {
            self.detect_convention(&fetched_skill.source_path).await?
        };

        // Organize skill according to convention
        self.organize_skill(&fetched_skill, &convention_name)
            .await?;

        // Update configuration (this should already be in config, but we might need to add metadata)
        self.update_config(&fetched_skill, &convention_name)?;

        println!("Successfully added skill: {}", skill_name);
        Ok(())
    }

    fn parse_reference(&self, reference: &str) -> Result<(String, String, String)> {
        // Check for explicit sources first
        if reference.starts_with("git:")
            || reference.starts_with("https://github.com")
            || reference.starts_with("git@")
        {
            let source_ref = reference
                .strip_prefix("git:")
                .unwrap_or(reference)
                .to_string();
            let skill_name = self.extract_skill_name_from_git(&source_ref)?;
            Ok(("git".to_string(), source_ref, skill_name))
        } else if reference.starts_with("oci:")
            || reference.contains("ghcr.io")
            || reference.contains("docker.io")
        {
            let source_ref = reference
                .strip_prefix("oci:")
                .unwrap_or(reference)
                .to_string();
            let skill_name = self.extract_skill_name_from_oci(&source_ref)?;
            Ok(("oci".to_string(), source_ref, skill_name))
        } else if reference.starts_with("/") || reference.starts_with("./") {
            let skill_name = self.extract_skill_name_from_path(reference)?;
            Ok(("local".to_string(), reference.to_string(), skill_name))
        } else {
            // Treat as simplified skill name that needs resolution
            // For now, we'll create a temporary config to resolve it
            // In practice, this would be handled at the CLI level
            Err(crate::error::SkillsetError::Source(format!(
                "Simplified skill names should be resolved at CLI level: {}",
                reference
            )))
        }
    }

    fn extract_skill_name_from_git(&self, git_url: &str) -> Result<String> {
        // Extract repo name from git URL
        // Example: https://github.com/user/skill-name.git -> skill-name
        let parts: Vec<&str> = git_url.split('/').collect();
        if parts.len() >= 2 {
            let last_part = parts.last().unwrap();
            let name = last_part.trim_end_matches(".git");
            Ok(name.to_string())
        } else {
            Err(crate::error::SkillsetError::Source(format!(
                "Unable to extract skill name from git URL: {}",
                git_url
            )))
        }
    }

    fn extract_skill_name_from_oci(&self, oci_ref: &str) -> Result<String> {
        // Extract repository name from OCI reference
        // Example: ghcr.io/user/skill-name:tag -> skill-name
        let parts: Vec<&str> = oci_ref.split('/').collect();
        if parts.len() >= 3 {
            let repo_part = parts.last().unwrap();
            let name_with_tag = repo_part.split(':').next().unwrap_or(repo_part);
            Ok(name_with_tag.to_string())
        } else {
            Err(crate::error::SkillsetError::Source(format!(
                "Unable to extract skill name from OCI reference: {}",
                oci_ref
            )))
        }
    }

    fn extract_skill_name_from_path(&self, path: &str) -> Result<String> {
        // Extract skill name from local path
        let path_buf = std::path::PathBuf::from(path);
        path_buf
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.to_string())
            .ok_or_else(|| {
                crate::error::SkillsetError::Source(format!(
                    "Unable to extract skill name from path: {}",
                    path
                ))
            })
    }

    pub async fn remove_skill(&mut self, skill_name: &str) -> Result<()> {
        // Check if skill exists
        if !self.config.skills.contains_key(skill_name) {
            return Err(crate::error::SkillsetError::SkillNotFound(
                skill_name.to_string(),
            ));
        }

        // Remove from configuration
        self.config.skills.remove(skill_name);

        // Remove from filesystem (this is complex, would need to find the actual location)
        // For now, just update config
        self.save_config()?;

        println!("Successfully removed skill: {}", skill_name);
        Ok(())
    }

    pub async fn show_skill_info(&self, skill_name: &str) -> Result<()> {
        if let Some(skill_config) = self.config.skills.get(skill_name) {
            let version = skill_config.get_version();
            println!("Skill: {}", skill_name);
            println!("Version: {}", version);

            if let Some(source) = skill_config.get_explicit_source() {
                println!("Source: {}", source);
            } else {
                // Show resolved reference
                let resolved = self
                    .config
                    .resolve_skill_reference(skill_name, skill_config)?;
                println!("Resolved Source: {}", resolved);
            }

            if let Some(convention) = skill_config.get_explicit_convention() {
                println!("Convention: {}", convention);
            }

            // TODO: Add more info like installation date, file location, etc.
        } else {
            return Err(crate::error::SkillsetError::SkillNotFound(
                skill_name.to_string(),
            ));
        }

        Ok(())
    }

    async fn fetch_skill(
        &self,
        source_type: &str,
        source_ref: &str,
        _version: Option<String>,
    ) -> Result<FetchedSkill> {
        match source_type {
            "git" => {
                // TODO: Implement Git source fetching
                todo!("Implement Git source fetching")
            }
            "oci" => {
                // TODO: Implement OCI source fetching
                todo!("Implement OCI source fetching")
            }
            "local" => {
                // TODO: Implement local source handling
                todo!("Implement local source handling")
            }
            _ => Err(crate::error::SkillsetError::SourceNotFound(
                source_type.to_string(),
            )),
        }
    }

    async fn detect_convention(&self, path: &PathBuf) -> Result<String> {
        if let Some(detected) = self.convention_registry.detect_convention(path).await? {
            Ok(detected)
        } else {
            // Default to autogpt if nothing detected
            Ok("autogpt".to_string())
        }
    }

    async fn organize_skill(
        &self,
        fetched_skill: &FetchedSkill,
        convention_name: &str,
    ) -> Result<()> {
        if let Some(convention) = self.convention_registry.get(convention_name) {
            convention
                .organize(
                    &fetched_skill.name,
                    &fetched_skill.source_path,
                    &self.project_path,
                )
                .await
        } else {
            Err(crate::error::SkillsetError::ConventionNotFound(
                convention_name.to_string(),
            ))
        }
    }

    fn update_config(&mut self, fetched_skill: &FetchedSkill, convention_name: &str) -> Result<()> {
        // Add skill to configuration with explicit source and convention
        self.config.skills.insert(
            fetched_skill.name.clone(),
            crate::config::skillset::SkillConfig::Detailed {
                version: fetched_skill.version.clone(),
                source: Some(fetched_skill.source_path.display().to_string()),
                convention: Some(convention_name.to_string()),
            },
        );

        // Save configuration
        let config_path = self.project_path.join("skillset.json");
        self.config.save_to_file(&config_path)?;

        Ok(())
    }

    pub fn list_skills(&self, verbose: bool) -> Result<()> {
        if self.config.skills.is_empty() {
            println!("No skills installed.");
            return Ok(());
        }

        println!("Installed skills:");
        for (name, skill_config) in &self.config.skills {
            let version = skill_config.get_version();
            if verbose {
                if let (Some(source), Some(convention)) = (
                    skill_config.get_explicit_source(),
                    skill_config.get_explicit_convention(),
                ) {
                    println!(
                        "  {} (v{}) - Source: {} - Convention: {}",
                        name, version, source, convention
                    );
                } else if let Some(source) = skill_config.get_explicit_source() {
                    println!("  {} (v{}) - Source: {}", name, version, source);
                } else {
                    println!("  {} (v{})", name, version);
                }
            } else {
                println!("  {} (v{})", name, version);
            }
        }

        Ok(())
    }
}
