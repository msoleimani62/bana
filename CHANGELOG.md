# Changelog

فرمت این سند بر پایه‌ی [Keep a Changelog](https://keepachangelog.com) است.
This document follows the [Keep a Changelog](https://keepachangelog.com) format.

هر تغییر واقعی همان روز به این فایل اضافه می‌شود، نه فقط موقع انتشار نسخه‌ی
جدید — طبق فاز ۱۰ سند AGENTS.md.
Every real change is logged here the same day it happens, not only at
release time — per the Phase 10 checklist in AGENTS.md.

## [Unreleased]

### رفع شد / Fixed
- هشدار کامپایلر `unused import: PackageBackend` در `bana-ffi/src/lib.rs`
  رفع شد — متدهای trait روی `Box<dyn Trait>` بدون import کار می‌کنند.
  Compiler warning `unused import: PackageBackend` in
  `bana-ffi/src/lib.rs` fixed — trait methods on `Box<dyn Trait>` work
  without an import.

### افزوده شد / Added
- دستور واقعی `bana setup`: `bana-ffi::setup_bundled_tools` هماهنگ‌کننده‌ی
  idempotent نصب — قبل از هر نصب، تشخیص واقعی فاز ۱ چک می‌شود؛ اگر ابزار
  از قبل `Found` باشد، هیچ نصبی انجام نمی‌شود. سرویس پایتون
  (`bana-py/bana/services/setup.py`) و دستور CLI `bana setup` اضافه شدند.
  `SetupAction` به `bana-types` اضافه شد.
  Real `bana setup` command: `bana-ffi::setup_bundled_tools` orchestrates
  idempotent installation — before any install, Phase 1's real detection
  is checked; if a tool is already `Found`, no install happens. Python
  service (`bana-py/bana/services/setup.py`) and the `bana setup` CLI
  command added. `SetupAction` added to `bana-types`.
- `YayBackend` به `PackageBackend` اضافه شد — چون `yay` هم پکیج‌های رسمی
  هم AUR را پوشش می‌دهد (مثل `android-sdk` که فقط در AUR است)، وقتی
  موجود باشد به `pacman` خالص ترجیح داده می‌شود.
  `YayBackend` added to `PackageBackend` — since `yay` covers both
  official and AUR packages (like `android-sdk`, AUR-only), it's
  preferred over plain `pacman` when available.
- کاتالوگ Bundled Tier (`crates/bana-toolchain-mgr/src/bundled.rs`) با
  نام پکیج واقعی و سرچ‌شده (نه حدسی) برای JDK و Android SDK روی
  apt/pacman/yay/pkg. یک نامتقارنی واقعی مهم مستند شد: Android SDK روی
  `pacman` خالص پکیج رسمی ندارد (فقط AUR)، و Termux اصلاً meta-package
  SDK ندارد.
  Bundled Tier catalog (`crates/bana-toolchain-mgr/src/bundled.rs`) with
  real, researched (never guessed) package names for JDK and Android SDK
  across apt/pacman/yay/pkg. Documented a real, important asymmetry:
  Android SDK has no official pacman package (AUR only), and Termux has
  no SDK meta-package at all.
- ثبت خودکار رکورد نصب (`crates/bana-toolchain-mgr/src/recorder.rs`،
  `InstallRecorder`): بعد از هر تلاش نصب (موفق یا شکست‌خورده)، یک فایل
  JSON مستقل زیر `<home>/.bana/installs/` نوشته می‌شود — طبق درخواست صریح
  کاربر برای عیب‌یابی داخلی خودکار آینده. `InstallRecord` به `bana-types`
  اضافه شد.
  Automatic install-record logging (`crates/bana-toolchain-mgr/src/recorder.rs`,
  `InstallRecorder`): after every install attempt (success or failure), a
  standalone JSON file is written under `<home>/.bana/installs/` — per the
  user's explicit request for future automatic internal diagnostics.
  `InstallRecord` added to `bana-types`.
- شروع فاز ۲: trait `PackageBackend` و ۶ پیاده‌سازی (`AptBackend`,
  `PacmanBackend`, `TermuxPkgBackend`, `WingetBackend`, `ChocoBackend`,
  `HomebrewBackend`) در `crates/bana-toolchain-mgr/src/backend.rs`، طبق
  بند ۵.۱ RULES.md. انتخاب backend بر اساس `is_available` واقعی، نه فرض
  از روی `HostKind` (پوشش سناریوی Kali proot در برابر Arch لپ‌تاپ کاربر
  که هر دو خانواده‌ی «لینوکس معمولی»اند ولی package manager متفاوت دارند).
  `ToolchainError` typed (thiserror) اضافه شد. ۶ تست واحد با `MockRunner`.
  Phase 2 kickoff: the `PackageBackend` trait and 6 implementations
  (`AptBackend`, `PacmanBackend`, `TermuxPkgBackend`, `WingetBackend`,
  `ChocoBackend`, `HomebrewBackend`) in
  `crates/bana-toolchain-mgr/src/backend.rs`, per RULES.md section 5.1.
  Backend selection is based on real `is_available` checks, never assumed
  from `HostKind` alone (covers the Kali-proot-vs-Arch-laptop scenario,
  where both are "native Linux" family but use different package
  managers). Typed `ToolchainError` (thiserror) added. 6 unit tests with
  `MockRunner`.
- تشخیص واقعی Gradle wrapper داخل یک پروژه‌ی مشخص در
  `crates/bana-env-scanner/src/gradle.rs`: چک واقعی سه فایل و پارس واقعی
  نسخه از `distributionUrl`؛ برخلاف بقیه‌ی پروب‌های فاز ۱، سطح پروژه است
  نه میزبان، پس عمداً بیرون از `AndroidToolchainReport` نگه داشته شد و از
  طریق تابع مستقل sync `bana-ffi::scan_gradle_wrapper` در دسترس است.
  `GradleWrapperInfo` به `bana-types` اضافه شد. ۵ تست واحد. این آخرین قدم
  فاز ۱ بود — فاز ۱ از نظر پیاده‌سازی هسته‌ای کامل شد.
  Real Gradle wrapper detection inside a specific project in
  `crates/bana-env-scanner/src/gradle.rs`: checks three real files and
  parses the real version from `distributionUrl`; unlike the other Phase 1
  probes, this one is project-level, not host-level, so it deliberately
  stays outside `AndroidToolchainReport` and is exposed via a standalone
  sync `bana-ffi::scan_gradle_wrapper` function. `GradleWrapperInfo` added
  to `bana-types`. 5 unit tests. This was the last Phase 1 item — Phase 1's
  core implementation is now complete.
- `bana doctor` حالا یک گزارش سطح پروژه هم بر اساس مسیر کاری فعلی نشان
  می‌دهد.
  `bana doctor` now also shows a project-level report based on the current
  working directory.
- تست واقعی AAPT2 (نه فقط `command -v`) در `crates/bana-env-scanner/src/aapt2.rs`:
  خواندن واقعی هدر ELF (`e_machine`) و مقایسه با معماری هاست؛ مسیرهای
  محتمل اول build-tools هر SDK، بعد PATH؛ معماری غلط → `FoundButIncompatible`
  همراه آدرس‌دهی دقیق رفع. `EnvProbe` با `read_bytes` گسترش یافت.
  `Aapt2Info` به `bana-types` اضافه شد. ۵ تست واحد.
  Real AAPT2 testing (not just `command -v`) in
  `crates/bana-env-scanner/src/aapt2.rs`: actually reads the ELF header
  (`e_machine`) and compares it to the host architecture; candidate paths
  are SDK build-tools first, then PATH; a wrong-architecture binary
  becomes `FoundButIncompatible` with precise remediation. `EnvProbe`
  extended with `read_bytes`. `Aapt2Info` added to `bana-types`. 5 unit
  tests.
- `bana doctor` حالا گزارش AAPT2 را هم نشان می‌دهد.
  `bana doctor` now also shows the AAPT2 report.
- تشخیص واقعی NDK (گشتن دنبال `source.properties`، نه مسیر ثابت) در
  `crates/bana-env-scanner/src/ndk.rs`: اولویت اول env var صریح، بعد گشتن
  زیر `ndk/` هر مسیر محتمل SDK؛ فقط `Pkg.Desc = Android NDK` واقعی پذیرفته
  می‌شود؛ چند نسخه‌ی هم‌زمان → `AmbiguousMultiple`. `NdkInfo` به
  `bana-types` اضافه شد. ۶ تست واحد.
  Real NDK detection (searching for `source.properties`, never a fixed
  path) in `crates/bana-env-scanner/src/ndk.rs`: explicit env var first,
  then searching under `ndk/` in each candidate SDK path; only a real
  `Pkg.Desc = Android NDK` is accepted; multiple simultaneous versions →
  `AmbiguousMultiple`. `NdkInfo` added to `bana-types`. 6 unit tests.
- `bana doctor` حالا گزارش NDK را هم نشان می‌دهد.
  `bana doctor` now also shows the NDK report.
- تشخیص واقعی Android SDK (root + platforms + build-tools) در
  `crates/bana-env-scanner/src/sdk.rs`: چند مسیر محتمل بر اساس `HostKind`
  (env varها اول، بعد مسیرهای رایج هر میزبان)، فقط پس از تأیید واقعی
  ساختار دایرکتوری پذیرفته می‌شود. `EnvProbe` با متد `list_dir` گسترش
  یافت. `SdkInfo` به `bana-types` اضافه شد. ۵ تست واحد.
  Real Android SDK detection (root + platforms + build-tools) in
  `crates/bana-env-scanner/src/sdk.rs`: several candidate paths based on
  `HostKind`, only accepted after confirming the real directory structure.
  `EnvProbe` extended with `list_dir`. `SdkInfo` added to `bana-types`. 5
  unit tests.
- تشخیص واقعی JDK از طریق اجرای `java -version` (پشت `trait CommandRunner`
  برای تست‌پذیری کامل)، در `crates/bana-env-scanner/src/jdk.rs`. `JdkInfo`
  به `bana-types` اضافه شد. ۳ تست واحد.
  Real JDK detection via running `java -version` (behind a `CommandRunner`
  trait for full testability). `JdkInfo` added to `bana-types`. 3 unit
  tests.
- اسکن موازی توچین با `tokio::task::JoinSet` در `toolchain.rs`، با
  `enum ProbeResult` برای یکسان‌سازی پروب‌های ناهمگون (JDK و SDK).
  Parallel toolchain scanning with `tokio::task::JoinSet` in
  `toolchain.rs`, using a `ProbeResult` enum to unify heterogeneous probes
  (JDK and SDK).
- `AndroidToolchainReport` به `bana-types` اضافه شد (شامل `jdk`، `sdk`،
  و `ndk`).
  `AndroidToolchainReport` added to `bana-types` (includes `jdk`, `sdk`,
  and `ndk`).
- شروع فاز ۱: تشخیص واقعی `HostEnvironment` در `bana-env-scanner`، پشت
  انتزاع `EnvProbe`. تشخیص Termux، Kali NetHunter proot، لینوکس معمولی،
  ویندوز، macOS، معماری CPU، و systemd استاب‌شده. ۱۰ تست واحد.
  Phase 1 kickoff: real `HostEnvironment` detection, behind the `EnvProbe`
  abstraction. Detects Termux, Kali NetHunter proot, native Linux,
  Windows, macOS, CPU arch, and stubbed systemd. 10 unit tests.
- توابع `scan_host()` و `scan_toolchain()` در مرز `bana-ffi` (با یک tokio
  runtime موقت برای اجرای توابع async از PyO3 sync).
  `scan_host()` and `scan_toolchain()` functions at the `bana-ffi`
  boundary (with a throwaway tokio runtime to run async functions from
  sync PyO3).
- `bana doctor`: گزارش کامل و دوستانه‌ی میزبان + JDK + SDK.
  `bana doctor`: full, friendly host + JDK + SDK report.
- مخزن روی گیت‌هاب ساخته و اولین commit (فاز ۰ کامل) push شد:
  https://github.com/msoleimani62/bana
  Repository created on GitHub and the first commit (complete Phase 0)
  pushed: https://github.com/msoleimani62/bana
- اصول ۱۱ تا ۱۴ به RULES.md: آدرس‌دهی دقیق رفع نیازمندی، راهنمایی دوستانه
  برای کاربر آماتور، به‌روزرسانی داخلی (self-update)، و استانداردهای تست
  جامع. بخش‌های جدید ۷ تا ۹ (فلگ‌ها، راهنمایی/doctor/self-update، تست).
  فاز ۶.۱ (self-update) به AGENTS.md اضافه شد.
  Principles 11-14 added to RULES.md; new sections 7-9 (flags, guidance/
  doctor/self-update, testing). Phase 6.1 (self-update) added to
  AGENTS.md.

### تغییر یافت / Changed
- امضای `scan_toolchain` تغییر کرد تا `probe` و `host_kind` را از بیرون
  بگیرد، چون تشخیص SDK به `HostKind` وابسته است؛ `bana-ffi` اکنون یک‌بار
  میزبان را تشخیص می‌دهد و به هر دو تابع اسکن می‌دهد.
  `scan_toolchain`'s signature changed to take `probe` and `host_kind` as
  input, since SDK detection depends on `HostKind`; `bana-ffi` now detects
  the host once and hands it to both scan functions.
- دامنه‌ی v1 از «فقط موبایل/proot» به «همه‌ی سیستم‌عامل‌ها (لینوکس، ویندوز،
  مک)» گسترش یافت. معماری از قبل (`HostKind`, `PackageBackend` trait) برای
  این جهش آماده بود، نیازی به بازطراحی هسته نبود.
  Scope expanded from "mobile/proot only" to "every OS (Linux, Windows,
  macOS)". The existing architecture was already ready for this.
- `HostKind` در `bana-types` با `Windows` و `MacOs` گسترش یافت.
  `HostKind` in `bana-types` extended with `Windows` and `MacOs`.

### رفع شد / Fixed
- باگ سریالایز: واریانت `NotFound` به‌صورت رشته‌ی خام JSON سریالایز می‌شود
  (نه dict)؛ چک اولیه‌ی پایتون (`"Found" in jdk`) روی این رشته هم به‌اشتباه
  True برمی‌گرداند چون "Found" زیررشته‌ی "NotFound" است — پیش از رسیدن به
  کاربر پیدا و رفع شد.
  Serialization bug: the `NotFound` variant serializes as a plain JSON
  string (not a dict); the initial Python check wrongly matched it too —
  caught and fixed before reaching the user.
- heuristic تشخیص `KaliNetHunterProot` بازنویسی شد: نشانه‌ی نامعتبر
  `/system/build.prop` (که روی دستگاه واقعی کاربر اصلاً وجود نداشت) با دو
  نشانه‌ی تأییدشده جایگزین شد: `/termux` و `/sdcard`.
  The `KaliNetHunterProot` detection heuristic was rewritten: the invalid
  `/system/build.prop` signal was replaced with two confirmed signals:
  `/termux` and `/sdcard`.
- فارسی داخل رشته‌های خروجی برنامه حذف شد — نقض قانون «فارسی فقط داخل
  کامنت».
  Persian removed from program output strings — a violation of the
  "Persian only in comments" rule.

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
