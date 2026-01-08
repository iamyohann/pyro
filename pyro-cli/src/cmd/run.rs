use crate::manifest::Manifest;
use crate::util;
use anyhow::{Context, Result};
use pyro_core::interpreter::Interpreter;
use sha2::{Sha256, Digest};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn r#impl(file: PathBuf) -> Result<()> {
    // Check for pyro.mod and Rust dependencies
    // Check for pyro.mod and Rust dependencies
    let manifest = Manifest::resolve_from(file.parent().unwrap_or(Path::new(".")))
        .or_else(|_| Manifest::load())
        .ok();
    let has_native_deps = manifest.as_ref()
        .map(|m| m.rust.is_some())
        .unwrap_or(false);

    if has_native_deps {
        if let Some(m) = manifest {
            // Generate externs relative to pyro.mod
            let parent = file.parent().unwrap_or(Path::new("."));
            let search_path = if parent.as_os_str().is_empty() { Path::new(".") } else { parent };

            if let Ok(_manifest_path) = std::fs::canonicalize(search_path) {
                 // Ideally manifest should tell us its root, but we can infer from where we found it or just use current dir if we loaded it from there.
                 // Manifest::load uses current dir or recursive parent. Manifest::resolve_from uses file parent. 
                 // Let's re-resolve correctly to find root.
                 let mut current = search_path.to_path_buf();
                 loop {
                     if current.join("pyro.mod").exists() {
                         let externs_dir = current.join(".externs");
                         if let Err(e) = crate::cmd::externs::generate_externs(&externs_dir) {
                             eprintln!("Warning: Failed to generate externs: {}", e);
                         }
                         break;
                     }
                     if !current.pop() { break; }
                 }
            }
            run_with_rust_deps(file, m)
        } else {
            // Should be unreachable due to check above, but fallback
            run_interpreter(file)
        }
    } else {
         // Also check for extern generation even if no rust deps? No, only if pyro.mod exists
         // But wait, if has_native_deps is false, maybe we still want to generate if rust section exists but is empty? 
         // manifest.rust.is_some() checks this.
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
    let home = std::env::var("HOME").context("Could not find HOME directory")?;
    
    // Hash the project root path to get a unique folder name
    let project_root = file.parent().unwrap_or(Path::new("."));
    let mut hasher = Sha256::new();
    let abs_root = fs::canonicalize(project_root).unwrap_or(project_root.to_path_buf());
    hasher.update(abs_root.to_string_lossy().as_bytes());
    let hash = hex::encode(hasher.finalize());
    
    let build_dir = PathBuf::from(home).join(".pyro").join("rustpkg").join(&hash);
    
    if !build_dir.exists() {
        fs::create_dir_all(build_dir.join("src"))?;
    }


    // 2. Parse User Code to find Externs (recursively)
    let mut statements = Vec::new();
    let mut loaded = std::collections::HashSet::new();
    
    // We utilize the updated process_file which handles .externs resolution
    crate::util::process_file(file.clone(), &mut loaded, &mut statements)?;

    let mut extern_funcs = Vec::new();
    for stmt in statements {
        if let pyro_core::ast::Stmt::Extern { func_name, params, return_type, rust_path, .. } = stmt {
            extern_funcs.push((func_name, params, return_type, rust_path));
        }
    }

    // 3. Generate Cargo.toml
    let mut dependencies = String::new();
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

    // 4. Generate Bindings (native_auto.rs)
    let native_rs_path = file.parent().unwrap_or(Path::new(".")).join("native.rs");
    let has_native = native_rs_path.exists();

    if has_native {
        fs::copy(&native_rs_path, build_dir.join("src/native.rs"))?;
    }

    let mut auto_wrappers = String::new();
    let mut auto_registration = String::new();

    auto_wrappers.push_str("use pyro_core::interpreter::Value;\n");
    auto_wrappers.push_str("use anyhow::Result;\n\n");

    for (name, params, return_type, rust_path) in extern_funcs {
        if let Some(rust_func_path) = rust_path {
            // Generate auto-wrapper
            let wrapper_name = format!("wrapper_{}", name);
            auto_wrappers.push_str(&format!("pub fn {}(args: Vec<Value>) -> Result<Value, Value> {{\n", wrapper_name));
            
            // Argument parsing
            if !params.is_empty() {
                auto_wrappers.push_str("    let mut args = args.into_iter();\n");
            } else {
                auto_wrappers.push_str("    let _ = args;\n");
            }
            
            let mut rust_args = Vec::new();

            for (i, (param_name, param_type)) in params.iter().enumerate() {
                 let arg_var = format!("arg_{}", i);
                 auto_wrappers.push_str(&format!("    let {} = args.next().ok_or(Value::String(std::sync::Arc::new(\"Missing argument '{}'\".to_string())))?;\n", arg_var, param_name));
                 
                 // Type check and convert
                 // Only implementing basic types for now
                 let (_type_check, _type_cast) = match param_type {
                     pyro_core::ast::Type::Int => ("matches!(val, Value::Int(_))", "if let Value::Int(i) = val { i } else { unreachable!() }"),
                     pyro_core::ast::Type::Float => ("matches!(val, Value::Float(_))", "if let Value::Float(f) = val { f } else { unreachable!() }"),
                     pyro_core::ast::Type::Bool => ("matches!(val, Value::Bool(_))", "if let Value::Bool(b) = val { b } else { unreachable!() }"),
                     pyro_core::ast::Type::String => ("matches!(val, Value::String(_))", "if let Value::String(s) = val { s } else { unreachable!() }"),
                     _ => ("true", "val"), // Pass Value mostly as is or fail?
                 };

                 // For now, let's assume direct cast via if check
                 match param_type {
                     pyro_core::ast::Type::Int => {
                         auto_wrappers.push_str(&format!("    let {} = if let Value::Int(i) = {} {{ i }} else {{ return Err(Value::String(\"Expected int for argument '{}'\".to_string().into())); }};\n", arg_var, arg_var, param_name));
                         rust_args.push(arg_var);
                     },
                     pyro_core::ast::Type::Float => {
                         auto_wrappers.push_str(&format!("    let {} = if let Value::Float(f) = {} {{ f }} else {{ return Err(Value::String(\"Expected float for argument '{}'\".to_string().into())); }};\n", arg_var, arg_var, param_name));
                         rust_args.push(arg_var);
                     },
                     pyro_core::ast::Type::Bool => {
                         auto_wrappers.push_str(&format!("    let {} = if let Value::Bool(b) = {} {{ b }} else {{ return Err(Value::String(\"Expected bool for argument '{}'\".to_string().into())); }};\n", arg_var, arg_var, param_name));
                         rust_args.push(arg_var);
                     },
                     pyro_core::ast::Type::String => {
                         auto_wrappers.push_str(&format!("    let {} = if let Value::String(s) = {} {{ s.to_string() }} else {{ return Err(Value::String(\"Expected string for argument '{}'\".to_string().into())); }};\n", arg_var, arg_var, param_name));
                         rust_args.push(arg_var);
                     },
                     _ => {
                         // Pass raw Value
                         rust_args.push(arg_var);
                     }
                 }
            }
            
             // Call Rust function
            let args_str = rust_args.join(", ");
            
            // Determine Rust return type for annotation
            let rust_ret_type = match return_type {
                pyro_core::ast::Type::Int => "i64",
                pyro_core::ast::Type::Float => "f64",
                pyro_core::ast::Type::Bool => "bool",
                pyro_core::ast::Type::String => "String",
                pyro_core::ast::Type::Void => "()",
                _ => "_",
            };

            auto_wrappers.push_str(&format!("    let result: {} = ::{}({});\n", rust_ret_type, rust_func_path, args_str));

             match return_type {
                 pyro_core::ast::Type::Int => auto_wrappers.push_str("    Ok(Value::Int(result))\n"),
                 pyro_core::ast::Type::Float => {
                      auto_wrappers.push_str("    Ok(Value::Float(result as f64))\n")
                 },
                 pyro_core::ast::Type::Bool => auto_wrappers.push_str("    Ok(Value::Bool(result))\n"),
                 pyro_core::ast::Type::String => auto_wrappers.push_str("    Ok(Value::String(result.into()))\n"), // Fixed: Wrap in Arc (via From/Into)
                 pyro_core::ast::Type::Void => auto_wrappers.push_str("    Ok(Value::Bool(true)) // Void -> True\n"),
                 _ => auto_wrappers.push_str("    Ok(result) // Assume Value\n"),
            }

            auto_wrappers.push_str("}\n\n");

            // Register
            auto_registration.push_str(&format!(
                "    interpreter.register_native_function(\"{}\", native_auto::{});\n",
                name, wrapper_name
            ));

        } else if has_native {
            // Old behavior: assume function is in `native::`
            auto_registration.push_str(&format!(
                "    interpreter.register_native_function(\"{}\", native::{});\n",
                name, name
            ));
        }
    }
    
    fs::write(build_dir.join("src/native_auto.rs"), auto_wrappers)?;

    let native_mod = if has_native { "mod native;" } else { "" };
    
    // Combining manual `native::register` call if exists with auto-registration is tricky.
    // If strict auto-gen is used, we use `auto_registration`.
    // If `native.rs` exists but no externs have paths, we rely on old behavior.
    
    let native_reg = if !auto_registration.is_empty() {
        auto_registration
    } else if has_native {
         "native::register(&mut interpreter);".to_string()
    } else {
        String::new()
    };

    let abs_file = fs::canonicalize(&file).unwrap_or(file.clone());

    let main_rs = format!(r#"
use pyro_core::interpreter::Interpreter;
use pyro_core::stdlib;

{}
mod native_auto;

#[tokio::main]
async fn main() -> anyhow::Result<()> {{
    let mut interpreter = Interpreter::new();
    stdlib::register_std_libs(&mut interpreter);
    
    // Register native modules
    {}
    
    println!("Running with custom native support...");

    let path = std::path::PathBuf::from({:?});
    let content = std::fs::read_to_string(&path)?;
    
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
