//! کاتالوگ لایه‌ی پایه (Bundled Tier) — طبق بخش ۵ RULES.md. هر ورودی فقط
//! برای backendهایی نام پکیج دارد که واقعاً سرچ و با منبع تأیید شده‌اند؛
//! نبودِ یک کلید یعنی «هنوز تأیید نشده»، نه یک حدس جای‌گذاری‌شده.
//! The Bundled Tier catalog — per RULES.md section 5. Each entry only has
//! a package name for backends that were actually researched and source-
//! confirmed; a missing key means "not yet verified", never a placeholder
//! guess.

use crate::backend::PackageBackend;
use crate::error::ToolchainError;
use crate::recorder::{current_unix_timestamp, InstallRecorder};
use bana_env_scanner::CommandRunner;
use bana_types::InstallRecord;

pub struct BundledToolSpec {
    pub id: &'static str,
    pub package_by_backend: &'static [(&'static str, &'static str)],
}

/// JDK — سازگار با AGP امروزی. `default-jdk` روی apt یک meta-package است
/// که همیشه resolve می‌شود، حتی وقتی Debian/Kali پکیج‌های شماره‌دار
/// (مثل `openjdk-17-jdk`) را در نسخه‌های بعدی drop می‌کنند؛ `jdk-openjdk`
/// پکیج رسمی و رول‌شونده‌ی مخزن `extra` آرچ است، همیشه به آخرین LTS.
/// JDK — compatible with today's AGP. `default-jdk` on apt is a meta-
/// package that always resolves, even across Debian/Kali releases that
/// drop numbered packages (e.g. `openjdk-17-jdk`); `jdk-openjdk` is
/// Arch's official, rolling `extra`-repo package, always the latest LTS.
pub const JDK: BundledToolSpec = BundledToolSpec {
    id: "jdk",
    package_by_backend: &[
        ("apt", "default-jdk"),
        ("pacman", "jdk-openjdk"),
        ("yay", "jdk-openjdk"),
        ("pkg", "openjdk-17"),
    ],
};

/// Android SDK meta. نامتقارنی واقعی و مهم: روی `pacman` خالص هیچ پکیج
/// رسمی‌ای برایش وجود ندارد — فقط AUR (از طریق `yay`) این را پوشش
/// می‌دهد؛ Termux اصلاً meta-package ندارد (باید دستی از طریق
/// `sdkmanager` نصب شود، خارج از مسیر `toolchain_mgr` — به همین دلیل
/// عمداً کلید `pkg` ندارد).
/// Android SDK meta. A real, important asymmetry: plain `pacman` has no
/// official package for this at all — only AUR (via `yay`) covers it;
/// Termux has no meta-package at all (must be installed manually via
/// `sdkmanager`, outside `toolchain_mgr`'s path — which is why it
/// deliberately has no `pkg` key).
pub const ANDROID_SDK: BundledToolSpec = BundledToolSpec {
    id: "android_sdk",
    package_by_backend: &[("apt", "android-sdk"), ("yay", "android-sdk")],
};

/// جست‌وجوی نام واقعی پکیج برای یک backend مشخص؛ `None` یعنی این ترکیب
/// هنوز تحقیق و تأیید نشده است.
/// Looks up the real package name for a specific backend; `None` means
/// this combination hasn't been researched and confirmed yet.
pub fn package_name_for(spec: &BundledToolSpec, backend_name: &str) -> Option<&'static str> {
    spec.package_by_backend
        .iter()
        .find(|(name, _)| *name == backend_name)
        .map(|(_, pkg)| *pkg)
}

/// نصب یک ابزار از کاتالوگ Bundled Tier، از طریق backend انتخاب‌شده. اگر
/// این ترکیب ابزار/backend هنوز تأیید نشده باشد، خطای صریح می‌دهد — هرگز
/// حدس نمی‌زند و هرگز یک backend دیگر را خودسرانه امتحان نمی‌کند. بعد از
/// هر تلاش (چه موفق چه شکست‌خورده)، یک رکورد جداگانه از طریق `recorder`
/// ثبت می‌شود — طبق درخواست صریح کاربر برای این‌که عیب‌یابی داخلی آینده
/// بتواند خودکار بفهمد هر ابزار از کجا آمده.
/// Installs a tool from the Bundled Tier catalog, via the selected
/// backend. If this tool/backend combination hasn't been confirmed yet,
/// it fails with a clear error — never guesses, never silently tries a
/// different backend on its own. After every attempt (success or
/// failure), a separate record is written via `recorder` — per the user's
/// explicit request so future internal diagnostics can automatically
/// understand where each tool came from.
pub fn install_bundled_tool(
    runner: &dyn CommandRunner,
    backend: &dyn PackageBackend,
    spec: &BundledToolSpec,
    recorder: &dyn InstallRecorder,
) -> Result<(), ToolchainError> {
    let package = package_name_for(spec, backend.name()).ok_or(ToolchainError::NoPackageForBackend {
        tool: spec.id,
        backend: backend.name(),
    })?;

    let result = backend.install(runner, package);

    let record = InstallRecord {
        tool_id: spec.id.to_string(),
        backend_used: backend.name().to_string(),
        package_name: package.to_string(),
        timestamp_unix: current_unix_timestamp(),
        success: result.is_ok(),
        failure_reason: result.as_ref().err().map(|e| e.to_string()),
    };

    // نوشتن رکورد حتی اگر خودِ نصب شکست خورده باشد؛ اگر نوشتن رکورد هم
    // شکست خورد، فقط لاگ می‌شود — خطای واقعی نصب هرگز پنهان نمی‌شود.
    // The record is written even when the install itself failed; if
    // writing the record also fails, it's only logged — the real install
    // error is never hidden by it.
    if let Err(record_err) = recorder.record(&record) {
        tracing::warn!(error = %record_err, "failed to write install record");
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::AptBackend;
    use crate::recorder::InMemoryRecorder;
    use bana_env_scanner::CommandOutput;
    use std::collections::HashSet;

    struct MockRunner {
        available_programs: HashSet<&'static str>,
    }

    impl CommandRunner for MockRunner {
        fn run(&self, program: &str, args: &[&str]) -> Option<CommandOutput> {
            if !self.available_programs.contains(program) {
                return None;
            }
            Some(CommandOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: args.first() != Some(&"__force_fail__"),
            })
        }
    }

    #[test]
    fn finds_confirmed_package_name() {
        assert_eq!(package_name_for(&JDK, "apt"), Some("default-jdk"));
        assert_eq!(package_name_for(&JDK, "pacman"), Some("jdk-openjdk"));
    }

    #[test]
    fn returns_none_for_unconfirmed_combination() {
        // Termux (pkg) هنوز برای Android SDK تأیید نشده — عمداً.
        // Termux (pkg) is deliberately not yet confirmed for Android SDK.
        assert_eq!(package_name_for(&ANDROID_SDK, "pkg"), None);
    }

    #[test]
    fn install_uses_the_confirmed_package_name() {
        let runner = MockRunner {
            available_programs: HashSet::from(["apt-get"]),
        };
        let backend = AptBackend;
        let recorder = InMemoryRecorder::default();
        assert!(install_bundled_tool(&runner, &backend, &JDK, &recorder).is_ok());
    }

    #[test]
    fn install_fails_clearly_when_combination_unconfirmed() {
        let runner = MockRunner {
            available_programs: HashSet::from(["apt-get"]),
        };
        let backend = AptBackend;
        let recorder = InMemoryRecorder::default();
        // فرض کنیم یک ابزار ساختگی که هیچ ورودی apt ندارد.
        // Assume a fake tool with no apt entry at all.
        let unconfirmed = BundledToolSpec {
            id: "made_up_tool",
            package_by_backend: &[("pacman", "whatever")],
        };
        assert!(matches!(
            install_bundled_tool(&runner, &backend, &unconfirmed, &recorder),
            Err(ToolchainError::NoPackageForBackend { .. })
        ));
    }

    #[test]
    fn writes_a_record_after_successful_install() {
        let runner = MockRunner {
            available_programs: HashSet::from(["apt-get"]),
        };
        let backend = AptBackend;
        let recorder = InMemoryRecorder::default();
        install_bundled_tool(&runner, &backend, &JDK, &recorder).unwrap();

        let records = recorder.records.lock().unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].tool_id, "jdk");
        assert_eq!(records[0].backend_used, "apt");
        assert_eq!(records[0].package_name, "default-jdk");
        assert!(records[0].success);
        assert!(records[0].failure_reason.is_none());
    }

    #[test]
    fn writes_a_record_after_failed_install_too() {
        // حتی نصب شکست‌خورده هم باید رکورد بشود، نه فقط موفق‌ها.
        // Even a failed install must still be recorded, not just successes.
        let runner = MockRunner {
            available_programs: HashSet::new(), // apt اصلاً موجود نیست
        };
        let backend = AptBackend;
        let recorder = InMemoryRecorder::default();
        let result = install_bundled_tool(&runner, &backend, &JDK, &recorder);
        assert!(result.is_err());

        let records = recorder.records.lock().unwrap();
        assert_eq!(records.len(), 1);
        assert!(!records[0].success);
        assert!(records[0].failure_reason.is_some());
    }
}
