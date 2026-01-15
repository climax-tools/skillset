use crate::error::Result;
use crate::config::skillset::RegistryConfig;
use reqwest::Client;

pub struct OciClient {
    client: Client,
    registry_url: String,
    auth_config: Option<RegistryConfig>,
}

impl OciClient {
    pub fn new(registry_url: &str, auth_config: Option<RegistryConfig>) -> Result<Self> {
        Ok(Self {
            client: Client::new(),
            registry_url: registry_url.to_string(),
            auth_config,
        })
    }
    
    pub async fn pull_manifest(&self, reference: &str) -> Result<String> {
        // Parse reference to get repository and tag
        let (repo, tag) = self.parse_reference(reference)?;
        
        // Construct the manifest URL
        let manifest_url = format!("{}/v2/{}/manifests/{}", self.registry_url, repo, tag);
        
        // TODO: Implement OCI authentication and manifest fetching
        println!("Would fetch manifest from: {}", manifest_url);
        Ok("manifest".to_string())
    }
    
    pub async fn pull_blob(&self, digest: &str) -> Result<Vec<u8>> {
        // TODO: Implement blob downloading
        todo!("Implement OCI blob downloading")
    }
    
    fn parse_reference(&self, reference: &str) -> Result<(String, String)> {
        // Parse "repository:tag" format
        let parts: Vec<&str> = reference.split(':').collect();
        if parts.len() != 2 {
            return Err(crate::error::SkillsetError::Oci(format!("Invalid reference format: {}", reference)));
        }
        
        Ok((parts[0].to_string(), parts[1].to_string()))
    }
}