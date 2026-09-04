//! اجرای واقعی build لایه‌ی native با `cargo-ndk`، برای همه‌ی ABIهای
//! موردنیاز — دقیقاً معادل مکانیزه‌ی مرحله‌ی ۶-۷ سند تشخیص بی‌مرز
//! (`scripts/build-android.sh`).
//! Real native-layer build via `cargo-ndk`, for every required ABI —
//! the mechanized equivalent of the bimarz detection doc's step 6-7
//! (`scripts/build-android.sh`).

use bana_env_scanner::CommandRunner;
use std::path::{Path, PathBuf};

/// چهار ABI استاندارد اندروید که پروژه‌ی Hybrid به‌طور پیش‌فرض برایشان
/// build می‌شود، طبق تصمیم بند ۴ RULES.md (چهارگانه را اول سالم کنیم).
/// The four standard Android ABIs a Hybrid project builds for by default,
/// per the RULES.md section 4 decision (get the quad build healthy first).
pub const ALL_ABIS: &[&str] = &["arm64-v8a", "armeabi-v7a", "x86_64", "x86"];

#[derive(Debug, thiserror::Error)]
pub enum NativeBuildError {
    #[error(
        "cargo-ndk is not installed or not on PATH. Install it with \
         `cargo install cargo-ndk` and run again."
    )]
    CargoNdkMissing,

    #[error("cargo-ndk build failed for package '{package}': {reason}")]
    BuildFailed { package: String, reason: String },
}

/// اجرای واقعی `cargo ndk -t <abi>... -o <jni_libs_out> build --release -p <package>`
/// داخل `workspace_root`. قبل از اجرا، وجود واقعی `cargo-ndk` تأیید
/// می‌شود — نه فرض.
/// Actually runs
/// `cargo ndk -t <abi>... -o <jni_libs_out> build --release -p <package>`
/// inside `workspace_root`. Before running, the real presence of
/// `cargo-ndk` is confirmed — never assumed.
pub fn build_native_layer(
    runner: &dyn CommandRunner,
    workspace_root: &Path,
    package: &str,
    jni_libs_out: &Path,
    abis: &[&str],
) -> Result<(), NativeBuildError> {
    if runner.run("cargo", &["ndk", "--version"]).is_none() {
        return Err(NativeBuildError::CargoNdkMissing);
    }

    let mut args: Vec<String> = vec!["ndk".to_string()];
    for abi in abis {
        args.push("-t".to_string());
        args.push((*abi).to_string());
    }
    args.push("-o".to_string());
    args.push(jni_libs_out.to_string_lossy().to_string());
    args.push("build".to_string());
    args.push("--release".to_string());
    args.push("-p".to_string());
    args.push(package.to_string());

    let args_ref: Vec<&str> = args.iter().map(String::as_str).collect();

    match runner.run_in(workspace_root, "cargo", &args_ref) {
        Some(out) if out.success => Ok(()),
        Some(out) => Err(NativeBuildError::BuildFailed {
            package: package.to_string(),
            reason: if out.stderr.is_empty() {
                out.stdout
            } else {
                out.stderr
            },
        }),
        None => Err(NativeBuildError::BuildFailed {
            package: package.to_string(),
            reason: "could not execute `cargo ndk`".to_string(),
        }),
    }
}

/// build ساده و هاست‌بومی (بدون cargo-ndk، بدون پروفایل release) — فقط
/// برای `uniffi-bindgen`، نه برای توزیع در APK. دو دلیل جدا، هر دو روی
/// دستگاه واقعی پیدا شدند:
/// (۱) uniffi-bindgen در «library mode» با `dlopen` واقعیِ همین هاست
///     کتابخانه را می‌خواند؛ یک `.so`ی کراس‌کامپایل‌شده برای
///     اندروید/Bionic با `dlopen` لینوکسی/glibc اصلاً قابل‌لود نیست.
/// (۲) حتی با یک `.so`ی هاست‌بومی درست، اگر با تنظیمات پروفایل
///     release این پروژه (`lto = true`, `codegen-units = 1`,
///     `strip = "symbols"`) ساخته شود، سکشن‌های متادیتای uniffi
///     (`UNIFFI_META_*`) در حین LTO/بازآرایی خراب می‌شوند — سمبل‌ها در
///     `nm -D` هنوز دیده می‌شوند، ولی `uniffi-bindgen print-repr`
///     خروجی خالی `[]` می‌دهد. با پروفایل debug ساده (بدون این
///     تنظیمات)، متادیتا کامل و سالم خوانده می‌شود؛ تأیید عملی روی
///     دستگاه واقعی.
/// A plain, host-native build (no cargo-ndk, no release profile) —
/// for `uniffi-bindgen` only, never for shipping in the APK. Two
/// separate causes, both found on a real device:
/// (1) uniffi-bindgen in "library mode" does a real `dlopen` on this
///     same host; an Android/Bionic cross-compiled `.so` simply
///     cannot be loaded by a Linux/glibc `dlopen`.
/// (2) even with a correct host-native `.so`, if built with this
///     project's release profile settings (`lto = true`,
///     `codegen-units = 1`, `strip = "symbols"`), uniffi's metadata
///     sections (`UNIFFI_META_*`) get corrupted during
///     LTO/reordering — the symbols still show up in `nm -D`, but
///     `uniffi-bindgen print-repr` returns an empty `[]`. A plain
///     debug profile (none of those settings) reads the metadata
///     fully and correctly — confirmed live on a real device.
pub fn build_host_library(
    runner: &dyn CommandRunner,
    workspace_root: &Path,
    package: &str,
) -> Result<PathBuf, NativeBuildError> {
    let args = ["build", "-p", package];

    match runner.run_in(workspace_root, "cargo", &args) {
        Some(out) if out.success => {
            let file_name = format!(
                "{}{}.{}",
                std::env::consts::DLL_PREFIX,
                package.replace('-', "_"),
                std::env::consts::DLL_EXTENSION
            );
            Ok(workspace_root.join("target").join("debug").join(file_name))
        }
        Some(out) => Err(NativeBuildError::BuildFailed {
            package: package.to_string(),
            reason: if out.stderr.is_empty() {
                out.stdout
            } else {
                out.stderr
            },
        }),
        None => Err(NativeBuildError::BuildFailed {
            package: package.to_string(),
            reason: "could not execute `cargo build` (host library)".to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bana_env_scanner::CommandOutput;
    use std::sync::Mutex;

    /// شبیه‌ساز اجرای دستور که آرگومان‌های واقعی ساخته‌شده را هم ثبت
    /// می‌کند، تا مطمئن شویم دستور نهایی درست ساخته می‌شود. از `Mutex`
    /// استفاده شده، نه `RefCell`، چون `CommandRunner: Send + Sync` است و
    /// `RefCell` این الزام را برآورده نمی‌کند.
    /// Simulates command execution while also recording the actually
    /// built arguments, to confirm the final command is built correctly.
    /// Uses `Mutex`, not `RefCell`, since `CommandRunner: Send + Sync` and
    /// `RefCell` doesn't satisfy that requirement.
    struct MockRunner {
        ndk_available: bool,
        build_should_fail: bool,
        last_call: Mutex<Option<(String, Vec<String>)>>,
    }

    impl CommandRunner for MockRunner {
        fn run(&self, program: &str, args: &[&str]) -> Option<CommandOutput> {
            if program == "cargo" && args == ["ndk", "--version"] {
                return self.ndk_available.then(|| CommandOutput {
                    stdout: "cargo-ndk 4.1.2".to_string(),
                    stderr: String::new(),
                    success: true,
                });
            }
            None
        }

        fn run_in(&self, _cwd: &Path, program: &str, args: &[&str]) -> Option<CommandOutput> {
            *self.last_call.lock().unwrap() = Some((
                program.to_string(),
                args.iter().map(|s| s.to_string()).collect(),
            ));
            Some(CommandOutput {
                stdout: String::new(),
                stderr: if self.build_should_fail {
                    "error[E0433]: failed to resolve".to_string()
                } else {
                    String::new()
                },
                success: !self.build_should_fail,
            })
        }
    }

    #[test]
    fn builds_correct_cargo_ndk_command() {
        let runner = MockRunner {
            ndk_available: true,
            build_should_fail: false,
            last_call: Mutex::new(None),
        };

        let result = build_native_layer(
            &runner,
            Path::new("/home/kali/bimarz"),
            "mobile-core",
            Path::new("/home/kali/bimarz/android/app/src/main/jniLibs"),
            &["arm64-v8a"],
        );

        assert!(result.is_ok());
        let (program, args) = runner.last_call.lock().unwrap().clone().unwrap();
        assert_eq!(program, "cargo");
        assert_eq!(
            args,
            vec![
                "ndk",
                "-t",
                "arm64-v8a",
                "-o",
                "/home/kali/bimarz/android/app/src/main/jniLibs",
                "build",
                "--release",
                "-p",
                "mobile-core",
            ]
        );
    }

    #[test]
    fn includes_a_flag_per_requested_abi() {
        let runner = MockRunner {
            ndk_available: true,
            build_should_fail: false,
            last_call: Mutex::new(None),
        };

        build_native_layer(
            &runner,
            Path::new("/home/kali/bimarz"),
            "mobile-core",
            Path::new("/out"),
            ALL_ABIS,
        )
        .unwrap();

        let (_, args) = runner.last_call.lock().unwrap().clone().unwrap();
        let t_flag_count = args.iter().filter(|a| a.as_str() == "-t").count();
        assert_eq!(t_flag_count, ALL_ABIS.len());
    }

    #[test]
    fn reports_missing_cargo_ndk_clearly() {
        let runner = MockRunner {
            ndk_available: false,
            build_should_fail: false,
            last_call: Mutex::new(None),
        };

        let result = build_native_layer(
            &runner,
            Path::new("/home/kali/bimarz"),
            "mobile-core",
            Path::new("/out"),
            &["arm64-v8a"],
        );

        assert!(matches!(result, Err(NativeBuildError::CargoNdkMissing)));
    }

    #[test]
    fn reports_build_failure_with_real_stderr_reason() {
        let runner = MockRunner {
            ndk_available: true,
            build_should_fail: true,
            last_call: Mutex::new(None),
        };

        match build_native_layer(
            &runner,
            Path::new("/home/kali/bimarz"),
            "mobile-core",
            Path::new("/out"),
            &["arm64-v8a"],
        ) {
            Err(NativeBuildError::BuildFailed { reason, .. }) => {
                assert!(reason.contains("E0433"));
            }
            other => panic!("expected BuildFailed, got {other:?}"),
        }
    }

    #[test]
    fn host_library_uses_plain_debug_cargo_build_not_ndk() {
        // دو باگ واقعی، هر دو روی دستگاه واقعی پیدا شدند:
        // (۱) uniffi-bindgen به یک .so هاست‌بومی نیاز دارد (dlopen
        //     واقعی می‌کند)، نه یک .so کراس‌کامپایل‌شده برای اندروید —
        //     این تابع نباید هیچ‌وقت `cargo ndk` صدا بزند.
        // (۲) این تابع نباید `--release` بزند — تنظیمات پروفایل
        //     release این پروژه (lto, codegen-units=1, strip) سکشن‌های
        //     متادیتای uniffi را خراب می‌کنند (تأیید عملی: print-repr
        //     با release خروجی خالی می‌دهد، با debug کامل و درست).
        // Two real bugs, both found on a real device:
        // (1) uniffi-bindgen needs a host-native .so (it does a real
        //     dlopen), not an Android-cross-compiled one — this
        //     function must never call `cargo ndk`.
        // (2) this function must never pass `--release` — this
        //     project's release profile settings (lto,
        //     codegen-units=1, strip) corrupt uniffi's metadata
        //     sections (confirmed live: print-repr returns empty with
        //     release, full and correct with debug).
        let runner = MockRunner {
            ndk_available: true,
            build_should_fail: false,
            last_call: Mutex::new(None),
        };

        let path =
            build_host_library(&runner, Path::new("/home/kali/bimarz"), "mobile-core").unwrap();

        let (program, args) = runner.last_call.lock().unwrap().clone().unwrap();
        assert_eq!(program, "cargo");
        assert_eq!(args, vec!["build", "-p", "mobile-core"]);
        assert!(!args.contains(&"ndk".to_string()));
        assert!(!args.contains(&"--release".to_string()));
        assert!(path.ends_with(format!(
            "target/debug/{}mobile_core.{}",
            std::env::consts::DLL_PREFIX,
            std::env::consts::DLL_EXTENSION
        )));
    }

    #[test]
    fn host_library_reports_failure_with_real_reason() {
        let runner = MockRunner {
            ndk_available: true,
            build_should_fail: true,
            last_call: Mutex::new(None),
        };

        match build_host_library(&runner, Path::new("/home/kali/bimarz"), "mobile-core") {
            Err(NativeBuildError::BuildFailed { reason, .. }) => {
                assert!(reason.contains("E0433"));
            }
            other => panic!("expected BuildFailed, got {other:?}"),
        }
    }
}
