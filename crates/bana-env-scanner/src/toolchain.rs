//! هماهنگ‌کننده‌ی اسکن موازی همه‌ی ابزارهای توچین، با `tokio::task::JoinSet`.
//! چون هر پروب نوع داده‌ی متفاوتی برمی‌گرداند (JDK، بعداً SDK/NDK/...)، یک
//! `enum ProbeResult` واسط تعریف شده تا `JoinSet` بتواند همه را در یک نوع
//! واحد جمع کند — افزودن پروب جدید یعنی فقط یک واریانت جدید به این enum.
//!
//! Parallel toolchain-scan orchestrator using `tokio::task::JoinSet`. Since
//! each probe returns a different data type (JDK, later SDK/NDK/...), a
//! `ProbeResult` enum bridges them so `JoinSet` can collect everything as
//! one unified type — adding a new probe just means one new enum variant.

use crate::command::{CommandRunner, RealCommandRunner};
use crate::host::EnvProbe;
use crate::jdk::detect_jdk;
use crate::ndk::detect_ndk;
use crate::sdk::detect_sdk;
use bana_types::{AndroidToolchainReport, HostKind, JdkInfo, NdkInfo, SdkInfo, ToolStatus};
use std::sync::Arc;

enum ProbeResult {
    Jdk(ToolStatus<JdkInfo>),
    Sdk(ToolStatus<SdkInfo>),
    Ndk(ToolStatus<NdkInfo>),
    // پروب‌های بعدی (Aapt2, GradleWrapper) طبق چک‌لیست فاز ۱ اینجا اضافه
    // می‌شوند.
    // Later probes (Aapt2, GradleWrapper) get added here per the rest of
    // the Phase 1 checklist.
}

/// اسکن کامل و موازی توچین اندروید. `host_kind` باید از قبل توسط
/// `detect_host_environment` تشخیص داده شده باشد، چون تشخیص SDK به آن
/// وابسته است (مسیرهای پیش‌فرض متفاوت بر اساس میزبان).
/// Full, parallel Android toolchain scan. `host_kind` must already be
/// detected via `detect_host_environment`, since SDK detection depends on
/// it (default paths differ by host).
pub async fn scan_toolchain(
    probe: Arc<dyn EnvProbe + Send + Sync>,
    host_kind: HostKind,
) -> AndroidToolchainReport {
    let runner: Arc<dyn CommandRunner> = Arc::new(RealCommandRunner);
    let mut tasks = tokio::task::JoinSet::new();

    {
        let runner = runner.clone();
        // اجرای دستور خارجی مسدودکننده است، پس روی thread pool مخصوص
        // بلاک‌شدن اجرا می‌شود، نه روی executor async اصلی.
        // Running an external command is blocking, so it runs on the
        // dedicated blocking thread pool, not the main async executor.
        tasks.spawn_blocking(move || ProbeResult::Jdk(detect_jdk(runner.as_ref())));
    }

    {
        let probe = probe.clone();
        let host_kind = host_kind.clone();
        tasks.spawn_blocking(move || ProbeResult::Sdk(detect_sdk(probe.as_ref(), &host_kind)));
    }

    {
        let probe = probe.clone();
        let host_kind = host_kind.clone();
        tasks.spawn_blocking(move || ProbeResult::Ndk(detect_ndk(probe.as_ref(), &host_kind)));
    }

    let mut report = AndroidToolchainReport {
        jdk: ToolStatus::NotFound,
        sdk: ToolStatus::NotFound,
        ndk: ToolStatus::NotFound,
    };

    while let Some(result) = tasks.join_next().await {
        match result {
            Ok(ProbeResult::Jdk(status)) => report.jdk = status,
            Ok(ProbeResult::Sdk(status)) => report.sdk = status,
            Ok(ProbeResult::Ndk(status)) => report.ndk = status,
            // پنیک یا لغو یک تسک نباید کل اسکن را متوقف کند؛ فقط همان ابزار
            // NotFound باقی می‌ماند. لاگ دقیق در فازهای بعدی اضافه می‌شود.
            // A panicked or cancelled task shouldn't stop the whole scan;
            // that one tool just stays NotFound. Proper logging is added
            // in later phases.
            Err(_) => {}
        }
    }

    report
}
