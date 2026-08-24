//! اجرای واقعی `gradlew assembleDebug`/`assembleRelease` — همیشه از طریق
//! wrapper خودِ پروژه، هرگز Gradle سیستمی مستقیم (طبق اصل ۴ فلسفه‌ی
//! هسته‌ای RULES.md). خطاهای رایج به پیام قابل‌فهم دسته‌بندی می‌شوند، ولی
//! خروجی خام Gradle هم همیشه همراهش نگه داشته می‌شود — هیچ‌وقت اطلاعات
//! پنهان نمی‌شود، فقط خوانا می‌شود.
//! Real `gradlew assembleDebug`/`assembleRelease` execution — always
//! through the project's own wrapper, never system Gradle directly (per
//! RULES.md core philosophy principle 4). Common errors are classified
//! into a readable message, but the raw Gradle output always stays
//! attached — nothing is ever hidden, only made more readable.

use bana_env_scanner::CommandRunner;
use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub enum BuildVariant {
    Debug,
    Release,
}

impl BuildVariant {
    fn gradle_task(self) -> &'static str {
        match self {
            BuildVariant::Debug => "assembleDebug",
            BuildVariant::Release => "assembleRelease",
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GradleBuildError {
    #[error("gradlew could not be executed at {path} (missing or not executable)")]
    GradlewMissing { path: String },

    #[error("{friendly_reason}\n\n--- raw Gradle output ---\n{raw_output}")]
    BuildFailed {
        friendly_reason: String,
        raw_output: String,
    },
}

/// اجرای واقعی `./gradlew <task> --stacktrace` داخل `gradle_project_root`.
/// Actually runs `./gradlew <task> --stacktrace` inside `gradle_project_root`.
pub fn run_gradlew(
    runner: &dyn CommandRunner,
    gradle_project_root: &Path,
    variant: BuildVariant,
) -> Result<(), GradleBuildError> {
    let task = variant.gradle_task();
    match runner.run_in(gradle_project_root, "./gradlew", &[task, "--stacktrace"]) {
        Some(out) if out.success => Ok(()),
        Some(out) => {
            let raw = if out.stderr.is_empty() { out.stdout } else { out.stderr };
            let friendly_reason = classify_gradle_error(&raw);
            Err(GradleBuildError::BuildFailed { friendly_reason, raw_output: raw })
        }
        None => Err(GradleBuildError::GradlewMissing {
            path: gradle_project_root.join("gradlew").to_string_lossy().to_string(),
        }),
    }
}

/// دسته‌بندی خطاهای رایج به یک پیام قابل‌فهم؛ اگر هیچ الگوی شناخته‌شده‌ای
/// پیدا نشود، یک پیام عمومی برمی‌گرداند — خروجی خام همیشه جدا نگه داشته
/// می‌شود، هرگز جایگزین نمی‌شود.
/// Classifies common errors into a readable message; if no known pattern
/// matches, returns a generic message — the raw output is always kept
/// separately, never replaced.
fn classify_gradle_error(raw: &str) -> String {
    let lower = raw.to_lowercase();

    if lower.contains("ndk") && (lower.contains("not configured") || lower.contains("not found")) {
        return "NDK looks missing or misconfigured. Run `bana doctor` to check, then install \
                one via `sdkmanager --install \"ndk;<version>\"`."
            .to_string();
    }
    if lower.contains("no toolchains found") || lower.contains("unsupported class file major version") {
        return "This looks like a JDK version mismatch. Run `bana doctor` to check the \
                detected JDK version against what this Gradle/AGP version expects."
            .to_string();
    }
    if lower.contains("aapt2") {
        return "AAPT2 failed to run — likely an architecture mismatch. Run `bana doctor`; \
                bana should be able to auto-patch this via android.aapt2FromMavenOverride."
            .to_string();
    }

    "Gradle build failed. See the raw output below for details.".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bana_env_scanner::CommandOutput;

    struct MockRunner {
        gradlew_available: bool,
        stderr: &'static str,
        success: bool,
    }

    impl CommandRunner for MockRunner {
        fn run(&self, _program: &str, _args: &[&str]) -> Option<CommandOutput> {
            None
        }
        fn run_in(&self, _cwd: &Path, _program: &str, _args: &[&str]) -> Option<CommandOutput> {
            self.gradlew_available.then(|| CommandOutput {
                stdout: String::new(),
                stderr: self.stderr.to_string(),
                success: self.success,
            })
        }
    }

    #[test]
    fn successful_build_is_ok() {
        let runner = MockRunner {
            gradlew_available: true,
            stderr: "",
            success: true,
        };
        assert!(run_gradlew(&runner, Path::new("/home/kali/bimarz/android"), BuildVariant::Debug).is_ok());
    }

    #[test]
    fn reports_missing_gradlew_clearly() {
        let runner = MockRunner {
            gradlew_available: false,
            stderr: "",
            success: false,
        };
        let result = run_gradlew(&runner, Path::new("/home/kali/bimarz/android"), BuildVariant::Debug);
        assert!(matches!(result, Err(GradleBuildError::GradlewMissing { .. })));
    }

    #[test]
    fn classifies_ndk_error_but_keeps_raw_output() {
        let runner = MockRunner {
            gradlew_available: true,
            stderr: "NDK not configured. Download it using sdkmanager.",
            success: false,
        };
        match run_gradlew(&runner, Path::new("/home/kali/bimarz/android"), BuildVariant::Debug) {
            Err(GradleBuildError::BuildFailed { friendly_reason, raw_output }) => {
                assert!(friendly_reason.contains("NDK"));
                assert!(raw_output.contains("sdkmanager"));
            }
            other => panic!("expected BuildFailed, got {other:?}"),
        }
    }

    #[test]
    fn falls_back_to_generic_message_for_unrecognized_errors() {
        let runner = MockRunner {
            gradlew_available: true,
            stderr: "some totally novel error nobody has seen before",
            success: false,
        };
        match run_gradlew(&runner, Path::new("/home/kali/bimarz/android"), BuildVariant::Debug) {
            Err(GradleBuildError::BuildFailed { friendly_reason, raw_output }) => {
                assert!(friendly_reason.contains("Gradle build failed"));
                assert!(raw_output.contains("novel error"));
            }
            other => panic!("expected BuildFailed, got {other:?}"),
        }
    }
}
