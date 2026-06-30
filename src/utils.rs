/*
    utils.rs
    SPDX-License-Identifier: MIT
    --=================================--
    Author: parkuoa <parkuoa@gmail.com>
*/

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use walkdir::WalkDir;

// recursive copy directory
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

// check if we have r/w permission to path
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

// ensure directory exists, if not, create it
pub fn ensure_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path)
        .with_context(|| format!("failed to create directory: {}", path.display()))
}

// copy a file following permission checks
pub fn copy_file(src: &Path, dst: &Path) -> Result<()> {
    if !can_write(dst) {
        anyhow::bail!(
            // we don't have permission to write to the dest path
            "no write permission for destination: {}. Try running with sudo?",
            dst.display()
        );
    }
    if let Some(parent) = dst.parent() {
        ensure_dir(parent)?;
    }
    fs::copy(src, dst)
        .with_context(|| format!("failed to copy {} to {}", src.display(), dst.display()))?;
    Ok(())
}

// get the running user's home path
pub fn home_dir() -> Result<PathBuf> {
    dirs::home_dir().context("failed to get home directory..")
}

// stupid fn to expand tilde prefixed (home) path
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