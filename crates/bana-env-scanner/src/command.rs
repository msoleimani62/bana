//! انتزاع اجرای دستور خارجی — دقیقاً به همان دلیل `EnvProbe`: تست‌پذیری
//! کامل بدون نیاز به وجود واقعی هر ابزار (java، gradle، ...) روی دستگاه تست.
//! Abstraction over running external commands — for the exact same reason
//! as `EnvProbe`: full testability without every tool (java, gradle, ...)
//! actually needing to exist on the test machine.

use std::path::Path;
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

    /// مثل `run`، ولی با تعیین صریح دایرکتوری کاری — لازم برای دستورهایی
    /// مثل `cargo` که باید داخل ریشه‌ی پروژه اجرا شوند. پیاده‌سازی پیش‌فرض
    /// دایرکتوری کاری را نادیده می‌گیرد و فقط `run` را صدا می‌زند، تا
    /// Mockهای موجود در بقیه‌ی پروژه (که به cwd کاری ندارند) نشکنند؛
    /// `RealCommandRunner` این متد را به‌درستی override می‌کند.
    /// Like `run`, but with an explicit working directory — needed for
    /// commands like `cargo` that must run inside the project root. The
    /// default implementation ignores the working directory and just
    /// calls `run`, so existing mocks elsewhere in the project (which
    /// don't care about cwd) keep compiling; `RealCommandRunner` properly
    /// overrides this.
    fn run_in(&self, _cwd: &Path, program: &str, args: &[&str]) -> Option<CommandOutput> {
        self.run(program, args)
    }
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

    fn run_in(&self, cwd: &Path, program: &str, args: &[&str]) -> Option<CommandOutput> {
        let output = Command::new(program).args(args).current_dir(cwd).output().ok()?;
        Some(CommandOutput {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            success: output.status.success(),
        })
    }
}
