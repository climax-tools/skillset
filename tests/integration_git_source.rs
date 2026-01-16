use assert_cmd::prelude::*;

mod common;

#[tokio::test]
async fn test_add_git_source_happy_path() {
    let test_project = common::TestProject::new().expect("Failed to create test project");

    // Add a skill from git source
    let mut cmd = test_project
        .run_skillset_command(&["add", "git:https://github.com/octocat/Hello-World.git"]);

    // Verify command succeeds
    cmd.assert().success();

    // Verify skill was added to configuration
    let config = test_project
        .read_skillset_config()
        .expect("Failed to read skillset.json");
    assert!(config.contains("Hello-World"));
    assert!(config.contains("latest"));

    // Verify skill was cached in user-wide cache
    let user_cache_dir = dirs::cache_dir()
        .expect("No cache directory found")
        .join("skillset/git/checkouts/Hello-World");
    assert!(user_cache_dir.exists());
    assert!(user_cache_dir.join("README").exists());

    // Verify skill was organized
    let skill_dir = test_project
        .project_path()
        .join("skills/autogpt/Hello-World");
    assert!(skill_dir.exists());
    assert!(skill_dir.join("README").exists());
}

#[tokio::test]
async fn test_add_git_source_with_convention_override() {
    let test_project = common::TestProject::new().expect("Failed to create test project");

    // Add a skill from git source with explicit convention
    let mut cmd = test_project.run_skillset_command(&[
        "add",
        "git:https://github.com/octocat/Hello-World.git",
        "--convention",
        "langchain",
    ]);

    // Verify command succeeds
    cmd.assert().success();

    // Verify skill was added with langchain convention
    let config = test_project
        .read_skillset_config()
        .expect("Failed to read skillset.json");
    assert!(config.contains("Hello-World"));
    assert!(config.contains("langchain"));

    // Verify skill was organized under langchain convention
    let skill_dir = test_project
        .project_path()
        .join("skills/langchain/Hello-World");
    assert!(skill_dir.exists());
    assert!(skill_dir.join("README").exists());
}

#[tokio::test]
async fn test_add_git_source_preserves_existing_config() {
    let test_project = common::TestProject::new().expect("Failed to create test project");

    // Create initial configuration
    test_project
        .write_skillset_config(
            r#"
    {
        "skills": {
            "existing-skill": "1.0.0"
        },
        "registry": "ghcr.io/skillset",
        "conventions": ["autogpt", "langchain"]
    }
    "#,
        )
        .expect("Failed to write initial config");

    // Add a git skill
    let mut cmd = test_project
        .run_skillset_command(&["add", "git:https://github.com/octocat/Hello-World.git"]);

    // Verify command succeeds
    cmd.assert().success();

    // Verify both skills exist in config
    let config = test_project
        .read_skillset_config()
        .expect("Failed to read skillset.json");
    assert!(config.contains("existing-skill"));
    assert!(config.contains("Hello-World"));
}
