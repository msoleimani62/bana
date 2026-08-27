# Changelog

فرمت این سند بر پایه‌ی [Keep a Changelog](https://keepachangelog.com) است.
This document follows the [Keep a Changelog](https://keepachangelog.com) format.

هر تغییر واقعی همان روز به این فایل اضافه می‌شود، نه فقط موقع انتشار نسخه‌ی
جدید — طبق فاز ۱۰ سند AGENTS.md.
Every real change is logged here the same day it happens, not only at
release time — per the Phase 10 checklist in AGENTS.md.

## [Unreleased]

### افزوده شد / Added
- pipeline کامل build زنجیره شد و به دستور واقعی `bana build [--variant]`
  وصل شد: `crates/bana-build-driver/src/pipeline.rs::build_hybrid_project`
  → `bana-ffi::run_build` → CLI. تشخیص سناریو، عضو workspace حاوی uniffi
  (بدون حدس)، Gradle wrapper، build native، uniffi bindgen، پچ AAPT2، و
  gradlew همه پشت‌سرهم زنجیر شدند. `BuildResult` به `bana-types` اضافه
  شد. ۳ تست یکپارچه با محیط شبیه‌سازی‌شده‌ی کامل مطابق ساختار بی‌مرز.
  Full build pipeline chained and wired to a real `bana build [--variant]`
  command: scenario detection, the uniffi-bearing workspace member
  (never guessed), Gradle wrapper, native build, uniffi bindgen, AAPT2
  patch, and gradlew all run in sequence. `BuildResult` added to
  `bana-types`. 3 integration tests with a full simulated environment
  matching bimarz's real structure.
- اجرای واقعی `gradlew assembleDebug`/`assembleRelease`
  (`crates/bana-build-driver/src/gradlew.rs`) همیشه از طریق wrapper
  خودِ پروژه؛ خطاهای رایج (NDK، نسخه‌ی JDK، AAPT2) به پیام قابل‌فهم
  دسته‌بندی می‌شوند بدون این‌که خروجی خام Gradle مخفی شود. ۴ تست واحد.
  این آخرین قطعه‌ی pipeline بود — هر ۵ گام build_driver حالا جداگانه
  پیاده و تست شده‌اند.
  Real `gradlew assembleDebug`/`assembleRelease` execution
  (`crates/bana-build-driver/src/gradlew.rs`) always via the project's
  own wrapper; common errors (NDK, JDK version, AAPT2) are classified
  into a readable message without hiding the raw Gradle output. 4 unit
  tests. This was the last pipeline piece — all 5 build_driver steps are
  now individually implemented and tested.
- پچ واقعی AAPT2 (`crates/bana-build-driver/src/aapt2_patch.rs`): نوشتن
  `android.aapt2FromMavenOverride` در `gradle.properties` پروژه —
  تأییدشده از سرچ که این کلید مسیر فایل مستقیم می‌خواهد، نه Maven
  coordinate. idempotent، جایگزین‌کننده‌ی خط قدیمی، و خطای صریح وقتی
  هیچ باینری هم‌معماری پیدا نشود. ۴ تست واحد.
  Real AAPT2 patch (`crates/bana-build-driver/src/aapt2_patch.rs`):
  writes `android.aapt2FromMavenOverride` into the project's
  `gradle.properties` — confirmed via search that this key takes a
  direct file path, not a Maven coordinate. Idempotent, replaces stale
  lines, and fails clearly when no matching-architecture binary exists.
  4 unit tests.
- ساخت/تأیید idempotent Gradle wrapper
  (`crates/bana-build-driver/src/wrapper.rs`): اگر از قبل `Found` باشد
  Gradle سیستمی اصلاً صدا زده نمی‌شود؛ در غیر این‌صورت فقط همین یک‌بار
  برای `gradle wrapper` استفاده می‌شود. ۴ تست واحد.
  Idempotent Gradle wrapper ensure/generate
  (`crates/bana-build-driver/src/wrapper.rs`): if already `Found`, system
  Gradle is never touched at all; otherwise it's used exactly once for
  `gradle wrapper`. 4 unit tests.
- تولید واقعی Kotlin bindings با `uniffi-bindgen`
  (`crates/bana-build-driver/src/bindgen.rs`)، سینتکس «library mode»
  تأییدشده از مستندات رسمی UniFFI، منطبق با الگوی خودِ بی‌مرز. ۲ تست واحد.
  Real Kotlin binding generation via `uniffi-bindgen`
  (`crates/bana-build-driver/src/bindgen.rs`), "library mode" syntax
  confirmed from official UniFFI docs, matching bimarz's own pattern. 2
  unit tests.
- شروع فاز ۴: اجرای واقعی build لایه‌ی native با `cargo-ndk`
  (`crates/bana-build-driver/src/native.rs`)، سینتکس تأییدشده از مستندات
  رسمی. `CommandRunner` با متد `run_in` (اجرا با working directory
  مشخص) گسترش یافت — پیاده‌سازی پیش‌فرض به `run` fallback می‌کند تا
  Mockهای فازهای قبلی نشکنند. ۴ تست واحد. هنوز به دستور کاربرنهایی وصل
  نشده، منتظر uniffi bindgen و gradlew.
  Phase 4 kickoff: real native-layer build via `cargo-ndk`
  (`crates/bana-build-driver/src/native.rs`), syntax confirmed from
  official docs. `CommandRunner` extended with a `run_in` method (run
  with an explicit working directory) — the default implementation falls
  back to `run` so earlier phases' mocks keep compiling. 4 unit tests.
  Not yet wired to a user-facing command, waiting on uniffi bindgen and
  gradlew.

### رفع شد / Fixed
- باگ کامپایل واقعی در `native.rs`: `MockRunner` تست از `RefCell` استفاده
  می‌کرد که با `CommandRunner: Send + Sync` سازگار نیست؛ با `Mutex`
  جایگزین شد.
  Real compile bug in `native.rs`: the test `MockRunner` used `RefCell`,
  incompatible with `CommandRunner: Send + Sync`; replaced with `Mutex`.
- تشخیص `uniffi` در سناریوی Hybrid حالا اعضای workspace را هم چک می‌کند
  (نه فقط `Cargo.toml` ریشه) — طبق ساختار واقعی بی‌مرز که وابستگی uniffi
  داخل عضو `mobile-core` است، نه ریشه.
  Hybrid scenario's `uniffi` detection now also checks workspace members
  (not just the root `Cargo.toml`) — matching the real bimarz structure,
  where the uniffi dependency lives inside the `mobile-core` member, not
  the root.
- مسیر چک Gradle wrapper در `bana doctor` حالا سناریو-آگاه است: برای
  پروژه‌ی Hybrid، `android/` را چک می‌کند نه ریشه‌ی مخزن را — پیدا شد
  چون `android/gradlew` واقعاً وجود داشت ولی داکتر اشتباه گزارش می‌داد.
  `bana doctor`'s Gradle wrapper check path is now scenario-aware: for a
  Hybrid project, it checks `android/`, not the repo root — found because
  `android/gradlew` genuinely existed but doctor wrongly reported it missing.

### افزوده شد / Added
- شروع فاز ۳: دو سناریوی v1 (`PureKotlinScenario`،
  `HybridRustUniffiScenario`) به‌عنوان پیاده‌سازی `ProjectScenario`، و
  رجیستری `analyze_project` که بین سناریوهای ثبت‌شده بالاترین `confidence`
  را انتخاب می‌کند. متصل به `bana-ffi::detect_project_scenario` و بخش
  «Scenario» در گزارش پروژه‌ی `bana doctor`. ۱۱ تست واحد.
  Phase 3 kickoff: two v1 scenarios (`PureKotlinScenario`,
  `HybridRustUniffiScenario`) as `ProjectScenario` implementations, and an
  `analyze_project` registry that picks the highest-confidence match among
  registered scenarios. Wired to `bana-ffi::detect_project_scenario` and a
  new "Scenario" section in `bana doctor`'s project report. 11 unit tests.

### تغییر یافت / Changed
- trait `ProjectScenario` با پارامتر `probe: &dyn EnvProbe` گسترش یافت —
  تا الان `detect`/`fingerprint` بدون تست‌پذیری مستقیم فایل‌سیستم واقعی
  می‌خواندند؛ الان مثل بقیه‌ی پروژه (`env_scanner`، `toolchain_mgr`) پشت
  انتزاع تست‌پذیر رفتند. `bana-plugin-api` حالا به `bana-env-scanner`
  وابسته است.
  The `ProjectScenario` trait was extended with a `probe: &dyn EnvProbe`
  parameter — until now `detect`/`fingerprint` read the real filesystem
  directly, with no testability; they now sit behind a testable
  abstraction like the rest of the project (`env_scanner`,
  `toolchain_mgr`). `bana-plugin-api` now depends on `bana-env-scanner`.
- تصمیم معماری: پچ AAPT2 (نوشتن `android.aapt2FromMavenOverride`) از فاز
  ۲ به فاز ۴ (`build_driver`) موکول شد، چون خودِ پچ سطح پروژه است
  (`gradle.properties` یک پروژه‌ی خاص)، نه سطح میزبان — همان مرز Gradle
  wrapper. فاز ۲ از نظر مسئولیت‌های سطح میزبان کامل اعلام شد.
  Architecture decision: the AAPT2 patch (writing
  `android.aapt2FromMavenOverride`) was deferred from Phase 2 to Phase 4
  (`build_driver`), since the patch itself is project-level (a specific
  project's `gradle.properties`), not host-level — the same boundary as
  the Gradle wrapper. Phase 2 is declared complete for its host-level
  responsibilities.

### رفع شد / Fixed
- هشدار کامپایلر `unused import: PackageBackend` در `bana-ffi/src/lib.rs`
  رفع شد — متدهای trait روی `Box<dyn Trait>` بدون import کار می‌کنند.
  Compiler warning `unused import: PackageBackend` in
  `bana-ffi/src/lib.rs` fixed — trait methods on `Box<dyn Trait>` work
  without an import.

### افزوده شد / Added
- کش content-addressed (`crates/bana-toolchain-mgr/src/cache.rs`) زیر
  `~/.cache/bana/<hash>` با هش FNV-1a قطعی. `ensure_cache_dir` می‌گوید
  مسیر از قبل reuse شده یا تازه ساخته شده. ۵ تست واحد.
  Content-addressed cache (`crates/bana-toolchain-mgr/src/cache.rs`) under
  `~/.cache/bana/<hash>`, using a deterministic FNV-1a hash.
  `ensure_cache_dir` reports whether the path was reused or freshly
  created. 5 unit tests.
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
