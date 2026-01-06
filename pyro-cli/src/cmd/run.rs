use crate::manifest::Manifest;
use crate::util;
use anyhow::{Context, Result};
use pyro_core::interpreter::Interpreter;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn r#impl(file: PathBuf) -> Result<()> {
    // Check for pyro.mod and Rust dependencies
    let manifest = Manifest::load().ok();
    let has_rust_deps = manifest.as_ref()
        .map(|m| m.rust.is_some())
        .unwrap_or(false);

    if has_rust_deps {
        if let Some(m) = manifest {
            run_with_rust_deps(file, m)
        } else {
            // Should be unreachable due to check above, but fallback
            run_interpreter(file)
        }
    } else {
        run_interpreter(file)
    }
}

fn run_interpreter(file: PathBuf) -> Result<()> {
    let mut statements = Vec::new();
    let mut loaded = HashSet::new();
    
    util::process_file(file.clone(), &mut loaded, &mut statements)?;

    let mut interpreter = Interpreter::new();
    match interpreter.run(statements) {
        Ok(_) => Ok(()), 
        Err(e) => Err(anyhow::anyhow!("Runtime error: {:?}", e)),
    }
}

fn run_with_rust_deps(file: PathBuf, manifest: Manifest) -> Result<()> {
    println!("Found native dependencies. Building custom runner...");

    // 1. Determine Build Directory (~/.pyro/rustpkg/<hash>)
    // For simplicity, we just use one global dir `~/.pyro/rustpkg/runner` for now,
    // or we can hash the manifest dependencies to separate builds.
    // Let's use `~/.pyro/rustpkg/current` for iteration speed.
    let home = std::env::var("HOME").context("Could not find HOME directory")?;
    let build_dir = PathBuf::from(home).join(".pyro").join("rustpkg").join("current");
    
    if build_dir.exists() {
        fs::remove_dir_all(&build_dir)?;
    }
    fs::create_dir_all(build_dir.join("src"))?;

    // 2. Parse User Code to find Externs
    let user_code = std::fs::read_to_string(&file)?;
    let tokens = pyro_core::lexer::Lexer::new(&user_code).tokenize();
    // We assume the file is parseable; if not, the runner will fail later anyway.
    // But we need to scan for externs.
    let program = pyro_core::parser::Parser::new(&tokens).parse()
        .map_err(|e| anyhow::anyhow!("Parser error during binding scan: {:?}", e))?;

    let mut extern_funcs = Vec::new();
    for stmt in program.statements {
        if let pyro_core::ast::Stmt::Extern { func_name, params, return_type, .. } = stmt {
            extern_funcs.push((func_name, params, return_type));
        }
    }

    // 3. Generate Cargo.toml
    let mut dependencies = String::new();
    
    // Resolve pyro-core path
    let cw_dir = std::env::current_dir()?;
    let search_paths = vec![
        cw_dir.join("pyro-core"),
        cw_dir.join("../pyro-core"),
        cw_dir.join("../../pyro-core"),
    ];

    let mut pyro_core_dep = r#"pyro-core = "0.1.0""#.to_string();
    for path in search_paths {
        if path.exists() {
             pyro_core_dep = format!("pyro-core = {{ path = {:?} }}", path);
             break;
        }
    }

    if let Some(rust_config) = &manifest.rust {
        for (name, version) in &rust_config.dependencies {
            dependencies.push_str(&format!("{} = \"{}\"\n", name, version));
        }
    }

    let cargo_toml = format!(r#"[package]
name = "pyro_runner"
version = "0.1.0"
edition = "2021"

[workspace]

[dependencies]
{}
{}
tokio = {{ version = "1", features = ["full"] }}
anyhow = "1.0"
"#, pyro_core_dep, dependencies);

    fs::write(build_dir.join("Cargo.toml"), cargo_toml)?;

    // 4. Generate Bindings
    // We generate a `bindings.rs` that registers these functions.
    // NOTE: This assumes the rust function exists in the dependency crates or standard library
    // matching the name! The user is responsible for names matching.
    // Or, for the user example `rand::random`, they might want to alias?
    // The `extern func rand_float` implies `rand_float` exists in Rust.
    // If it comes from a crate like `rand`, they normally can't call `rand::random` directly via simple name.
    // They would need a wrapper.
    // BUT the task is "auto generate native modules".
    // If the user says `extern func my_func`, we need to generate `fn my_func(...) { ... }`?  No.
    // We need to generate the *glue* code that calls the Rust function.
    // But where is the Rust function defined? 
    // If they import a crate, they can't customize the runner code easily without `native.rs`.
    // The prompt says: "Can you auto generate these native modules dynamically and store the rust package in `~/.pyro/rustpkg` so the user doesn't have to worry about all of this?"
    // And refers to the `native.rs` example.
    // It seems the user wants to avoid writing `native.rs`.
    // If so, the Rust code must be INLINE in the pyro file? Or inferred?
    // "auto generate these native modules dynamically"
    // If I use `extern func rand_float()`, how do I know `rand_float` calls `rand::random()`? I don't.
    // Perhaps the user implies they want to write the Rust code IN the generated runner?
    // Or maybe they just want the *boilerplate* of `native.rs` gone?
    // If they strictly want to use *existing* Rust crates, they might need to specify the crate path?
    // `extern "crate:func" func name` ?
    // The AST `Extern` I added has a `func_name`.
    // Let's assume for now purely *shim* generation for *existing* functions provided by crates or `native.rs` is NOT what is asked.
    // Wait, the user showed `native.rs` code and asked to auto-generate *these native modules*.
    // That implies generating the `native.rs` content itself?
    // But `rand_float` logic came from the user.
    // I can't guess `rand::random`.
    // Maybe the user wants to write the Rust implementation *inline* in Pyro? 
    // "native modules dynamically"
    // Let's look at the request again.
    // "Can you auto generate ... so the user doesn't have to worry about all of this?"
    // "All of this" refers to the `native.rs` boilerplate + manual registration.
    // IF the user simply adds `rand="0.8"` and wants to use `rand::random`, they assume mapped bindings?
    // But `rand::random` is generic.
    // Maybe `extern "rand::random" func rand_float() -> Float`.
    // The string I ignored in `parser.rs`!
    // I should use that string to specify the Rust path!
    // `extern "rust_path" func name(...)`
    
    // Let's modify Parser/AST slightly to capture the "rust:..." string if possible?
    // In strict mode I didn't add it.
    // But for now, let's assume the user has a `native.rs` BUT we auto-wire it?
    // No, "auto generate these native modules".
    // If I strictly follow "auto generate", I should likely allow the user to specify the implementation or at least the target.
    // I will assume for this step that I generate the *registration* boilerplate.
    // Creating `native.rs` might still be needed for CUSTOM logic.
    // But for direct crate functions?
    // Let's stick to generating the boilerplate.
    // I will look for `native.rs` as before. If it exists, I use it.
    // If not, maybe I generate a default one?
    // The user's request might effectively be "Handle the boilerplate".
    // Let's start with moving to `~/.pyro/rustpkg`.
    
    // Check for native.rs
    let native_rs_path = file.parent().unwrap_or(Path::new(".")).join("native.rs");
    let has_native = native_rs_path.exists();

    if has_native {
        fs::copy(&native_rs_path, build_dir.join("src/native.rs"))?;
    } else {
        // Create an empty dummy native.rs if none exists, to simplify main.rs logic?
        // Or just handle it in main.rs generation.
    }

    let native_mod = if has_native { "mod native;" } else { "" };
    
    // Auto-generate registration from externs
    let mut auto_registration = String::new();
    if has_native {
        for (name, _, _) in extern_funcs {
            // Assume the rust function has the same name and is in `native` module
            // and signature matches `fn(Vec<Value>) -> Result<Value, Value>`
            auto_registration.push_str(&format!(
                "    interpreter.register_native_function(\"{}\", native::{});\n",
                name, name
            ));
        }
    }

    // Combine manual registration (if any) with auto registration
    // We strictly check for `register` function existence? 
    // The previous code called `native::register`. 
    // If the user wants to use BOTH auto-wire and manual register, we should support it.
    // But how do we know if `native::register` exists without parsing rust?
    // We can try to call it in a `try` block or just append it and let rustc fail if missing?
    // Or, we assume if `extern` is used, we use auto-wire. 
    // If `native.rs` has `register`, we can call it too.
    // For safety, let's just emit the auto-registration code.
    // If the user *also* has a `register` function, they might call it manually?
    // Maybe we simply try to call `native::register` if we didn't find any externs?
    // Or just always try to call it? No, checking existence is hard.
    // Let's assume:
    // If `native.rs` exists:
    //   If `extern`s found: Auto-register them.
    //   Else: Call `native::register`.
    
    let native_reg = if has_native {
        if !auto_registration.is_empty() {
            auto_registration
        } else {
             "native::register(&mut interpreter);".to_string()
        }
    } else {
        String::new()
    };

    let abs_file = fs::canonicalize(&file).unwrap_or(file.clone());

    let main_rs = format!(r#"
use pyro_core::interpreter::Interpreter;
use pyro_core::stdlib;
use std::collections::HashSet;

{}

#[tokio::main]
async fn main() -> anyhow::Result<()> {{
    let mut interpreter = Interpreter::new();
    stdlib::register_std_libs(&mut interpreter);
    
    // Register native modules
    {}
    
    println!("Running with custom native support...");

    // Basic file loader and execution
    let path = std::path::PathBuf::from({:?});
    let content = std::fs::read_to_string(&path)?;
    
    // Use fully qualified names to avoid confusion if user imports things
    let tokens = pyro_core::lexer::Lexer::new(&content).tokenize();
    let program = pyro_core::parser::Parser::new(&tokens).parse()
        .map_err(|e| anyhow::anyhow!("Parser error: {{:?}}", e))?;
    
    interpreter.run(program.statements).map_err(|e| anyhow::anyhow!("Runtime error: {{:?}}", e))?;

    Ok(())
}}
"#, native_mod, native_reg, abs_file);

    fs::write(build_dir.join("src/main.rs"), main_rs)?;

    // 5. Run cargo run
    println!("Compiling and running...");
    let status = Command::new("cargo")
        .arg("run")
        .arg("--release")
        .current_dir(&build_dir)
        .status()
        .context("Failed to run cargo run")?;

    if !status.success() {
        anyhow::bail!("Execution failed");
    }

    Ok(())
}
