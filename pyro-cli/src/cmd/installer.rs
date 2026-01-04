use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use crate::manifest::{Manifest, LockFile, LockPackage};
use sha2::{Sha256, Digest};
use walkdir::WalkDir;

pub fn r#impl() -> Result<()> {
    println!("Installing dependencies...");
    
    // 1. Load manifest
    let manifest = Manifest::load()?;
    
    // 2. Load lockfile (or create empty)
    let mut lockfile = LockFile::load()?;

    // Simple resolution: If lockfile is empty/stale, resolve from manifest.
    // For now, let's just assume we iterate manifest and ensure lockfile matches.
    // Real dependency resolution is complex; we'll implement a basic one:
    // Sync manifest -> lockfile.
    
    // Identify packages in manifest that are not in lockfile or versions differ
    // (For this iteration, we might just regenerate lock entries based on manifest)
    
    let mut new_lock_packages = Vec::new();
    
    for (url, version) in &manifest.dependencies {
        // Check if already in lockfile
        let existing = lockfile.package.iter().find(|p| &p.name == url);
        
        let lock_pkg = if let Some(pkg) = existing {
             // If version matches, keep it. If not, we'd need to update.
             // For simplicity, let's assume if it exists in lock, we trust it, 
             // unless we are forcing update. 
             // BUT, user asked for "Maintain a dependencies file and a dependencies lock file to ensure consistency"
             // So if manifest version differs, we should update.
             if &pkg.version == version {
                 pkg.clone()
             } else {
                 resolve_package(url, version)?
             }
        } else {
            resolve_package(url, version)?
        };
        
        // Install the package
        install_package(&lock_pkg)?;
        new_lock_packages.push(lock_pkg);
    }
    
    lockfile.package = new_lock_packages;
    lockfile.save()?;
    
    println!("Dependencies installed.");
    Ok(())
}

pub fn resolve_package(url: &str, version: &str) -> Result<LockPackage> {
    println!("Resolving {}@{}", url, version);
    // 1. Clone to temp/cache to get checksum and latest commit for 'version'
    // This is expensive. In Go modules, there's a proxy. Here we might just clone to ~/.pyro/cache first?
    // Let's reuse existing logic: clone to ~/.pyro/pkg directly, checkout version, then checksum.
    
    let home = std::env::var("HOME").context("Could not find HOME directory")?;
    let pkg_root = PathBuf::from(home).join(".pyro/pkg");
    
    let mut dest = pkg_root.clone();
    for part in url.split('/') {
        dest.push(part);
    }
    
    if !dest.exists() {
         let git_url = if url.contains("://") {
             url.to_string()
         } else {
             format!("https://{}", url)
         };
         
         fs::create_dir_all(dest.parent().unwrap())?;
         let status = Command::new("git")
            .arg("clone")
            .arg(&git_url)
            .arg(dest.to_str().unwrap())
            .status()
            .context("Failed to git clone")?;
            
         if !status.success() {
             anyhow::bail!("Failed to clone {}", url);
         }
    }
    
    // Checkout version
    // If version is "latest" or "HEAD", we might pull.
    // Ideally version is a semver tag or commit hash.
    // For now allow simple tags/branches.
    
    let status = Command::new("git")
        .current_dir(&dest)
        .arg("checkout")
        .arg(version)
        .status()?;
        
    if !status.success() {
        // try fetching?
        Command::new("git").current_dir(&dest).arg("fetch").status()?;
        let status = Command::new("git").current_dir(&dest).arg("checkout").arg(version).status()?;
        if !status.success() {
             anyhow::bail!("Failed to checkout version {} for {}", version, url);
        }
    }
    
    // Get Commit Hash
    let output = Command::new("git")
        .current_dir(&dest)
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .context("Failed to get commit hash")?;
    let commit = String::from_utf8(output.stdout)?.trim().to_string();

    // Calculate Checksum
    let checksum = calculate_dir_checksum(&dest)?;
    
    Ok(LockPackage {
        name: url.to_string(),
        version: version.to_string(),
        source: format!("https://{}", url),
        commit: Some(commit),
        checksum,
        dependencies: None, // We are not recursive yet in this step, but we will need to be eventually.
    })
}

fn install_package(pkg: &LockPackage) -> Result<()> {
    let home = std::env::var("HOME").context("Could not find HOME directory")?;
    let mut dest = PathBuf::from(home).join(".pyro/pkg");
    for part in pkg.name.split('/') {
        dest.push(part);
    }
    
    if !dest.exists() {
        // clone logic duplicated, refactor later
        let git_url = &pkg.source;
        fs::create_dir_all(dest.parent().unwrap())?;
         let status = Command::new("git")
            .arg("clone")
            .arg(git_url)
            .arg(dest.to_str().unwrap())
            .status()?;
        if !status.success() { anyhow::bail!("Clone failed"); }
    }
    
    // Ensure correct version
    // If locked, we want to be sure.
    // We already resolved it above if we called resolve. If we came from lockfile, we might need to checkout.
    let target_ref = pkg.commit.as_ref().unwrap_or(&pkg.version);

     let status = Command::new("git")
        .current_dir(&dest)
        .arg("checkout")
        .arg(target_ref)
        .status()?;
            
    if !status.success() {
        // Maybe fetch?
         Command::new("git").current_dir(&dest).arg("fetch").status()?;
         let status = Command::new("git").current_dir(&dest).arg("checkout").arg(target_ref).status()?;
         if !status.success() { anyhow::bail!("Checkout failed for locked version {}", target_ref); }
    }
    
    // Verify checksum
    let current_checksum = calculate_dir_checksum(&dest)?;
    if current_checksum != pkg.checksum {
        anyhow::bail!("Checksum mismatch for package {}! Lockfile says {}, found {}", pkg.name, pkg.checksum, current_checksum);
    }
    
    Ok(())
}

pub fn calculate_dir_checksum(path: &Path) -> Result<String> {
    let mut hasher = Sha256::new();
    
    for entry in WalkDir::new(path).sort_by_file_name() {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            if path.file_name().unwrap() == ".git" {
                continue; // Skip .git directory
            }
             continue;
        }
        
        // skip .git files if walkdir doesn't skip dir children when skipping dir
        if path.components().any(|c| c.as_os_str() == ".git") {
            continue;
        }

        let content = fs::read(path)?;
        hasher.update(&content);
        hasher.update(path.to_string_lossy().as_bytes()); // Include filename in hash
    }
    
    let result = hasher.finalize();
    Ok(hex::encode(result))
}
