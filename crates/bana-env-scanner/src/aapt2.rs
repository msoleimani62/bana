//! تست واقعی AAPT2 — نه فقط `command -v`، بلکه خواندن واقعی هدر ELF برای
//! تطابق معماری باینری با معماری هاست (دقیقاً مرحله‌ی ۸ سند تشخیص بی‌مرز)،
//! و اجرای واقعی `aapt2 version` برای گرفتن نسخه.
//! Real AAPT2 testing — not just `command -v`, but actually reading the
//! ELF header to check the binary's architecture against the host's
//! (exactly step 8 of the bimarz detection doc), and actually running
//! `aapt2 version` to get the version string.

use crate::command::CommandRunner;
use crate::host::EnvProbe;
use crate::sdk::candidate_paths as sdk_candidate_paths;
use bana_types::{Aapt2Info, HostArch, HostKind, ToolStatus};
use std::path::{Path, PathBuf};

const EM_386: u16 = 3;
const EM_ARM: u16 = 40;
const EM_X86_64: u16 = 62;
const EM_AARCH64: u16 = 183;

fn expected_machine(arch: &HostArch) -> Option<u16> {
    match arch {
        HostArch::Aarch64 => Some(EM_AARCH64),
        HostArch::X86_64 => Some(EM_X86_64),
        HostArch::Unknown => None,
    }
}

fn machine_name(machine: u16) -> &'static str {
    match machine {
        EM_386 => "x86 (32-bit)",
        EM_ARM => "ARM (32-bit)",
        EM_X86_64 => "x86_64",
        EM_AARCH64 => "aarch64",
        _ => "an unrecognized architecture",
    }
}

/// استخراج فیلد `e_machine` از هدر ELF (آفست ۱۸، دو بایت little-endian)؛
/// `None` یعنی فایل اصلاً یک باینری ELF معتبر نیست.
/// Extracts the `e_machine` field from the ELF header (offset 18, two
/// little-endian bytes); `None` means the file isn't a valid ELF binary
/// at all.
fn read_elf_machine(probe: &dyn EnvProbe, path: &Path) -> Option<u16> {
    let bytes = probe.read_bytes(path, 20)?;
    if bytes.len() < 20 || &bytes[0..4] != b"\x7fELF" {
        return None;
    }
    Some(u16::from_le_bytes([bytes[18], bytes[19]]))
}

/// فهرست مسیرهای محتمل aapt2: اول build-tools هر SDK محتمل (رسمی‌ترین
/// منبع)، بعد هر چیزی روی PATH.
/// Candidate aapt2 paths: SDK build-tools first (the most official
/// source), then anything on PATH.
fn candidate_aapt2_paths(probe: &dyn EnvProbe, host_kind: &HostKind) -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    for sdk_root in sdk_candidate_paths(probe, host_kind) {
        let build_tools_dir = sdk_root.join("build-tools");
        for version in probe.list_dir(&build_tools_dir) {
            candidates.push(build_tools_dir.join(version).join("aapt2"));
        }
    }

    if let Some(path_var) = probe.read_env("PATH") {
        for dir in path_var.split(':') {
            candidates.push(PathBuf::from(dir).join("aapt2"));
        }
    }

    candidates
}

pub fn detect_aapt2(
    probe: &dyn EnvProbe,
    runner: &dyn CommandRunner,
    host_kind: &HostKind,
    host_arch: &HostArch,
) -> ToolStatus<Aapt2Info> {
    let expected = expected_machine(host_arch);
    let mut wrong_arch_found: Option<(PathBuf, u16)> = None;

    for path in candidate_aapt2_paths(probe, host_kind) {
        if !probe.path_exists(&path) {
            continue;
        }
        let Some(machine) = read_elf_machine(probe, &path) else {
            // فایل وجود دارد ولی ELF معتبر نیست (شاید یک wrapper اسکریپت
            // است، نه باینری خام)؛ به کاندید بعدی برو.
            // The file exists but isn't a valid ELF (maybe a wrapper
            // script, not a raw binary); move to the next candidate.
            continue;
        };

        if Some(machine) != expected {
            if wrong_arch_found.is_none() {
                wrong_arch_found = Some((path.clone(), machine));
            }
            continue;
        }

        // معماری تطابق دارد؛ حالا واقعاً اجرا کن تا نسخه‌ی واقعی را بگیری.
        // Architecture matches; now actually run it to get the real version.
        let version = runner
            .run(&path.to_string_lossy(), &["version"])
            .map(|out| {
                let raw = if out.stdout.is_empty() {
                    out.stderr
                } else {
                    out.stdout
                };
                raw.trim().to_string()
            })
            .unwrap_or_else(|| "unknown".to_string());

        return ToolStatus::Found {
            path,
            info: Aapt2Info {
                version,
                arch_mismatch: false,
            },
            verified: true,
        };
    }

    if let Some((path, machine)) = wrong_arch_found {
        let expected_name = expected
            .map(machine_name)
            .unwrap_or("an unknown architecture");
        return ToolStatus::FoundButIncompatible {
            path,
            info: Aapt2Info {
                version: "unknown".to_string(),
                arch_mismatch: true,
            },
            reason: format!(
                "found an aapt2 binary built for {}, but this host is {} -- override it via \
                 `android.aapt2FromMavenOverride` in gradle.properties, pointing at a native \
                 {} aapt2",
                machine_name(machine),
                expected_name,
                expected_name
            ),
        };
    }

    ToolStatus::NotFound
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::CommandOutput;
    use std::collections::HashMap;

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

    struct MockRunner {
        stdout: &'static str,
    }

    impl CommandRunner for MockRunner {
        fn run(&self, _program: &str, _args: &[&str]) -> Option<CommandOutput> {
            Some(CommandOutput {
                stdout: self.stdout.to_string(),
                stderr: String::new(),
                success: true,
            })
        }
    }

    fn fake_elf(machine: u16) -> Vec<u8> {
        let mut bytes = vec![0u8; 20];
        bytes[0..4].copy_from_slice(b"\x7fELF");
        bytes[18..20].copy_from_slice(&machine.to_le_bytes());
        bytes
    }

    #[test]
    fn detects_matching_architecture_and_runs_version() {
        let mut probe = MockProbe::default();
        let path = PathBuf::from("/usr/lib/android-sdk/build-tools/34.0.0/aapt2");
        probe
            .existing_paths
            .push(PathBuf::from("/usr/lib/android-sdk/build-tools"));
        probe.dirs.insert(
            PathBuf::from("/usr/lib/android-sdk/build-tools"),
            vec!["34.0.0".to_string()],
        );
        probe.existing_paths.push(path.clone());
        probe.bytes.insert(path.clone(), fake_elf(EM_AARCH64));
        let runner = MockRunner {
            stdout: "Android Asset Packaging Tool (aapt) 2.19-...",
        };

        match detect_aapt2(
            &probe,
            &runner,
            &HostKind::KaliNetHunterProot,
            &HostArch::Aarch64,
        ) {
            ToolStatus::Found { info, .. } => {
                assert!(!info.arch_mismatch);
                assert!(info.version.contains("2.19"));
            }
            other => panic!("expected Found, got {other:?}"),
        }
    }

    #[test]
    fn reports_arch_mismatch_when_only_wrong_binary_found() {
        let mut probe = MockProbe::default();
        let path = PathBuf::from("/usr/lib/android-sdk/build-tools/34.0.0/aapt2");
        probe
            .existing_paths
            .push(PathBuf::from("/usr/lib/android-sdk/build-tools"));
        probe.dirs.insert(
            PathBuf::from("/usr/lib/android-sdk/build-tools"),
            vec!["34.0.0".to_string()],
        );
        probe.existing_paths.push(path.clone());
        probe.bytes.insert(path, fake_elf(EM_X86_64));
        let runner = MockRunner { stdout: "" };

        match detect_aapt2(
            &probe,
            &runner,
            &HostKind::KaliNetHunterProot,
            &HostArch::Aarch64,
        ) {
            ToolStatus::FoundButIncompatible { info, reason, .. } => {
                assert!(info.arch_mismatch);
                assert!(reason.contains("x86_64"));
                assert!(reason.contains("aarch64"));
            }
            other => panic!("expected FoundButIncompatible, got {other:?}"),
        }
    }

    #[test]
    fn matching_candidate_wins_even_after_wrong_arch_candidate() {
        let mut probe = MockProbe::default();
        let wrong = PathBuf::from("/usr/bin/aapt2");
        let right = PathBuf::from("/usr/lib/android-sdk/build-tools/34.0.0/aapt2");

        probe
            .existing_paths
            .push(PathBuf::from("/usr/lib/android-sdk/build-tools"));
        probe.dirs.insert(
            PathBuf::from("/usr/lib/android-sdk/build-tools"),
            vec!["34.0.0".to_string()],
        );
        probe.existing_paths.push(right.clone());
        probe.bytes.insert(right, fake_elf(EM_AARCH64));

        probe.env.insert("PATH".to_string(), "/usr/bin".to_string());
        probe.existing_paths.push(wrong.clone());
        probe.bytes.insert(wrong, fake_elf(EM_X86_64));

        let runner = MockRunner { stdout: "aapt2 ok" };
        match detect_aapt2(
            &probe,
            &runner,
            &HostKind::KaliNetHunterProot,
            &HostArch::Aarch64,
        ) {
            ToolStatus::Found { info, .. } => assert!(!info.arch_mismatch),
            other => panic!("expected Found, got {other:?}"),
        }
    }

    #[test]
    fn reports_not_found_when_nothing_present() {
        let probe = MockProbe::default();
        let runner = MockRunner { stdout: "" };
        assert!(matches!(
            detect_aapt2(&probe, &runner, &HostKind::NativeLinux, &HostArch::Aarch64),
            ToolStatus::NotFound
        ));
    }

    #[test]
    fn skips_non_elf_file_and_keeps_searching() {
        let mut probe = MockProbe::default();
        let non_elf = PathBuf::from("/usr/lib/android-sdk/build-tools/34.0.0/aapt2");
        probe
            .existing_paths
            .push(PathBuf::from("/usr/lib/android-sdk/build-tools"));
        probe.dirs.insert(
            PathBuf::from("/usr/lib/android-sdk/build-tools"),
            vec!["34.0.0".to_string()],
        );
        probe.existing_paths.push(non_elf.clone());
        probe.bytes.insert(non_elf, b"not an elf file".to_vec());
        let runner = MockRunner { stdout: "" };

        assert!(matches!(
            detect_aapt2(
                &probe,
                &runner,
                &HostKind::KaliNetHunterProot,
                &HostArch::Aarch64
            ),
            ToolStatus::NotFound
        ));
    }
}
