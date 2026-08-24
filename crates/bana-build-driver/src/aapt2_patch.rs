//! پچ واقعی عدم‌تطابق معماری AAPT2 — نوشتن `android.aapt2FromMavenOverride`
//! در `gradle.properties` همان پروژه، طبق مرحله‌ی موکول‌شده از فاز ۲.
//! تأیید شد که این کلید (برخلاف اسمش) یک **مسیر فایل مستقیم** می‌گیرد،
//! نه مختصات Maven — دقیقاً همان روشی که کاربرهای واقعی Termux/proot با
//! معماری ARM استفاده می‌کنند.
//! Real AAPT2 arch-mismatch patch — writing
//! `android.aapt2FromMavenOverride` into the project's `gradle.properties`,
//! per the step deferred from Phase 2. Confirmed this key (despite its
//! name) takes a **direct file path**, not a Maven coordinate — exactly
//! how real Termux/proot ARM users fix it.

use bana_env_scanner::{detect_aapt2, CommandRunner, EnvProbe};
use bana_types::{HostArch, HostKind, ToolStatus};
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Eq)]
pub enum Aapt2PatchOutcome {
    /// override از قبل با همین مسیر درست تنظیم بود؛ هیچ نوشتنی انجام نشد.
    /// The override was already set to this exact correct path; nothing was written.
    AlreadyCorrect,
    /// `gradle.properties` نوشته/به‌روز شد.
    /// `gradle.properties` was written/updated.
    Patched,
}

#[derive(Debug, thiserror::Error)]
pub enum Aapt2PatchError {
    #[error("no working native-architecture aapt2 binary found on this host: {reason}")]
    NoNativeAapt2Available { reason: String },

    #[error("could not write gradle.properties: {reason}")]
    WriteFailed { reason: String },
}

/// انتزاع خواندن/نوشتن `gradle.properties` — برای تست‌پذیری کامل، بدون
/// نیاز به فایل واقعی روی دیسک.
/// Abstraction for reading/writing `gradle.properties` — for full
/// testability, without needing a real file on disk.
pub trait PropertiesWriter: Send + Sync {
    fn read(&self, path: &Path) -> Option<String>;
    fn write(&self, path: &Path, content: &str) -> Result<(), String>;
}

pub struct RealPropertiesWriter;

impl PropertiesWriter for RealPropertiesWriter {
    fn read(&self, path: &Path) -> Option<String> {
        std::fs::read_to_string(path).ok()
    }
    fn write(&self, path: &Path, content: &str) -> Result<(), String> {
        std::fs::write(path, content).map_err(|e| e.to_string())
    }
}

/// اطمینان idempotent از این‌که `gradle.properties` به aapt2 معتبر و
/// هم‌معماری اشاره می‌کند. اگر هیچ باینری هم‌معماری‌ای روی کل هاست پیدا
/// نشود (طبق همان جست‌وجوی فاز ۱)، خطای صریح می‌دهد — هرگز یک مسیر
/// نامعتبر یا حدسی نمی‌نویسد.
/// Idempotently ensures `gradle.properties` points at a valid,
/// architecture-matching aapt2. If no matching binary exists anywhere on
/// the host (via the same Phase 1 search), fails with a clear error —
/// never writes an invalid or guessed path.
pub fn ensure_aapt2_override(
    probe: &dyn EnvProbe,
    runner: &dyn CommandRunner,
    props: &dyn PropertiesWriter,
    host_kind: &HostKind,
    host_arch: &HostArch,
    gradle_project_root: &Path,
) -> Result<Aapt2PatchOutcome, Aapt2PatchError> {
    let aapt2_path: PathBuf = match detect_aapt2(probe, runner, host_kind, host_arch) {
        ToolStatus::Found { path, .. } => path,
        ToolStatus::FoundButIncompatible { reason, .. } => {
            return Err(Aapt2PatchError::NoNativeAapt2Available { reason })
        }
        _ => {
            return Err(Aapt2PatchError::NoNativeAapt2Available {
                reason: "aapt2 not found anywhere on this host".to_string(),
            })
        }
    };

    let props_path = gradle_project_root.join("gradle.properties");
    let desired_line = format!(
        "android.aapt2FromMavenOverride={}",
        aapt2_path.to_string_lossy()
    );

    let existing = props.read(&props_path).unwrap_or_default();
    if existing.lines().any(|l| l.trim() == desired_line.trim()) {
        return Ok(Aapt2PatchOutcome::AlreadyCorrect);
    }

    // هر خط override قدیمی (با مسیر متفاوت یا اشتباه) حذف و خط درست
    // جایگزین می‌شود — نه این‌که تکراری اضافه شود.
    // Any old override line (wrong or different path) is removed and
    // replaced with the correct one — not duplicated.
    let mut new_content: String = existing
        .lines()
        .filter(|l| !l.trim_start().starts_with("android.aapt2FromMavenOverride="))
        .collect::<Vec<_>>()
        .join("\n");
    if !new_content.is_empty() && !new_content.ends_with('\n') {
        new_content.push('\n');
    }
    new_content.push_str(&desired_line);
    new_content.push('\n');

    props
        .write(&props_path, &new_content)
        .map_err(|reason| Aapt2PatchError::WriteFailed { reason })?;

    Ok(Aapt2PatchOutcome::Patched)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bana_env_scanner::CommandOutput;
    use std::collections::HashMap;
    use std::sync::Mutex;

    #[derive(Default)]
    struct MockProbe {
        env: HashMap<String, String>,
        existing_paths: Vec<PathBuf>,
        dirs: HashMap<PathBuf, Vec<String>>,
        bytes: HashMap<PathBuf, Vec<u8>>,
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
        fn read_bytes(&self, path: &Path, _max_len: usize) -> Option<Vec<u8>> {
            self.bytes.get(path).cloned()
        }
    }

    struct MockRunner;
    impl CommandRunner for MockRunner {
        fn run(&self, _program: &str, _args: &[&str]) -> Option<CommandOutput> {
            Some(CommandOutput {
                stdout: "aapt2 ok".to_string(),
                stderr: String::new(),
                success: true,
            })
        }
    }

    #[derive(Default)]
    struct MockProps {
        files: Mutex<HashMap<PathBuf, String>>,
    }

    impl PropertiesWriter for MockProps {
        fn read(&self, path: &Path) -> Option<String> {
            self.files.lock().unwrap().get(path).cloned()
        }
        fn write(&self, path: &Path, content: &str) -> Result<(), String> {
            self.files
                .lock()
                .unwrap()
                .insert(path.to_path_buf(), content.to_string());
            Ok(())
        }
    }

    const EM_AARCH64: u16 = 183;

    fn fake_elf(machine: u16) -> Vec<u8> {
        let mut bytes = vec![0u8; 20];
        bytes[0..4].copy_from_slice(b"\x7fELF");
        bytes[18..20].copy_from_slice(&machine.to_le_bytes());
        bytes
    }

    fn probe_with_matching_aapt2() -> (MockProbe, PathBuf) {
        let mut probe = MockProbe::default();
        let aapt2_path = PathBuf::from("/usr/lib/android-sdk/build-tools/34.0.0/aapt2");
        probe
            .existing_paths
            .push(PathBuf::from("/usr/lib/android-sdk/build-tools"));
        probe.dirs.insert(
            PathBuf::from("/usr/lib/android-sdk/build-tools"),
            vec!["34.0.0".to_string()],
        );
        probe.existing_paths.push(aapt2_path.clone());
        probe.bytes.insert(aapt2_path.clone(), fake_elf(EM_AARCH64));
        (probe, aapt2_path)
    }

    #[test]
    fn writes_override_when_none_exists() {
        let (probe, aapt2_path) = probe_with_matching_aapt2();
        let runner = MockRunner;
        let props = MockProps::default();
        let root = PathBuf::from("/home/kali/bimarz/android");

        let outcome = ensure_aapt2_override(
            &probe,
            &runner,
            &props,
            &HostKind::KaliNetHunterProot,
            &HostArch::Aarch64,
            &root,
        )
        .unwrap();

        assert_eq!(outcome, Aapt2PatchOutcome::Patched);
        let written = props.files.lock().unwrap()[&root.join("gradle.properties")].clone();
        assert!(written.contains(&format!(
            "android.aapt2FromMavenOverride={}",
            aapt2_path.to_string_lossy()
        )));
    }

    #[test]
    fn is_idempotent_when_already_correct() {
        let (probe, aapt2_path) = probe_with_matching_aapt2();
        let runner = MockRunner;
        let props = MockProps::default();
        let root = PathBuf::from("/home/kali/bimarz/android");
        props.files.lock().unwrap().insert(
            root.join("gradle.properties"),
            format!(
                "android.useAndroidX=true\nandroid.aapt2FromMavenOverride={}\n",
                aapt2_path.to_string_lossy()
            ),
        );

        let outcome = ensure_aapt2_override(
            &probe,
            &runner,
            &props,
            &HostKind::KaliNetHunterProot,
            &HostArch::Aarch64,
            &root,
        )
        .unwrap();

        assert_eq!(outcome, Aapt2PatchOutcome::AlreadyCorrect);
    }

    #[test]
    fn replaces_stale_override_instead_of_duplicating() {
        let (probe, aapt2_path) = probe_with_matching_aapt2();
        let runner = MockRunner;
        let props = MockProps::default();
        let root = PathBuf::from("/home/kali/bimarz/android");
        props.files.lock().unwrap().insert(
            root.join("gradle.properties"),
            "android.aapt2FromMavenOverride=/some/stale/wrong/aapt2\n".to_string(),
        );

        let outcome = ensure_aapt2_override(
            &probe,
            &runner,
            &props,
            &HostKind::KaliNetHunterProot,
            &HostArch::Aarch64,
            &root,
        )
        .unwrap();

        assert_eq!(outcome, Aapt2PatchOutcome::Patched);
        let written = props.files.lock().unwrap()[&root.join("gradle.properties")].clone();
        assert_eq!(
            written.matches("android.aapt2FromMavenOverride=").count(),
            1
        );
        assert!(written.contains(&aapt2_path.to_string_lossy().to_string()));
        assert!(!written.contains("stale"));
    }

    #[test]
    fn errors_clearly_when_no_native_aapt2_available() {
        let probe = MockProbe::default(); // هیچ کاندیدی وجود ندارد
        let runner = MockRunner;
        let props = MockProps::default();
        let root = PathBuf::from("/home/kali/bimarz/android");

        let result = ensure_aapt2_override(
            &probe,
            &runner,
            &props,
            &HostKind::NativeLinux,
            &HostArch::Aarch64,
            &root,
        );

        assert!(matches!(
            result,
            Err(Aapt2PatchError::NoNativeAapt2Available { .. })
        ));
    }
}
