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

/// نام اعضای workspace را از یک خط `members = [...]` استخراج می‌کند —
/// پارس ساده و سطحی، نه یک پارسر کامل TOML، ولی برای این الگوی رایج کافی
/// است.
/// Extracts workspace member names from a `members = [...]` line — a
/// shallow, simple parse, not a full TOML parser, but enough for this
/// common pattern.
fn workspace_members(cargo_toml_content: &str) -> Vec<String> {
    let Some(members_line) = cargo_toml_content
        .lines()
        .find(|l| l.trim_start().starts_with("members"))
    else {
        return Vec::new();
    };
    members_line
        .split('"')
        .skip(1)
        .step_by(2)
        .map(|s| s.to_string())
        .collect()
}

/// آیا Cargo.toml ریشه یا هر کدام از اعضای workspace واقعاً به uniffi
/// اشاره می‌کنند — بررسی سطحی محتوا، نه فقط وجود فایل. طبق داده‌ی واقعی
/// بی‌مرز: Cargo.toml ریشه فقط یک workspace manifest ساده است
/// (`members = [...]`)، خودِ وابستگی uniffi داخل یکی از اعضا (مثل
/// mobile-core) است، نه ریشه — پس باید هر دو سطح چک شود.
/// Whether the root Cargo.toml or any workspace member actually mentions
/// uniffi — a shallow content check, not just file existence. Per real
/// bimarz data: the root Cargo.toml is just a plain workspace manifest
/// (`members = [...]`), the uniffi dependency itself lives inside a
/// member (like mobile-core), not the root — so both levels must be
/// checked.
fn mentions_uniffi(probe: &dyn EnvProbe, project_root: &Path) -> bool {
    let Some(root_content) = probe.read_to_string(&project_root.join("Cargo.toml")) else {
        return false;
    };
    if root_content.contains("uniffi") {
        return true;
    }
    workspace_members(&root_content).into_iter().any(|member| {
        probe
            .read_to_string(&project_root.join(&member).join("Cargo.toml"))
            .map(|content| content.contains("uniffi"))
            .unwrap_or(false)
    })
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
    fn detects_uniffi_inside_a_workspace_member_not_just_root() {
        // دقیقاً ساختار واقعی بی‌مرز: Cargo.toml ریشه فقط یک workspace
        // manifest ساده است، خودِ uniffi داخل یکی از اعضا (mobile-core) است.
        // Exactly the real bimarz structure: the root Cargo.toml is just a
        // plain workspace manifest, uniffi itself lives inside a member
        // (mobile-core).
        let root = PathBuf::from("/home/kali/bimarz");
        let mut probe = MockProbe::default();
        probe.existing_paths.push(root.join("Cargo.toml"));
        probe
            .existing_paths
            .push(root.join("android").join("settings.gradle.kts"));
        probe.files.insert(
            root.join("Cargo.toml"),
            "[workspace]\nresolver = \"2\"\nmembers = [\"engine-core\", \"mobile-core\"]\n"
                .to_string(),
        );
        probe.files.insert(
            root.join("mobile-core").join("Cargo.toml"),
            "[dependencies]\nuniffi = \"0.27\"\n".to_string(),
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
