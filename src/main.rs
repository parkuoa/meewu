/*
    main.rs
    SPDX-License-Identifier: MIT
    --=================================--
    Author: parkuoa <parkuoa@gmail.com>
*/

use clap::{Parser, Subcommand};
use anyhow::{Context, Result};
use colored::*;

mod modules;
mod utils;

use modules::installer::MewModInstaller;

use utils::init_meewu;
use utils::is_meewu_setup_done;

#[derive(Parser)]
#[command(name = "meewu")]
#[command(about = "imperative module manager", long_about = None)]
#[command(subcommand_required = false)]

struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum ModRegOpts {
    /// edit the mod registry (sudo for global)
    Edit,
}

#[derive(Subcommand)]
enum Commands {
    /// do first-time directory setup
    Init,
    /// install module.zip
    Install {
        /// path to the module.zip
        #[arg(value_name = "module.zip")]
        module: std::path::PathBuf,
    },
    /// list all installed modules
    List,
    /// uninstall [module]
    Uninstall {
        /// name of module to remove
        #[arg(value_name = "module")]
        name: String,
    },
    /// edit the global/user mod registry
    Registry {
        #[command(subcommand)]
        action: ModRegOpts,
    },
    /// show meewu help
    Help,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Some(Commands::Init) = cli.command {
        init_meewu()?;
        return Ok(());
    }

    if !is_meewu_setup_done() {
        println!("{}", "meewu is not set up!".red().bold());
        println!("Please run 'meewu init'.");
        std::process::exit(1);
    }

    // case: help/noarg
    match cli.command {
    None | Some(Commands::Help) => {
        use clap::CommandFactory;

        let mut cmd = Cli::command();

        cmd.print_help()?;
        Ok(())
    }

        // case: install
        Some(Commands::Install { module }) => {
            if !module.exists() {
                anyhow::bail!("file not found: {}", module.display());
            }
            let installer = MewModInstaller::from_zip(&module)?;
            installer.install()?;
            Ok(())
        }

        // case: list
        Some(Commands::List) => {
            // #task:consider_meewu_path
            /* list installed modules by reading meewu's registry file,
            {meewu_root}/modules.json
            currently ~/.meewu/modules.json
            */
            let meewu_dir = dirs::home_dir().unwrap().join(".meewu");
            let mod_registry = meewu_dir.join("modules.json");
            if mod_registry.exists() {
                let content = std::fs::read_to_string(&mod_registry)?;
                let registry: serde_json::Value = serde_json::from_str(&content)?;
                if let Some(obj) = registry.as_object() {
                    if obj.is_empty() {
                        println!("there are no modules installed..");
                    } else {
                        println!("installed modules:");
                        for (name, info) in obj {
                            let version = info.get("version").and_then(|v| v.as_str()).unwrap_or("unknown");
                            println!("  {} (v{})", name, version);
                        }
                    }
                }
            } else {
                println!("there are no modules installed..");
            }
            Ok(())
        }

        // case: uninstall
        Some(Commands::Uninstall { name }) => {
            // #task:meewu_uninstall
            println!("(hi)");
            Ok(())
        }

        // case: registry
        Some(Commands::Registry { action: ModRegOpts::Edit }) => {
            use modules::paths::{MEEWU_GLOBAL_MOD_REGISTRY, user_mod_registry, current_user};
            
            let is_root = unsafe { libc::getuid() == 0 };
            let mod_registry = if is_root {
                MEEWU_GLOBAL_MOD_REGISTRY.clone()
            } else {
                user_mod_registry(&current_user())
            };
            
            if !mod_registry.exists() {
                anyhow::bail!(
                    "{}",
                    format!("registry not found: {}", mod_registry.display()).red().bold()
                );
            }
            
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "/usr/bin/nano".to_string());

            println!();
            
            let status = std::process::Command::new(&editor)
            .arg(&mod_registry)
            .status()
            .context(format!("failed to launch editor: {}", editor))?;
            
            if !status.success() {
                anyhow::bail!("editor exited with error".red().bold());
            }
            Ok(())
        }
        _ => Ok(()),
    }
}