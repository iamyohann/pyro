use anyhow::{Context, Result};
use crate::manifest::{Manifest, LockFile};
use crate::cmd::installer::resolve_package;

pub fn r#impl(url: String) -> Result<()> {
    
    // Parse url for version: url@version
    let (clean_url, version) = if let Some((u, v)) = url.split_once('@') {
        (u.to_string(), v.to_string())
    } else {
        (url, "HEAD".to_string())
    };

    println!("Getting package: {} version: {}", clean_url, version);

    // 1. Load Manifest
    let mut manifest = Manifest::load().context("Could not find pyro.mod. Run 'pyro mod init' first.")?;
    
    if manifest.dependencies.contains_key(&clean_url) {
        println!("Package {} already in pyro.mod", clean_url);
        // We could update version here if different
    }

    // 2. Resolve package (Download, Checkout, Checksum)
    let lock_pkg = resolve_package(&clean_url, &version)?;
    
    // 3. Update Manifest
    manifest.dependencies.insert(clean_url.clone(), version.clone()); // store the requested version (e.g. HEAD or v1.0)
    manifest.save()?;
    
    // 4. Update Lockfile
    let mut lockfile = LockFile::load()?;
    
    // Remove existing entry if any
    if let Some(pos) = lockfile.package.iter().position(|p| p.name == clean_url) {
        lockfile.package.remove(pos);
    }
    
    lockfile.package.push(lock_pkg);
    lockfile.save()?;
    
    println!("Package {} added.", clean_url);
    Ok(())
}
