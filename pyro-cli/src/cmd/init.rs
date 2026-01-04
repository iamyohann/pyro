use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn r#impl(name: String) -> Result<()> {
    println!("Initializing new Pyro project: {}", name);
    
    let toml_content = format!(r#"[package]
name = "{}"
version = "0.1.0"

[dependencies]
"#, name);

    let path = Path::new("pyro.mod");
    if path.exists() {
        println!("pyro.mod already exists, skipping creation.");
    } else {
        fs::write(path, toml_content).with_context(|| "Failed to write pyro.mod")?;
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
