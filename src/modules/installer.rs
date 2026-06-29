/*
    installer.rs
    SPDX-License-Identifier: MIT
    --=================================--
    Author: parkuoa <parkuoa@gmail.com>
*/

/* NOTE: add SIP disabled check to if self.manifest.metadata.requires_sip_off
you lazy ass

- myself
*/

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{Context, Result};
use tempfile::TempDir;
use zip::ZipArchive;
use std::io::{self, Write};

use crate::modules::manifest::mewModManifest;
use crate::utils::ensure_dir;

pub struct mewModInstaller {
    manifest: mewModManifest,
    module_root: PathBuf,
    temp_dir: TempDir,
}

impl mewModInstaller {
    pub fn from_zip(zip_path: &Path) -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();

        // println!("- Decompressing module"); println!();

        let file = fs::File::open(zip_path)
            .with_context(|| format!("failed to open zip: {}", zip_path.display()))?;
        let mut archive = ZipArchive::new(file)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = temp_path.join(file.name());

            if file.name().ends_with('/') {
                fs::create_dir_all(&outpath)?;
            } else {
                if let Some(parent) = outpath.parent() {
                    fs::create_dir_all(parent)?;
                }
                let mut outfile = fs::File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
        }

        /* look for the root of the module */
        let entries = fs::read_dir(temp_path)?;
        let mut module_root = None;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                /* all modules follow the 'meewu_module/' structure"
                (see https://github.com/parkuoa/meewu/tree/main/examples) */
                let meewu_module = path.join("meewu_module");
                if meewu_module.exists() && meewu_module.is_dir() {
                    module_root = Some(path);
                    break;
                }
            }
        }
        
        let module_root = module_root.context("unable to find module root (meewu_module/)")?;

        /* load the module manifest */
        let manifest_path = module_root.join("meewu_module/module.toml");

        let manifest_content = fs::read_to_string(&manifest_path).with_context(
            || format!("failed to read module.toml at {}", manifest_path.display()))?;

        let manifest: mewModManifest = toml::from_str(&manifest_content).context(
            "failed to parse module.toml")?;

        Ok(Self {
            manifest,
            module_root,
            temp_dir,
        })
    }

    pub fn install(&self) -> Result<()> {
        // Self::clear();
        // print the ascii art
        Self::print_meewu_ascii();
        
        // print module info
        println!();
        println!("[꠹] Installing module: {}", self.manifest.package.name);
        println!("version: {}", self.manifest.package.version);
        println!("author: {}", self.manifest.package.author);
        
        /* for modules that require SIP to be off, first, check if user can install
        (a.k.a. if SIP is disabled), then warn the user */
        if self.manifest.metadata.requires_sip_off {
            println!();
            println!("⚠️  This module requires SIP to be disabled!");
            println!("Proceed with caution. Remember meewu isn't responsible for the module's actions.");
            
            loop {
                println!();
                print!("Are you sure you wish to continue? (y/n): ");
                io::stdout().flush()?;

                let mut input = String::new();
                io::stdin().read_line(&mut input)?;

                let trimmed = input.trim().to_lowercase();

                match trimmed.as_str() {
                    "y" | "yes" => break,
                    "n" | "no" => {
                        println!("exiting...");
                        std::process::exit(0);
                    }
                    _ => {
                        println!("Please enter 'y' or 'n'.");
                    }
                }
            }
        }

        println!();

        /* module root = ${MOD_DIR} / meewu_module */
        // =================================================
        let base = self.module_root.join("meewu_module");
        // =================================================

        // run the installer script if specified
        if let Some(script) = &self.manifest.hooks.install {
            let script_path = base.join(script);
            if !script_path.exists() {
                anyhow::bail!("manifest declared installer: {}, but file not found.", script_path.display());
            }

            // set it as executable
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&script_path)?.permissions();
                perms.set_mode(perms.mode() | 0o111);
                fs::set_permissions(&script_path, perms)?;
            }

            println!("located installer: {}", script_path.display());
            println!("running installer script... ↓");
            println!();

            // the module's self-contained installer script output starts here --
            println!("======================================");
            
            // run the script with MOD_DIR envvar
            let status = Command::new(&script_path)
            .current_dir(&base)
            .env("MOD_DIR", &base)
            .status()
            .context("failed to run module installer script!")?;

            if !status.success() {
                anyhow::bail!("installer script failed, exit code: {}", status);
            }
        } else {
            anyhow::bail!("can't proceed because module doesn't have an installer script (missing install hook)");
        }

        // module's installer script finished running here!!
        println!("======================================");

        // we're done, finish the install
        self.finish_install()?;
        println!();
        println!("[*] Done!");
        Ok(())
    }

    // #task:consider_meewu_path
    /* register mod install to meewu's registry: ~/.meewu/modules.json 
    this path isn't ideal and will be changed. */
    fn finish_install(&self) -> Result<()> {
        let meewu_dir = dirs::home_dir().unwrap().join(".meewu");
        ensure_dir(&meewu_dir)?;

        let mod_registry = meewu_dir.join("modules.json");

        let mut registry: serde_json::Value = if mod_registry.exists() {
            let content = fs::read_to_string(&mod_registry)?;
            serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
        } else {
            serde_json::json!({})
        };
        
        let entry = serde_json::json!({
            "version": self.manifest.package.version,
            "module_root": self.module_root,
        });

        registry[self.manifest.package.name.clone()] = entry;
        fs::write(&mod_registry, serde_json::to_string_pretty(&registry)?)?;
        Ok(())
    }

    pub fn uninstall(&self) -> Result<()> {
        Ok(())
    }

    fn print_meewu_ascii() {
        let art = r#"
  _____   ____   ______  _  ____ __
 /     \_/ __ \_/ __ \ \/ \/ /  |  |
|__|_|  /\___  >\___  >\/\_/ |____/
      \/     \/     \/
"#;
        println!("{}", art);
    }

    fn clear() {
        print!("\x1B[2J\x1B[1;1H");
        let _ = io::stdout().flush();
    }
}