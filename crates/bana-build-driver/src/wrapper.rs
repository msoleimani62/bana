//! ساخت/تأیید Gradle wrapper. طبق مرحله‌ی ۵ سند تشخیص بی‌مرز، Gradle
//! سیستمی فقط همین یک‌بار (برای تولید خودِ wrapper) مجاز به استفاده است؛
//! از این پس پروژه همیشه فقط از طریق wrapper خودش build می‌شود، هرگز
//! دوباره از Gradle سیستمی مستقیم.
//! Ensures the Gradle wrapper exists/is valid. Per the bimarz detection
//! doc's step 5, system Gradle is only ever used this one time (to
//! generate the wrapper itself); from then on the project always builds
//! through its own wrapper, never system Gradle directly again.

use bana_env_scanner::{detect_gradle_wrapper, CommandRunner, EnvProbe};
use bana_types::ToolStatus;
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum WrapperError {
    #[error(
        "system `gradle` is not installed or not on PATH; install it once \
         to generate the wrapper (it won't be needed again afterward)"
    )]
    SystemGradleMissing,

    #[error("`gradle wrapper` failed: {reason}")]
    GenerationFailed { reason: String },

    #[error("wrapper generation ran but the project still has no valid wrapper afterward")]
    StillIncompleteAfterGeneration,
}

/// اطمینان idempotent از وجود Gradle wrapper معتبر: اگر از قبل واقعاً
/// `Found` باشد، هیچ کاری انجام نمی‌شود (حتی به Gradle سیستمی سر هم زده
/// نمی‌شود). فقط در غیر این صورت، Gradle سیستمی یک‌بار برای تولید wrapper
/// صدا زده می‌شود.
/// Idempotently ensures a valid Gradle wrapper exists: if it's already
/// really `Found`, nothing happens at all (system Gradle isn't even
/// touched). Only otherwise is system Gradle called once to generate it.
pub fn ensure_gradle_wrapper(
    probe: &dyn EnvProbe,
    runner: &dyn CommandRunner,
    project_root: &Path,
) -> Result<(), WrapperError> {
    if matches!(
        detect_gradle_wrapper(probe, project_root),
        ToolStatus::Found { .. }
    ) {
        return Ok(());
    }

    if runner.run("gradle", &["--version"]).is_none() {
        return Err(WrapperError::SystemGradleMissing);
    }

    match runner.run_in(project_root, "gradle", &["wrapper"]) {
        Some(out) if out.success => {}
        Some(out) => {
            return Err(WrapperError::GenerationFailed {
                reason: if out.stderr.is_empty() { out.stdout } else { out.stderr },
            })
        }
        None => {
            return Err(WrapperError::GenerationFailed {
                reason: "could not execute `gradle wrapper`".to_string(),
            })
        }
    }

    if matches!(
        detect_gradle_wrapper(probe, project_root),
        ToolStatus::Found { .. }
    ) {
        Ok(())
    } else {
        Err(WrapperError::StillIncompleteAfterGeneration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bana_env_scanner::CommandOutput;
    use std::path::PathBuf;
    use std::sync::Mutex;

    /// یک پروژه‌ی شبیه‌سازی‌شده که هم `EnvProbe` هم `CommandRunner` است؛
    /// وقتی `gradle wrapper` با موفقیت اجرا شود، خودش را «دارای wrapper»
    /// علامت می‌زند — دقیقاً همان چیزی که در دنیای واقعی اتفاق می‌افتد.
    /// A simulated project that is both `EnvProbe` and `CommandRunner`;
    /// once `gradle wrapper` runs successfully, it marks itself as "now
    /// having a wrapper" — exactly what happens in the real world.
    struct FakeProject {
        wrapper_exists: Mutex<bool>,
        gradle_available: bool,
        generation_should_fail: bool,
        generation_attempted: Mutex<bool>,
    }

    const REAL_PROPS: &str =
        "distributionUrl=https\\://services.gradle.org/distributions/gradle-8.7-bin.zip\n";

    impl EnvProbe for FakeProject {
        fn read_env(&self, _key: &str) -> Option<String> {
            None
        }
        fn path_exists(&self, path: &Path) -> bool {
            *self.wrapper_exists.lock().unwrap()
                && (path.ends_with("gradlew") || path.ends_with("gradle-wrapper.jar"))
        }
        fn read_to_string(&self, path: &Path) -> Option<String> {
            if *self.wrapper_exists.lock().unwrap() && path.ends_with("gradle-wrapper.properties") {
                Some(REAL_PROPS.to_string())
            } else {
                None
            }
        }
        fn list_dir(&self, _path: &Path) -> Vec<String> {
            Vec::new()
        }
        fn read_bytes(&self, _path: &Path, _max_len: usize) -> Option<Vec<u8>> {
            None
        }
    }

    impl CommandRunner for FakeProject {
        fn run(&self, program: &str, args: &[&str]) -> Option<CommandOutput> {
            if program == "gradle" && args == ["--version"] {
                return self.gradle_available.then(|| CommandOutput {
                    stdout: "Gradle 8.7".to_string(),
                    stderr: String::new(),
                    success: true,
                });
            }
            None
        }

        fn run_in(&self, _cwd: &Path, program: &str, args: &[&str]) -> Option<CommandOutput> {
            if program == "gradle" && args == ["wrapper"] {
                *self.generation_attempted.lock().unwrap() = true;
                if self.generation_should_fail {
                    return Some(CommandOutput {
                        stdout: String::new(),
                        stderr: "could not resolve distribution URL".to_string(),
                        success: false,
                    });
                }
                *self.wrapper_exists.lock().unwrap() = true;
                return Some(CommandOutput {
                    stdout: String::new(),
                    stderr: String::new(),
                    success: true,
                });
            }
            None
        }
    }

    #[test]
    fn already_satisfied_skips_generation_entirely() {
        let project = FakeProject {
            wrapper_exists: Mutex::new(true),
            gradle_available: false, // عمداً false — نباید اصلاً لازم شود
            generation_should_fail: false,
            generation_attempted: Mutex::new(false),
        };
        let root = PathBuf::from("/home/kali/bimarz/android");

        assert!(ensure_gradle_wrapper(&project, &project, &root).is_ok());
        assert!(!*project.generation_attempted.lock().unwrap());
    }

    #[test]
    fn generates_wrapper_when_missing() {
        let project = FakeProject {
            wrapper_exists: Mutex::new(false),
            gradle_available: true,
            generation_should_fail: false,
            generation_attempted: Mutex::new(false),
        };
        let root = PathBuf::from("/home/kali/bimarz/android");

        assert!(ensure_gradle_wrapper(&project, &project, &root).is_ok());
        assert!(*project.generation_attempted.lock().unwrap());
        assert!(*project.wrapper_exists.lock().unwrap());
    }

    #[test]
    fn reports_missing_system_gradle_clearly() {
        let project = FakeProject {
            wrapper_exists: Mutex::new(false),
            gradle_available: false,
            generation_should_fail: false,
            generation_attempted: Mutex::new(false),
        };
        let root = PathBuf::from("/home/kali/bimarz/android");

        assert!(matches!(
            ensure_gradle_wrapper(&project, &project, &root),
            Err(WrapperError::SystemGradleMissing)
        ));
    }

    #[test]
    fn reports_generation_failure_with_real_reason() {
        let project = FakeProject {
            wrapper_exists: Mutex::new(false),
            gradle_available: true,
            generation_should_fail: true,
            generation_attempted: Mutex::new(false),
        };
        let root = PathBuf::from("/home/kali/bimarz/android");

        match ensure_gradle_wrapper(&project, &project, &root) {
            Err(WrapperError::GenerationFailed { reason }) => {
                assert!(reason.contains("distribution URL"));
            }
            other => panic!("expected GenerationFailed, got {other:?}"),
        }
    }
}
