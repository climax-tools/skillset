use assert_cmd::prelude::*;

mod common;

#[tokio::test]
async fn test_add_simple_skill_happy_path() {
    let test_project = common::TestProject::new().expect("Failed to create test project");

    // Add a simple skill with version
    let mut cmd = test_project.run_skillset_command(&["add", "file-analyzer@1.0.0"]);

    // Verify command succeeds (this will fail due to unimplemented OCI fetching)
    cmd.assert().success();
}

#[tokio::test]
async fn test_add_scoped_skill_happy_path() {
    let test_project = common::TestProject::new().expect("Failed to create test project");

    // Add a scoped skill with version
    let mut cmd = test_project.run_skillset_command(&["add", "@johndoe/web-scraper@2.1.0"]);

    // Verify command succeeds (this will fail due to unimplemented OCI fetching)
    cmd.assert().success();
}

#[tokio::test]
async fn test_add_skill_with_convention_override() {
    let test_project = common::TestProject::new().expect("Failed to create test project");

    // Add a skill with convention override
    let mut cmd =
        test_project.run_skillset_command(&["add", "custom-tool@1.5.0", "--convention", "autogpt"]);

    // Verify command succeeds (this will fail due to unimplemented OCI fetching)
    cmd.assert().success();
}

#[tokio::test]
async fn test_add_skill_with_version_override() {
    let test_project = common::TestProject::new().expect("Failed to create test project");

    // Add a skill with version override
    let mut cmd =
        test_project.run_skillset_command(&["add", "versioned-tool@2.0.0", "--version", "3.0.0"]);

    // Verify command succeeds (this will fail due to unimplemented OCI fetching)
    cmd.assert().success();
}

#[tokio::test]
async fn test_add_multiple_skills() {
    let test_project = common::TestProject::new().expect("Failed to create test project");

    // Add first skill
    let mut cmd1 = test_project.run_skillset_command(&["add", "first-skill@1.0.0"]);
    cmd1.assert().success();

    // Add second skill
    let mut cmd2 = test_project.run_skillset_command(&["add", "@user/second-skill@2.0.0"]);
    cmd2.assert().success();
}

#[tokio::test]
async fn test_add_skill_preserves_existing_config() {
    let test_project = common::TestProject::new().expect("Failed to create test project");

    // First, manually add a skill to the config to simulate existing state
    let initial_config = r#"
{
  "skills": {
    "existing-skill": "0.1.0"
  },
  "registry": "ghcr.io/skillset",
  "conventions": ["autogpt", "langchain"]
}
"#;
    std::fs::write(test_project.skillset_config_path(), initial_config)
        .expect("Failed to write initial config");

    // Add a new skill
    let mut cmd = test_project.run_skillset_command(&["add", "new-skill@1.0.0"]);
    cmd.assert().success();
}
