/*
    manifest.rs
    SPDX-License-Identifier: MIT
    --=================================--
    Author: parkuoa <parkuoa@gmail.com>
*/

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use once_cell::sync::Lazy;

#[derive(Debug, Deserialize, Serialize)]
pub struct MewModManifest {
    pub package: MewModPackage,
    pub metadata: MewModMetadata,
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
    pub hooks: MewModHooks,
    #[serde(default)]
    pub security: Security,
}

#[derive(Debug, Deserialize, Serialize)]
/// Defines identifiable information regarding the module itself.
///
/// Example:
/// ```bash  
/// [package]  
/// name = "battery-saver"  
/// version = "1.2.0"  
/// author = "parkuoa"  
/// description = "A bunch of tweaks to improve battery life"  
///
pub struct MewModPackage {
    /// Module name (e.g., "hello-world", "disable-glass")
    pub name: String,

    /// (Semantic) module version (e.g., "1.0.0", "2.6.7")
    pub version: String,

    /// Author identifier (e.g. username, email, profile link)
    pub author: String,

    /// (Advisely brief) description of what the module does
    pub description: String,
    #[serde(default)]

    /// SPDX license identifier (e.g., "MIT", "GPL-3.0-or-later)
    pub license: Option<String>,
    #[serde(default)]

     /// URL to module repository / source code
    pub repository: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MewModMetadata {
    /// Defines the target macOS version of the module.
    pub target_os: String,

    /// Defines the privilege level of the module.
    #[serde(default = "default_type")]
    pub r#type: MewModType,

    /// Defines the risk level of the module.
    #[serde(default = "default_risk")]
    pub risk_level: MewModRiskLevel,
    #[serde(default)]

    /// Defines whether the module requires a reboot for changes
    /// to take effect.
    pub requires_reboot: bool,
    #[serde(default)]

    /// Defines whether the module requires System Integrity Protection
    /// (SIP) to be disabled.
    pub requires_sip_off: bool,
    #[serde(default)]
    pub extra: HashMap<String, toml::Value>,
}

fn default_type() -> MewModType { MewModType::UserSpace }
fn default_risk() -> MewModRiskLevel { MewModRiskLevel::Low }

#[derive(Debug, Deserialize, Serialize, PartialEq)]
/// Defines the privileges required for a module
pub enum MewModType {
    /// User-space modules run with standard user privileges.
    /// 
    /// User-space modules can modify user files and config,
    /// but cannot modify system nor protected files
    /// Some examples are: UI tweaks, user preferences
    #[serde(rename = "user-space")]
    UserSpace,

    /// Privileged modules are those that require elevated
    /// privileges to function.
    /// 
    /// Privileged modules can modify protected files to the extent of
    /// SIP/SSV regulations.
    ///
    /// Examples are:
    /// - Creating system daemons  
    /// - Modifying /etc, /Library, /Users, /usr/local, /var  
    /// - Run commands that require root privileges (e.g. bless,
    /// pfctl, security, sysctl)
    #[serde(rename = "privileged")]
    Privileged,

    /// System-level modules require elevated privileges and SIP /
    /// authenticated root disabled.
    /// 
    /// These modules can modify the **signed system volume**, and
    /// in turn, will break the macOS cryptographic seal and make
    /// your Mac unable to install OTAs!!!
    ///
    /// Note: reverting to a sealed snapshot can be done using `bless`
    /// or `apfs_systemsnapshot`
    #[serde(rename = "system")]
    SystemLevel,

    #[serde(rename = "kernel")]
    Kernel,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
/// Defines the risk level for a module
pub enum MewModRiskLevel {
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
pub struct MewModHooks {
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