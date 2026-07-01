/*
    installer.rs
    SPDX-License-Identifier: MIT
    --=================================--
    Author: parkuoa <parkuoa@gmail.com>
*/

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{Context, Result};
use tempfile::TempDir;
use zip::ZipArchive;
use std::io::{self, Write};
use colored::*;

use crate::modules::paths::
{MEEWU_ROOT, MEEWU_GLOBAL_MOD_REGISTRY, user_mod_registry, current_user};

use crate::modules::paths::*;
use crate::modules::manifest::*;
use crate::utils::{ensure_dir_exists, is_sip_disabled, copy_dir_all, get_meewu_modules_directory};

pub struct MewModInstaller {
    manifest: MewModManifest,
    module_root: PathBuf,
    temp_dir: TempDir,
    pub zip_path: PathBuf,
}

impl MewModInstaller {
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

        let manifest: MewModManifest = toml::from_str(&manifest_content).context(
            "failed to parse module.toml")?;

        /* determine where to copy the module to based on if sudo was used */
        let module_install_path = get_meewu_modules_directory(&manifest.package.name);

        /* if the module -reportedly- already exists, remove it first */
        if module_install_path.exists() {
            fs::remove_dir_all(&module_install_path)?;
        }

        /* copy the module to data/modules */
        copy_dir_all(&module_root, &module_install_path)?;

        Ok(Self {
            manifest,
            module_root: module_install_path,
            temp_dir,
            zip_path: zip_path.to_path_buf(),
        })
    }

    pub fn install(&self) -> Result<()> {
        // Self::clear();
        // print the ascii art
        Self::print_meewu_ascii();

        let is_root = unsafe { libc::getuid() == 0 };

        if !is_root && matches!(
            self.manifest.metadata.r#type,
            MewModType::SystemLevel | MewModType::Kernel
        ) {
            eprintln!("{}", "This module requires root privileges to be installed!".red().bold());
            eprintln!("Try: sudo meewu install {}\n", self.zip_path.display());
            anyhow::bail!("access denied".red().bold());
        }
        
        // print module info
        println!();
        println!("{}{} {}", "[*] module:".white().bold(), "", self.manifest.package.name);
        println!("{} {}", "[*] version:".white().bold(), self.manifest.package.version);
        println!("{} {}", "[*] author:".white().bold(), self.manifest.package.author);
        
        /* for modules that require SIP to be off, first, check if user can install
        (a.k.a. if SIP is disabled), then warn the user */
        if self.manifest.metadata.requires_sip_off {
            if !is_sip_disabled() {
                println!("");
                eprintln!("{}", "can't proceed! This module requires SIP to be disabled!".red().bold());
                eprintln!
                ("{}", "see: \
                https://developer.apple.com/documentation/\
                security/disabling-and-enabling-system-integrity-protection\n".red().bold());
                anyhow::bail!("SIP enabled".red().bold());
            }

            println!();
            println!("{}", "⚠️  This module requires SIP to be disabled!".bright_yellow());
            println!("{}", "Proceed with caution. Remember meewu isn't responsible for the module's actions.".bold());
            
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

            println!("- located installer: {}", script_path.display());
            println!("- running installer script... ↓");
            println!();

            // the module's self-contained installer script output starts here --
            println!("======================================");
            
            // run the script with MOD_DIR envvar
            let status = Command::new(&script_path)
            .current_dir(&base)
            .env("MOD_DIR", &base)
            .status()
            .context("failed to run module installer script!".red().bold())?;

            if !status.success() {
                anyhow::bail!("installer script failed, exit code: {}", status);
            }
        } else {
            anyhow::bail!("can't proceed because module doesn't have an installer script (missing install hook)".red().bold());
        }

        // module's installer script finished running here!!
        println!("======================================");

        // we're done, finish the install
        self.finish_install()?;
        println!();
        println!("[*] Done!");

        if self.manifest.metadata.requires_reboot {
            loop {
                println!();
                print!("This module requires a system reboot. Reboot now? (y/n): ");

                io::stdout().flush()?;

                let mut input = String::new();
                io::stdin().read_line(&mut input)?;

                let trimmed = input.trim().to_lowercase();

                match trimmed.as_str() {
                    "y" | "yes" => {
                        println!("{}", "rebooting..".bright_yellow().bold());
                        std::process::Command::new("/sbin/shutdown")
                        .args(["-r", "now"])
                        .spawn()?;
                    }
                    
                    "n" | "no" => {
                        println!("{}", "[*] Changes will take effect after rebooting..".bright_yellow().bold());
                        std::process::exit(0);
                    }
                    _ => {
                        println!("Please enter 'y' or 'n'.");
                    }
                }
            }
        }
        Ok::<(), anyhow::Error>(())
    }

    fn finish_install(&self) -> Result<()> {
        let is_root = unsafe { libc::getuid() == 0 };

        let mod_registry = if is_root {
            MEEWU_GLOBAL_MOD_REGISTRY.clone()
        } else {
            user_mod_registry(&current_user())
        };

        let meewu_dir = mod_registry.parent().unwrap();
        ensure_dir_exists(&MEEWU_ROOT)?;

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

    fn unregister_mod(&self) -> Result<()> {
        let mod_registry = MEEWU_ROOT.join("modules.json");
        
        if !mod_registry.exists() {
            return Ok(());
        }
        
        let content = fs::read_to_string(&mod_registry)?;
        let mut registry: serde_json::Value = serde_json::from_str(&content)?;
        
        // remove the module by its package name
        if let Some(obj) = registry.as_object_mut() {
            obj.remove(&self.manifest.package.name);
        }
        
        // serialize the new registry back to pretty JSON and save it
        fs::write(&mod_registry, serde_json::to_string_pretty(&registry)?)?;
        Ok(())
    }

    fn print_meewu_ascii() {
        let art = r#"
  _____   ____   ______  _  ____ __ 
 /     \_/ __ \_/ __ \ \/ \/ /  |  \
|  Y Y  \  ___/\  ___/\     /|  |  /
|__|_|  /\___  >\___  >\/\_/ |____/ 
      \/     \/     \/              
"#;
        println!("{}", art.bright_green().bold());
    }

    fn clear() {
        print!("\x1B[2J\x1B[1;1H");
        let _ = io::stdout().flush();
    }
}