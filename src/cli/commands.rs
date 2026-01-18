use crate::{cli::ConventionCommands, error::Result};

pub async fn handle_add(
    reference: String,
    convention: Option<String>,
    version: Option<String>,
) -> Result<()> {
    // Get current directory as project path
    let project_path = std::env::current_dir()?;

    // Initialize skill manager
    let mut skill_manager = crate::skill::manager::SkillManager::new(project_path)?;

    // Parse the reference to determine if it's a simplified name or explicit source
    if is_simplified_skill_reference(&reference) {
        // Handle simplified skill names like "file-analyzer@1.0.0" or "@user/skill@2.0.0"
        let (skill_name, skill_version) = parse_skill_reference(&reference)?;

        // Create or update configuration with the new skill
        let skill_config = if version.is_some() {
            // Version was overridden in CLI
            crate::config::skillset::SkillConfig::Detailed {
                version: version.unwrap(),
                source: None,
                convention,
            }
        } else {
            crate::config::skillset::SkillConfig::Simple(skill_version.to_string())
        };

        // Add to configuration first
        {
            let config = skill_manager.config_mut();
            config
                .skills
                .insert(skill_name.clone(), skill_config.clone());
        }

        // Save configuration before adding skill
        skill_manager.save_config()?;

        // Add skill using simplified name resolution
        skill_manager
            .add_skill_by_name(&skill_name, &skill_config)
            .await?;
    } else {
        // Handle explicit source references
        skill_manager
            .add_skill(&reference, convention, version)
            .await?;
    }

    Ok(())
}

fn is_simplified_skill_reference(reference: &str) -> bool {
    // Check if it's a simplified skill name (not an explicit source)
    !reference.starts_with("git:")
        && !reference.starts_with("oci:")
        && !reference.starts_with("https://")
        && !reference.starts_with("http://")
        && !reference.starts_with("/")
        && !reference.starts_with("./")
        && !reference.starts_with("../")
        && reference.contains('@') // Should have @version
}

fn parse_skill_reference(reference: &str) -> Result<(String, &str)> {
    // Parse skill references like "file-analyzer@1.0.0" or "@user/skill@2.0.0"
    if let Some(at_pos) = reference.rfind('@') {
        let skill_name = &reference[..at_pos];
        let version = &reference[at_pos + 1..];

        // Validate skill name format
        validate_skill_name(skill_name)?;
        validate_version(version)?;

        Ok((skill_name.to_string(), version))
    } else {
        Err(crate::error::SkillsetError::InvalidSkillName(
            "Skill reference must include version with @. Expected: name@version".to_string(),
        ))
    }
}

fn validate_skill_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(crate::error::SkillsetError::InvalidSkillName(
            "Skill name cannot be empty".to_string(),
        ));
    }

    // Handle scoped names like @user/skill
    if name.starts_with('@') {
        let scoped_part = &name[1..]; // Remove '@'
        if scoped_part.is_empty() {
            return Err(crate::error::SkillsetError::InvalidSkillName(
                "Scoped skill name cannot be empty after @".to_string(),
            ));
        }

        let parts: Vec<&str> = scoped_part.split('/').collect();
        if parts.len() != 2 {
            return Err(crate::error::SkillsetError::InvalidSkillName(
                "Scoped skill name must be in format @user/skill".to_string(),
            ));
        }

        let (user, skill) = (parts[0], parts[1]);

        // Validate username part
        if user.is_empty() || user.len() > 39 {
            return Err(crate::error::SkillsetError::InvalidSkillName(
                "Username in scoped skill name must be 1-39 characters".to_string(),
            ));
        }

        // Validate skill name part
        if skill.is_empty() || skill.len() > 100 {
            return Err(crate::error::SkillsetError::InvalidSkillName(
                "Skill name must be 1-100 characters".to_string(),
            ));
        }

        // Check for valid characters (alphanumeric, hyphens, underscores)
        if !user
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(crate::error::SkillsetError::InvalidSkillName(
                "Username can only contain alphanumeric characters, hyphens, and underscores"
                    .to_string(),
            ));
        }

        if !skill
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(crate::error::SkillsetError::InvalidSkillName(
                "Skill name can only contain alphanumeric characters, hyphens, and underscores"
                    .to_string(),
            ));
        }
    } else {
        // Validate simple skill names
        if name.len() > 100 {
            return Err(crate::error::SkillsetError::InvalidSkillName(
                "Skill name must be 1-100 characters".to_string(),
            ));
        }

        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(crate::error::SkillsetError::InvalidSkillName(
                "Skill name can only contain alphanumeric characters, hyphens, and underscores"
                    .to_string(),
            ));
        }
    }

    Ok(())
}

fn validate_version(version: &str) -> Result<()> {
    if version.is_empty() {
        return Err(crate::error::SkillsetError::InvalidSkillName(
            "Version cannot be empty".to_string(),
        ));
    }

    // Allow "latest" or semantic version patterns
    if version == "latest" {
        return Ok(());
    }

    // Basic semantic version validation (X.Y.Z or vX.Y.Z)
    let version_to_check = if version.starts_with('v') {
        &version[1..]
    } else {
        version
    };

    // Split by dots and validate each part is numeric
    let parts: Vec<&str> = version_to_check.split('.').collect();
    if parts.len() < 2 || parts.len() > 4 {
        return Err(crate::error::SkillsetError::InvalidSkillName(
            "Version must be in semantic version format (e.g., 1.0.0 or v1.0.0)".to_string(),
        ));
    }

    for part in parts {
        if part.is_empty() {
            return Err(crate::error::SkillsetError::InvalidSkillName(
                "Version parts cannot be empty".to_string(),
            ));
        }

        // Allow numeric parts and pre-release identifiers
        if part
            .chars()
            .any(|c| !c.is_alphanumeric() && c != '-' && c != '+')
        {
            return Err(crate::error::SkillsetError::InvalidSkillName(
                "Version parts can only contain alphanumeric characters, hyphens, and plus signs"
                    .to_string(),
            ));
        }
    }

    Ok(())
}

pub async fn handle_remove(name: String) -> Result<()> {
    let project_path = std::env::current_dir()?;
    let mut skill_manager = crate::skill::manager::SkillManager::new(project_path)?;

    skill_manager.remove_skill(&name).await
}

pub async fn handle_list(verbose: bool) -> Result<()> {
    let project_path = std::env::current_dir()?;
    let skill_manager = crate::skill::manager::SkillManager::new(project_path)?;

    skill_manager.list_skills(verbose)
}

pub async fn handle_update(name: Option<String>) -> Result<()> {
    println!("Updating skills (specific: {:?})", name);
    todo!("Implement update command")
}

pub async fn handle_info(name: String) -> Result<()> {
    let project_path = std::env::current_dir()?;
    let skill_manager = crate::skill::manager::SkillManager::new(project_path)?;

    skill_manager.show_skill_info(&name).await
}

pub async fn handle_convention(command: ConventionCommands) -> Result<()> {
    let project_path = std::env::current_dir().map_err(|e| crate::error::SkillsetError::Io(e))?;
    let mut manager = crate::skill::manager::SkillManager::new(project_path)?;

    match command {
        ConventionCommands::List => {
            let conventions = manager.config().get_conventions();
            if conventions.is_empty() {
                println!("No conventions are enabled");
            } else {
                println!("Enabled conventions:");
                for convention in conventions {
                    println!("  - {}", convention);
                }
            }
            Ok(())
        }
        ConventionCommands::Enable { name } => {
            let config = manager.config_mut();
            let mut conventions = config.get_conventions();

            if conventions.contains(&name) {
                println!("Convention '{}' is already enabled", name);
                return Ok(());
            }

            // Validate convention name
            if !["autogpt", "langchain", "agent-skills"].contains(&name.as_str()) {
                return Err(crate::error::SkillsetError::Config(format!(
                    "Unknown convention: {}. Available: autogpt, langchain, agent-skills",
                    name
                )));
            }

            conventions.push(name.clone());
            config.conventions = Some(conventions);
            manager.save_config()?;
            println!("Enabled convention: {}", name);
            Ok(())
        }
        ConventionCommands::Disable { name } => {
            let config = manager.config_mut();
            let mut conventions = config.get_conventions();

            if let Some(pos) = conventions.iter().position(|c| c == &name) {
                conventions.remove(pos);
                config.conventions = Some(conventions);
                manager.save_config()?;
                println!("Disabled convention: {}", name);
            } else {
                println!("Convention '{}' is not enabled", name);
            }
            Ok(())
        }
        ConventionCommands::Configure { name } => {
            println!("Configuring convention: {}", name);
            todo!("Implement convention configure")
        }
    }
}

pub async fn handle_publish(
    path: String,
    reference: String,
    registry: Option<String>,
) -> Result<()> {
    println!(
        "Publishing skill from {} to {} (registry: {:?})",
        path, reference, registry
    );
    todo!("Implement publish command")
}
