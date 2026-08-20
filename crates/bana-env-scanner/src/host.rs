//! تشخیص واقعی محیط میزبان، پشت انتزاع `EnvProbe` تا بدون دستگاه واقعی هر
//! سیستم‌عامل هم کاملاً قابل تست باشد.
//! Real host-environment detection behind the `EnvProbe` abstraction, so it
//! stays fully testable without needing real hardware for every OS.

use bana_types::{HostArch, HostEnvironment, HostKind};
use std::path::{Path, PathBuf};

/// انتزاع منابع محیطی (env var، فایل‌سیستم) که منطق تشخیص به آن نیاز دارد.
/// پیاده‌سازی واقعی از سیستم واقعی می‌خواند؛ پیاده‌سازی تست از داده‌ی ساختگی.
///
/// Abstraction over the environment resources (env vars, filesystem) the
/// detection logic needs. The real implementation reads the real system;
/// the test implementation reads fake data.
pub trait EnvProbe {
    fn read_env(&self, key: &str) -> Option<String>;
    fn path_exists(&self, path: &Path) -> bool;
    fn read_to_string(&self, path: &Path) -> Option<String>;
    /// نام زیرپوشه‌های مستقیم یک مسیر؛ در صورت هر خطا (وجود نداشتن، عدم
    /// دسترسی) لیست خالی برمی‌گردد، نه panic.
    /// Names of a path's direct subdirectories; any error (missing, no
    /// permission) returns an empty list, never a panic.
    fn list_dir(&self, path: &Path) -> Vec<String>;
}

/// پیاده‌سازی واقعی `EnvProbe` که مستقیم از سیستم عامل واقعی می‌خواند.
/// The real `EnvProbe` implementation, reading directly from the real OS.
pub struct RealEnvProbe;

impl EnvProbe for RealEnvProbe {
    fn read_env(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }

    fn path_exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn read_to_string(&self, path: &Path) -> Option<String> {
        std::fs::read_to_string(path).ok()
    }

    fn list_dir(&self, path: &Path) -> Vec<String> {
        std::fs::read_dir(path)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().is_dir())
                    .filter_map(|e| e.file_name().into_string().ok())
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// تشخیص `HostKind` — ترتیب چک‌ها عمداً از خاص‌ترین به عمومی‌ترین است، چون
/// Kali NetHunter proot هم خودش را «لینوکس» و هم گاهی متغیرهای Termux را
/// نشان می‌دهد؛ باید اول حالت خاص‌تر (proot روی اندروید) رد شود.
///
/// Detects `HostKind` — checks deliberately go from most specific to most
/// generic, since a Kali NetHunter proot both looks like plain Linux and
/// sometimes carries Termux env vars; the more specific case must be ruled
/// out first.
fn detect_host_kind(probe: &dyn EnvProbe) -> HostKind {
    if cfg!(target_os = "windows") {
        return HostKind::Windows;
    }
    if cfg!(target_os = "macos") {
        return HostKind::MacOs;
    }
    if !cfg!(target_os = "linux") {
        return HostKind::Unknown;
    }

    // نشانه‌ی proot روی اندروید: طبق داده‌ی واقعی دستگاه کاربر، مسیر
    // `/system/build.prop` وجود نداشت (فرض اولیه‌ی اشتباه)، ولی `/termux`
    // (حتی بدون هیچ مجوزی، چون فقط stat لازم است نه خواندن) و `/sdcard`
    // به‌طور قابل‌اتکا فقط داخل این نوع chroot دیده می‌شوند.
    // Sign of proot on Android: per the user's real device data,
    // `/system/build.prop` did not exist (a wrong initial assumption), but
    // `/termux` (reliable even with zero permissions, since only stat is
    // needed, not read) and `/sdcard` are only reliably seen inside this
    // kind of chroot.
    let android_proot_signal =
        probe.path_exists(Path::new("/termux")) || probe.path_exists(Path::new("/sdcard"));
    let os_release = probe.read_to_string(Path::new("/etc/os-release"));
    let is_kali_os_release = os_release
        .as_deref()
        .map(|s| s.to_lowercase().contains("kali"))
        .unwrap_or(false);

    if android_proot_signal && is_kali_os_release {
        return HostKind::KaliNetHunterProot;
    }

    // نشانه‌ی خودِ Termux (بدون proot اضافه): متغیر محیطی اختصاصی Termux.
    // Sign of plain Termux (no extra proot): Termux's own env var.
    if probe.read_env("TERMUX_VERSION").is_some() {
        return HostKind::Termux;
    }

    HostKind::NativeLinux
}

/// تشخیص معماری پردازنده از طریق ثابت‌های زمان کامپایل Rust.
/// Detects CPU architecture via Rust's compile-time constants.
fn detect_host_arch() -> HostArch {
    match std::env::consts::ARCH {
        "aarch64" => HostArch::Aarch64,
        "x86_64" => HostArch::X86_64,
        _ => HostArch::Unknown,
    }
}

/// تشخیص اینکه آیا `systemd` واقعی در حال اجراست یا استاب‌شده — نشانه‌اش
/// نبودِ `/run/systemd/system` (که فقط systemd واقعی در زمان boot می‌سازد)
/// در حالی که باینری‌های systemd در دسترس‌اند. دقیقاً همون مشکلی که خودِ
/// کاربر روی proot با wrapperهای استاب حل کرده بود.
///
/// Detects whether a real `systemd` is running versus stubbed — the signal
/// is the absence of `/run/systemd/system` (which only a real, booted
/// systemd creates) while systemd binaries are present. This mirrors the
/// exact problem the user solved on proot with stub wrappers.
fn detect_systemd_stubbed(probe: &dyn EnvProbe) -> bool {
    let systemd_binary_present = probe.path_exists(Path::new("/usr/bin/systemd-sysusers"))
        || probe.path_exists(Path::new("/bin/systemd-sysusers"));
    let systemd_actually_running = probe.path_exists(Path::new("/run/systemd/system"));
    systemd_binary_present && !systemd_actually_running
}

fn detect_home_dir(probe: &dyn EnvProbe) -> PathBuf {
    probe
        .read_env("HOME")
        .or_else(|| probe.read_env("USERPROFILE")) // ویندوز HOME ندارد / Windows has no HOME
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn detect_shell(probe: &dyn EnvProbe) -> String {
    probe
        .read_env("SHELL")
        .unwrap_or_else(|| "unknown".to_string())
}

/// نقطه‌ی ورود اصلی: تشخیص کامل `HostEnvironment` با یک `EnvProbe` دلخواه.
/// Main entry point: full `HostEnvironment` detection against any `EnvProbe`.
pub fn detect_host_environment(probe: &dyn EnvProbe) -> HostEnvironment {
    HostEnvironment {
        kind: detect_host_kind(probe),
        arch: detect_host_arch(),
        home_dir: detect_home_dir(probe),
        shell: detect_shell(probe),
        systemd_stubbed: detect_systemd_stubbed(probe),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// پروب ساختگی برای تست — بدون هیچ نیاز به دستگاه واقعی.
    /// Fake probe for tests — no real device needed at all.
    #[derive(Default)]
    struct MockEnvProbe {
        env: HashMap<String, String>,
        existing_paths: Vec<PathBuf>,
        files: HashMap<PathBuf, String>,
        dirs: HashMap<PathBuf, Vec<String>>,
    }

    impl EnvProbe for MockEnvProbe {
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
    }

    #[test]
    fn detects_kali_nethunter_proot_via_termux_dir() {
        let mut probe = MockEnvProbe::default();
        probe.existing_paths.push(PathBuf::from("/termux"));
        probe.files.insert(
            PathBuf::from("/etc/os-release"),
            "NAME=\"Kali GNU/Linux\"".to_string(),
        );
        assert_eq!(detect_host_kind(&probe), HostKind::KaliNetHunterProot);
    }

    #[test]
    fn detects_kali_nethunter_proot_via_sdcard_dir() {
        // /sdcard تنها نشانه‌ی موجود است ولی همراه os-release کالی، همچنان
        // باید کافی باشد — طبق طراحی OR بین دو نشانه.
        // /sdcard is the only present signal, but combined with Kali
        // os-release it should still be enough — the two signals are OR'd
        // by design.
        let mut probe = MockEnvProbe::default();
        probe.existing_paths.push(PathBuf::from("/sdcard"));
        probe.files.insert(
            PathBuf::from("/etc/os-release"),
            "NAME=\"Kali GNU/Linux\"".to_string(),
        );
        assert_eq!(detect_host_kind(&probe), HostKind::KaliNetHunterProot);
    }

    #[test]
    fn android_signal_without_kali_os_release_is_not_proot() {
        // فقط نشانه‌ی اندروید بدون تأیید os-release کافی نیست — باید هر دو
        // باشند.
        // The Android signal alone without os-release confirmation isn't
        // enough — both signals must agree.
        let mut probe = MockEnvProbe::default();
        probe.existing_paths.push(PathBuf::from("/termux"));
        assert_ne!(detect_host_kind(&probe), HostKind::KaliNetHunterProot);
    }

    #[test]
    fn detects_plain_termux() {
        let mut probe = MockEnvProbe::default();
        probe
            .env
            .insert("TERMUX_VERSION".to_string(), "0.118".to_string());
        assert_eq!(detect_host_kind(&probe), HostKind::Termux);
    }

    #[test]
    fn detects_native_linux_when_no_special_signal() {
        let probe = MockEnvProbe::default();
        assert_eq!(detect_host_kind(&probe), HostKind::NativeLinux);
    }

    #[test]
    fn detects_systemd_stubbed_when_binary_present_but_not_running() {
        let mut probe = MockEnvProbe::default();
        probe
            .existing_paths
            .push(PathBuf::from("/usr/bin/systemd-sysusers"));
        assert!(detect_systemd_stubbed(&probe));
    }

    #[test]
    fn detects_systemd_not_stubbed_when_actually_running() {
        let mut probe = MockEnvProbe::default();
        probe
            .existing_paths
            .push(PathBuf::from("/usr/bin/systemd-sysusers"));
        probe
            .existing_paths
            .push(PathBuf::from("/run/systemd/system"));
        assert!(!detect_systemd_stubbed(&probe));
    }

    #[test]
    fn detects_systemd_not_stubbed_when_binary_absent() {
        let probe = MockEnvProbe::default();
        assert!(!detect_systemd_stubbed(&probe));
    }

    #[test]
    fn falls_back_to_userprofile_when_home_missing() {
        let mut probe = MockEnvProbe::default();
        probe
            .env
            .insert("USERPROFILE".to_string(), "C:\\Users\\test".to_string());
        assert_eq!(detect_home_dir(&probe), PathBuf::from("C:\\Users\\test"));
    }

    #[test]
    fn full_detection_wires_all_fields_together() {
        let mut probe = MockEnvProbe::default();
        probe
            .env
            .insert("HOME".to_string(), "/home/kali".to_string());
        probe
            .env
            .insert("SHELL".to_string(), "/bin/zsh".to_string());
        let env = detect_host_environment(&probe);
        assert_eq!(env.home_dir, PathBuf::from("/home/kali"));
        assert_eq!(env.shell, "/bin/zsh");
    }
}
