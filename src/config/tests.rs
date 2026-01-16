#[cfg(test)]
mod tests {
    use crate::config::skillset::{SkillConfig, SkillsetConfig};
    use std::collections::HashMap;
    use tempfile::TempDir;

    #[test]
    fn test_default_config_uses_implicit_defaults() {
        let config = SkillsetConfig::default();

        // Both fields should be None initially
        assert_eq!(config.registry, None);
        assert_eq!(config.conventions, None);

        // Helper methods should provide runtime defaults
        assert_eq!(config.get_registry(), "ghcr.io/skillset");
        assert_eq!(config.get_conventions(), vec!["autogpt", "langchain"]);
    }

    #[test]
    fn test_explicit_registry_override() {
        let config = SkillsetConfig {
            skills: HashMap::new(),
            registry: Some("my-registry.example.com/custom".to_string()),
            conventions: None,
        };

        assert_eq!(config.get_registry(), "my-registry.example.com/custom");
        assert_eq!(config.get_conventions(), vec!["autogpt", "langchain"]); // Still uses default
    }

    #[test]
    fn test_explicit_conventions_override() {
        let config = SkillsetConfig {
            skills: HashMap::new(),
            registry: None,
            conventions: Some(vec!["autogpt".to_string()]), // Only autogpt
        };

        assert_eq!(config.get_registry(), "ghcr.io/skillset"); // Still uses default
        assert_eq!(config.get_conventions(), vec!["autogpt"]);
    }

    #[test]
    fn test_empty_conventions_disables_all() {
        let config = SkillsetConfig {
            skills: HashMap::new(),
            registry: None,
            conventions: Some(vec![]), // Empty list
        };

        assert_eq!(config.get_conventions(), Vec::<String>::new());
    }

    #[test]
    fn test_minimal_config_serialization() {
        let config = SkillsetConfig {
            skills: HashMap::new(),
            registry: None,
            conventions: None,
        };

        let json = serde_json::to_string_pretty(&config).unwrap();

        // Should not include registry or conventions fields when None
        assert!(!json.contains("registry"));
        assert!(!json.contains("conventions"));
        assert!(json.contains("\"skills\": {}"));
    }

    #[test]
    fn test_partial_config_serialization() {
        let mut skills = HashMap::new();
        skills.insert(
            "test-skill".to_string(),
            SkillConfig::Simple("1.0.0".to_string()),
        );

        let config = SkillsetConfig {
            skills,
            registry: Some("custom-registry.com".to_string()),
            conventions: None,
        };

        let json = serde_json::to_string_pretty(&config).unwrap();

        // Should include registry but not conventions
        assert!(json.contains("registry"));
        assert!(json.contains("custom-registry.com"));
        assert!(!json.contains("conventions"));
    }

    #[test]
    fn test_load_minimal_config_from_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("skillset.json");

        // Write minimal config with only skills
        std::fs::write(
            &config_path,
            r#"{
  "skills": {
    "test-skill": "1.0.0"
  }
}"#,
        )
        .unwrap();

        let config = SkillsetConfig::load_from_file(&config_path).unwrap();

        // Should use implicit defaults for missing fields
        assert_eq!(config.registry, None);
        assert_eq!(config.conventions, None);
        assert_eq!(config.get_registry(), "ghcr.io/skillset");
        assert_eq!(config.get_conventions(), vec!["autogpt", "langchain"]);
    }

    #[test]
    fn test_resolve_reference_with_default_registry() {
        let config = SkillsetConfig {
            skills: HashMap::new(),
            registry: None, // Use default
            conventions: None,
        };

        let skill_config = SkillConfig::Simple("1.0.0".to_string());
        let resolved = config
            .resolve_skill_reference("file-analyzer", &skill_config)
            .unwrap();

        // Should use default registry
        assert_eq!(resolved, "oci:ghcr.io/skillset/file-analyzer:v1.0.0");
    }

    #[test]
    fn test_resolve_reference_with_custom_registry() {
        let config = SkillsetConfig {
            skills: HashMap::new(),
            registry: Some("my-registry.com/ns".to_string()),
            conventions: None,
        };

        let skill_config = SkillConfig::Simple("1.0.0".to_string());
        let resolved = config
            .resolve_skill_reference("file-analyzer", &skill_config)
            .unwrap();

        // Should use custom registry
        assert_eq!(resolved, "oci:my-registry.com/ns/file-analyzer:v1.0.0");
    }

    #[test]
    fn test_resolve_scoped_reference_with_default_registry() {
        let config = SkillsetConfig::default();

        let skill_config = SkillConfig::Simple("2.1.0".to_string());
        let resolved = config
            .resolve_skill_reference("@johndoe/web-scraper", &skill_config)
            .unwrap();

        // Should extract domain from default registry and user from scope
        assert_eq!(resolved, "oci:ghcr.io/johndoe/web-scraper:v2.1.0");
    }
}
