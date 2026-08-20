//! تشخیص واقعی JDK — هیچ‌وقت مسیر یا نسخه حدس زده نمی‌شود، همیشه از خروجی
//! واقعی `java -version` استخراج می‌شود.
//! Real JDK detection — the path or version is never guessed, always
//! extracted from the real `java -version` output.

use crate::command::CommandRunner;
use bana_types::{JdkInfo, ToolStatus};
use std::path::PathBuf;

/// اجرای `java -version` و تفسیر خروجی. جاوا حتی در حالت موفق، نسخه را روی
/// stderr چاپ می‌کند نه stdout — این رفتار شناخته‌شده‌ی خودِ JDK است.
/// Runs `java -version` and parses the output. Java prints its version to
/// stderr even on success, not stdout — this is JDK's own known behavior.
pub fn detect_jdk(runner: &dyn CommandRunner) -> ToolStatus<JdkInfo> {
    let Some(output) = runner.run("java", &["-version"]) else {
        return ToolStatus::NotFound;
    };

    let raw = if output.stderr.is_empty() {
        &output.stdout
    } else {
        &output.stderr
    };

    match parse_java_version(raw) {
        Some(version) => ToolStatus::Found {
            // مسیر دقیق باینری در قدم بعدی (جست‌وجوی PATH) تکمیل می‌شود؛
            // فعلاً فقط اسم دستور ثبت می‌شود، نه یک مسیر حدسی.
            // The exact binary path gets filled in by the next step (PATH
            // search); for now only the command name is recorded, never a
            // guessed path.
            path: PathBuf::from("java"),
            info: JdkInfo { version },
            verified: true,
        },
        None => ToolStatus::FoundButIncompatible {
            path: PathBuf::from("java"),
            info: JdkInfo {
                version: "unknown".to_string(),
            },
            reason: "could not parse `java -version` output".to_string(),
        },
    }
}

/// استخراج نسخه از خطی مثل: `openjdk version "21.0.11" 2024-04-16`.
/// Extracts the version from a line like: `openjdk version "21.0.11" 2024-04-16`.
fn parse_java_version(raw: &str) -> Option<String> {
    let line = raw.lines().find(|l| l.contains("version"))?;
    let start = line.find('"')? + 1;
    let rest = &line[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::CommandOutput;

    struct MockRunner {
        stderr: &'static str,
        exists: bool,
    }

    impl CommandRunner for MockRunner {
        fn run(&self, _program: &str, _args: &[&str]) -> Option<CommandOutput> {
            if !self.exists {
                return None;
            }
            Some(CommandOutput {
                stdout: String::new(),
                stderr: self.stderr.to_string(),
                success: true,
            })
        }
    }

    #[test]
    fn detects_real_openjdk_version_string() {
        let runner = MockRunner {
            stderr: "openjdk version \"21.0.11\" 2024-04-16\nOpenJDK Runtime Environment",
            exists: true,
        };
        match detect_jdk(&runner) {
            ToolStatus::Found { info, verified, .. } => {
                assert_eq!(info.version, "21.0.11");
                assert!(verified);
            }
            other => panic!("expected Found, got {other:?}"),
        }
    }

    #[test]
    fn reports_not_found_when_command_missing() {
        let runner = MockRunner {
            stderr: "",
            exists: false,
        };
        assert!(matches!(detect_jdk(&runner), ToolStatus::NotFound));
    }

    #[test]
    fn reports_incompatible_when_output_unparseable() {
        let runner = MockRunner {
            stderr: "some garbage output with no quotes",
            exists: true,
        };
        assert!(matches!(
            detect_jdk(&runner),
            ToolStatus::FoundButIncompatible { .. }
        ));
    }
}
