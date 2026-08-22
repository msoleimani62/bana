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
    let report = runtime.block_on(bana_env_scanner::scan_toolchain(
        probe,
        host.kind,
        host.arch,
    ));
    serde_json::to_string(&report)
        .map_err(|e| PyRuntimeError::new_err(format!("toolchain scan serialize failed: {e}")))
}

/// تشخیص واقعی Gradle wrapper داخل یک پروژه‌ی مشخص و بازگرداندن آن به‌صورت
/// JSON. برخلاف `scan_host`/`scan_toolchain`، sync است چون فقط چند فایل
/// می‌خواند، نیازی به runtime موقت tokio ندارد.
///
/// Real Gradle wrapper detection inside a specific project, returned as
/// JSON. Unlike `scan_host`/`scan_toolchain`, this is sync since it only
/// reads a few files — no throwaway tokio runtime needed.
#[pyfunction]
fn scan_gradle_wrapper(project_root: String) -> PyResult<String> {
    let probe = bana_env_scanner::RealEnvProbe;
    let status =
        bana_env_scanner::detect_gradle_wrapper(&probe, std::path::Path::new(&project_root));
    serde_json::to_string(&status)
        .map_err(|e| PyRuntimeError::new_err(format!("gradle wrapper scan serialize failed: {e}")))
}

/// اجرای idempotent «هرچی لازم است برآورده کن»: هر ابزار کاتالوگ Bundled
/// Tier را چک می‌کند؛ اگر از قبل واقعاً پیدا شده باشد (`Found`)، هیچ
/// نصبی انجام نمی‌شود؛ در غیر این‌صورت backend مناسب انتخاب و نصب واقعی
/// انجام می‌شود، و طبق `InstallRecorder`، رکوردش هم نوشته می‌شود.
/// نتیجه‌ی هر ابزار به‌صورت `SetupAction` جمع‌آوری و JSON برمی‌گردد.
///
/// Idempotent "make sure everything needed is satisfied": checks each
/// Bundled Tier catalog entry; if it's already really found (`Found`), no
/// install happens; otherwise a suitable backend is selected and a real
/// install runs, with its record written per `InstallRecorder`. Each
/// tool's result is collected as a `SetupAction` and returned as JSON.
#[pyfunction]
fn setup_bundled_tools() -> PyResult<String> {
    use bana_toolchain_mgr::{install_bundled_tool, select_backend, RealInstallRecorder, ANDROID_SDK, JDK};
    use bana_types::{SetupAction, ToolStatus};

    let probe: Arc<dyn bana_env_scanner::EnvProbe + Send + Sync> =
        Arc::new(bana_env_scanner::RealEnvProbe);
    let host = bana_env_scanner::detect_host_environment(probe.as_ref());
    let runner = bana_env_scanner::RealCommandRunner;
    let recorder = RealInstallRecorder::under_home(&host.home_dir);

    let mut actions: Vec<SetupAction> = Vec::new();

    // JDK: idempotency واقعی — اگر تشخیص واقعی env_scanner از قبل آن را
    // پیدا کرده باشد، اصلاً به نصب نمی‌رسیم.
    // JDK: real idempotency — if env_scanner's real detection already
    // found it, we never even reach the install step.
    match bana_env_scanner::detect_jdk(&runner) {
        ToolStatus::Found { .. } => actions.push(SetupAction {
            tool_id: "jdk".to_string(),
            outcome: "already_satisfied".to_string(),
            detail: None,
        }),
        _ => match select_backend(&runner, &host.kind) {
            Ok(backend) => {
                let result = install_bundled_tool(&runner, backend.as_ref(), &JDK, &recorder);
                actions.push(SetupAction {
                    tool_id: "jdk".to_string(),
                    outcome: if result.is_ok() { "installed".to_string() } else { "failed".to_string() },
                    detail: Some(match result {
                        Ok(()) => format!("via {}", backend.name()),
                        Err(e) => e.to_string(),
                    }),
                });
            }
            Err(e) => actions.push(SetupAction {
                tool_id: "jdk".to_string(),
                outcome: "no_backend".to_string(),
                detail: Some(e.to_string()),
            }),
        },
    }

    // Android SDK: همان منطق idempotency.
    // Android SDK: the same idempotency logic.
    match bana_env_scanner::detect_sdk(probe.as_ref(), &host.kind) {
        ToolStatus::Found { .. } => actions.push(SetupAction {
            tool_id: "android_sdk".to_string(),
            outcome: "already_satisfied".to_string(),
            detail: None,
        }),
        _ => match select_backend(&runner, &host.kind) {
            Ok(backend) => {
                let result = install_bundled_tool(&runner, backend.as_ref(), &ANDROID_SDK, &recorder);
                actions.push(SetupAction {
                    tool_id: "android_sdk".to_string(),
                    outcome: if result.is_ok() { "installed".to_string() } else { "failed".to_string() },
                    detail: Some(match result {
                        Ok(()) => format!("via {}", backend.name()),
                        Err(e) => e.to_string(),
                    }),
                });
            }
            Err(e) => actions.push(SetupAction {
                tool_id: "android_sdk".to_string(),
                outcome: "no_backend".to_string(),
                detail: Some(e.to_string()),
            }),
        },
    }

    serde_json::to_string(&actions)
        .map_err(|e| PyRuntimeError::new_err(format!("setup result serialize failed: {e}")))
}

/// تشخیص سناریوی یک پروژه‌ی مشخص و بازگرداندن آن به‌صورت JSON. اگر هیچ
/// سناریوی شناخته‌شده‌ای مطابقت نداشت، `null` برمی‌گردد (نه خطا — عدم
/// تطابق یک نتیجه‌ی معتبر است).
/// Detects a specific project's scenario, returned as JSON. If no known
/// scenario matched, returns `null` (not an error — no match is a valid
/// outcome).
#[pyfunction]
fn detect_project_scenario(project_root: String) -> PyResult<String> {
    let probe = bana_env_scanner::RealEnvProbe;
    let fingerprint =
        bana_project_analyzer::analyze_project(&probe, std::path::Path::new(&project_root));
    serde_json::to_string(&fingerprint)
        .map_err(|e| PyRuntimeError::new_err(format!("project scenario serialize failed: {e}")))
}

/// نقطه‌ی ثبت ماژول پایتون؛ نام باید با [lib].name در Cargo.toml یکی باشد.
/// Python module entry point; name must match [lib].name in Cargo.toml.
#[pymodule]
fn _bana_ffi(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(ping, m)?)?;
    m.add_function(wrap_pyfunction!(scan_host, m)?)?;
    m.add_function(wrap_pyfunction!(scan_toolchain, m)?)?;
    m.add_function(wrap_pyfunction!(scan_gradle_wrapper, m)?)?;
    m.add_function(wrap_pyfunction!(setup_bundled_tools, m)?)?;
    m.add_function(wrap_pyfunction!(detect_project_scenario, m)?)?;
    Ok(())
}
