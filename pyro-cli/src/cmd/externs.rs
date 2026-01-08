use crate::manifest::Manifest;
use anyhow::{Context, Result};

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use syn::visit::Visit;

pub fn run() -> Result<()> {
    // Default to current directory for CLI usage
    // Or we could try to find pyro.mod and use .externs if we wanted, but let's stick to current dir for explicit command
    // unless we want to match the new behavior.
    // For now, let's just use current dir to match previous behavior
    generate_externs(&std::env::current_dir()?)
}

pub fn generate_externs(output_dir: &Path) -> Result<()> {
    // We assume output_dir is .externs inside the project root where pyro.mod exists
    let project_root = output_dir.parent().context("Invalid output directory")?;
    // println!("Resolving dependencies in {}...", project_root.display());
    
    let manifest = Manifest::resolve_from(project_root).context("No pyro.mod found. Cannot generate externs.")?;
    
    // We need to resolve dependencies to find their source code.
    // We'll create a temporary cargo project similar to how `run` does it.
    
    // 1. Determine Build Directory (~/.pyro/rustpkg/<hash>)
    let home = std::env::var("HOME").context("Could not find HOME directory")?;
    let current_dir = std::env::current_dir()?;
    
    // Hash the project root path to get a unique folder name
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    let abs_root = fs::canonicalize(&current_dir).unwrap_or(current_dir.clone());
    hasher.update(abs_root.to_string_lossy().as_bytes());
    let hash = hex::encode(hasher.finalize());
    
    let build_dir = PathBuf::from(home).join(".pyro").join("rustpkg").join(&hash);
    
    if !build_dir.exists() {
        fs::create_dir_all(build_dir.join("src"))?;
    }

    // 2. Generate Cargo.toml
    let mut dependencies = String::new();
    if let Some(rust_config) = &manifest.rust {
        for (name, version) in &rust_config.dependencies {
            dependencies.push_str(&format!("{} = \"{}\"\n", name, version));
        }
    } else {
        println!("No [rust] dependencies found in pyro.mod");
        return Ok(());
    }

    // We don't strictly need pyro-core for metadata generation usually, but let's include a dummy or real one if needed.
    // Actually, simple metadata check on just dependencies might be enough.
    let cargo_toml = format!(r#"[package]
name = "pyro_extern_gen"
version = "0.1.0"
edition = "2021"

[dependencies]
{}
"#, dependencies);

    fs::write(build_dir.join("Cargo.toml"), cargo_toml)?;
    // Create a dummy main.rs so cargo doesn't complain
    fs::write(build_dir.join("src/main.rs"), "fn main() {}")?;

    println!("Fetching metadata...");
    
    // 3. Run cargo metadata
    let output = Command::new("cargo")
        .arg("metadata")
        .arg("--format-version")
        .arg("1")
        .current_dir(&build_dir)
        .output()
        .context("Failed to run cargo metadata")?;

    if !output.status.success() {
        anyhow::bail!("Failed to resolve dependencies: {}", String::from_utf8_lossy(&output.stderr));
    }

    let metadata: cargo_metadata::Metadata = serde_json::from_slice(&output.stdout)?;

    // Ensure output dir exists
    if !output_dir.exists() {
        fs::create_dir_all(output_dir)?;
    }

    // 4. For each dependency, find the source and parse
    if let Some(rust_config) = &manifest.rust {
        for (dep_name, _) in &rust_config.dependencies {
            // Find package in metadata
            if let Some(pkg) = metadata.packages.iter().find(|p| &p.name == dep_name) {
                // Find lib target
                if let Some(target) = pkg.targets.iter().find(|t| t.kind.contains(&"lib".to_string())) {
                    let src_path = &target.src_path;
                    // println!("Generating externs for {} ({})", dep_name, src_path);
                    
                    if let Ok(content) = fs::read_to_string(src_path) {
                        let ast = syn::parse_file(&content)?;
                        let mut visitor = FunctionVisitor {
                            module_path: dep_name.clone(),
                            externs: Vec::new(),
                        };
                        visitor.visit_file(&ast);
                        
                        // Write to file
                        let output_file = output_dir.join(format!("extern.{}.pyro", dep_name));
                        let output_content = visitor.externs.join("\n");
                        fs::write(&output_file, output_content)?;
                        println!("Created {}", output_file.display());
                    }
                }
            } else {
                println!("Warning: Could not find package {} in metadata", dep_name);
            }
        }
    }

    Ok(())
}

struct FunctionVisitor {
    module_path: String,
    externs: Vec<String>,
}

impl<'ast> Visit<'ast> for FunctionVisitor {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        // Only public functions
        if let syn::Visibility::Public(_) = node.vis {
             // Skip tests
             if node.attrs.iter().any(|attr| attr.path().is_ident("test")) {
                 return;
             }

             let func_name = node.sig.ident.to_string();
             
             // Analyze generics
             // Check if it's strictly `fn foo<T>() -> T` pattern like rand::random
             let mut handled_generic = false;
             if node.sig.generics.params.len() == 1 {
                 if let Some(syn::GenericParam::Type(ty_param)) = node.sig.generics.params.first() {
                      // Check if return type is this generic
                      if let syn::ReturnType::Type(_, ret_ty) = &node.sig.output {
                          if let syn::Type::Path(p) = &**ret_ty {
                              if p.path.is_ident(&ty_param.ident) {
                                   // Pattern match: fn foo<T>() -> T
                                   // Generate variants
                                   handled_generic = true;
                                   
                                   let variants = vec![
                                       ("int", "i64"),
                                       ("float", "f64"), 
                                       ("bool", "bool")
                                   ];

                                   for (pyro_type, rust_type) in variants {
                                        let variant_name = format!("extern_{}_{}_{}", self.module_path, func_name, pyro_type);
                                        // e.g. extern "rand::random::<f64>" def random_float() -> float
                                        let pyro_ret = pyro_type; // same name
                                        self.externs.push(format!(
                                            "extern \"{}::{}::<{}>\" def {}() -> {}", 
                                            self.module_path, func_name, rust_type, variant_name, pyro_ret
                                        ));
                                   }
                              }
                          }
                      }
                 }
             }

             if handled_generic {
                 // Comment out original to skip generic error
                 self.externs.push(format!("// Generic base function {} skipped in favor of specialized variants", func_name));
                 return;
             }

             let mut param_map = std::collections::HashMap::new();
             
             for param in &node.sig.generics.params {
                 if let syn::GenericParam::Type(type_param) = param {
                     // Check bounds
                     for bound in &type_param.bounds {
                         if let Some(ty) = check_bound(bound) {
                             param_map.insert(type_param.ident.to_string(), ty);
                         }
                     }
                 }
             }
             
             if let Some(where_clause) = &node.sig.generics.where_clause {
                 for predicate in &where_clause.predicates {
                     if let syn::WherePredicate::Type(pt) = predicate {
                         if let syn::Type::Path(tp) = &pt.bounded_ty {
                             if let Some(ident) = tp.path.get_ident() {
                                 for bound in &pt.bounds {
                                     if let Some(ty) = check_bound(bound) {
                                         param_map.insert(ident.to_string(), ty);
                                     }
                                 }
                             }
                         }
                     }
                 }
             }

            // Map arguments
            let mut params = Vec::new();
            let mut valid = true;
            
            for input in &node.sig.inputs {
                if let syn::FnArg::Typed(pat_type) = input {
                    if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                        let arg_name = pat_ident.ident.to_string();
                        if let Some(pyro_type) = map_rust_type(&pat_type.ty, &param_map) {
                            params.push(format!("{}: {}", arg_name, pyro_type));
                        } else {
                            valid = false;
                        }
                    }
                } else {
                    valid = false;
                }
            }
            
            // Map return type
            let return_type = match &node.sig.output {
                syn::ReturnType::Default => "void".to_string(),
                syn::ReturnType::Type(_, ty) => {
                    if let Some(t) = map_rust_type(ty, &param_map) {
                        t
                    } else {
                        valid = false;
                        "unknown".to_string()
                    }
                }
            };
            
            if valid {
                let extern_line = format!("extern \"{}::{}\" def extern_{}_{}({}) -> {}", 
                    self.module_path, func_name, self.module_path, func_name, params.join(", "), return_type);
                self.externs.push(extern_line);
            } else {
                // Emit comment
                let extern_line = format!("// extern \"{}::{}\" def extern_{}_{}({}) -> {} // Generic/Unsupported", 
                    self.module_path, func_name, self.module_path, func_name, params.join(", "), return_type);
                self.externs.push(extern_line);
            }
        }
        
        syn::visit::visit_item_fn(self, node);
    }
}

fn check_bound(bound: &syn::TypeParamBound) -> Option<String> {
    if let syn::TypeParamBound::Trait(trait_bound) = bound {
        if let Some(segment) = trait_bound.path.segments.last() {
            let ident = segment.ident.to_string();
            if ident == "AsRef" || ident == "Into" || ident == "Borrow" {
                // Check args
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                   if let Some(syn::GenericArgument::Type(ty)) = args.args.first() {
                       return map_rust_type(ty, &std::collections::HashMap::new());
                   }
                }
            }
        }
    }
    None
}

fn map_rust_type(ty: &syn::Type, generics: &std::collections::HashMap<String, String>) -> Option<String> {
    match ty {
        syn::Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                let ident = segment.ident.to_string();
                if let Some(mapped) = generics.get(&ident) {
                    return Some(mapped.clone());
                }
                match ident.as_str() {
                    "i64" | "i32" | "isize" | "u64" | "u32" | "usize" => Some("int".to_string()),
                    "f64" | "f32" => Some("float".to_string()),
                    "bool" => Some("bool".to_string()),
                    "String" | "str" => Some("string".to_string()),
                    "Vec" => {
                        // Check if Vec<u8>
                        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                           if let Some(syn::GenericArgument::Type(ty)) = args.args.first() {
                               if let syn::Type::Path(p) = ty {
                                   if p.path.is_ident("u8") {
                                        return Some("string".to_string());
                                   }
                               }
                           }
                        }
                        None
                    }
                    _ => None,
                }
            } else {
                None
            }
        },
        syn::Type::Reference(type_ref) => {
             map_rust_type(&type_ref.elem, generics)
        },
        syn::Type::Slice(slice) => {
            // Check for [u8]
             if let syn::Type::Path(p) = &*slice.elem {
                 if p.path.is_ident("u8") {
                      return Some("string".to_string());
                 }
             }
             None
        },
        _ => None,
    }
}
