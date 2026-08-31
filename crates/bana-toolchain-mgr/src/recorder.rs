//! ثبت خودکار هر تلاش نصب (موفق یا شکست‌خورده) در یک فایل جداگانه، تا
//! عیب‌یابی داخلی آینده‌ی bana بتواند خودکار بفهمد هر ابزار از کجا و چطور
//! آمده — بدون نیاز به کاوش دستی لاگ‌های خام.
//! Automatically records every install attempt (success or failure) into
//! a separate file, so bana's future internal diagnostics can
//! automatically understand where and how each tool arrived — without
//! manually digging through raw logs.

use crate::error::ToolchainError;
use bana_types::InstallRecord;
use std::path::{Path, PathBuf};

/// انتزاع ثبت رکورد نصب — برای تست‌پذیری کامل، دقیقاً همان دلیل
/// `EnvProbe`/`CommandRunner`.
/// The install-record abstraction — for full testability, the exact same
/// reason as `EnvProbe`/`CommandRunner`.
pub trait InstallRecorder: Send + Sync {
    fn record(&self, record: &InstallRecord) -> Result<PathBuf, ToolchainError>;
}

/// پیاده‌سازی واقعی: هر رکورد را به‌صورت یک فایل JSON مستقل زیر
/// `<home>/.bana/installs/` می‌نویسد.
/// The real implementation: writes each record as an independent JSON
/// file under `<home>/.bana/installs/`.
pub struct RealInstallRecorder {
    pub installs_dir: PathBuf,
}

impl RealInstallRecorder {
    /// ساخت recorder با مسیر استاندارد `<home>/.bana/installs`.
    /// Builds a recorder using the standard `<home>/.bana/installs` path.
    pub fn under_home(home_dir: &Path) -> Self {
        Self {
            installs_dir: home_dir.join(".bana").join("installs"),
        }
    }
}

impl InstallRecorder for RealInstallRecorder {
    fn record(&self, record: &InstallRecord) -> Result<PathBuf, ToolchainError> {
        std::fs::create_dir_all(&self.installs_dir).map_err(|e| {
            ToolchainError::CacheUnavailable {
                path: self.installs_dir.clone(),
                reason: e.to_string(),
            }
        })?;

        let file_name = format!(
            "{}-{}-{}.json",
            record.timestamp_unix, record.tool_id, record.backend_used
        );
        let path = self.installs_dir.join(file_name);

        let json =
            serde_json::to_string_pretty(record).map_err(|e| ToolchainError::CacheUnavailable {
                path: path.clone(),
                reason: e.to_string(),
            })?;
        std::fs::write(&path, json).map_err(|e| ToolchainError::CacheUnavailable {
            path: path.clone(),
            reason: e.to_string(),
        })?;

        Ok(path)
    }
}

/// زمان یونیکس فعلی، برای برچسب‌زدن رکورد نصب.
/// Current Unix timestamp, for tagging the install record.
pub fn current_unix_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// recorder ساختگی که رکوردها را فقط در حافظه نگه می‌دارد، برای تست
/// منطق فراخوانی بدون I/O واقعی. سطح فایل (نه داخل `mod tests`) تا از
/// ماژول‌های دیگر (مثل تست‌های `bundled.rs`) هم در حالت تست قابل‌دسترسی
/// باشد.
/// A fake recorder that only keeps records in memory, for testing
/// call-site logic without real I/O. Kept at file level (not inside
/// `mod tests`) so other modules' tests (like `bundled.rs`'s) can reach it
/// too.
#[cfg(test)]
#[derive(Default)]
pub struct InMemoryRecorder {
    pub records: std::sync::Mutex<Vec<InstallRecord>>,
}

#[cfg(test)]
impl InstallRecorder for InMemoryRecorder {
    fn record(&self, record: &InstallRecord) -> Result<PathBuf, ToolchainError> {
        self.records.lock().unwrap().push(record.clone());
        Ok(PathBuf::from("in-memory"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn real_recorder_writes_a_real_json_file() {
        let tmp = std::env::temp_dir().join(format!("bana-test-{}", current_unix_timestamp()));
        let recorder = RealInstallRecorder::under_home(&tmp);
        let record = InstallRecord {
            tool_id: "jdk".to_string(),
            backend_used: "apt".to_string(),
            package_name: "default-jdk".to_string(),
            timestamp_unix: current_unix_timestamp(),
            success: true,
            failure_reason: None,
        };

        let path = recorder.record(&record).expect("record should succeed");
        assert!(path.exists());
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("default-jdk"));

        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn in_memory_recorder_keeps_records_for_inspection() {
        let recorder = InMemoryRecorder::default();
        let record = InstallRecord {
            tool_id: "android_sdk".to_string(),
            backend_used: "yay".to_string(),
            package_name: "android-sdk".to_string(),
            timestamp_unix: 0,
            success: false,
            failure_reason: Some("network unreachable".to_string()),
        };
        recorder.record(&record).unwrap();
        assert_eq!(recorder.records.lock().unwrap().len(), 1);
    }
}
