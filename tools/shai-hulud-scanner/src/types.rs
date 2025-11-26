//! Common type definitions for the Shai-Hulud Scanner

use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

/// Represents an affected package from the CSV
#[derive(Debug, Clone)]
pub struct AffectedPackage {
    pub name: String,
    pub versions: Vec<VersionConstraint>,
}

/// Version constraint (e.g., "= 0.0.7" or "= 3.24.1")
#[derive(Debug, Clone)]
pub struct VersionConstraint {
    pub version: String,
}

/// Represents a package found in the project
#[derive(Debug, Clone)]
pub struct FoundPackage {
    pub name: String,
    pub version: String,
    pub location: PathBuf,
    pub file_type: String,
    pub source: PackageSource,
}

/// Source of a package (local or global from various package managers)
#[derive(Debug, Clone, PartialEq)]
pub enum PackageSource {
    Local,      // Project-level packages
    GlobalNpm,  // npm global
    GlobalYarn, // yarn global
    GlobalPnpm, // pnpm global
    GlobalBun,  // bun global
}

impl std::fmt::Display for PackageSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackageSource::Local => write!(f, "Local"),
            PackageSource::GlobalNpm => write!(f, "npm (global)"),
            PackageSource::GlobalYarn => write!(f, "yarn (global)"),
            PackageSource::GlobalPnpm => write!(f, "pnpm (global)"),
            PackageSource::GlobalBun => write!(f, "bun (global)"),
        }
    }
}

/// Detection result
#[derive(Debug)]
pub struct Detection {
    pub package: FoundPackage,
    pub affected_versions: Vec<String>,
    pub severity: Severity,
}

/// Suspicious file detection
#[derive(Debug)]
pub struct SuspiciousFile {
    pub path: PathBuf,
    pub reason: String,
    pub severity: Severity,
}

/// Severity level for detections
#[derive(Debug, Clone, Copy)]
pub enum Severity {
    Critical, // Exact version match or suspicious file
    Warning,  // Package name match but version unclear
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Critical => write!(f, "CRITICAL"),
            Severity::Warning => write!(f, "WARNING"),
        }
    }
}

/// Package.json structure
#[derive(Debug, Deserialize)]
pub struct PackageJson {
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
    #[serde(default, rename = "devDependencies")]
    pub dev_dependencies: HashMap<String, String>,
    #[serde(default, rename = "optionalDependencies")]
    pub optional_dependencies: HashMap<String, String>,
}

/// Package-lock.json structure (simplified)
#[derive(Debug, Deserialize)]
pub struct PackageLockJson {
    #[serde(default)]
    pub packages: HashMap<String, PackageLockEntry>,
    #[serde(default)]
    pub dependencies: HashMap<String, PackageLockDependency>,
}

#[derive(Debug, Deserialize)]
pub struct PackageLockEntry {
    #[serde(default)]
    pub version: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PackageLockDependency {
    #[serde(default)]
    pub version: Option<String>,
}

