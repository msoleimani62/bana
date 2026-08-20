//! تشخیص کامل و بدون‌حدس وضعیت محیط و توچین اندروید.
//! Complete, guess-free detection of the host and its Android toolchain.
//!
//! فاز ۱ در حال پیشرفت: `HostEnvironment` و تشخیص واقعی JDK پیاده‌سازی
//! شدند. بقیه‌ی چک‌لیست (SDK/NDK/AAPT2/Gradle) طبق docs/AGENTS.md ادامه
//! پیدا می‌کند.
//! Phase 1 in progress: `HostEnvironment` and real JDK detection are
//! implemented. The rest of the checklist (SDK/NDK/AAPT2/Gradle) continues
//! per docs/AGENTS.md.

mod command;
mod host;
mod jdk;
mod toolchain;

pub use command::{CommandOutput, CommandRunner, RealCommandRunner};
pub use host::{detect_host_environment, EnvProbe, RealEnvProbe};
pub use jdk::detect_jdk;
pub use toolchain::scan_toolchain;

/// تابع اتصال، فقط برای تأیید build/link صحیح workspace در فاز ۰.
/// Wiring sanity check only, to confirm the workspace builds/links in Phase 0.
pub fn ping() -> &'static str {
    "pong"
}
