//! سناریوی پروژه‌ی Hybrid با لایه‌ی native Rust از طریق uniffi + cargo-ndk
//! — دقیقاً الگوی بی‌مرز: یک Cargo workspace در ریشه، به‌علاوه یک پوشه‌ی
//! `android/` که خودش یک پروژه‌ی Gradle کامل است.
//! The Hybrid project scenario with a native Rust layer via uniffi +
//! cargo-ndk — exactly the bimarz pattern: a Cargo workspace at the root,
//! plus an `android/` folder that is itself a complete Gradle project.

use bana_env_scanner::EnvProbe;
use bana_plugin_api::ProjectScenario;
use bana_types::ProjectFingerprint;
use std::path::Path;

pub struct HybridRustUniffiScenario;

fn has_android_gradle_project(probe: &dyn EnvProbe, project_root: &Path) -> bool {
    let android_dir = project_root.join("android");
    probe.path_exists(&android_dir.join("settings.gradle.kts"))
        || probe.path_exists(&android_dir.join("settings.gradle"))
}

/// آیا Cargo.toml ریشه واقعاً به uniffi اشاره می‌کند — بررسی سطحی محتوای
/// فایل، نه فقط وجودش.
/// Whether the root Cargo.toml actually mentions uniffi — a shallow
/// content check, not just its existence.
fn mentions_uniffi(probe: &dyn EnvProbe, project_root: &Path) -> bool {
    probe
        .read_to_string(&project_root.join("Cargo.toml"))
        .map(|content| content.contains("uniffi"))
        .unwrap_or(false)
}

impl ProjectScenario for HybridRustUniffiScenario {
    fn scenario_id(&self) -> &'static str {
        "hybrid-rust-uniffi"
    }

    fn detect(&self, probe: &dyn EnvProbe, project_root: &Path) -> Option<f32> {
        let has_cargo = probe.path_exists(&project_root.join("Cargo.toml"));
        let has_android = has_android_gradle_project(probe, project_root);
        if !has_cargo || !has_android {
            return None;
        }
        Some(if mentions_uniffi(probe, project_root) {
            0.95
        } else {
            // Cargo workspace و پروژه‌ی اندروید هر دو هستند، ولی uniffi
            // دیده نشد — شاید یک الگوی native دیگر باشد (خارج از دامنه‌ی
            // v1، طبق بخش ۴ RULES.md)، پس اطمینان کمتر.
            // Both a Cargo workspace and an Android project exist, but no
            // uniffi mention — might be a different native pattern
            // (outside v1 scope, per RULES.md section 4), so lower
            // confidence.
            0.5
        })
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
    fn detects_high_confidence_matching_real_bimarz_layout() {
        let root = PathBuf::from("/home/kali/bimarz");
        let mut probe = MockProbe::default();
        probe.existing_paths.push(root.join("Cargo.toml"));
        probe
            .existing_paths
            .push(root.join("android").join("settings.gradle.kts"));
        probe.files.insert(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"engine-core\"]\n[dependencies]\nuniffi = \"0.27\"\n"
                .to_string(),
        );

        let confidence = HybridRustUniffiScenario.detect(&probe, &root).unwrap();
        assert!(confidence > 0.9);
    }

    #[test]
    fn lowers_confidence_when_uniffi_not_mentioned() {
        let root = PathBuf::from("/home/kali/some-other-hybrid");
        let mut probe = MockProbe::default();
        probe.existing_paths.push(root.join("Cargo.toml"));
        probe
            .existing_paths
            .push(root.join("android").join("settings.gradle.kts"));
        probe
            .files
            .insert(root.join("Cargo.toml"), "[package]\nname = \"foo\"\n".to_string());

        let confidence = HybridRustUniffiScenario.detect(&probe, &root).unwrap();
        assert!(confidence < 0.6);
    }

    #[test]
    fn returns_none_without_android_subproject() {
        let root = PathBuf::from("/home/kali/pure-rust-lib");
        let mut probe = MockProbe::default();
        probe.existing_paths.push(root.join("Cargo.toml"));
        assert!(HybridRustUniffiScenario.detect(&probe, &root).is_none());
    }

    #[test]
    fn returns_none_without_cargo_toml() {
        let root = PathBuf::from("/home/kali/pure-android");
        let mut probe = MockProbe::default();
        probe
            .existing_paths
            .push(root.join("android").join("settings.gradle.kts"));
        assert!(HybridRustUniffiScenario.detect(&probe, &root).is_none());
    }
}
