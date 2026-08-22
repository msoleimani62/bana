//! رجیستری سناریوها — نقطه‌ای که همه‌ی پیاده‌سازی‌های `ProjectScenario`
//! ثبت می‌شوند و بهترین تطابق (بالاترین اطمینان) انتخاب می‌شود. افزودن
//! سناریوی جدید یعنی فقط یک خط اینجا اضافه شود، بدون تغییر بقیه‌ی منطق —
//! دقیقاً طبق اصل ۹ RULES.md.
//! The scenario registry — where every `ProjectScenario` implementation is
//! registered and the best match (highest confidence) is picked. Adding a
//! new scenario means just one line here, no other logic changes — per
//! RULES.md principle 9.

use crate::hybrid_rust_uniffi::HybridRustUniffiScenario;
use crate::pure_kotlin::PureKotlinScenario;
use bana_env_scanner::EnvProbe;
use bana_plugin_api::ProjectScenario;
use bana_types::ProjectFingerprint;
use std::path::Path;

/// فهرست همه‌ی سناریوهای v1 ثبت‌شده، به هیچ ترتیب خاصی وابسته نیستند چون
/// انتخاب نهایی بر اساس بالاترین `confidence` است، نه اولین تطابق.
/// The list of all registered v1 scenarios, independent of order since the
/// final pick is by highest `confidence`, not first match.
fn registered_scenarios() -> Vec<Box<dyn ProjectScenario>> {
    vec![
        Box::new(HybridRustUniffiScenario),
        Box::new(PureKotlinScenario),
    ]
}

/// تحلیل یک پروژه: بین همه‌ی سناریوهای ثبت‌شده، بهترین تطابق را برمی‌گرداند.
/// `None` یعنی هیچ سناریوی شناخته‌شده‌ای با این پروژه مطابقت نداشت.
/// Analyzes a project: returns the best match among all registered
/// scenarios. `None` means no known scenario matched this project.
pub fn analyze_project(probe: &dyn EnvProbe, project_root: &Path) -> Option<ProjectFingerprint> {
    let mut best: Option<(f32, Box<dyn ProjectScenario>)> = None;

    for scenario in registered_scenarios() {
        if let Some(confidence) = scenario.detect(probe, project_root) {
            let is_better = best.as_ref().map(|(c, _)| confidence > *c).unwrap_or(true);
            if is_better {
                best = Some((confidence, scenario));
            }
        }
    }

    best.map(|(_, scenario)| scenario.fingerprint(probe, project_root))
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
    fn picks_hybrid_over_pure_kotlin_when_both_signals_present() {
        // دقیقاً همان لحظه‌ای که PureKotlin هم مطابقت جزئی می‌دهد (چون
        // Cargo.toml هم‌سطح settings.gradle.kts دیده می‌شود)، ولی Hybrid
        // باید با اطمینان بالاتر برنده شود.
        // Exactly the moment where PureKotlin also partially matches
        // (since Cargo.toml sits alongside settings.gradle.kts), but
        // Hybrid must win with higher confidence.
        let root = PathBuf::from("/home/kali/bimarz");
        let mut probe = MockProbe::default();
        probe.existing_paths.push(root.join("Cargo.toml"));
        probe
            .existing_paths
            .push(root.join("android").join("settings.gradle.kts"));
        probe.files.insert(
            root.join("Cargo.toml"),
            "[dependencies]\nuniffi = \"0.27\"\n".to_string(),
        );

        let fp = analyze_project(&probe, &root).unwrap();
        assert_eq!(fp.scenario_id, "hybrid-rust-uniffi");
    }

    #[test]
    fn picks_pure_kotlin_when_no_rust_signals() {
        let root = PathBuf::from("/home/kali/my-simple-app");
        let mut probe = MockProbe::default();
        probe.existing_paths.push(root.join("settings.gradle.kts"));

        let fp = analyze_project(&probe, &root).unwrap();
        assert_eq!(fp.scenario_id, "pure-kotlin");
    }

    #[test]
    fn returns_none_when_nothing_matches() {
        let root = PathBuf::from("/home/kali/random-folder");
        let probe = MockProbe::default();
        assert!(analyze_project(&probe, &root).is_none());
    }
}
