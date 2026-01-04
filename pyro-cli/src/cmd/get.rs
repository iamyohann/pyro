use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub fn r#impl(url: String) -> Result<()> {
    println!("Getting package: {}", url);

    // Naive URL parsing
    // github.com/user/repo -> ~/.pyro/pkg/github.com/user/repo
    // https://github.com/user/repo -> error or handle?
    // Let's assume the user passes "github.com/user/repo" for now as per Go style.
    
    let home = std::env::var("HOME").context("Could not find HOME directory")?;
    let mut dest = PathBuf::from(home);
    dest.push(".pyro");
    dest.push("pkg");
    
    // Normalize url
    let parts: Vec<&str> = url.split('/').collect();
    if parts.len() < 3 {
        anyhow::bail!("Invalid package path. Expected format: host/user/repo");
    }
    
    for part in &parts {
        dest.push(part);
    }
    
    if dest.exists() {
        println!("Package already exists at {:?}", dest);
        // git pull?
        return Ok(());
    }

    fs::create_dir_all(dest.parent().unwrap())?;

    let git_url = format!("https://{}", url);

    println!("Cloning {} into {:?}", git_url, dest);

    let status = Command::new("git")
        .arg("clone")
        .arg(&git_url)
        .arg(dest.to_str().unwrap())
        .status()
        .context("Failed to execute git clone")?;

    if !status.success() {
        anyhow::bail!("Git clone failed");
    }

    println!("Package installed successfully.");
    Ok(())
}
