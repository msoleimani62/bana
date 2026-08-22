//! سناریوی پروژه‌ی خالص Kotlin/Java با Gradle استاندارد، بدون لایه‌ی
//! native Rust.
//! The pure Kotlin/Java project scenario with standard Gradle, no native
//! Rust layer.

use bana_env_scanner::EnvProbe;
use bana_plugin_api::ProjectScenario;
use bana_types::ProjectFingerprint;
use std::path::Path;

pub struct PureKotlinScenario;

fn has_gradle_settings(probe: &dyn EnvProbe, project_root: &Path) -> bool {
    probe.path_exists(&project_root.join("settings.gradle.kts"))
        || probe.path_exists(&project_root.join("settings.gradle"))
}

impl ProjectScenario for PureKotlinScenario {
    fn scenario_id(&self) -> &'static str {
        "pure-kotlin"
    }

    fn detect(&self, probe: &dyn EnvProbe, project_root: &Path) -> Option<f32> {
        if !has_gradle_settings(probe, project_root) {
            return None;
        }
        // Cargo.toml هم‌سطح settings.gradle.kts یعنی احتمالاً این پروژه
        // در واقع Hybrid است، فقط اطمینان کمتری به این سناریو می‌دهیم.
        // A Cargo.toml alongside settings.gradle.kts likely means this
        // project is actually Hybrid — lower confidence for this scenario.
        let has_cargo = probe.path_exists(&project_root.join("Cargo.toml"));
        Some(if has_cargo { 0.3 } else { 0.9 })
    }

    fn fingerprint(&self, probe: &dyn EnvProbe, project_root: &Path) -> ProjectFingerprint {
        ProjectFingerprint {
            scenario_id: self.scenario_id().to_string(),
            root: project_root.to_path_buf(),
            confidence: self.detect(probe, project_root).unwrap_or(0.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[derive(Default)]
    struct MockProbe {
        existing_paths: Vec<PathBuf>,
        files: HashMap<PathBuf, String>,
    }

    impl EnvProbe for MockProbe {
        fn read_env(&self, _key: &str) -> Option<String> {
            None
        }
        fn path_exists(&self, path: &Path) -> bool {
            self.existing_paths.iter().any(|p| p == path)
        }
        fn read_to_string(&self, path: &Path) -> Option<String> {
            self.files.get(path).cloned()
        }
        fn list_dir(&self, _path: &Path) -> Vec<String> {
            Vec::new()
        }
        fn read_bytes(&self, _path: &Path, _max_len: usize) -> Option<Vec<u8>> {
            None
        }
    }

    #[test]
    fn detects_high_confidence_when_only_gradle_present() {
        let root = PathBuf::from("/home/kali/my-app");
        let mut probe = MockProbe::default();
        probe.existing_paths.push(root.join("settings.gradle.kts"));

        let confidence = PureKotlinScenario.detect(&probe, &root).unwrap();
        assert!(confidence > 0.8);
    }

    #[test]
    fn lowers_confidence_when_cargo_toml_also_present() {
        let root = PathBuf::from("/home/kali/bimarz/android");
        let mut probe = MockProbe::default();
        probe.existing_paths.push(root.join("settings.gradle.kts"));
        probe.existing_paths.push(root.join("Cargo.toml"));

        let confidence = PureKotlinScenario.detect(&probe, &root).unwrap();
        assert!(confidence < 0.5);
    }

    #[test]
    fn returns_none_without_gradle_settings() {
        let root = PathBuf::from("/home/kali/not-a-project");
        let probe = MockProbe::default();
        assert!(PureKotlinScenario.detect(&probe, &root).is_none());
    }

    #[test]
    fn fingerprint_carries_the_same_confidence_as_detect() {
        let root = PathBuf::from("/home/kali/my-app");
        let mut probe = MockProbe::default();
        probe.existing_paths.push(root.join("settings.gradle.kts"));

        let fp = PureKotlinScenario.fingerprint(&probe, &root);
        assert_eq!(fp.scenario_id, "pure-kotlin");
        assert_eq!(fp.root, root);
        assert!(fp.confidence > 0.8);
    }
}
