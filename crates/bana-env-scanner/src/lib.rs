//! تشخیص کامل و بدون‌حدس وضعیت محیط و توچین اندروید.
//! Complete, guess-free detection of the host and its Android toolchain.
//!
//! فاز ۱ در حال پیشرفت: `HostEnvironment` واقعی پیاده‌سازی شد (بند اول
//! چک‌لیست فاز ۱). بقیه‌ی چک‌لیست (JDK/SDK/NDK/AAPT2/Gradle) طبق
//! docs/AGENTS.md ادامه پیدا می‌کند.
//! Phase 1 in progress: real `HostEnvironment` detection is implemented
//! (first item of the Phase 1 checklist). The rest (JDK/SDK/NDK/AAPT2/
//! Gradle) continues per docs/AGENTS.md.

mod host;

pub use host::{detect_host_environment, EnvProbe, RealEnvProbe};

/// تابع اتصال، فقط برای تأیید build/link صحیح workspace در فاز ۰.
/// Wiring sanity check only, to confirm the workspace builds/links in Phase 0.
pub fn ping() -> &'static str {
    "pong"
}
