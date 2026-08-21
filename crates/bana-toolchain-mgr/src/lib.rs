//! نصب/پچ ابزارهای گمشده‌ی توچین اندروید، به‌صورت idempotent و بدون حدس.
//! Installs/patches missing Android toolchain pieces, idempotently and without guessing.
//!
//! فاز فعلی: trait `PackageBackend` و پیاده‌سازی‌های v1 (apt، pacman، pkg،
//! winget، choco، brew) پیاده‌سازی شدند، طبق بند ۵.۱ RULES.md. لایه‌بندی
//! Bundled/On-Demand (بخش ۵ RULES.md) و کش content-addressed طبق ادامه‌ی
//! چک‌لیست فاز ۲ در docs/AGENTS.md پیاده‌سازی می‌شوند.
//! Current phase: the `PackageBackend` trait and v1 implementations (apt,
//! pacman, pkg, winget, choco, brew) are implemented, per RULES.md section
//! 5.1. The Bundled/On-Demand split (RULES.md section 5) and the
//! content-addressed cache will be implemented per the rest of the Phase 2
//! checklist in docs/AGENTS.md.

mod backend;
mod error;

pub use backend::{
    select_backend, AptBackend, ChocoBackend, HomebrewBackend, PackageBackend, PacmanBackend,
    TermuxPkgBackend, WingetBackend,
};
pub use error::ToolchainError;

/// تابع اتصال، فقط برای تأیید build/link صحیح workspace در فاز ۰.
/// Wiring sanity check only, to confirm the workspace builds/links in Phase 0.
pub fn ping() -> &'static str {
    "pong"
}
