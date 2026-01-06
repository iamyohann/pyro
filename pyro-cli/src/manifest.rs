use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::fs;
use anyhow::{Result, Context};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Manifest {
    pub package: Package,
    #[serde(default)]
    pub dependencies: HashMap<String, String>, 
    #[serde(default)]
    pub rust: Option<RustConfig>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RustConfig {
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Package {
    pub name: String,
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LockFile {
    #[serde(default)]
    pub package: Vec<LockPackage>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LockPackage {
    pub name: String,
    pub version: String,
    pub source: String,
    pub commit: Option<String>,
    pub checksum: String,
    pub dependencies: Option<Vec<String>>,
}

impl Manifest {
    pub fn new(name: String) -> Self {
        Self {
            package: Package {
                name,
                version: "0.1.0".to_string(),
            },
            dependencies: HashMap::new(),
            rust: None,
        }
    }

    pub fn load() -> Result<Self> {
        let path = Path::new("pyro.mod");
        if !path.exists() {
            anyhow::bail!("pyro.mod not found");
        }
        let content = fs::read_to_string(path).context("Failed to read pyro.mod")?;
        let manifest: Manifest = toml::from_str(&content).context("Failed to parse pyro.mod")?;
        Ok(manifest)
    }

    pub fn save(&self) -> Result<()> {
        let content = toml::to_string_pretty(self).context("Failed to serialize pyro.mod")?;
        fs::write("pyro.mod", content).context("Failed to write pyro.mod")?;
        Ok(())
    }
}

impl LockFile {
    pub fn load() -> Result<Self> {
        let path = Path::new("pyro.lock");
        if !path.exists() {
            return Ok(LockFile { package: vec![] });
        }
        let content = fs::read_to_string(path).context("Failed to read pyro.lock")?;
        let lock: LockFile = toml::from_str(&content).context("Failed to parse pyro.lock")?;
        Ok(lock)
    }

    pub fn save(&self) -> Result<()> {
        let content = toml::to_string_pretty(self).context("Failed to serialize pyro.lock")?;
        fs::write("pyro.lock", content).context("Failed to write pyro.lock")?;
        Ok(())
    }
}
