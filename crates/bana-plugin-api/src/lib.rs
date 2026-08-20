//! نقطه‌ی رسمی توسعه‌ی bana: هر سناریوی نوع پروژه این trait را پیاده می‌کند.
//! bana's official extension point: every project scenario implements this trait.

use bana_types::ProjectFingerprint;
use std::path::Path;

/// یک سناریوی شناخته‌شده‌ی پروژه (مثلاً Pure Kotlin یا Hybrid Rust/uniffi).
/// افزودن سناریوی جدید یعنی نوشتن یک پیاده‌سازی جدید از همین trait —
/// هیچ کریت دیگری (project_analyzer، build_driver) نباید برای این کار تغییر کند.
///
/// A recognized project scenario (e.g. Pure Kotlin or Hybrid Rust/uniffi).
/// Adding a new scenario means writing a new implementation of this trait —
/// no other crate (project_analyzer, build_driver) should need to change.
pub trait ProjectScenario {
    /// شناسه‌ی یکتای این سناریو، مثل "pure-kotlin" یا "hybrid-rust-uniffi".
    /// Unique identifier for this scenario, e.g. "pure-kotlin" or "hybrid-rust-uniffi".
    fn scenario_id(&self) -> &'static str;

    /// آیا این سناریو با پروژه‌ی داده‌شده مطابقت دارد؛ اگر بله چقدر مطمئن است.
    /// Whether this scenario matches the given project root, and how confident.
    fn detect(&self, project_root: &Path) -> Option<f32>;

    /// ساخت اثرانگشت کامل، فقط پس از تأیید detect().
    /// Build the full fingerprint, only after detect() has confirmed a match.
    fn fingerprint(&self, project_root: &Path) -> ProjectFingerprint;
}
