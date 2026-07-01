/*
    utils.rs
    SPDX-License-Identifier: MIT
    --=================================--
    Author: parkuoa <parkuoa@gmail.com>
*/

use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use walkdir::WalkDir;
use crate::modules::paths::*;
use colored::*;

/// Recursively copy directory
pub fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in WalkDir::new(src).into_iter().filter_map(|e| e.ok()) {
        let rel = entry.path().strip_prefix(src)?;
        let dest = dst.join(rel);
        
        if entry.file_type().is_dir() {
            fs::create_dir_all(&dest)?;
        } else {
            fs::copy(entry.path(), &dest)?;
        }
    }
    Ok(())
}

/// Check if we have R/W permission to path
pub fn can_write(path: &Path) -> bool {
    if path.exists() {
        fs::metadata(path)
            .map(|m| m.permissions().readonly() == false)
            .unwrap_or(false)
    } else {
        // try to create the parent directory
        if let Some(parent) = path.parent() {
            can_write(parent)
        } else {
            false
        }
    }
}

/// Ensure directory exists, if not, create it
pub fn ensure_dir_exists(path: &Path) -> Result<()> {
    fs::create_dir_all(path)
        .with_context(|| format!("failed to create directory: {}", path.display()))
}

/// Copy a file following permission checks
pub fn copy_file(src: &Path, dst: &Path) -> Result<()> {
    if !can_write(dst) {
        anyhow::bail!(
            // we don't have permission to write to the dest path
            "no write permission for destination: {}. Try running with sudo?",
            dst.display()
        );
    }
    if let Some(parent) = dst.parent() {
        ensure_dir_exists(parent)?;
    }
    fs::copy(src, dst)
        .with_context(|| format!("failed to copy {} to {}", src.display(), dst.display()))?;
    Ok(())
}

/// Get the running user's home path
pub fn home_dir() -> Result<PathBuf> {
    dirs::home_dir().context("failed to get home directory..")
}

/// Stupid fn to expand tilde prefixed (home) path
pub fn expand_home_dir(path: &Path) -> PathBuf {
    let s = path.to_string_lossy();
    if s.starts_with("~") {
        let home = home_dir().unwrap_or_else(|_| PathBuf::from("/"));
        let rest = s.strip_prefix("~").unwrap_or("");
        home.join(rest.trim_start_matches('/'))
    } else {
        path.to_path_buf()
    }
}

/// Check if System Integrity Protection (SIP) is disabled
pub fn is_sip_disabled() -> bool {
    /* run csrutil status */
    // missing unknown
    let output = std::process::Command::new("/usr/bin/csrutil")
        .arg("status")
        .output();
    
    match output {
        Ok(output) => {
            /* outputs:
            System Integrity Protection status: disabled.
            System Integrity Protection status: enabled.
            */
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.contains("disabled")
        }
        Err(_) => {
            false
        }
    }
}

/// Run meewu's first-time directory setup
///
/// This creates the following files/directories:
///
/// `/opt/meewu`  
/// `/opt/meewu/bin`  
/// `/opt/meewu/data/modules`  
/// `/opt/meewu/modules.json`  
///
/// User-specific:  
/// `/opt/meewu/u/{username}`  
/// `/opt/meewu/u/{username}/bin`  
/// `/opt/meewu/u/{username}/data/modules`  
/// `/opt/meewu/u/{username}/modules.json`  
pub fn init_meewu() -> Result<()> {
    let username = current_user();
    
    /* check for /opt/meewu */
    if !MEEWU_ROOT.exists() {
        println!("meewu is setting base at {}", MEEWU_ROOT.display());
        println!("Some directories will be created(note that this requires sudo)");

        /* meewu follows the same permission scheme for all files/directories globally:
        770 = owner(7, r/w/x) | group(7, r/w/x) | others(0)

        owner = running user (the user who ran meewu init)
        group = admin

        That means users can't interact with meewu's files outside of their user directory
        without sudo/root access, and therefore can't install 
        */
        
        /* try to create meewu's struct with sudo */
        let status = std::process::Command::new("sudo")
            .args([
                "/bin/mkdir", "-p",
                "/opt/meewu/bin",
                "/opt/meewu/data",
            ])
            .status()
            .context("failed to create /opt/meewu. Did you type your password correctly?")?;
        
        // ?
        if !status.success() {
            anyhow::bail!("failed to create /opt/meewu!".red().bold());
        }
        
        /* create the global mod registry -- {MEEWU_ROOT}/modules.json */
        let status = std::process::Command::new("sudo")
            .args([
                "/usr/bin/touch",
                "/opt/meewu/modules.json"
            ])
            .status()?;
        
        if !status.success() {
            anyhow::bail!("failed to create mod registry!".red().bold());
        }
        
        /* set permissions for global meewu directories
        Owner here is root and group is admin, so keep that in mind.
        */
        let status = std::process::Command::new("sudo")
            .args([
                "/bin/chmod", "-R", "770",
                "/opt/meewu/data",
                "/opt/meewu/bin",
            ])
            .status()?;
        
        if !status.success() {
            anyhow::bail!("Failed to set permissions on /opt/meewu");
        }
        
        /* set ownership for global files and directories to root:admin */
        let status = std::process::Command::new("sudo")
            .args([
                "/usr/sbin/chown", "-R", "root:admin",
                "/opt/meewu/data",
                "/opt/meewu/bin",
                "/opt/meewu/modules.json",
            ])
            .status()?;
        
        if !status.success() {
            anyhow::bail!("Failed to set ownership on /opt/meewu");
        }
    }
    
    /* create running user's directory */
    let user_data_dir = user_data_dir(&username);
    if !user_data_dir.exists() {
        println!("creating user directory for {}...", username);
        
        let status = std::process::Command::new("sudo")
            .args([
                "/bin/mkdir", "-p",
                // {MEEWU_ROOT}/u/user_dir/data
                &user_data_dir.to_string_lossy(),
                // {MEEWU_ROOT}/u/user_dir/bin
                &user_dir(&username).join("bin").to_string_lossy(),
            ])
            .status()?;
        
        if !status.success() {
            anyhow::bail!("failed to create user directory!".red().bold());
        }
        
        // {MEEWU_ROOT}/u/user_dir/modules.json
        let status = std::process::Command::new("sudo")
            .args([
                "/usr/bin/touch",
                &user_dir(&username).join("modules.json").to_string_lossy(),
            ])
            .status()?;
        
        if !status.success() {
            anyhow::bail!("failed to create user module registry!".red().bold());
        }
        
        /* give running user ownership of their files */
        let status = std::process::Command::new("sudo")
            .args([
                "/usr/sbin/chown", "-R", &format!("{}:admin", username),
                &user_dir(&username).to_string_lossy(),
            ])
            .status()?;
        
        if !status.success() {
            anyhow::bail!("failed to give user directory ownership!".red().bold());
        }
        
        // set permissions
        let status = std::process::Command::new("sudo")
            .args([
                "/bin/chmod", "-R", "770",
                &user_data_dir.to_string_lossy(),
            ])
            .status()?;
        
        if !status.success() {
            anyhow::bail!("Failed to set user directory permissions");
        }
    }
    
    println!("[*] Done");
    Ok(())
}

/*
pub fn is_meewu_setup_done() -> bool {
    MEEWU_ROOT.exists() && 
    MEEWU_GLOBAL_MOD_REGISTRY.exists() && 
    user_dir(&current_user()).exists()
}
*/

/// Check if meewu first-time setup was done
pub fn is_meewu_setup_done() -> bool {
    /* check if we're root */
    let is_root = unsafe { libc::getuid() == 0 };
    
    // global paths (for system modules)
    let global_initialized = MEEWU_ROOT.exists() && MEEWU_GLOBAL_MOD_REGISTRY.exists();
    
    // per-user paths (for user-level modules)
    let username = current_user();
    let user_initialized = user_dir(&username).exists() && 
                           user_mod_registry(&username).exists();
    
    if is_root {
        global_initialized
    } else {
        global_initialized && user_initialized
    }
}

pub fn get_meewu_modules_directory(module_name: &str) -> PathBuf {
    let is_root = unsafe { libc::getuid() == 0 };
    if is_root {
        // if mod is global(?): /opt/meewu/data/modules/{mod}
        MEEWU_GLOBAL_MOD_PATH.join(module_name)
    } else {
        // if it's user-space: /opt/meewu/u/{user}/data/modules/{mod}
        user_modules_dir(&current_user()).join(module_name)
    }
}