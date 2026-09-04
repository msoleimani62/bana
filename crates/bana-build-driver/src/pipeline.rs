//! زنجیر کردن کل pipeline build — از تشخیص سناریو تا APK خام. طبق دامنه‌ی
//! v1 (بخش ۴ RULES.md)، فقط سناریوی `hybrid-rust-uniffi` پشتیبانی
//! می‌شود؛ بقیه با خطای صریح رد می‌شوند، نه رفتار حدسی یا سکوت.
//!
//! **فرض صریح v1 (مستند، نه حدس پنهان):** ماژول Gradle داخل `android/`
//! همیشه `app` نام‌گذاری شده — دقیقاً مطابق ساختار واقعی بی‌مرز. تعمیم به
//! نام‌های دیگر (خواندن از `settings.gradle.kts`) یک بهبود آینده است، نه
//! بخشی از این pipeline.
//! Chains the entire build pipeline — from scenario detection to a raw
//! APK. Per v1 scope (RULES.md section 4), only the `hybrid-rust-uniffi`
//! scenario is supported; anything else is rejected with a clear error,
//! never guessed or silently handled.
//!
//! **Explicit v1 assumption (documented, not a hidden guess):** the
//! Gradle module inside `android/` is always named `app` — exactly
//! matching bimarz's real layout. Generalizing to other names (reading
//! `settings.gradle.kts`) is a future improvement, not part of this
//! pipeline.

use crate::aapt2_patch::{ensure_aapt2_override, Aapt2PatchError, PropertiesWriter};
use crate::bindgen::{generate_kotlin_bindings, BindgenError};
use crate::gradlew::{run_gradlew, BuildVariant, GradleBuildError};
use crate::native::{build_host_library, build_native_layer, NativeBuildError, ALL_ABIS};
use crate::wrapper::{ensure_gradle_wrapper, WrapperError};
use bana_env_scanner::{CommandRunner, EnvProbe};
use bana_project_analyzer::{analyze_project, find_uniffi_bindgen_member};
use bana_types::{HostArch, HostKind};
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    #[error(
        "this project didn't match any known bana scenario; \
         `bana build` only supports recognized project types"
    )]
    UnrecognizedProject,

    #[error(
        "bana build v1 only supports the hybrid-rust-uniffi scenario; \
         detected '{scenario_id}' instead"
    )]
    UnsupportedScenario { scenario_id: String },

    #[error(
        "could not determine which workspace member contains the \
         uniffi-decorated native code"
    )]
    NoUniffiMemberFound,

    #[error(transparent)]
    Wrapper(#[from] WrapperError),

    #[error(transparent)]
    NativeBuild(#[from] NativeBuildError),

    #[error(transparent)]
    Bindgen(#[from] BindgenError),

    #[error(transparent)]
    Aapt2Patch(#[from] Aapt2PatchError),

    #[error(transparent)]
    GradleBuild(#[from] GradleBuildError),
}

/// اجرای کامل pipeline برای یک پروژه‌ی Hybrid؛ در پایان، مسیر APK خروجی
/// را برمی‌گرداند.
/// Runs the full pipeline for a Hybrid project; returns the output APK's
/// path at the end.
pub fn build_hybrid_project(
    probe: &dyn EnvProbe,
    runner: &dyn CommandRunner,
    props: &dyn PropertiesWriter,
    host_kind: &HostKind,
    host_arch: &HostArch,
    repo_root: &Path,
    variant: BuildVariant,
) -> Result<PathBuf, PipelineError> {
    // ۱. تأیید سناریو — هرگز فرض نمی‌شود.
    // 1. Confirm the scenario — never assumed.
    let fingerprint =
        analyze_project(probe, repo_root).ok_or(PipelineError::UnrecognizedProject)?;
    if fingerprint.scenario_id != "hybrid-rust-uniffi" {
        return Err(PipelineError::UnsupportedScenario {
            scenario_id: fingerprint.scenario_id,
        });
    }

    let android_root = repo_root.join("android");
    let app_module_root = android_root.join("app");
    let jni_libs_out = app_module_root.join("src").join("main").join("jniLibs");
    let kotlin_out = app_module_root.join("src").join("main").join("kotlin");

    // ۲. تعیین عضو workspace حاوی uniffi — هرگز حدس زده نمی‌شود.
    // 2. Determine the uniffi-bearing workspace member — never guessed.
    let bindgen_package =
        find_uniffi_bindgen_member(probe, repo_root).ok_or(PipelineError::NoUniffiMemberFound)?;

    // ۳. Gradle wrapper (idempotent، Gradle سیستمی حداکثر یک‌بار).
    // 3. Gradle wrapper (idempotent, system Gradle at most once).
    ensure_gradle_wrapper(probe, runner, &android_root)?;

    // ۴. build لایه‌ی native برای همه‌ی ABIهای استاندارد.
    // 4. Native-layer build for all standard ABIs.
    build_native_layer(runner, repo_root, &bindgen_package, &jni_libs_out, ALL_ABIS)?;

    // ۵. تولید Kotlin bindings — طبق مستندات uniffi، library mode یک
    //    `dlopen` واقعی روی همین هاست انجام می‌دهد، پس باید از یک build
    //    هاست‌بومی (نه .so کراس‌کامپایل‌شده‌ی اندروید که با dlopen
    //    لینوکسی اصلاً لود نمی‌شود) استفاده کند — باگ واقعی، روی دستگاه
    //    واقعی پیدا و رفع شد.
    // 5. Kotlin binding generation — per uniffi's own docs, library mode
    //    does a real `dlopen` on this same host, so it must use a
    //    host-native build (not the Android-cross-compiled .so, which a
    //    Linux dlopen simply cannot load) — real bug, found and fixed on
    //    a real device.
    let host_library_path = build_host_library(runner, repo_root, &bindgen_package)?;
    generate_kotlin_bindings(
        runner,
        repo_root,
        &bindgen_package,
        &host_library_path,
        &kotlin_out,
    )?;

    // ۶. پچ AAPT2 در صورت نیاز (idempotent).
    // 6. AAPT2 patch if needed (idempotent).
    ensure_aapt2_override(probe, runner, props, host_kind, host_arch, &android_root)?;

    // ۷. اجرای واقعی gradlew.
    // 7. Actually running gradlew.
    run_gradlew(runner, &android_root, variant)?;

    let (variant_dir, apk_name) = match variant {
        BuildVariant::Debug => ("debug", "app-debug.apk"),
        BuildVariant::Release => ("release", "app-release-unsigned.apk"),
    };
    Ok(app_module_root
        .join("build")
        .join("outputs")
        .join("apk")
        .join(variant_dir)
        .join(apk_name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bana_env_scanner::CommandOutput;
    use std::collections::HashMap;
    use std::sync::Mutex;

    const EM_AARCH64: u16 = 183;

    fn fake_elf(machine: u16) -> Vec<u8> {
        let mut bytes = vec![0u8; 20];
        bytes[0..4].copy_from_slice(b"\x7fELF");
        bytes[18..20].copy_from_slice(&machine.to_le_bytes());
        bytes
    }

    /// یک محیط شبیه‌سازی‌شده‌ی کامل — دقیقاً ساختار واقعی بی‌مرز — که هر
    /// سه‌تا `EnvProbe`/`CommandRunner`/`PropertiesWriter` را پیاده
    /// می‌کند، برای این‌که کل زنجیره‌ی pipeline را واقعاً تست کنیم، نه
    /// فقط قطعات جدا.
    /// A complete simulated environment — exactly bimarz's real
    /// structure — implementing all three of
    /// `EnvProbe`/`CommandRunner`/`PropertiesWriter`, so the whole
    /// pipeline chain gets tested for real, not just isolated pieces.
    struct FakeEnv {
        files: HashMap<PathBuf, String>,
        existing_paths: Vec<PathBuf>,
        dirs: HashMap<PathBuf, Vec<String>>,
        bytes: HashMap<PathBuf, Vec<u8>>,
        props: Mutex<HashMap<PathBuf, String>>,
        gradlew_success: bool,
        calls: Mutex<Vec<Vec<String>>>,
    }

    impl EnvProbe for FakeEnv {
        fn read_env(&self, _key: &str) -> Option<String> {
            None
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
        fn read_bytes(&self, path: &Path, _max_len: usize) -> Option<Vec<u8>> {
            self.bytes.get(path).cloned()
        }
    }

    impl CommandRunner for FakeEnv {
        fn run(&self, _program: &str, _args: &[&str]) -> Option<CommandOutput> {
            Some(CommandOutput {
                stdout: "ok".to_string(),
                stderr: String::new(),
                success: true,
            })
        }
        fn run_in(&self, _cwd: &Path, program: &str, args: &[&str]) -> Option<CommandOutput> {
            self.calls.lock().unwrap().push(
                std::iter::once(program.to_string())
                    .chain(args.iter().map(|a| a.to_string()))
                    .collect(),
            );
            if program == "./gradlew" {
                return Some(CommandOutput {
                    stdout: String::new(),
                    stderr: if self.gradlew_success {
                        String::new()
                    } else {
                        "build failed".to_string()
                    },
                    success: self.gradlew_success,
                });
            }
            Some(CommandOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            })
        }
    }

    impl PropertiesWriter for FakeEnv {
        fn read(&self, path: &Path) -> Option<String> {
            self.props.lock().unwrap().get(path).cloned()
        }
        fn write(&self, path: &Path, content: &str) -> Result<(), String> {
            self.props
                .lock()
                .unwrap()
                .insert(path.to_path_buf(), content.to_string());
            Ok(())
        }
    }

    fn bimarz_env(gradlew_success: bool) -> FakeEnv {
        let root = PathBuf::from("/home/kali/bimarz");
        let mut files = HashMap::new();
        files.insert(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"engine-core\", \"mobile-core\"]\n".to_string(),
        );
        files.insert(
            root.join("mobile-core").join("Cargo.toml"),
            "[dependencies]\nuniffi = \"0.27\"\n".to_string(),
        );
        files.insert(
            root.join("android")
                .join("gradle")
                .join("wrapper")
                .join("gradle-wrapper.properties"),
            "distributionUrl=https\\://services.gradle.org/distributions/gradle-8.7-bin.zip\n"
                .to_string(),
        );

        let mut existing_paths = vec![
            root.join("Cargo.toml"),
            root.join("mobile-core").join("Cargo.toml"),
            root.join("android").join("settings.gradle.kts"),
            root.join("android").join("gradlew"),
            root.join("android")
                .join("gradle")
                .join("wrapper")
                .join("gradle-wrapper.jar"),
            root.join("android")
                .join("gradle")
                .join("wrapper")
                .join("gradle-wrapper.properties"),
        ];

        let mut dirs = HashMap::new();
        let build_tools_dir = PathBuf::from("/usr/lib/android-sdk/build-tools");
        existing_paths.push(build_tools_dir.clone());
        dirs.insert(build_tools_dir.clone(), vec!["34.0.0".to_string()]);
        let aapt2_path = build_tools_dir.join("34.0.0").join("aapt2");
        existing_paths.push(aapt2_path.clone());

        let mut bytes = HashMap::new();
        bytes.insert(aapt2_path, fake_elf(EM_AARCH64));

        FakeEnv {
            files,
            existing_paths,
            dirs,
            bytes,
            props: Mutex::new(HashMap::new()),
            gradlew_success,
            calls: Mutex::new(Vec::new()),
        }
    }

    #[test]
    fn full_pipeline_succeeds_and_returns_apk_path() {
        let env = bimarz_env(true);
        let root = PathBuf::from("/home/kali/bimarz");

        let apk = build_hybrid_project(
            &env,
            &env,
            &env,
            &HostKind::KaliNetHunterProot,
            &HostArch::Aarch64,
            &root,
            BuildVariant::Debug,
        )
        .unwrap();

        assert_eq!(
            apk,
            root.join("android/app/build/outputs/apk/debug/app-debug.apk")
        );
    }

    #[test]
    fn fails_clearly_when_project_not_recognized() {
        let env = FakeEnv {
            files: HashMap::new(),
            existing_paths: Vec::new(),
            dirs: HashMap::new(),
            bytes: HashMap::new(),
            props: Mutex::new(HashMap::new()),
            gradlew_success: true,
            calls: Mutex::new(Vec::new()),
        };
        let root = PathBuf::from("/home/kali/random-folder");

        let result = build_hybrid_project(
            &env,
            &env,
            &env,
            &HostKind::NativeLinux,
            &HostArch::Aarch64,
            &root,
            BuildVariant::Debug,
        );
        assert!(matches!(result, Err(PipelineError::UnrecognizedProject)));
    }

    #[test]
    fn propagates_real_gradlew_failure() {
        let env = bimarz_env(false);
        let root = PathBuf::from("/home/kali/bimarz");

        let result = build_hybrid_project(
            &env,
            &env,
            &env,
            &HostKind::KaliNetHunterProot,
            &HostArch::Aarch64,
            &root,
            BuildVariant::Debug,
        );
        assert!(matches!(result, Err(PipelineError::GradleBuild(_))));
    }

    #[test]
    fn bindgen_receives_host_native_library_not_android_one() {
        // باگ واقعی، پیدا‌شده روی دستگاه واقعی: قبل از این رفع، اینجا
        // مسیر jniLibs/arm64-v8a (یک .so اندرویدی/Bionic) به bindgen
        // داده می‌شد که با dlopen لینوکسی قابل‌لود نیست و uniffi-bindgen
        // ساکت هیچ فایلی تولید نمی‌کرد.
        // Real bug, found on a real device: before this fix, the
        // jniLibs/arm64-v8a path (an Android/Bionic .so) was handed to
        // bindgen, which a Linux dlopen cannot load, so uniffi-bindgen
        // silently generated nothing.
        let env = bimarz_env(true);
        let root = PathBuf::from("/home/kali/bimarz");

        build_hybrid_project(
            &env,
            &env,
            &env,
            &HostKind::KaliNetHunterProot,
            &HostArch::Aarch64,
            &root,
            BuildVariant::Debug,
        )
        .unwrap();

        let calls = env.calls.lock().unwrap();
        let bindgen_call = calls
            .iter()
            .find(|c| c.contains(&"uniffi-bindgen".to_string()))
            .expect("expected a uniffi-bindgen invocation");
        let library_arg_index = bindgen_call
            .iter()
            .position(|a| a == "--library")
            .expect("expected a --library flag");
        let library_path = &bindgen_call[library_arg_index + 1];
        assert!(
            library_path.contains("target/debug/"),
            "expected a host-native target/debug path (release profile corrupts uniffi metadata), got: {library_path}"
        );
        assert!(
            !library_path.contains("jniLibs"),
            "must not use the Android-cross-compiled jniLibs .so, got: {library_path}"
        );

        let host_build_call = calls.iter().find(|c| {
            c.first().map(String::as_str) == Some("cargo") && c.get(1) == Some(&"build".to_string())
        });
        assert!(
            host_build_call.is_some(),
            "expected a plain `cargo build` (no cargo-ndk) call for the host library"
        );
    }
}
