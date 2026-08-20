//! انتزاع اجرای دستور خارجی — دقیقاً به همان دلیل `EnvProbe`: تست‌پذیری
//! کامل بدون نیاز به وجود واقعی هر ابزار (java، gradle، ...) روی دستگاه تست.
//! Abstraction over running external commands — for the exact same reason
//! as `EnvProbe`: full testability without every tool (java, gradle, ...)
//! actually needing to exist on the test machine.

use std::process::Command;

#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
}

/// اجرای یک دستور و گرفتن خروجی آن؛ `None` یعنی خودِ دستور اصلاً پیدا/اجرا نشد.
/// Runs a command and captures its output; `None` means the command itself
/// couldn't even be found/executed.
pub trait CommandRunner: Send + Sync {
    fn run(&self, program: &str, args: &[&str]) -> Option<CommandOutput>;
}

/// پیاده‌سازی واقعی که مستقیم روی سیستم عامل واقعی دستور اجرا می‌کند.
/// The real implementation, actually executing commands on the real OS.
pub struct RealCommandRunner;

impl CommandRunner for RealCommandRunner {
    fn run(&self, program: &str, args: &[&str]) -> Option<CommandOutput> {
        let output = Command::new(program).args(args).output().ok()?;
        Some(CommandOutput {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            success: output.status.success(),
        })
    }
}
