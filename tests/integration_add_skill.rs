use assert_cmd::prelude::*;

mod common;

#[tokio::test]
async fn test_add_simple_skill_happy_path() {
    let test_project = common::TestProject::new().expect("Failed to create test project");

    // Add a simple skill with version (using git URL for now)
    let mut cmd = test_project
        .run_skillset_command(&["add", "git:https://github.com/user/file-analyzer.git"]);

    // Verify command succeeds (this will fail due to unimplemented OCI fetching)
    cmd.assert().success();
}

#[tokio::test]
async fn test_add_scoped_skill_happy_path() {
    let test_project = common::TestProject::new().expect("Failed to create test project");

    // Add a scoped skill with version
    let mut cmd = test_project
        .run_skillset_command(&["add", "git:https://github.com/johndoe/web-scraper.git"]);

    // Verify command succeeds (this will fail due to unimplemented OCI fetching)
    cmd.assert().success();
}

#[tokio::test]
async fn test_add_skill_with_convention_override() {
    let test_project = common::TestProject::new().expect("Failed to create test project");

    // Add a skill with convention override
    let mut cmd = test_project.run_skillset_command(&[
        "add",
        "git:https://github.com/user/custom-tool.git",
        "--convention",
        "autogpt",
    ]);

    // Verify command succeeds (this will fail due to unimplemented OCI fetching)
    cmd.assert().success();
}

#[tokio::test]
async fn test_add_skill_with_version_override() {
    let test_project = common::TestProject::new().expect("Failed to create test project");

    // Add a skill with version override
    let mut cmd = test_project.run_skillset_command(&[
        "add",
        "git:https://github.com/user/versioned-tool.git",
        "--version",
        "3.0.0",
    ]);

    // Verify command succeeds (this will fail due to unimplemented OCI fetching)
    cmd.assert().success();
}

#[tokio::test]
async fn test_add_multiple_skills() {
    let test_project = common::TestProject::new().expect("Failed to create test project");

    // Add first skill
    let mut cmd1 =
        test_project.run_skillset_command(&["add", "git:https://github.com/user/first-skill.git"]);
    cmd1.assert().success();

    // Add second skill
    let mut cmd2 =
        test_project.run_skillset_command(&["add", "git:https://github.com/user/second-skill.git"]);
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
    let mut cmd =
        test_project.run_skillset_command(&["add", "git:https://github.com/user/new-skill.git"]);
    cmd.assert().success();
}
