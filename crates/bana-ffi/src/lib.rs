//! تنها مرز PyO3 در bana — هیچ کریت منطقی دیگری این ماژول را نمی‌بیند.
//! The sole PyO3 boundary in bana — no other logical crate sees this module.
//!
//! فاز فعلی: فقط اتصال تست ping/pong برای تأیید صحت زنجیره‌ی
//! maturin → bana-ffi → bana-py. توابع واقعی طبق پیشرفت فازهای ۱ تا ۵ به
//! این‌جا اضافه می‌شوند.
//! Current phase: only a ping/pong wiring check to confirm the
//! maturin → bana-ffi → bana-py chain works. Real functions get added here
//! as Phases 1-5 progress.

use pyo3::prelude::*;

/// تابع تست ساده که از پایتون قابل فراخوانی است.
/// Simple test function callable from Python.
#[pyfunction]
fn ping() -> PyResult<String> {
    Ok(bana_env_scanner::ping().to_string())
}

/// نقطه‌ی ثبت ماژول پایتون؛ نام باید با [lib].name در Cargo.toml یکی باشد.
/// Python module entry point; name must match [lib].name in Cargo.toml.
#[pymodule]
fn _bana_ffi(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(ping, m)?)?;
    Ok(())
}
