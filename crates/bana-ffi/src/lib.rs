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
use std::sync::Arc;

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

/// تشخیص واقعی توچین اندروید (JDK + SDK) و بازگرداندن آن به‌صورت JSON.
/// چون تشخیص SDK به `HostKind` وابسته است، اول محیط میزبان تشخیص داده
/// می‌شود، سپس به اسکن توچین داده می‌شود. چون این تابع async است ولی PyO3
/// مستقیم async نمی‌پذیرد، یک runtime موقت و سبک tokio ساخته می‌شود؛ فقط
/// برای همین یک فراخوانی زنده می‌ماند.
///
/// Real Android toolchain detection (JDK + SDK), returned as JSON. Since
/// SDK detection depends on `HostKind`, the host environment is detected
/// first, then handed to the toolchain scan. Since the underlying function
/// is async but PyO3 doesn't take async directly, a lightweight, throwaway
/// tokio runtime is built here; it only lives for this single call.
#[pyfunction]
fn scan_toolchain() -> PyResult<String> {
    let probe: Arc<dyn bana_env_scanner::EnvProbe + Send + Sync> =
        Arc::new(bana_env_scanner::RealEnvProbe);
    let host = bana_env_scanner::detect_host_environment(probe.as_ref());

    let runtime = tokio::runtime::Runtime::new()
        .map_err(|e| PyRuntimeError::new_err(format!("failed to start async runtime: {e}")))?;
    let report = runtime.block_on(bana_env_scanner::scan_toolchain(probe, host.kind));
    serde_json::to_string(&report)
        .map_err(|e| PyRuntimeError::new_err(format!("toolchain scan serialize failed: {e}")))
}

/// نقطه‌ی ثبت ماژول پایتون؛ نام باید با [lib].name در Cargo.toml یکی باشد.
/// Python module entry point; name must match [lib].name in Cargo.toml.
#[pymodule]
fn _bana_ffi(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(ping, m)?)?;
    m.add_function(wrap_pyfunction!(scan_host, m)?)?;
    m.add_function(wrap_pyfunction!(scan_toolchain, m)?)?;
    Ok(())
}
