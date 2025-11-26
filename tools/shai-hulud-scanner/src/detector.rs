//! Detection logic for affected packages

use std::collections::{HashMap, HashSet};

use crate::parsers::clean_version;
use crate::types::{AffectedPackage, Detection, FoundPackage, Severity, VersionConstraint};

/// Detect affected packages from the found packages list
pub fn detect_affected_packages(
    found: &[FoundPackage],
    affected_map: &HashMap<String, &AffectedPackage>,
) -> Vec<Detection> {
    let mut detections = Vec::new();
    let mut seen: HashSet<(String, String, String)> = HashSet::new();

    for pkg in found {
        // Deduplicate
        let key = (
            pkg.name.clone(),
            pkg.version.clone(),
            pkg.location.to_string_lossy().to_string(),
        );
        if seen.contains(&key) {
            continue;
        }
        seen.insert(key);

        if let Some(&affected) = affected_map.get(&pkg.name) {
            let matching_versions: Vec<String> = affected
                .versions
                .iter()
                .filter(|vc| version_matches(&pkg.version, vc))
                .map(|vc| vc.version.clone())
                .collect();

            let severity = if matching_versions.is_empty() {
                Severity::Warning
            } else {
                Severity::Critical
            };

            let affected_versions: Vec<String> =
                affected.versions.iter().map(|vc| vc.version.clone()).collect();

            detections.push(Detection {
                package: pkg.clone(),
                affected_versions,
                severity,
            });
        }
    }

    detections
}

/// Check if a version matches a constraint
fn version_matches(installed: &str, constraint: &VersionConstraint) -> bool {
    let installed_clean = clean_version(installed);
    installed_clean == constraint.version
}

