//! نصب/پچ ابزارهای گمشده‌ی توچین اندروید، به‌صورت idempotent و بدون حدس.
//! Installs/patches missing Android toolchain pieces, idempotently and without guessing.
//!
//! فاز فعلی: trait `PackageBackend` و ۷ پیاده‌سازی (apt، pacman، yay، pkg،
//! winget، choco، brew) پیاده‌سازی شدند، طبق بند ۵.۱ RULES.md. کاتالوگ
//! Bundled Tier (`bundled.rs`) با نام پکیج واقعی و منبع‌دار برای JDK و
//! Android SDK شروع شد. باقی‌مانده: منطق `bana setup`، On-Demand Tier، و
//! کش content-addressed طبق ادامه‌ی چک‌لیست فاز ۲ در docs/AGENTS.md.
//! Current phase: the `PackageBackend` trait and 7 implementations (apt,
//! pacman, yay, pkg, winget, choco, brew) are implemented, per RULES.md
//! section 5.1. The Bundled Tier catalog (`bundled.rs`) has started, with
//! real, sourced package names for JDK and Android SDK. Remaining: `bana
//! setup` logic, the On-Demand Tier, and the content-addressed cache, per
//! the rest of the Phase 2 checklist in docs/AGENTS.md.

mod backend;
mod bundled;
mod cache;
mod error;
mod recorder;

pub use backend::{
    select_backend, AptBackend, ChocoBackend, HomebrewBackend, PackageBackend, PacmanBackend,
    TermuxPkgBackend, WingetBackend, YayBackend,
};
pub use bundled::{install_bundled_tool, package_name_for, BundledToolSpec, ANDROID_SDK, JDK};
pub use cache::{cache_dir_for, compute_cache_key, default_cache_root, ensure_cache_dir};
pub use error::ToolchainError;
pub use recorder::{current_unix_timestamp, InstallRecorder, RealInstallRecorder};

/// تابع اتصال، فقط برای تأیید build/link صحیح workspace در فاز ۰.
/// Wiring sanity check only, to confirm the workspace builds/links in Phase 0.
pub fn ping() -> &'static str {
    "pong"
}
