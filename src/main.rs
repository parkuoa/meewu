/*
    main.rs
    SPDX-License-Identifier: MIT
    --=================================--
    Author: parkuoa <parkuoa@gmail.com>
*/

use clap::{Parser, Subcommand};
use anyhow::Result;
use colored::*;

mod modules;
mod utils;

use modules::installer::mewModInstaller;

use utils::init_meewu;
use utils::is_meewu_setup_done;

#[derive(Parser)]
#[command(name = "meewu")]
#[command(about = "imperative module manager", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    //// Do first-time directory setup
    Init,
    /* install module.zip */
    Install {
        /* path to the module.zip */
        #[arg(value_name = "module.zip")]
        module: std::path::PathBuf,
    },
    /* list all installed modules */
    List,
    /* uninstall [module] */
    Uninstall {
        /* name of module to remove */
        #[arg(value_name = "module")]
        name: String,
    },
    /* show meewu help */
    Help,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Some(Commands::Init) = cli.command {
        init_meewu()?;
        return Ok(());
    }

    if !is_meewu_setup_done() {
        println!("{}", "meewu is not initialized!".red().bold());
        println!("Please run 'meewu init'.");
        std::process::exit(1);
    }

    // case: help/noarg
    match cli.command {
        None | Some(Commands::Help) => {
            println!("⊹ meewu");
            println!("imperative module manager");
            println!();
            println!("Usage:");
            println!("  meewu install [module.zip]   install given module");
            println!("  meewu list                   list installed modules");
            //println!("  meewu uninstall [module]     remove a module");
            println!("  meewu help                   Show this help");
            Ok(())
        }
        // case: install
        Some(Commands::Install { module }) => {
            if !module.exists() {
                anyhow::bail!("file not found: {}", module.display());
            }
            let installer = mewModInstaller::from_zip(&module)?;
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
        _ => Ok(()),
    }
}