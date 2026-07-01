/*
    paths.rs
    SPDX-License-Identifier: MIT
    --=================================--
    Author: parkuoa <parkuoa@gmail.com>
*/

use once_cell::sync::Lazy;
use std::process::Command;
use std::path::PathBuf;
use std::fs;

/// Base directory for meewu
///
/// e.g. /opt/meewu
pub static MEEWU_ROOT: Lazy<PathBuf> = Lazy::new(|| {
    PathBuf::from("/opt/meewu")
});

/// Global module registry path
///
/// e.g. /opt/meewu/modules.json
pub static MEEWU_GLOBAL_MOD_REGISTRY: Lazy<PathBuf> = Lazy::new(|| {
    MEEWU_ROOT.join("modules.json")
});

/// Directory for global data
///
/// e.g. /opt/meewu/data
pub static MEEWU_GLOBAL_DATA_DIR: Lazy<PathBuf> = Lazy::new(|| {
    MEEWU_ROOT.join("data")
});

/// Directory for global modules
///
/// e.g. /opt/meewu/data/modules
pub static MEEWU_GLOBAL_MOD_PATH: Lazy<PathBuf> = Lazy::new(|| {
    MEEWU_GLOBAL_DATA_DIR.join("modules")
});

/// Directory for meewu binaries/utilities
///
/// e.g. /opt/meewu/bin
pub static MEEWU_BIN: Lazy<PathBuf> = Lazy::new(|| {
    MEEWU_ROOT.join("bin")
});


/// Get user's base directory
///
/// e.g. /opt/meewu/u/alex
pub fn user_dir(username: &str) -> PathBuf {
    MEEWU_ROOT.join("u").join(username)
}


/// Get user's `data` directory
///
/// e.g. /opt/meewu/u/alex/data
pub fn user_data_dir(username: &str) -> PathBuf {
    user_dir(username).join("data")
}


/// Get user's `modules` directory
///
/// e.g. /opt/meewu/u/alex/data/modules
pub fn user_modules_dir(username: &str) -> PathBuf {
    user_data_dir(username).join("modules")
}


/// Get path to user's mod registry
/// `MEEWU_ROOT/u/user_dir/modules.json`
///
/// e.g. /opt/meewu/u/alex/data/modules.json
pub fn user_mod_registry(username: &str) -> PathBuf {
    user_dir(username).join("modules.json")
}


/// Get current/running user
pub fn current_user() -> String {
    let output = Command::new("/usr/bin/id")
        .arg("-un")
        .output();
    
    match output {
        Ok(output) => {
            String::from_utf8_lossy(&output.stdout)
                .trim()
                .to_string()
        }
        Err(_) => {
            std::env::var("USER")
                .or_else(|_| std::env::var("LOGNAME"))
                .unwrap_or_else(|_| "unknown".to_string())
        }
    }
}

/// Get all users that use meewu
pub fn get_all_meewu_users() -> Vec<String> {
    let users_dir = MEEWU_ROOT.join("u");
    if !users_dir.exists() {
        return Vec::new();
    }
    
    let entries = match std::fs::read_dir(&users_dir) {
        Ok(entries) => entries,
        Err(_) => return Vec::new(),
    };
    
    entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_dir() {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| name.to_string())
            } else {
                None
            }
        })
        .collect()
}
