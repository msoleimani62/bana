# Changelog

فرمت این سند بر پایه‌ی [Keep a Changelog](https://keepachangelog.com) است.
This document follows the [Keep a Changelog](https://keepachangelog.com) format.

هر تغییر واقعی همان روز به این فایل اضافه می‌شود، نه فقط موقع انتشار نسخه‌ی
جدید — طبق فاز ۱۰ سند AGENTS.md.
Every real change is logged here the same day it happens, not only at
release time — per the Phase 10 checklist in AGENTS.md.

## [Unreleased]

### افزوده شد / Added
- شروع فاز ۱: تشخیص واقعی `HostEnvironment` در `bana-env-scanner`
  (`crates/bana-env-scanner/src/host.rs`)، پشت انتزاع `EnvProbe` برای
  تست‌پذیری کامل. تشخیص Termux، Kali NetHunter proot (heuristic دو-نشانه‌ای:
  bind-mount اندروید + os-release کالی)، لینوکس معمولی، ویندوز، macOS،
  معماری CPU، و systemd استاب‌شده. ۸ تست واحد با `MockEnvProbe`.
  Phase 1 kickoff: real `HostEnvironment` detection in `bana-env-scanner`,
  behind the `EnvProbe` abstraction for full testability. Detects Termux,
  Kali NetHunter proot (two-signal heuristic), native Linux, Windows,
  macOS, CPU arch, and stubbed systemd. 8 unit tests via `MockEnvProbe`.
- تابع `scan_host()` در مرز `bana-ffi` که `HostEnvironment` را به‌صورت JSON
  به پایتون می‌دهد.
  `scan_host()` function at the `bana-ffi` boundary, handing
  `HostEnvironment` to Python as JSON.
- دستور `bana doctor` (نسخه‌ی اولیه): گزارش دوستانه‌ی وضعیت میزبان.
  `bana doctor` command (initial version): friendly host-status report.
- مخزن روی گیت‌هاب ساخته و اولین commit (فاز ۰ کامل) push شد:
  https://github.com/msoleimani62/bana
  Repository created on GitHub and the first commit (complete Phase 0)
  pushed: https://github.com/msoleimani62/bana

### تغییر یافت / Changed
- دامنه‌ی v1 از «فقط موبایل/proot» به «همه‌ی سیستم‌عامل‌ها (لینوکس، ویندوز،
  مک)» گسترش یافت. معماری از قبل (`HostKind`, `PackageBackend` trait) برای
  این جهش آماده بود، نیازی به بازطراحی هسته نبود.
  Scope expanded from "mobile/proot only" to "every OS (Linux, Windows,
  macOS)". The existing architecture (`HostKind`, `PackageBackend` trait)
  was already ready for this — no core redesign needed.
- `HostKind` در `bana-types` با `Windows` و `MacOs` گسترش یافت.
  `HostKind` in `bana-types` extended with `Windows` and `MacOs`.

### افزوده شد / Added
- اصول ۱۱ تا ۱۴ به RULES.md: آدرس‌دهی دقیق رفع نیازمندی، راهنمایی دوستانه
  برای کاربر آماتور، به‌روزرسانی داخلی (self-update)، و استانداردهای تست
  جامع.
  Principles 11-14 added to RULES.md: precise remediation addressing,
  friendly guidance for amateur users, internal self-update, and
  comprehensive testing standards.
- بخش‌های جدید RULES.md: فلسفه‌ی فلگ‌های CLI (بخش ۷)، لایه‌ی راهنمایی/
  `bana doctor`/self-update (بخش ۸)، استانداردهای تست (بخش ۹).
  New RULES.md sections: CLI flag philosophy (§7), guidance layer/`bana
  doctor`/self-update (§8), testing standards (§9).
- فاز ۶.۱ (self-update) به AGENTS.md اضافه شد.
  Phase 6.1 (self-update) added to AGENTS.md.

## [0.1.0] — فاز ۰: اسکلت مخزن / Phase 0: repo skeleton

### افزوده شد / Added
- Cargo workspace با ۸ کریت مستقل: `bana-types`, `bana-plugin-api`,
  `bana-env-scanner`, `bana-toolchain-mgr`, `bana-project-analyzer`,
  `bana-build-driver`, `bana-output-validator`, `bana-ffi`.
  Cargo workspace with 8 independent crates (same list as above).
- پکیج پایتون `bana-py` با ساختار `cli/`, `services/`, `config/` و CLI
  مبتنی بر `typer`.
  Python package `bana-py` with `cli/`, `services/`, `config/` layout and a
  `typer`-based CLI.
- trait `ProjectScenario` در `bana-plugin-api` به‌عنوان نقطه‌ی رسمی
  توسعه‌ی سناریوهای نوع پروژه.
  `ProjectScenario` trait in `bana-plugin-api` as the official extension
  point for project scenarios.
- CI اسکلتی (build+lint برای Rust و Python)، `.gitignore`، اسناد `RULES.md`
  و `AGENTS.md`.
  Skeleton CI (build+lint for Rust and Python), `.gitignore`, `RULES.md`
  and `AGENTS.md` docs.

### رفع شد / Fixed
- ناهماهنگی نام سیمبل PyO3: `PyInit__bana_ffi` تولید نمی‌شد چون
  `[lib].name` و اسم تابع `#[pymodule]` با `module-name` در `pyproject.toml`
  یکی نبودند. رفع شد با یکسان‌سازی هر دو روی `_bana_ffi`.
  PyO3 symbol-naming mismatch: `PyInit__bana_ffi` wasn't generated because
  `[lib].name` and the `#[pymodule]` function name didn't match
  `pyproject.toml`'s `module-name`. Fixed by aligning both to `_bana_ffi`.
- مود تک‌فرمانی پیش‌فرض `typer`: با فقط یک دستور ثبت‌شده، `bana ping` رد
  می‌شد. رفع شد با افزودن یک `@app.callback()` خالی.
  typer's default single-command collapse mode: with only one registered
  command, `bana ping` was rejected. Fixed by adding an empty
  `@app.callback()`.

### تأیید شد / Verified
- زنجیره‌ی `cargo build --workspace` → `maturin develop` → `bana ping`
  روی دستگاه واقعی (Kali NetHunter proot، aarch64) با خروجی `pong` تأیید شد.
  The `cargo build --workspace` → `maturin develop` → `bana ping` chain
  verified on real hardware (Kali NetHunter proot, aarch64), output `pong`.
