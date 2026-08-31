//! تولید واقعی Kotlin bindings با `uniffi-bindgen` — دقیقاً همان الگوی
//! خودِ بی‌مرز (`cargo run -p mobile-core --bin uniffi-bindgen --features
//! cli`)، با «library mode» (بدون نیاز به فایل UDL، مستقیم از روی `.so`
//! کامپایل‌شده که مرحله‌ی native قبلی تولید کرده).
//! Real Kotlin binding generation with `uniffi-bindgen` — the exact same
//! pattern bimarz itself uses (`cargo run -p mobile-core --bin
//! uniffi-bindgen --features cli`), using "library mode" (no UDL file
//! needed, straight from the `.so` the previous native step produced).

use bana_env_scanner::CommandRunner;
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum BindgenError {
    #[error("uniffi-bindgen generation failed for library '{library}': {reason}")]
    GenerationFailed { library: String, reason: String },
}

/// اجرای واقعی `cargo run -p <bindgen_package> --bin uniffi-bindgen
/// --features cli -- generate --library <library_path> --language kotlin
/// --out-dir <out_dir>` داخل `workspace_root`.
/// Actually runs `cargo run -p <bindgen_package> --bin uniffi-bindgen
/// --features cli -- generate --library <library_path> --language kotlin
/// --out-dir <out_dir>` inside `workspace_root`.
pub fn generate_kotlin_bindings(
    runner: &dyn CommandRunner,
    workspace_root: &Path,
    bindgen_package: &str,
    library_path: &Path,
    out_dir: &Path,
) -> Result<(), BindgenError> {
    let args: Vec<String> = vec![
        "run".to_string(),
        "-p".to_string(),
        bindgen_package.to_string(),
        "--bin".to_string(),
        "uniffi-bindgen".to_string(),
        "--features".to_string(),
        "cli".to_string(),
        "--".to_string(),
        "generate".to_string(),
        "--library".to_string(),
        library_path.to_string_lossy().to_string(),
        "--language".to_string(),
        "kotlin".to_string(),
        "--out-dir".to_string(),
        out_dir.to_string_lossy().to_string(),
    ];
    let args_ref: Vec<&str> = args.iter().map(String::as_str).collect();

    match runner.run_in(workspace_root, "cargo", &args_ref) {
        Some(out) if out.success => Ok(()),
        Some(out) => Err(BindgenError::GenerationFailed {
            library: library_path.to_string_lossy().to_string(),
            reason: if out.stderr.is_empty() {
                out.stdout
            } else {
                out.stderr
            },
        }),
        None => Err(BindgenError::GenerationFailed {
            library: library_path.to_string_lossy().to_string(),
            reason: "could not execute `cargo run ... uniffi-bindgen`".to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bana_env_scanner::CommandOutput;
    use std::sync::Mutex;

    /// از `Mutex` استفاده شده، نه `RefCell` — درسی که از باگ واقعی
    /// `native.rs` گرفتیم؛ `CommandRunner: Send + Sync` است.
    /// Uses `Mutex`, not `RefCell` — a lesson learned from `native.rs`'s
    /// real bug; `CommandRunner: Send + Sync`.
    struct MockRunner {
        should_fail: bool,
        last_call: Mutex<Option<Vec<String>>>,
    }

    impl CommandRunner for MockRunner {
        fn run(&self, _program: &str, _args: &[&str]) -> Option<CommandOutput> {
            None
        }

        fn run_in(&self, _cwd: &Path, _program: &str, args: &[&str]) -> Option<CommandOutput> {
            *self.last_call.lock().unwrap() = Some(args.iter().map(|s| s.to_string()).collect());
            Some(CommandOutput {
                stdout: String::new(),
                stderr: if self.should_fail {
                    "error: no UniFFI-decorated items found".to_string()
                } else {
                    String::new()
                },
                success: !self.should_fail,
            })
        }
    }

    #[test]
    fn builds_correct_uniffi_bindgen_command() {
        let runner = MockRunner {
            should_fail: false,
            last_call: Mutex::new(None),
        };

        let result = generate_kotlin_bindings(
            &runner,
            Path::new("/home/kali/bimarz"),
            "mobile-core",
            Path::new("/home/kali/bimarz/target/aarch64-linux-android/release/libmobile_core.so"),
            Path::new("/home/kali/bimarz/android/app/src/main/kotlin/ir/bimarz/app/uniffi"),
        );

        assert!(result.is_ok());
        let args = runner.last_call.lock().unwrap().clone().unwrap();
        assert_eq!(
            args,
            vec![
                "run",
                "-p",
                "mobile-core",
                "--bin",
                "uniffi-bindgen",
                "--features",
                "cli",
                "--",
                "generate",
                "--library",
                "/home/kali/bimarz/target/aarch64-linux-android/release/libmobile_core.so",
                "--language",
                "kotlin",
                "--out-dir",
                "/home/kali/bimarz/android/app/src/main/kotlin/ir/bimarz/app/uniffi",
            ]
        );
    }

    #[test]
    fn reports_generation_failure_with_real_stderr_reason() {
        let runner = MockRunner {
            should_fail: true,
            last_call: Mutex::new(None),
        };

        match generate_kotlin_bindings(
            &runner,
            Path::new("/home/kali/bimarz"),
            "mobile-core",
            Path::new("/out/libmobile_core.so"),
            Path::new("/out/kotlin"),
        ) {
            Err(BindgenError::GenerationFailed { reason, .. }) => {
                assert!(reason.contains("no UniFFI-decorated items"));
            }
            other => panic!("expected GenerationFailed, got {other:?}"),
        }
    }
}
