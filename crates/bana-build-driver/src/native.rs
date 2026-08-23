//! اجرای واقعی build لایه‌ی native با `cargo-ndk`، برای همه‌ی ABIهای
//! موردنیاز — دقیقاً معادل مکانیزه‌ی مرحله‌ی ۶-۷ سند تشخیص بی‌مرز
//! (`scripts/build-android.sh`).
//! Real native-layer build via `cargo-ndk`, for every required ABI —
//! the mechanized equivalent of the bimarz detection doc's step 6-7
//! (`scripts/build-android.sh`).

use bana_env_scanner::CommandRunner;
use std::path::Path;

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
            reason: if out.stderr.is_empty() { out.stdout } else { out.stderr },
        }),
        None => Err(NativeBuildError::BuildFailed {
            package: package.to_string(),
            reason: "could not execute `cargo ndk`".to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bana_env_scanner::CommandOutput;
    use std::cell::RefCell;

    /// شبیه‌ساز اجرای دستور که آرگومان‌های واقعی ساخته‌شده را هم ثبت
    /// می‌کند، تا مطمئن شویم دستور نهایی درست ساخته می‌شود.
    /// Simulates command execution while also recording the actually
    /// built arguments, to confirm the final command is built correctly.
    struct MockRunner {
        ndk_available: bool,
        build_should_fail: bool,
        last_call: RefCell<Option<(String, Vec<String>)>>,
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
            *self.last_call.borrow_mut() = Some((
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
            last_call: RefCell::new(None),
        };

        let result = build_native_layer(
            &runner,
            Path::new("/home/kali/bimarz"),
            "mobile-core",
            Path::new("/home/kali/bimarz/android/app/src/main/jniLibs"),
            &["arm64-v8a"],
        );

        assert!(result.is_ok());
        let (program, args) = runner.last_call.borrow().clone().unwrap();
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
            last_call: RefCell::new(None),
        };

        build_native_layer(
            &runner,
            Path::new("/home/kali/bimarz"),
            "mobile-core",
            Path::new("/out"),
            ALL_ABIS,
        )
        .unwrap();

        let (_, args) = runner.last_call.borrow().clone().unwrap();
        let t_flag_count = args.iter().filter(|a| a.as_str() == "-t").count();
        assert_eq!(t_flag_count, ALL_ABIS.len());
    }

    #[test]
    fn reports_missing_cargo_ndk_clearly() {
        let runner = MockRunner {
            ndk_available: false,
            build_should_fail: false,
            last_call: RefCell::new(None),
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
            last_call: RefCell::new(None),
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
}
