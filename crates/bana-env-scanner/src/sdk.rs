//! تشخیص واقعی Android SDK — چند مسیر محتمل (بسته به `HostKind`) بررسی
//! می‌شوند، ولی هیچ‌کدام بدون تأیید ساختار واقعی دایرکتوری (وجود واقعی
//! platforms/ و build-tools/ با محتوای واقعی) پذیرفته نمی‌شود.
//! Real Android SDK detection — several candidate paths (based on
//! `HostKind`) are checked, but none is accepted without confirming the
//! real directory structure (platforms/ and build-tools/ actually exist
//! with real content).

use crate::host::EnvProbe;
use bana_types::{HostKind, SdkInfo, ToolStatus};
use std::path::{Path, PathBuf};

/// فهرست مسیرهای محتمل SDK، به ترتیب اولویت: ابتدا env varهای صریح کاربر،
/// بعد مسیرهای رایج بسته به نوع میزبان. `pub(crate)` است چون ماژول `ndk`
/// هم همین مسیرهای پایه را برای پیدا کردن `ndk/` زیر آن‌ها استفاده می‌کند.
/// Candidate SDK paths, in priority order: explicit user env vars first,
/// then common paths based on host kind. `pub(crate)` because the `ndk`
/// module reuses these same base paths to find `ndk/` underneath them.
pub(crate) fn candidate_paths(probe: &dyn EnvProbe, host_kind: &HostKind) -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(v) = probe.read_env("ANDROID_HOME") {
        candidates.push(PathBuf::from(v));
    }
    if let Some(v) = probe.read_env("ANDROID_SDK_ROOT") {
        candidates.push(PathBuf::from(v));
    }

    let home = probe
        .read_env("HOME")
        .or_else(|| probe.read_env("USERPROFILE"))
        .map(PathBuf::from);

    match host_kind {
        HostKind::KaliNetHunterProot | HostKind::Termux | HostKind::NativeLinux => {
            candidates.push(PathBuf::from("/usr/lib/android-sdk"));
            if let Some(home) = &home {
                candidates.push(home.join("Android").join("Sdk"));
            }
        }
        HostKind::MacOs => {
            if let Some(home) = &home {
                candidates.push(home.join("Library").join("Android").join("sdk"));
            }
        }
        HostKind::Windows => {
            if let Some(local) = probe.read_env("LOCALAPPDATA") {
                candidates.push(PathBuf::from(local).join("Android").join("Sdk"));
            }
        }
        HostKind::Unknown => {}
    }

    // اگر ANDROID_HOME و ANDROID_SDK_ROOT دقیقاً یک مسیر باشند (خیلی رایج
    // است)، بدون این خط همان NDK/SDK واقعی دوبار شمرده می‌شد و نتیجه‌ی
    // کاذب AmbiguousMultiple تولید می‌کرد؛ حذف تکراری‌ها با حفظ ترتیب.
    // If ANDROID_HOME and ANDROID_SDK_ROOT are set to the exact same path
    // (very common), without this the same real NDK/SDK got counted twice,
    // producing a false AmbiguousMultiple result; dedupe while keeping
    // first-seen order.
    let mut seen = std::collections::HashSet::new();
    candidates.retain(|p| seen.insert(p.clone()));

    candidates
}

/// یک مسیر فقط وقتی SDK معتبر حساب می‌شود که هم `platforms/` هم
/// `build-tools/` واقعاً زیرش وجود داشته باشند.
/// A path only counts as a valid SDK once both `platforms/` and
/// `build-tools/` genuinely exist under it.
fn has_sdk_structure(probe: &dyn EnvProbe, root: &Path) -> bool {
    probe.path_exists(&root.join("platforms")) && probe.path_exists(&root.join("build-tools"))
}

pub fn detect_sdk(probe: &dyn EnvProbe, host_kind: &HostKind) -> ToolStatus<SdkInfo> {
    for root in candidate_paths(probe, host_kind) {
        if !has_sdk_structure(probe, &root) {
            continue;
        }

        let installed_platforms = probe.list_dir(&root.join("platforms"));
        let installed_build_tools = probe.list_dir(&root.join("build-tools"));

        if installed_platforms.is_empty() || installed_build_tools.is_empty() {
            // ساختار پوشه هست ولی هیچ نسخه‌ای واقعاً نصب نشده — SDK
            // پوسته‌ی خالی است، نه یک نصب کاربردی.
            // The folder structure exists but no version is actually
            // installed — this is an empty SDK shell, not a usable install.
            return ToolStatus::FoundButIncompatible {
                path: root,
                info: SdkInfo {
                    installed_platforms,
                    installed_build_tools,
                },
                reason: "SDK root found but has no installed platforms/build-tools".to_string(),
            };
        }

        return ToolStatus::Found {
            path: root,
            info: SdkInfo {
                installed_platforms,
                installed_build_tools,
            },
            verified: true,
        };
    }

    ToolStatus::NotFound
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Default)]
    struct MockProbe {
        env: HashMap<String, String>,
        existing_paths: Vec<PathBuf>,
        dirs: HashMap<PathBuf, Vec<String>>,
    }

    impl EnvProbe for MockProbe {
        fn read_env(&self, key: &str) -> Option<String> {
            self.env.get(key).cloned()
        }
        fn path_exists(&self, path: &Path) -> bool {
            self.existing_paths.iter().any(|p| p == path)
        }
        fn read_to_string(&self, _path: &Path) -> Option<String> {
            None
        }
        fn list_dir(&self, path: &Path) -> Vec<String> {
            self.dirs.get(path).cloned().unwrap_or_default()
        }
        fn read_bytes(&self, _path: &Path, _max_len: usize) -> Option<Vec<u8>> {
            None
        }
    }

    #[test]
    fn detects_sdk_via_android_home_env_var() {
        let mut probe = MockProbe::default();
        probe
            .env
            .insert("ANDROID_HOME".to_string(), "/opt/my-sdk".to_string());
        probe
            .existing_paths
            .push(PathBuf::from("/opt/my-sdk/platforms"));
        probe
            .existing_paths
            .push(PathBuf::from("/opt/my-sdk/build-tools"));
        probe.dirs.insert(
            PathBuf::from("/opt/my-sdk/platforms"),
            vec!["android-34".to_string()],
        );
        probe.dirs.insert(
            PathBuf::from("/opt/my-sdk/build-tools"),
            vec!["34.0.0".to_string()],
        );

        match detect_sdk(&probe, &HostKind::NativeLinux) {
            ToolStatus::Found { path, info, .. } => {
                assert_eq!(path, PathBuf::from("/opt/my-sdk"));
                assert_eq!(info.installed_platforms, vec!["android-34"]);
                assert_eq!(info.installed_build_tools, vec!["34.0.0"]);
            }
            other => panic!("expected Found, got {other:?}"),
        }
    }

    #[test]
    fn falls_back_to_kali_default_path_when_no_env_var() {
        let mut probe = MockProbe::default();
        probe
            .existing_paths
            .push(PathBuf::from("/usr/lib/android-sdk/platforms"));
        probe
            .existing_paths
            .push(PathBuf::from("/usr/lib/android-sdk/build-tools"));
        probe.dirs.insert(
            PathBuf::from("/usr/lib/android-sdk/platforms"),
            vec!["android-35".to_string()],
        );
        probe.dirs.insert(
            PathBuf::from("/usr/lib/android-sdk/build-tools"),
            vec!["35.0.0".to_string()],
        );

        match detect_sdk(&probe, &HostKind::KaliNetHunterProot) {
            ToolStatus::Found { path, .. } => {
                assert_eq!(path, PathBuf::from("/usr/lib/android-sdk"));
            }
            other => panic!("expected Found, got {other:?}"),
        }
    }

    #[test]
    fn reports_incompatible_when_structure_exists_but_empty() {
        let mut probe = MockProbe::default();
        probe
            .env
            .insert("ANDROID_HOME".to_string(), "/opt/empty-sdk".to_string());
        probe
            .existing_paths
            .push(PathBuf::from("/opt/empty-sdk/platforms"));
        probe
            .existing_paths
            .push(PathBuf::from("/opt/empty-sdk/build-tools"));
        // عمداً هیچ‌چیزی داخل dirs ثبت نشده — یعنی پوشه‌ها خالی‌اند.
        // Deliberately nothing registered in dirs — meaning the folders are empty.

        assert!(matches!(
            detect_sdk(&probe, &HostKind::NativeLinux),
            ToolStatus::FoundButIncompatible { .. }
        ));
    }

    #[test]
    fn reports_not_found_when_no_candidate_has_structure() {
        let probe = MockProbe::default();
        assert!(matches!(
            detect_sdk(&probe, &HostKind::NativeLinux),
            ToolStatus::NotFound
        ));
    }

    #[test]
    fn env_var_takes_priority_over_default_path() {
        // اگر هم env var هم مسیر پیش‌فرض معتبر باشند، env var باید انتخاب شود.
        // If both the env var and the default path are valid, the env var wins.
        let mut probe = MockProbe::default();
        probe
            .env
            .insert("ANDROID_HOME".to_string(), "/custom/sdk".to_string());
        for root in ["/custom/sdk", "/usr/lib/android-sdk"] {
            probe
                .existing_paths
                .push(PathBuf::from(root).join("platforms"));
            probe
                .existing_paths
                .push(PathBuf::from(root).join("build-tools"));
            probe.dirs.insert(
                PathBuf::from(root).join("platforms"),
                vec!["android-34".to_string()],
            );
            probe.dirs.insert(
                PathBuf::from(root).join("build-tools"),
                vec!["34.0.0".to_string()],
            );
        }

        match detect_sdk(&probe, &HostKind::KaliNetHunterProot) {
            ToolStatus::Found { path, .. } => assert_eq!(path, PathBuf::from("/custom/sdk")),
            other => panic!("expected Found, got {other:?}"),
        }
    }

    #[test]
    fn dedupes_when_android_home_and_sdk_root_are_identical() {
        // باگ واقعی: وقتی ANDROID_HOME و ANDROID_SDK_ROOT دقیقاً یک مقدار
        // دارند (خیلی رایج)، نباید همان مسیر دوبار در لیست بیاید.
        // Real bug: when ANDROID_HOME and ANDROID_SDK_ROOT hold the exact
        // same value (very common), the same path must not appear twice.
        let mut probe = MockProbe::default();
        probe.env.insert(
            "ANDROID_HOME".to_string(),
            "/home/kali/android-sdk".to_string(),
        );
        probe.env.insert(
            "ANDROID_SDK_ROOT".to_string(),
            "/home/kali/android-sdk".to_string(),
        );

        let paths = candidate_paths(&probe, &HostKind::KaliNetHunterProot);
        let occurrences = paths
            .iter()
            .filter(|p| *p == &PathBuf::from("/home/kali/android-sdk"))
            .count();
        assert_eq!(occurrences, 1);
    }
}
