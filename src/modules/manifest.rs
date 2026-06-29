/*
    manifest.rs
    SPDX-License-Identifier: MIT
    --=================================--
    Author: parkuoa <parkuoa@gmail.com>
*/

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
pub struct mewModManifest {
    pub package: mewModPackage,
    pub metadata: mewModMetadata,
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
    #[serde(default)]
    pub files: Files,
    #[serde(default)]
    pub launch_agents: HashMap<String, LaunchAgent>,
    #[serde(default)]
    pub launch_daemons: HashMap<String, LaunchDaemon>,
    #[serde(default)]
    pub system_modifications: SystemModifications,
    #[serde(default)]
    pub hooks: mewModHooks,
    #[serde(default)]
    pub security: Security,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct mewModPackage {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub repository: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct mewModMetadata {
    pub target_os: String,
    #[serde(default = "default_type")]
    pub r#type: mewModType,
    #[serde(default = "default_risk")]
    pub risk_level: mewModRiskLevel,
    #[serde(default)]
    pub requires_reboot: bool,
    #[serde(default)]
    pub requires_sip_off: bool,
    #[serde(default)]
    pub extra: HashMap<String, toml::Value>,
}

fn default_type() -> mewModType { mewModType::UserSpace }
fn default_risk() -> mewModRiskLevel { mewModRiskLevel::Low }

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum mewModType {
    #[serde(rename = "user-space")]
    UserSpace,
    #[serde(rename = "system-patch")]
    SystemPatch,
    #[serde(rename = "kernel")]
    Kernel,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum mewModRiskLevel {
    #[serde(rename = "low")]
    Low,
    #[serde(rename = "medium")]
    Medium,
    #[serde(rename = "high")]
    High,
    #[serde(rename = "experimental")]
    Experimental,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Files {
    #[serde(default)]
    pub copy: Vec<FileOperation>,
    #[serde(default)]
    pub symlink: Vec<FileOperation>,
    #[serde(default)]
    pub move_files: Vec<FileOperation>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FileOperation {
    pub src: PathBuf,
    pub dest: PathBuf,
    #[serde(default)]
    pub permissions: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LaunchAgent {
    pub name: String,
    pub file: PathBuf,
    #[serde(default)]
    pub run_at_load: bool,
    #[serde(default)]
    pub keep_alive: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LaunchDaemon {
    pub file: PathBuf,
    #[serde(default)]
    pub run_at_load: bool,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct SystemModifications {
    #[serde(default)]
    pub overlay: Vec<FileOperation>,
    #[serde(default)]
    pub kexts: Vec<PathBuf>,
    #[serde(default)]
    pub frameworks: Vec<FileOperation>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct mewModHooks {
    #[serde(default)]
    pub pre_install: Option<PathBuf>,
    #[serde(default)]
    pub install: Option<PathBuf>,
    #[serde(default)]
    pub post_install: Option<PathBuf>,
    #[serde(default)]
    pub pre_uninstall: Option<PathBuf>,
    #[serde(default)]
    pub uninstall: Option<PathBuf>,
    #[serde(default)]
    pub post_uninstall: Option<PathBuf>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Security {
    #[serde(default)]
    pub require_signature: bool,
    #[serde(default)]
    pub allowed_paths: Vec<PathBuf>,
    #[serde(default)]
    pub blocked_paths: Vec<PathBuf>,
}