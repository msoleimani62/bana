//! اجرای واقعی فرایند build بر اساس ProjectFingerprint، بدون دخالت کاربر.
//! Drives the real build process from a ProjectFingerprint, with zero user interaction.
//!
//! فاز فعلی: اجرای واقعی لایه‌ی native با cargo-ndk و تولید Kotlin bindings
//! با uniffi-bindgen پیاده‌سازی شدند. بقیه‌ی چک‌لیست (Gradle wrapper، پچ
//! AAPT2، gradlew) طبق فاز ۴ در docs/AGENTS.md ادامه پیدا می‌کند.
//! Current phase: real native-layer build via cargo-ndk and Kotlin
//! binding generation via uniffi-bindgen are implemented. The rest of the
//! checklist (Gradle wrapper, AAPT2 patch, gradlew) continues per Phase 4
//! in docs/AGENTS.md.

mod bindgen;
mod native;

pub use bindgen::{generate_kotlin_bindings, BindgenError};
pub use native::{build_native_layer, NativeBuildError, ALL_ABIS};

/// تابع اتصال، فقط برای تأیید build/link صحیح workspace در فاز ۰.
/// Wiring sanity check only, to confirm the workspace builds/links in Phase 0.
pub fn ping() -> &'static str {
    "pong"
}
