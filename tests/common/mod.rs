use std::path::PathBuf;
use tempfile::TempDir;

pub struct TestProject {
    temp_dir: TempDir,
    project_path: PathBuf,
}

impl TestProject {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = tempfile::tempdir()?;
        let project_path = temp_dir.path().to_path_buf();

        // Create initial skillset.json with default config
        let config_path = project_path.join("skillset.json");
        let default_config = r#"
{
  "skills": {},
  "registry": "ghcr.io/skillset",
  "conventions": ["autogpt", "langchain"]
}
"#;
        std::fs::write(&config_path, default_config)?;

        Ok(Self {
            temp_dir,
            project_path,
        })
    }

    pub fn project_path(&self) -> &PathBuf {
        &self.project_path
    }

    pub fn skillset_config_path(&self) -> PathBuf {
        self.project_path.join("skillset.json")
    }

    pub fn run_skillset_command(&self, args: &[&str]) -> std::process::Command {
        // Use the built binary from target/debug
        let binary_path = format!("{}/target/debug/skillset", env!("CARGO_MANIFEST_DIR"));

        let mut cmd = std::process::Command::new(binary_path);
        cmd.args(args)
            .current_dir(&self.project_path)
            .env("RUST_LOG", "off");
        cmd
    }

    pub fn read_skillset_config(&self) -> Result<String, Box<dyn std::error::Error>> {
        let config_path = self.skillset_config_path();
        Ok(std::fs::read_to_string(&config_path)?)
    }
}

impl Drop for TestProject {
    fn drop(&mut self) {
        // TempDir will automatically clean up
    }
}
