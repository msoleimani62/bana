//! تشخیص سناریوی پروژه از میان پیاده‌سازی‌های ثبت‌شده‌ی ProjectScenario.
//! Detects the project's scenario among registered ProjectScenario implementations.
//!
//! فاز فعلی: دو سناریوی v1 (`pure-kotlin`، `hybrid-rust-uniffi`) و رجیستری
//! انتخاب بهترین تطابق پیاده‌سازی شدند، طبق چک‌لیست فاز ۳ در docs/AGENTS.md.
//! Current phase: the two v1 scenarios (`pure-kotlin`,
//! `hybrid-rust-uniffi`) and the best-match registry are implemented, per
//! the Phase 3 checklist in docs/AGENTS.md.

mod hybrid_rust_uniffi;
mod pure_kotlin;
mod registry;

pub use hybrid_rust_uniffi::HybridRustUniffiScenario;
pub use pure_kotlin::PureKotlinScenario;
pub use registry::analyze_project;

/// تابع اتصال، فقط برای تأیید build/link صحیح workspace در فاز ۰.
/// Wiring sanity check only, to confirm the workspace builds/links in Phase 0.
pub fn ping() -> &'static str {
    "pong"
}
