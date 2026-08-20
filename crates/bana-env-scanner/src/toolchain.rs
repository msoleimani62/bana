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
use crate::jdk::detect_jdk;
use bana_types::{AndroidToolchainReport, JdkInfo, ToolStatus};
use std::sync::Arc;

enum ProbeResult {
    Jdk(ToolStatus<JdkInfo>),
    // پروب‌های بعدی (Sdk, Ndk, Aapt2, GradleWrapper) طبق چک‌لیست فاز ۱ اینجا
    // اضافه می‌شوند.
    // Later probes (Sdk, Ndk, Aapt2, GradleWrapper) get added here per the
    // rest of the Phase 1 checklist.
}

/// اسکن کامل و موازی توچین اندروید. فعلاً فقط JDK پیاده‌سازی شده است.
/// Full, parallel Android toolchain scan. Only JDK is implemented so far.
pub async fn scan_toolchain() -> AndroidToolchainReport {
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

    let mut report = AndroidToolchainReport {
        jdk: ToolStatus::NotFound,
    };

    while let Some(result) = tasks.join_next().await {
        match result {
            Ok(ProbeResult::Jdk(status)) => report.jdk = status,
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
