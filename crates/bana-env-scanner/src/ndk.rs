//! تشخیص واقعی NDK — هیچ‌وقت با اسم پوشه تشخیص داده نمی‌شود؛ همیشه با
//! خواندن واقعی `source.properties` تأیید می‌شود (دقیقاً همان روش سند
//! تشخیص بی‌مرز، مرحله‌ی ۴).
//! Real NDK detection — never identified by folder name; always confirmed
//! by actually reading `source.properties` (the exact method from the
//! bimarz detection doc, step 4).

use crate::host::EnvProbe;
use crate::sdk::candidate_paths as sdk_candidate_paths;
use bana_types::{HostKind, NdkInfo, ToolStatus};
use std::path::{Path, PathBuf};

/// یک `source.properties` فقط وقتی NDK واقعی حساب می‌شود که صراحتاً
/// `Pkg.Desc = Android NDK` را داشته باشد — نه هر فایلی با همین اسم.
/// A `source.properties` only counts as a real NDK when it explicitly
/// contains `Pkg.Desc = Android NDK` — not just any file with that name.
fn parse_ndk_source_properties(content: &str) -> Option<String> {
    let is_ndk = content
        .lines()
        .any(|l| l.trim() == "Pkg.Desc = Android NDK");
    if !is_ndk {
        return None;
    }
    content
        .lines()
        .find_map(|l| l.trim().strip_prefix("Pkg.Revision = "))
        .map(|v| v.to_string())
}

/// جست‌وجوی همه‌ی نسخه‌های معتبر NDK زیر یک مسیر `ndk/` (هر زیرپوشه یک
/// نسخه‌ی محتمل است).
/// Searches for every valid NDK version under one `ndk/` path (each
/// subfolder is a candidate version).
fn find_ndk_versions(probe: &dyn EnvProbe, ndk_root: &Path) -> Vec<(PathBuf, String)> {
    probe
        .list_dir(ndk_root)
        .into_iter()
        .filter_map(|name| {
            let dir = ndk_root.join(&name);
            let content = probe.read_to_string(&dir.join("source.properties"))?;
            let version = parse_ndk_source_properties(&content)?;
            Some((dir, version))
        })
        .collect()
}

pub fn detect_ndk(probe: &dyn EnvProbe, host_kind: &HostKind) -> ToolStatus<NdkInfo> {
    // اولویت اول: override صریح کاربر.
    // First priority: the user's explicit override.
    if let Some(explicit) = probe
        .read_env("ANDROID_NDK_HOME")
        .or_else(|| probe.read_env("ANDROID_NDK_ROOT"))
    {
        let dir = PathBuf::from(explicit);
        return match probe
            .read_to_string(&dir.join("source.properties"))
            .and_then(|c| parse_ndk_source_properties(&c))
        {
            Some(version) => ToolStatus::Found {
                path: dir,
                info: NdkInfo { version },
                verified: true,
            },
            None => ToolStatus::FoundButIncompatible {
                path: dir,
                info: NdkInfo {
                    version: "unknown".to_string(),
                },
                reason: "ANDROID_NDK_HOME/ANDROID_NDK_ROOT is set but no valid \
                         source.properties was found there"
                    .to_string(),
            },
        };
    }

    // اولویت دوم: گشتن زیر `ndk/` هر کدام از مسیرهای محتمل SDK.
    // Second priority: searching under `ndk/` in every candidate SDK path.
    let mut found = Vec::new();
    for sdk_root in sdk_candidate_paths(probe, host_kind) {
        let ndk_root = sdk_root.join("ndk");
        if probe.path_exists(&ndk_root) {
            found.extend(find_ndk_versions(probe, &ndk_root));
        }
    }

    match found.len() {
        0 => ToolStatus::NotFound,
        1 => {
            let (path, version) = found.into_iter().next().expect("checked len == 1");
            ToolStatus::Found {
                path,
                info: NdkInfo { version },
                verified: true,
            }
        }
        _ => ToolStatus::AmbiguousMultiple {
            candidates: found.into_iter().map(|(path, _)| path).collect(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Default)]
    struct MockProbe {
        env: HashMap<String, String>,
        existing_paths: Vec<PathBuf>,
        files: HashMap<PathBuf, String>,
        dirs: HashMap<PathBuf, Vec<String>>,
    }

    impl EnvProbe for MockProbe {
        fn read_env(&self, key: &str) -> Option<String> {
            self.env.get(key).cloned()
        }
        fn path_exists(&self, path: &Path) -> bool {
            self.existing_paths.iter().any(|p| p == path)
        }
        fn read_to_string(&self, path: &Path) -> Option<String> {
            self.files.get(path).cloned()
        }
        fn list_dir(&self, path: &Path) -> Vec<String> {
            self.dirs.get(path).cloned().unwrap_or_default()
        }
        fn read_bytes(&self, _path: &Path, _max_len: usize) -> Option<Vec<u8>> {
            None
        }
    }

    const REAL_NDK_PROPS: &str = "Pkg.Desc = Android NDK\nPkg.Revision = 26.1.10909125\n";

    #[test]
    fn detects_ndk_via_explicit_env_var() {
        let mut probe = MockProbe::default();
        probe
            .env
            .insert("ANDROID_NDK_HOME".to_string(), "/opt/my-ndk".to_string());
        probe.files.insert(
            PathBuf::from("/opt/my-ndk/source.properties"),
            REAL_NDK_PROPS.to_string(),
        );

        match detect_ndk(&probe, &HostKind::NativeLinux) {
            ToolStatus::Found { path, info, .. } => {
                assert_eq!(path, PathBuf::from("/opt/my-ndk"));
                assert_eq!(info.version, "26.1.10909125");
            }
            other => panic!("expected Found, got {other:?}"),
        }
    }

    #[test]
    fn env_var_set_but_invalid_is_incompatible() {
        let mut probe = MockProbe::default();
        probe.env.insert(
            "ANDROID_NDK_HOME".to_string(),
            "/opt/not-really-ndk".to_string(),
        );
        // عمداً هیچ source.properties ثبت نشده.
        // Deliberately no source.properties registered.

        assert!(matches!(
            detect_ndk(&probe, &HostKind::NativeLinux),
            ToolStatus::FoundButIncompatible { .. }
        ));
    }

    #[test]
    fn finds_single_version_under_sdk_ndk_folder() {
        let mut probe = MockProbe::default();
        probe
            .existing_paths
            .push(PathBuf::from("/usr/lib/android-sdk/ndk"));
        probe.dirs.insert(
            PathBuf::from("/usr/lib/android-sdk/ndk"),
            vec!["26.1.10909125".to_string()],
        );
        probe.files.insert(
            PathBuf::from("/usr/lib/android-sdk/ndk/26.1.10909125/source.properties"),
            REAL_NDK_PROPS.to_string(),
        );

        match detect_ndk(&probe, &HostKind::KaliNetHunterProot) {
            ToolStatus::Found { info, .. } => assert_eq!(info.version, "26.1.10909125"),
            other => panic!("expected Found, got {other:?}"),
        }
    }

    #[test]
    fn ignores_non_ndk_folders_without_correct_pkg_desc() {
        let mut probe = MockProbe::default();
        probe
            .existing_paths
            .push(PathBuf::from("/usr/lib/android-sdk/ndk"));
        probe.dirs.insert(
            PathBuf::from("/usr/lib/android-sdk/ndk"),
            vec!["not-actually-ndk".to_string()],
        );
        probe.files.insert(
            PathBuf::from("/usr/lib/android-sdk/ndk/not-actually-ndk/source.properties"),
            "Pkg.Desc = Something Else Entirely\nPkg.Revision = 1.0\n".to_string(),
        );

        assert!(matches!(
            detect_ndk(&probe, &HostKind::KaliNetHunterProot),
            ToolStatus::NotFound
        ));
    }

    #[test]
    fn reports_ambiguous_when_multiple_versions_found() {
        let mut probe = MockProbe::default();
        probe
            .existing_paths
            .push(PathBuf::from("/usr/lib/android-sdk/ndk"));
        probe.dirs.insert(
            PathBuf::from("/usr/lib/android-sdk/ndk"),
            vec!["25.2.9519653".to_string(), "26.1.10909125".to_string()],
        );
        probe.files.insert(
            PathBuf::from("/usr/lib/android-sdk/ndk/25.2.9519653/source.properties"),
            "Pkg.Desc = Android NDK\nPkg.Revision = 25.2.9519653\n".to_string(),
        );
        probe.files.insert(
            PathBuf::from("/usr/lib/android-sdk/ndk/26.1.10909125/source.properties"),
            REAL_NDK_PROPS.to_string(),
        );

        match detect_ndk(&probe, &HostKind::KaliNetHunterProot) {
            ToolStatus::AmbiguousMultiple { candidates } => assert_eq!(candidates.len(), 2),
            other => panic!("expected AmbiguousMultiple, got {other:?}"),
        }
    }

    #[test]
    fn does_not_double_count_one_ndk_when_home_and_sdk_root_are_identical() {
        // باگ واقعی، پیدا‌شده روی دستگاه واقعی: وقتی ANDROID_HOME و
        // ANDROID_SDK_ROOT دقیقاً یک مقدار دارند، همان یک NDK واقعی
        // نباید به‌اشتباه AmbiguousMultiple گزارش شود.
        // Real bug, found on a real device: when ANDROID_HOME and
        // ANDROID_SDK_ROOT hold the exact same value, the one real NDK
        // must not be falsely reported as AmbiguousMultiple.
        let mut probe = MockProbe::default();
        probe.env.insert(
            "ANDROID_HOME".to_string(),
            "/home/kali/android-sdk".to_string(),
        );
        probe.env.insert(
            "ANDROID_SDK_ROOT".to_string(),
            "/home/kali/android-sdk".to_string(),
        );
        probe
            .existing_paths
            .push(PathBuf::from("/home/kali/android-sdk/ndk"));
        probe.dirs.insert(
            PathBuf::from("/home/kali/android-sdk/ndk"),
            vec!["27.3.13750724".to_string()],
        );
        probe.files.insert(
            PathBuf::from("/home/kali/android-sdk/ndk/27.3.13750724/source.properties"),
            "Pkg.Desc = Android NDK\nPkg.Revision = 27.3.13750724\n".to_string(),
        );

        match detect_ndk(&probe, &HostKind::KaliNetHunterProot) {
            ToolStatus::Found { info, .. } => assert_eq!(info.version, "27.3.13750724"),
            other => panic!("expected Found, got {other:?}"),
        }
    }

    #[test]
    fn reports_not_found_when_nothing_present() {
        let probe = MockProbe::default();
        assert!(matches!(
            detect_ndk(&probe, &HostKind::NativeLinux),
            ToolStatus::NotFound
        ));
    }
}
