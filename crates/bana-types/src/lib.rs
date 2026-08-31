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
    Found {
        path: PathBuf,
        info: T,
        verified: bool,
    },
    FoundButIncompatible {
        path: PathBuf,
        info: T,
        reason: String,
    },
    NotFound,
    AmbiguousMultiple {
        candidates: Vec<PathBuf>,
    },
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

/// نتیجه‌ی اجرای کامل pipeline ساخت — یا موفق با مسیر APK، یا شکست‌خورده
/// با پیام خطای واقعی.
/// The result of a full build pipeline run — either successful with the
/// APK path, or failed with the real error message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildResult {
    pub success: bool,
    pub apk_path: Option<String>,
    pub error: Option<String>,
}

/// یک اقدام ثبت‌شده در جریان `bana setup` — یا از قبل برآورده بوده، یا
/// نصب شده، یا شکست خورده.
/// One recorded action during `bana setup` — either already satisfied,
/// installed, or failed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupAction {
    pub tool_id: String,
    pub outcome: String,
    pub detail: Option<String>,
}

/// یک رکورد ثبت‌شده از تلاش نصب — چه موفق چه شکست‌خورده — برای این‌که
/// عیب‌یابی داخلی آینده‌ی bana بتواند خودکار بفهمد هر ابزار از کجا و چطور
/// آمده. یک فایل جداگانه به ازای هر نصب.
/// A recorded install attempt — success or failure — so bana's future
/// internal diagnostics can automatically understand where and how each
/// tool arrived. One separate file per install.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallRecord {
    pub tool_id: String,
    pub backend_used: String,
    pub package_name: String,
    pub timestamp_unix: u64,
    pub success: bool,
    pub failure_reason: Option<String>,
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

/// اطلاعات تشخیص‌داده‌شده از یک باینری واقعی AAPT2 — شامل این‌که آیا
/// معماری باینری با معماری هاست تطابق دارد یا نه.
/// Detected information from a real AAPT2 binary — including whether the
/// binary's architecture matches the host's.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Aapt2Info {
    pub version: String,
    pub arch_mismatch: bool,
}

/// گزارش کامل توچین اندروید؛ در فاز ۱ فقط `jdk`، `sdk`، `ndk`، و `aapt2`
/// پر می‌شوند، فیلد باقی‌مانده (gradle_wrapper) طبق ادامه‌ی چک‌لیست فاز ۱
/// اضافه می‌شود.
/// Full Android toolchain report; only `jdk`, `sdk`, `ndk`, and `aapt2`
/// are populated in Phase 1 for now, the remaining field
/// (gradle_wrapper) lands per the rest of the Phase 1 checklist.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AndroidToolchainReport {
    pub jdk: ToolStatus<JdkInfo>,
    pub sdk: ToolStatus<SdkInfo>,
    pub ndk: ToolStatus<NdkInfo>,
    pub aapt2: ToolStatus<Aapt2Info>,
}

/// اطلاعات تشخیص‌داده‌شده از Gradle wrapper داخل یک پروژه‌ی مشخص. برخلاف
/// JdkInfo/SdkInfo/NdkInfo/Aapt2Info که سطح میزبان‌اند، این یکی سطح پروژه
/// است — به همین دلیل عمداً بیرون از `AndroidToolchainReport` نگه داشته
/// شده، دقیقاً همان مرزی که `project_analyzer` (فاز ۳) بهش نیاز دارد.
/// Detected information about a Gradle wrapper inside a specific project.
/// Unlike JdkInfo/SdkInfo/NdkInfo/Aapt2Info, which are host-level, this one
/// is project-level — deliberately kept outside `AndroidToolchainReport`,
/// exactly the boundary `project_analyzer` (Phase 3) will need.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradleWrapperInfo {
    pub gradlew_present: bool,
    pub wrapper_jar_present: bool,
    pub distribution_version: String,
}

/// گزارش نهایی اسکن محیط، خروجی نهایی env_scanner برای مصرف لایه‌های بعدی.
/// Final environment scan report, the output env_scanner hands to later stages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanReport {
    pub host: HostEnvironment,
    pub blocking_issues: Vec<Issue>,
    pub warnings: Vec<Issue>,
}
