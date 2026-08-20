//! تنها مرز PyO3 در bana — هیچ کریت منطقی دیگری این ماژول را نمی‌بیند.
//! The sole PyO3 boundary in bana — no other logical crate sees this module.
//!
//! فاز فعلی: فقط اتصال تست ping/pong برای تأیید صحت زنجیره‌ی
//! maturin → bana-ffi → bana-py. توابع واقعی طبق پیشرفت فازهای ۱ تا ۵ به
//! این‌جا اضافه می‌شوند.
//! Current phase: only a ping/pong wiring check to confirm the
//! maturin → bana-ffi → bana-py chain works. Real functions get added here
//! as Phases 1-5 progress.

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

/// تابع تست ساده که از پایتون قابل فراخوانی است.
/// Simple test function callable from Python.
#[pyfunction]
fn ping() -> PyResult<String> {
    Ok(bana_env_scanner::ping().to_string())
}

/// تشخیص واقعی محیط میزبان و بازگرداندن آن به‌صورت JSON برای پایتون.
/// خطای سریالایز عملاً غیرممکن است (نوع‌های ساده)، ولی هر خطایی به یک
/// PyRuntimeError واضح تبدیل می‌شود، نه یک panic خام.
///
/// Real host-environment detection, returned as JSON for Python. Serialize
/// failure is practically impossible (plain data types), but any error is
/// turned into a clear PyRuntimeError rather than a raw panic.
#[pyfunction]
fn scan_host() -> PyResult<String> {
    let probe = bana_env_scanner::RealEnvProbe;
    let env = bana_env_scanner::detect_host_environment(&probe);
    serde_json::to_string(&env)
        .map_err(|e| PyRuntimeError::new_err(format!("host scan serialize failed: {e}")))
}

/// نقطه‌ی ثبت ماژول پایتون؛ نام باید با [lib].name در Cargo.toml یکی باشد.
/// Python module entry point; name must match [lib].name in Cargo.toml.
#[pymodule]
fn _bana_ffi(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(ping, m)?)?;
    m.add_function(wrap_pyfunction!(scan_host, m)?)?;
    Ok(())
}
