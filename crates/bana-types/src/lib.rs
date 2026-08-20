//! انواع داده‌ی مشترک بین همه‌ی کریت‌های bana — این کریت هیچ منطقی ندارد.
//! Shared data types used across every bana crate — this crate holds no logic.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// نوع محیط میزبانی که bana در آن اجرا می‌شود.
/// The kind of host environment bana is currently running inside.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HostKind {
    Termux,
    KaliNetHunterProot,
    NativeLinux,
    Windows,
    MacOs,
    Unknown,
}

/// معماری پردازنده‌ی میزبان.
/// CPU architecture of the host.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HostArch {
    Aarch64,
    X86_64,
    Unknown,
}

/// خلاصه‌ی وضعیت خودِ محیط اجرا، پیش از هر تشخیص ابزار خاص.
/// Summary of the execution environment itself, before scanning any specific tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostEnvironment {
    pub kind: HostKind,
    pub arch: HostArch,
    pub home_dir: PathBuf,
    pub shell: String,
    pub systemd_stubbed: bool,
}

/// وضعیت تشخیص هر ابزار — هیچ‌وقت حدس زده نمی‌شود، فقط یکی از این چهار حالت.
/// Detection status for any single tool — never guessed, always one of these four.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolStatus<T> {
    Found { path: PathBuf, info: T, verified: bool },
    FoundButIncompatible { path: PathBuf, info: T, reason: String },
    NotFound,
    AmbiguousMultiple { candidates: Vec<PathBuf> },
}

/// یک مسئله‌ی گزارش‌شده به کاربر، همراه با شدت آن.
/// A single issue reported to the user, tagged with its severity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub message: String,
    pub blocking: bool,
}

/// اثرانگشت پروژه‌ی ورودی — خروجی project_analyzer، ورودی build_driver.
/// The input project's fingerprint — output of project_analyzer, input to build_driver.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectFingerprint {
    pub scenario_id: String,
    pub root: PathBuf,
    pub confidence: f32,
}

/// اطلاعات تشخیص‌داده‌شده از یک نصب واقعی JDK.
/// Detected information from a real JDK installation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JdkInfo {
    pub version: String,
}

/// اطلاعات تشخیص‌داده‌شده از یک نصب واقعی Android SDK — نام واقعی
/// پوشه‌های زیرِ platforms/ و build-tools/، نه یک عدد یا مسیر حدسی.
/// Detected information from a real Android SDK install — the actual
/// folder names under platforms/ and build-tools/, never a guessed
/// number or path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkInfo {
    pub installed_platforms: Vec<String>,
    pub installed_build_tools: Vec<String>,
}

/// اطلاعات تشخیص‌داده‌شده از یک نصب واقعی NDK — نسخه از خودِ
/// `source.properties` واقعی استخراج می‌شود، نه از نام پوشه (که می‌تواند
/// گمراه‌کننده باشد).
/// Detected information from a real NDK install — the version is
/// extracted from the real `source.properties`, not the folder name
/// (which can be misleading).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NdkInfo {
    pub version: String,
}

/// گزارش کامل توچین اندروید؛ در فاز ۱ فقط `jdk`، `sdk`، و `ndk` پر
/// می‌شوند، بقیه‌ی فیلدها (aapt2، gradle_wrapper) طبق ادامه‌ی چک‌لیست فاز ۱
/// اضافه می‌شوند.
/// Full Android toolchain report; only `jdk`, `sdk`, and `ndk` are
/// populated in Phase 1 for now, the rest (aapt2, gradle_wrapper) land per
/// the rest of the Phase 1 checklist.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AndroidToolchainReport {
    pub jdk: ToolStatus<JdkInfo>,
    pub sdk: ToolStatus<SdkInfo>,
    pub ndk: ToolStatus<NdkInfo>,
}

/// گزارش نهایی اسکن محیط، خروجی نهایی env_scanner برای مصرف لایه‌های بعدی.
/// Final environment scan report, the output env_scanner hands to later stages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanReport {
    pub host: HostEnvironment,
    pub blocking_issues: Vec<Issue>,
    pub warnings: Vec<Issue>,
}
