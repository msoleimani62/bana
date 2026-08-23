//! اجرای واقعی فرایند build بر اساس ProjectFingerprint، بدون دخالت کاربر.
//! Drives the real build process from a ProjectFingerprint, with zero user interaction.
//!
//! فاز فعلی: اجرای واقعی لایه‌ی native با cargo-ndk پیاده‌سازی شد. بقیه‌ی
//! چک‌لیست (uniffi bindgen، Gradle wrapper، پچ AAPT2، gradlew) طبق فاز ۴
//! در docs/AGENTS.md ادامه پیدا می‌کند.
//! Current phase: real native-layer build via cargo-ndk is implemented.
//! The rest of the checklist (uniffi bindgen, Gradle wrapper, AAPT2
//! patch, gradlew) continues per Phase 4 in docs/AGENTS.md.

mod native;

pub use native::{build_native_layer, NativeBuildError, ALL_ABIS};

/// تابع اتصال، فقط برای تأیید build/link صحیح workspace در فاز ۰.
/// Wiring sanity check only, to confirm the workspace builds/links in Phase 0.
pub fn ping() -> &'static str {
    "pong"
}
