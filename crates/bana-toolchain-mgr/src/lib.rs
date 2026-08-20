//! نصب/پچ ابزارهای گمشده‌ی توچین اندروید، به‌صورت idempotent و بدون حدس.
//! Installs/patches missing Android toolchain pieces, idempotently and without guessing.
//!
//! فاز فعلی: فقط اسکلت. لایه‌بندی Bundled/On-Demand طبق بخش ۵ RULES.md و
//! چک‌لیست فاز ۲ در docs/AGENTS.md پیاده‌سازی می‌شود.
//! Current phase: skeleton only. The Bundled/On-Demand split from RULES.md
//! section 5 and the Phase 2 checklist in docs/AGENTS.md will be implemented next.

/// تابع اتصال، فقط برای تأیید build/link صحیح workspace در فاز ۰.
/// Wiring sanity check only, to confirm the workspace builds/links in Phase 0.
pub fn ping() -> &'static str {
    "pong"
}
