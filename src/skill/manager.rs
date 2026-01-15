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

    pub async fn add_skill(
        &mut self,
        reference: &str,
        convention: Option<String>,
        version: Option<String>,
    ) -> Result<()> {
        // Parse reference to determine source type
        let (source_type, source_ref) = self.parse_reference(reference)?;

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

    fn parse_reference(&self, reference: &str) -> Result<(String, String)> {
        if reference.starts_with("git:")
            || reference.starts_with("https://github.com")
            || reference.starts_with("git@")
        {
            Ok((
                "git".to_string(),
                reference
                    .strip_prefix("git:")
                    .unwrap_or(reference)
                    .to_string(),
            ))
        } else if reference.starts_with("oci:")
            || reference.contains("ghcr.io")
            || reference.contains("docker.io")
        {
            Ok((
                "oci".to_string(),
                reference
                    .strip_prefix("oci:")
                    .unwrap_or(reference)
                    .to_string(),
            ))
        } else if reference.starts_with("/") || reference.starts_with("./") {
            Ok(("local".to_string(), reference.to_string()))
        } else {
            Err(crate::error::SkillsetError::Source(format!(
                "Unable to determine source type for reference: {}",
                reference
            )))
        }
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
        // Add skill to configuration
        self.config.skills.insert(
            fetched_skill.name.clone(),
            crate::config::skillset::SkillConfig {
                version: fetched_skill.version.clone(),
                source: "unknown".to_string(), // TODO: Track original source
                installed_at: Some("2025-01-14T18:30:00Z".to_string()), // TODO: Use chrono
                repo_path: Some(fetched_skill.source_path.display().to_string()),
                convention: Some(convention_name.to_string()),
                checksum: None, // TODO: Calculate checksum
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
            if verbose {
                if let (Some(installed_at), Some(convention)) =
                    (&skill_config.installed_at, &skill_config.convention)
                {
                    println!(
                        "  {} (v{}) - {} - Convention: {}",
                        name, skill_config.version, installed_at, convention
                    );
                } else {
                    println!("  {} (v{})", name, skill_config.version);
                }
            } else {
                println!("  {} (v{})", name, skill_config.version);
            }
        }

        Ok(())
    }
}
