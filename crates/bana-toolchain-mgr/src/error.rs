//! خطاهای typed لایه‌ی toolchain_mgr، طبق استاندارد thiserror در کریت‌های
//! کتابخانه‌ای (بخش ۶ RULES.md).
//! Typed errors for the toolchain_mgr layer, per the thiserror-in-library-
//! crates standard (RULES.md section 6).

use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum ToolchainError {
    #[error("no supported package backend found for this host")]
    NoBackendAvailable,

    #[error("installing '{package}' via {backend} failed: {reason}")]
    InstallFailed {
        backend: &'static str,
        package: String,
        reason: String,
    },

    #[error("cache directory '{path}' could not be prepared: {reason}")]
    CacheUnavailable { path: PathBuf, reason: String },

    #[error(
        "no confirmed package name for tool '{tool}' on backend '{backend}' \
         (not researched yet, never guessed)"
    )]
    NoPackageForBackend {
        tool: &'static str,
        backend: &'static str,
    },
}
