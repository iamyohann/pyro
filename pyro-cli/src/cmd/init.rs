use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn r#impl(name: String) -> Result<()> {
    println!("Initializing new Pyro project: {}", name);
    
    let path = Path::new("pyro.mod");
    if path.exists() {
        println!("pyro.mod already exists, skipping creation.");
    } else {
        let manifest = crate::manifest::Manifest::new(name);
        manifest.save().context("Failed to save pyro.mod")?;
        println!("Created pyro.mod");
    }

    // Create src directory
    fs::create_dir_all("src")?;
    
    // Create main.pyro
    let main_path = Path::new("src/main.pyro");
    if !main_path.exists() {
        fs::write(main_path, r#"def main():
    print("Hello, Pyro!")

main()
"#)?;
        println!("Created src/main.pyro");
    }

    Ok(())
}
