//! تشخیص سناریوی پروژه از میان پیاده‌سازی‌های ثبت‌شده‌ی ProjectScenario.
//! Detects the project's scenario among registered ProjectScenario implementations.
//!
//! فاز فعلی: فقط اسکلت. registry سناریوها و منطق تشخیص طبق چک‌لیست فاز ۳
//! در docs/AGENTS.md پیاده‌سازی می‌شود.
//! Current phase: skeleton only. The scenario registry and detection logic
//! will be implemented per the Phase 3 checklist in docs/AGENTS.md.

/// تابع اتصال، فقط برای تأیید build/link صحیح workspace در فاز ۰.
/// Wiring sanity check only, to confirm the workspace builds/links in Phase 0.
pub fn ping() -> &'static str {
    "pong"
}
