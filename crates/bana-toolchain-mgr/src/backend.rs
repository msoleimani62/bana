//! انتزاع رسمی نصب پکیج — طبق بند ۵.۱ RULES.md، هیچ کد دیگری در پروژه
//! نباید مستقیم `apt`/`pacman`/`pkg`/`winget`/`choco`/`brew` را صدا بزند،
//! فقط از طریق پیاده‌سازی‌های همین trait.
//! The official package-installation abstraction — per RULES.md section
//! 5.1, no other code in the project may call `apt`/`pacman`/`pkg`/
//! `winget`/`choco`/`brew` directly, only through implementations of this
//! trait.

use crate::error::ToolchainError;
use bana_env_scanner::CommandRunner;
use bana_types::HostKind;

/// یک راه واقعی برای نصب پکیج روی یک میزبان مشخص.
/// A real way to install a package on a specific host.
pub trait PackageBackend: Send + Sync {
    /// نام backend برای گزارش خطا و لاگ، نه برای منطق تصمیم‌گیری.
    /// Backend name, for error reporting and logging — not decision logic.
    fn name(&self) -> &'static str;

    /// آیا این backend واقعاً روی این میزبان قابل‌استفاده است؛ همیشه با
    /// اجرای واقعی یک دستور بی‌خطر تأیید می‌شود، نه فرض از روی HostKind.
    /// Whether this backend is actually usable on this host; always
    /// confirmed by really running a harmless command, never assumed from
    /// HostKind alone.
    fn is_available(&self, runner: &dyn CommandRunner) -> bool;

    /// نصب واقعی یک پکیج؛ خطا شامل خروجی واقعی دستور شکست‌خورده است.
    /// Actually installs a package; the error includes the real output of
    /// the failed command.
    fn install(&self, runner: &dyn CommandRunner, package: &str) -> Result<(), ToolchainError>;
}

fn run_install(
    runner: &dyn CommandRunner,
    backend_name: &'static str,
    program: &str,
    args: &[&str],
    package: &str,
) -> Result<(), ToolchainError> {
    match runner.run(program, args) {
        Some(out) if out.success => Ok(()),
        Some(out) => Err(ToolchainError::InstallFailed {
            backend: backend_name,
            package: package.to_string(),
            reason: if out.stderr.is_empty() {
                out.stdout
            } else {
                out.stderr
            },
        }),
        None => Err(ToolchainError::InstallFailed {
            backend: backend_name,
            package: package.to_string(),
            reason: format!("could not execute `{program}`"),
        }),
    }
}

pub struct AptBackend;
impl PackageBackend for AptBackend {
    fn name(&self) -> &'static str {
        "apt"
    }
    fn is_available(&self, runner: &dyn CommandRunner) -> bool {
        runner.run("apt-get", &["--version"]).is_some()
    }
    fn install(&self, runner: &dyn CommandRunner, package: &str) -> Result<(), ToolchainError> {
        run_install(
            runner,
            self.name(),
            "apt-get",
            &["install", "-y", package],
            package,
        )
    }
}

pub struct PacmanBackend;
impl PackageBackend for PacmanBackend {
    fn name(&self) -> &'static str {
        "pacman"
    }
    fn is_available(&self, runner: &dyn CommandRunner) -> bool {
        runner.run("pacman", &["--version"]).is_some()
    }
    fn install(&self, runner: &dyn CommandRunner, package: &str) -> Result<(), ToolchainError> {
        run_install(
            runner,
            self.name(),
            "pacman",
            &["-S", "--noconfirm", package],
            package,
        )
    }
}

/// wrapper واقعی روی `yay` — superset کاربردی `pacman`، چون هم پکیج‌های
/// رسمی هم AUR را پوشش می‌دهد (مثل `android-sdk` که فقط توی AUR است).
/// وقتی موجود باشد، به `pacman` خالص ترجیح داده می‌شود دقیقاً به همین
/// دلیل: کار هرگز فقط به‌خاطر AUR-only بودن یک پکیج شکست نمی‌خورد.
///
/// A real wrapper around `yay` — a practical superset of `pacman`, since
/// it covers both official-repo and AUR packages (like `android-sdk`,
/// which is AUR-only). When available, it's preferred over plain `pacman`
/// for exactly this reason: the install never fails just because a
/// package happens to be AUR-only.
pub struct YayBackend;
impl PackageBackend for YayBackend {
    fn name(&self) -> &'static str {
        "yay"
    }
    fn is_available(&self, runner: &dyn CommandRunner) -> bool {
        runner.run("yay", &["--version"]).is_some()
    }
    fn install(&self, runner: &dyn CommandRunner, package: &str) -> Result<(), ToolchainError> {
        run_install(
            runner,
            self.name(),
            "yay",
            &["-S", "--noconfirm", package],
            package,
        )
    }
}

pub struct TermuxPkgBackend;
impl PackageBackend for TermuxPkgBackend {
    fn name(&self) -> &'static str {
        "pkg"
    }
    fn is_available(&self, runner: &dyn CommandRunner) -> bool {
        runner.run("pkg", &["--version"]).is_some()
    }
    fn install(&self, runner: &dyn CommandRunner, package: &str) -> Result<(), ToolchainError> {
        run_install(
            runner,
            self.name(),
            "pkg",
            &["install", "-y", package],
            package,
        )
    }
}

pub struct WingetBackend;
impl PackageBackend for WingetBackend {
    fn name(&self) -> &'static str {
        "winget"
    }
    fn is_available(&self, runner: &dyn CommandRunner) -> bool {
        runner.run("winget", &["--version"]).is_some()
    }
    fn install(&self, runner: &dyn CommandRunner, package: &str) -> Result<(), ToolchainError> {
        run_install(
            runner,
            self.name(),
            "winget",
            &["install", "-e", "--id", package],
            package,
        )
    }
}

pub struct ChocoBackend;
impl PackageBackend for ChocoBackend {
    fn name(&self) -> &'static str {
        "choco"
    }
    fn is_available(&self, runner: &dyn CommandRunner) -> bool {
        runner.run("choco", &["--version"]).is_some()
    }
    fn install(&self, runner: &dyn CommandRunner, package: &str) -> Result<(), ToolchainError> {
        run_install(
            runner,
            self.name(),
            "choco",
            &["install", "-y", package],
            package,
        )
    }
}

pub struct HomebrewBackend;
impl PackageBackend for HomebrewBackend {
    fn name(&self) -> &'static str {
        "brew"
    }
    fn is_available(&self, runner: &dyn CommandRunner) -> bool {
        runner.run("brew", &["--version"]).is_some()
    }
    fn install(&self, runner: &dyn CommandRunner, package: &str) -> Result<(), ToolchainError> {
        run_install(runner, self.name(), "brew", &["install", package], package)
    }
}

/// فهرست backendهای کاندید بر اساس نوع میزبان، به ترتیب اولویت. این فقط
/// **کاندیدها** را محدود می‌کند؛ تعیین این‌که کدام واقعاً موجود است همیشه
/// با `is_available` واقعی انجام می‌شود.
/// Candidate backends by host kind, in priority order. This only narrows
/// the **candidates**; which one is actually present is always decided by
/// a real `is_available` check.
fn candidate_backends(host_kind: &HostKind) -> Vec<Box<dyn PackageBackend>> {
    match host_kind {
        HostKind::Termux => vec![Box::new(TermuxPkgBackend)],
        HostKind::KaliNetHunterProot | HostKind::NativeLinux => vec![
            Box::new(AptBackend),
            Box::new(YayBackend),
            Box::new(PacmanBackend),
        ],
        HostKind::Windows => vec![Box::new(WingetBackend), Box::new(ChocoBackend)],
        HostKind::MacOs => vec![Box::new(HomebrewBackend)],
        HostKind::Unknown => vec![],
    }
}

/// انتخاب واقعی backend: از میان کاندیدهای همان خانواده‌ی میزبان، اولین
/// موردی که واقعاً `is_available` است انتخاب می‌شود. نتیجه لاگ می‌شود تا
/// بعداً لایه‌ی CLI بتواند به کاربر گزارش بدهد کدام ابزار واقعاً استفاده
/// شد (طبق درخواست صریح کاربر: «هر وسیله‌ای لازم است استفاده کنیم، ولی
/// گزارشش را به کاربر بدهیم»).
/// Real backend selection: among the candidates for that host family, the
/// first one that's actually `is_available` wins. The result is logged so
/// the CLI layer can later report to the user which tool was actually
/// used (per the user's explicit request: "use whatever tool is needed,
/// but report it to the user").
pub fn select_backend(
    runner: &dyn CommandRunner,
    host_kind: &HostKind,
) -> Result<Box<dyn PackageBackend>, ToolchainError> {
    let backend = candidate_backends(host_kind)
        .into_iter()
        .find(|backend| backend.is_available(runner))
        .ok_or(ToolchainError::NoBackendAvailable)?;
    tracing::info!(backend = backend.name(), "selected package backend");
    Ok(backend)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bana_env_scanner::CommandOutput;
    use std::collections::HashSet;

    /// شبیه‌ساز محیطی که فقط دستورات مشخصی در آن واقعاً «نصب شده‌اند».
    /// Simulates an environment where only certain commands are actually
    /// "installed".
    struct MockRunner {
        available_programs: HashSet<&'static str>,
        install_should_fail: bool,
    }

    impl CommandRunner for MockRunner {
        fn run(&self, program: &str, args: &[&str]) -> Option<CommandOutput> {
            if !self.available_programs.contains(program) {
                return None;
            }
            if args.first() == Some(&"install") || args.contains(&"-S") {
                return Some(CommandOutput {
                    stdout: String::new(),
                    stderr: if self.install_should_fail {
                        "unable to locate package".to_string()
                    } else {
                        String::new()
                    },
                    success: !self.install_should_fail,
                });
            }
            Some(CommandOutput {
                stdout: "some version string".to_string(),
                stderr: String::new(),
                success: true,
            })
        }
    }

    #[test]
    fn selects_apt_when_only_apt_available_on_linux() {
        let runner = MockRunner {
            available_programs: HashSet::from(["apt-get"]),
            install_should_fail: false,
        };
        let backend = select_backend(&runner, &HostKind::KaliNetHunterProot).unwrap();
        assert_eq!(backend.name(), "apt");
    }

    #[test]
    fn selects_pacman_when_only_pacman_available_on_linux() {
        // دقیقاً سناریوی لپ‌تاپ آرچ کاربر — HostKind شبیه به کالی‌ست ولی
        // package manager واقعی متفاوت است.
        // Exactly the user's Arch laptop scenario — HostKind-family looks
        // like Kali, but the real package manager differs.
        let runner = MockRunner {
            available_programs: HashSet::from(["pacman"]),
            install_should_fail: false,
        };
        let backend = select_backend(&runner, &HostKind::NativeLinux).unwrap();
        assert_eq!(backend.name(), "pacman");
    }

    #[test]
    fn apt_takes_priority_when_both_available() {
        let runner = MockRunner {
            available_programs: HashSet::from(["apt-get", "pacman"]),
            install_should_fail: false,
        };
        let backend = select_backend(&runner, &HostKind::NativeLinux).unwrap();
        assert_eq!(backend.name(), "apt");
    }

    #[test]
    fn yay_takes_priority_over_plain_pacman_when_both_available() {
        // چون yay هم پکیج‌های رسمی هم AUR را پوشش می‌دهد، وقتی موجود است
        // باید به pacman خالص ترجیح داده شود.
        // Since yay covers both official and AUR packages, it should be
        // preferred over plain pacman when both are available.
        let runner = MockRunner {
            available_programs: HashSet::from(["yay", "pacman"]),
            install_should_fail: false,
        };
        let backend = select_backend(&runner, &HostKind::NativeLinux).unwrap();
        assert_eq!(backend.name(), "yay");
    }

    #[test]
    fn falls_back_to_plain_pacman_when_yay_absent() {
        let runner = MockRunner {
            available_programs: HashSet::from(["pacman"]),
            install_should_fail: false,
        };
        let backend = select_backend(&runner, &HostKind::NativeLinux).unwrap();
        assert_eq!(backend.name(), "pacman");
    }

    #[test]
    fn returns_no_backend_available_when_nothing_matches() {
        let runner = MockRunner {
            available_programs: HashSet::new(),
            install_should_fail: false,
        };
        let result = select_backend(&runner, &HostKind::NativeLinux);
        assert!(matches!(result, Err(ToolchainError::NoBackendAvailable)));
    }

    #[test]
    fn install_success_is_ok() {
        let runner = MockRunner {
            available_programs: HashSet::from(["pacman"]),
            install_should_fail: false,
        };
        let backend = PacmanBackend;
        assert!(backend.install(&runner, "jdk-openjdk").is_ok());
    }

    #[test]
    fn install_failure_includes_real_stderr_reason() {
        let runner = MockRunner {
            available_programs: HashSet::from(["apt-get"]),
            install_should_fail: true,
        };
        let backend = AptBackend;
        match backend.install(&runner, "totally-fake-package") {
            Err(ToolchainError::InstallFailed { reason, .. }) => {
                assert!(reason.contains("unable to locate package"));
            }
            other => panic!("expected InstallFailed, got {other:?}"),
        }
    }
}
