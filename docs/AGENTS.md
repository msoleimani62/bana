# AGENTS.md — نقشه‌ی فازها و چک‌لیست پیشرفت bana

> این سند برای هر AI/agent (یا خودمان در آینده) است که می‌خواهد بداند دقیقاً
> در کدام فاز هستیم و چه چیزی انجام شده. قبل از شروع هر کار، `RULES.md` را
> کامل بخوان — این سند فقط اجرای مرحله‌به‌مرحله‌ی همان قوانین است.
>
> علامت‌گذاری: `[ ]` انجام‌نشده، `[~]` در حال انجام، `[x]` تمام و تأییدشده.

---

## فاز ۰ — اسکلت مخزن

**هدف:** پایه‌ی قابل build (خالی ولی کامپایل‌شونده) برای هر دو زبان.

- [x] ساخت مخزن `msoleimani62/bana` — https://github.com/msoleimani62/bana
- [x] Cargo workspace با `workspace.dependencies` مشترک در ریشه
- [x] کریت `bana-types` (اسکلت، فقط تایپ‌های خالی + derive‌های serde)
- [x] کریت `bana-env-scanner` (اسکلت، وابسته به `bana-types`)
- [x] کریت `bana-toolchain-mgr` (اسکلت)
- [x] کریت `bana-project-analyzer` (اسکلت)
- [x] کریت `bana-plugin-api` (تعریف اولیه‌ی trait `ProjectScenario`)
- [x] کریت `bana-build-driver` (اسکلت)
- [x] کریت `bana-output-validator` (اسکلت)
- [x] کریت `bana-ffi` (اسکلت PyO3، تابع تست `ping() -> "pong"`)
- [x] پکیج پایتون `bana-py` با `pyproject.toml` + ساختار `cli/`, `services/`,
      `config/` (طبق الگوی موفق بازآرایی cli.py بی‌مرز)
- [x] اتصال `bana-py` به `bana-ffi` از طریق maturin و تست `ping()` — کامل
      روی دستگاه واقعی (Kali NetHunter proot، aarch64) تأیید شد، خروجی `pong`
- [~] `tests/fixtures/` — فقط README توضیحی فعلاً؛ فایل‌های واقعی Gradle در
      فاز ۳/۴ اضافه می‌شود
- [x] `.gitignore` مناسب هر دو اکوسیستم
- [x] CI اسکلت (فقط build+lint، بدون تست واقعی هنوز)
- [x] کپی `RULES.md` و همین `AGENTS.md` داخل `docs/`

---

## فاز ۱ — env_scanner

**هدف:** تشخیص کامل و قابل‌اتکای وضعیت واقعی محیط (بدون هیچ حدسی).

- [x] تعریف `HostEnvironment` (نوع میزبان، معماری، shell، وضعیت systemd) —
      پیاده‌سازی کامل در `crates/bana-env-scanner/src/host.rs`، پشت انتزاع
      `EnvProbe` برای تست‌پذیری کامل بدون نیاز به دستگاه واقعی هر سیستم‌عامل
- [x] گسترش `HostKind` به `Windows`, `MacOs` — منطق تشخیص واقعی از طریق
      `cfg!(target_os = ...)` برای ویندوز/مک، و heuristic دو-نشانه‌ای
      (bind-mount اندروید + os-release کالی) برای تفکیک Termux از Kali
      NetHunter proot
- [x] تشخیص `systemd_stubbed` با heuristic (باینری systemd-sysusers موجود
      ولی `/run/systemd/system` غایب)
- [x] تعریف عمومی `ToolStatus<T>` (Found / FoundButIncompatible / NotFound /
      AmbiguousMultiple) — از فاز ۰ در `bana-types` تعریف شده بود
- [x] راه‌اندازی اسکن موازی با `tokio::task::JoinSet` (الگوی `healthcheck.rs`
      بی‌مرز) برای همه‌ی پروب‌های مستقل — پیاده‌سازی در
      `crates/bana-env-scanner/src/toolchain.rs` با `enum ProbeResult` برای
      یکسان‌سازی نوع بازگشتی پروب‌های ناهمگون
- [x] پیاده‌سازی تشخیص JDK — `crates/bana-env-scanner/src/jdk.rs`، پشت
      `trait CommandRunner` برای تست‌پذیری کامل بدون نیاز به وجود واقعی
      java روی دستگاه تست؛ ۳ تست واحد (نسخه‌ی معتبر، NotFound، خروجی
      غیرقابل‌تفسیر)
- [x] پیاده‌سازی تشخیص Android SDK root + platformها + build-toolsها —
      `crates/bana-env-scanner/src/sdk.rs`، چند مسیر محتمل بر اساس
      `HostKind` (env varها اول، بعد مسیرهای رایج هر میزبان)، فقط پس از
      تأیید ساختار واقعی `platforms/`+`build-tools/` پذیرفته می‌شود؛ ۵ تست
      واحد
- [x] پیاده‌سازی تشخیص NDK (گشتن دنبال `source.properties`، نه مسیر ثابت) —
      `crates/bana-env-scanner/src/ndk.rs`؛ اولویت اول env var صریح
      (`ANDROID_NDK_HOME`/`ANDROID_NDK_ROOT`)، بعد گشتن زیر `ndk/` هر
      کدام از مسیرهای محتمل SDK؛ فقط `source.properties` با
      `Pkg.Desc = Android NDK` واقعی پذیرفته می‌شود؛ چند نسخه‌ی هم‌زمان
      → `AmbiguousMultiple`؛ ۶ تست واحد
- [x] پیاده‌سازی تست واقعی AAPT2 (اجرای واقعی + بررسی تطابق معماری باینری با
      هاست، نه فقط `command -v`) — `crates/bana-env-scanner/src/aapt2.rs`؛
      خواندن واقعی هدر ELF (`e_machine`) و مقایسه با معماری هاست؛ مسیرهای
      محتمل: اول build-tools هر SDK محتمل، بعد PATH؛ در صورت پیدا نشدن
      نسخه‌ی هم‌معماری ولی وجود نسخه‌ی معماری غلط → `FoundButIncompatible`
      با آدرس‌دهی دقیق رفع (`android.aapt2FromMavenOverride`)؛ `EnvProbe`
      با متد `read_bytes` گسترش یافت؛ ۵ تست واحد
- [x] پیاده‌سازی تشخیص Gradle wrapper موجود در پروژه —
      `crates/bana-env-scanner/src/gradle.rs`؛ برخلاف بقیه‌ی پروب‌های این
      فاز، سطح پروژه است نه میزبان (یک `project_root: &Path` می‌گیرد)، پس
      عمداً بیرون از `AndroidToolchainReport` نگه داشته شده و از طریق تابع
      مستقل `bana-ffi::scan_gradle_wrapper(project_root)` در دسترس است؛
      چک واقعی `gradlew` + `gradle-wrapper.jar` + پارس واقعی
      `distributionUrl` از `gradle-wrapper.properties`؛ ۵ تست واحد
- [x] ترکیب همه در `AndroidToolchainReport` — انجام شد (`jdk`، `sdk`،
      `ndk`، `aapt2`؛ Gradle wrapper عمداً بیرون از این گزارش ماند، طبق
      تصمیم معماری بالا)
- [~] تولید `ScanReport` نهایی + سریالایز JSON — سریالایز JSON برای هر
      پروب به‌طور مستقل انجام می‌شود (`scan_host`/`scan_toolchain`/
      `scan_gradle_wrapper`)؛ خودِ نوع رسمی `ScanReport` (با
      `blocking_issues`/`warnings` یکپارچه) هنوز جمع‌آوری نشده — به فاز
      CLI/`--explain` موکول شد
- [~] هر `Issue` گزارش‌شده شامل «آدرس‌دهی دقیق رفع» باشد — از نظر عملی
      برآورده شده: هر پیام `FoundButIncompatible`/`NotFound` در
      `bana doctor` همین الان شامل دستور/راهنمای دقیق رفع است (مثلاً
      `sdkmanager --install "ndk;..."`، `android.aapt2FromMavenOverride`)؛
      ولی نوع رسمی `Issue`/`ScanReport` یکپارچه هنوز ساخته نشده
- [x] تست‌های واحد برای هر تشخیص روی محیط واقعی فعلی (Kali proot + Arch)؛
      تست واحد Windows/macOS حداقل در سطح mock/unit (بدون دستگاه واقعی) —
      همه‌ی ۵ پروب با `MockEnvProbe`/`MockProbe` تست شده‌اند، مستقل از
      سیستم‌عامل واقعی
- [x] دستور `bana doctor` که این گزارش را انسانی و دوستانه نمایش می‌دهد
      (طبق اصل ۱۲ — قابل‌فهم برای کاربر آماتور) — کامل: میزبان + توچین
      (JDK/SDK/NDK/AAPT2) + گزارش پروژه‌ی مسیر کاری فعلی (Gradle wrapper)

---

## فاز ۲ — toolchain_mgr

**هدف:** فراهم‌کردن/پچ‌کردن ابزارهای گمشده، بدون حدس.

- [ ] تعریف لایه‌ی Bundled Tier (چه چیزهایی، چه نسخه‌هایی — طبق بخش ۵ قانون)
- [x] تعریف trait `PackageBackend` و پیاده‌سازی‌های v1: `TermuxPkgBackend`,
      `AptBackend`, `PacmanBackend` (طبق بخش ۵.۱ RULES.md — هیچ فراخوانی
      مستقیم pkg/apt/pacman خارج از این پیاده‌سازی‌ها مجاز نیست) —
      `crates/bana-toolchain-mgr/src/backend.rs`؛ انتخاب backend بر اساس
      `is_available` واقعی (نه فرض از روی HostKind)، چون هم گوشی هم لپ‌تاپ
      آرچ کاربر می‌توانند خانواده‌ی «لینوکس معمولی» باشند ولی package
      manager واقعی‌شان فرق دارد؛ ۶ تست واحد با `MockRunner`
- [x] پیاده‌سازی‌های چندسکویی: `WingetBackend`/`ChocoBackend` (ویندوز),
      `HomebrewBackend` (macOS) — همان الگوی trait، بدون تغییر در بقیه‌ی
      toolchain_mgr — انجام شد همراه بند بالا
- [ ] منطق نصب لایه‌ی پایه هنگام `bana setup`
- [ ] منطق فراهم‌کردن On-Demand Tier وقتی `project_analyzer` نیازش را اعلام
      می‌کند
- [ ] کش content-addressed زیر `~/.cache/bana/<hash>` (مستقل از پروژه، قابل
      استفاده‌ی مجدد بین چند پروژه‌ی مختلف)
- [ ] پچ AAPT2 برای عدم‌تطابق معماری (مشکل شناخته‌شده‌ی x86_64 روی aarch64)
- [ ] idempotency کامل — اجرای دوباره هیچ کاری تکراری انجام ندهد
- [ ] تست: شبیه‌سازی محیط با ابزار گمشده → نصب صحیح → تشخیص مجدد موفق

---

## فاز ۳ — project_analyzer

**هدف:** تشخیص نوع پروژه و تولید `ProjectFingerprint`.

- [ ] تعریف enum سناریوهای v1 (Pure Kotlin/Java, Hybrid Rust/uniffi) به‌عنوان
      پیاده‌سازی‌های trait `ProjectScenario` از `bana-plugin-api`
- [ ] منطق تشخیص خودکار (وجود `Cargo.toml` + uniffi، تعداد و نوع ماژول‌های
      Gradle، ABIهای هدف)
- [ ] پیاده‌سازی registry ثبت سناریوها بر پایه‌ی `bana-plugin-api` (نقطه‌ی
      رسمی افزودن سناریوی جدید در آینده، بدون تغییر این کریت)
- [ ] تست روی بی‌مرز واقعی (باید Hybrid تشخیص داده شود)

---

## فاز ۴ — build_driver

**هدف:** اجرای واقعی build بر اساس fingerprint، بدون دخالت کاربر.

- [ ] اجرای build لایه‌ی native (cargo-ndk برای همه‌ی ABIهای لازم)
- [ ] تولید Kotlin bindings (uniffi) در مسیر درست پروژه
- [ ] ساخت/تأیید Gradle wrapper (هیچ‌وقت Gradle سیستمی مستقیم)
- [ ] اجرای `gradlew assembleDebug` / `assembleRelease` با مدیریت خطا
- [ ] مدیریت خطاهای رایج (NDK ناسازگار، ABI missing) با پیام قابل‌فهم، نه
      استک‌تریس خام
- [ ] تست end-to-end روی بی‌مرز تا مرحله‌ی APK خام

---

## فاز ۵ — output_validator

**هدف:** اطمینان از صحت واقعی خروجی نهایی.

- [ ] بررسی وجود native libها برای همه‌ی ABIهای موردنظر داخل APK
- [ ] بررسی معماری واقعی هر `.so` (ELF header) در برابر ABI موردانتظار
- [ ] بررسی امضای APK
- [ ] بررسی صحت پایه‌ای manifest (package name، minSdk/targetSdk)
- [ ] گزارش نهایی خوانا برای کاربر

---

## فاز ۶ — CLI و تجربه‌ی کاربر (bana-py)

**هدف:** یک دستور، صفر درگیری — با پشتیبانی هم‌زمان آماتور و حرفه‌ای.

- [ ] `bana setup` — فراهم‌کردن یک‌باره‌ی لایه‌ی پایه
- [ ] `bana` (بدون آرگومان داخل یک پروژه) — تشخیص + build + validate کامل،
      بهترین تصمیم پیش‌فرض بدون نیاز به هیچ فلگی (طبق بخش ۷ RULES.md)
- [ ] فلگ‌های عمومی/آماتور: `-y`/`--yes`, `--verbose`, `--quiet`
- [ ] فلگ‌های پیشرفته/حرفه‌ای: انتخاب دستی سناریو، override مسیر NDK/SDK،
      محدودکردن ABI هدف، انتخاب build type — هیچ‌کدام برای مسیر پیش‌فرض
      اجباری نیست
- [ ] `bana doctor` — نمایش `ScanReport` کامل + آدرس‌دهی دقیق هر کمبود
- [ ] `bana --explain` — توضیح شکست تشخیص/build با جزئیات فنی‌تر برای
      کاربر حرفه‌ای (پیام پیش‌فرض همیشه ساده می‌ماند)
- [ ] `bana.toml` — override دستی برای موارد لبه‌ای (اختیاری، نه اجباری)
- [ ] پیام‌های خطا انسانی، دوستانه، و راهنما — نه استک‌تریس خام Rust/Python

---

## فاز ۶.۱ — به‌روزرسانی داخلی (self-update)

- [ ] `bana self-update` (یا معادل) که نسخه‌ی جدید منتشرشده روی crates.io/
      PyPI را بررسی می‌کند
- [ ] نصب نسخه‌ی جدید بدون نیاز به دخالت دستی کاربر (طبق اصل ۱۳ RULES.md)
- [ ] گزارش تغییرات نسخه‌ی جدید از CHANGELOG.md به کاربر پیش از نصب

---

## فاز ۷ — تست و کیفیت

**هدف:** پوشش واقعی سناریوهای لبه‌ای، نه فقط سبز بودن CI (طبق بخش ۹ RULES.md).

- [ ] پوشش تست هسته‌ی منطقی (هدف مشخص، مثل بی‌مرز: coverage روی pure logic)
- [ ] تست سناریوی «نیازمندی کاملاً گمشده» برای هر ماژول
- [ ] تست سناریوی «نسخه‌ی ناسازگار» (`FoundButIncompatible`)
- [ ] تست سناریوی «چند نسخه‌ی هم‌زمان» (`AmbiguousMultiple`)
- [ ] تست قطع/خطای میانه‌ی build (بدون حالت نیمه‌خراب باقی‌مانده)
- [ ] تست واحد هر سه `PackageBackend` (لینوکس/ویندوز/مک)، حتی وقتی تست
      یکپارچه فقط روی لینوکس واقعی اجرا می‌شود
- [ ] `cargo clippy -D warnings` تمیز
- [ ] `ruff`/فرمتر پایتون تمیز
- [ ] CI سبز پایدار روی هر سه‌ سیستم‌عامل (matrix build) — نه فقط لینوکس

---

## فاز ۸ — scaffold پروژه‌ی جدید (اختیاری v1، طبق تصمیم scope کامل)

- [ ] قالب پروژه‌ی خالص Kotlin/Java
- [ ] قالب پروژه‌ی Hybrid Rust/uniffi
- [ ] `bana init` برای ساخت پروژه‌ی جدید از این قالب‌ها

---

## فاز ۹ — اعتبارسنجی روی پروژه‌های واقعی

- [ ] تست کامل end-to-end روی بی‌مرز (اولین و اصلی‌ترین مرجع)
- [ ] تست روی خبرخوان/بینا (در صورت رسیدن به مرحله‌ی نسخه‌ی اندروید)
- [ ] تست روی open-downloader-cli (در صورت رسیدن به مرحله‌ی نسخه‌ی اندروید)
- [ ] تأیید صریح: bana هیچ وابستگی سخت‌کدشده‌ای به این پروژه‌ها ندارد

---

## فاز ۱۰ — بسته‌بندی و انتشار

- [ ] انتشار `bana-core` روی crates.io
- [ ] انتشار `bana-py` روی PyPI
- [ ] README دوزبانه (فارسی/انگلیسی) کامل طبق استاندارد ثابت کاربر
- [ ] اسکریپت نصب یک‌خطی
- [ ] `CHANGELOG.md` همیشه هم‌زمان با هر تغییر واقعی (نه فقط موقع انتشار)
      به‌روز نگه داشته می‌شود — از همین فاز ۰ شروع شده

---

## وضعیت فعلی

**فاز ۰: تأیید عملی شد.** `cargo build --workspace` روی دستگاه واقعی (Kali
NetHunter proot، aarch64) با موفقیت هر ۸ کریت را کامپایل کرد. باقی‌مانده:
ساخت خودِ مخزن روی گیت‌هاب توسط کاربر، اجرای `maturin develop` داخل
`bana-py` و تست `bana ping`، و تکمیل fixture واقعی Pure Kotlin.

**باگ اول (پیدا و رفع‌شده):** `maturin develop` روی دستگاه واقعی با خطای
`ImportError: dynamic module does not define module export function
(PyInit__bana_ffi)` شکست خورد. علت: اسم تابع `#[pymodule]` و `[lib].name` در
`Cargo.toml` هر دو `bana_ffi` بودند، در حالی که `module-name = "bana._bana_ffi"`
در `pyproject.toml` انتظار سیمبل `_bana_ffi` را داشت. هر دو به `_bana_ffi`
اصلاح شدند.

**باگ دوم (پیدا و رفع‌شده):** بعد از رفع باگ اول، build/install موفق شد ولی
`bana ping` با خطای `Got unexpected extra argument(s) (ping)` شکست خورد.
علت: typer وقتی فقط یک دستور ثبت‌شده دارد، مود تک‌فرمانی می‌گیرد. رفع شد با
افزودن یک `@app.callback()` خالی در `cli/main.py`.

هر دو باگ با تست واقعی روی دستگاه (`cargo build --workspace` →
`maturin develop` → `bana ping` → خروجی `pong`) تأیید شدند.

**نکته‌ی بی‌خطر:** هشدار `failed to garbage collect ... Operation not
permitted` در build روی proot به‌خاطر محدودیت مجوز فایل‌سیستم proot است، نه
باگ واقعی — قابل نادیده‌گرفتن.

باقی‌مانده‌ی فاز ۰ (غیرمسدودکننده برای فاز ۱): تکمیل fixture واقعی Pure
Kotlin (در فاز ۳/۴ انجام می‌شود).

مخزن ساخته و push شد: https://github.com/msoleimani62/bana (commit
`57f2440` و `13d45a8`، تغییرنام به `main`).

**فاز ۱: در حال پیشرفت.** میزبان، JDK، SDK، NDK، و AAPT2 کامل و همگی روی
دستگاه واقعی تأیید شده‌اند:

- `HostEnvironment`: تشخیص Termux/KaliNetHunterProot/NativeLinux/Windows/
  MacOs، معماری، systemd استاب‌شده. ۱۰ تست واحد.
- JDK: اجرای واقعی `java -version` پشت `trait CommandRunner`. ۳ تست واحد.
- SDK: چند مسیر محتمل بر اساس `HostKind` (اول `ANDROID_HOME`/
  `ANDROID_SDK_ROOT`، بعد مسیرهای رایج هر میزبان)، فقط پس از تأیید واقعی
  `platforms/`+`build-tools/` غیرخالی پذیرفته می‌شود. `EnvProbe` با
  `list_dir` گسترش یافت. ۵ تست واحد.
- NDK: اولویت اول `ANDROID_NDK_HOME`/`ANDROID_NDK_ROOT`، بعد گشتن زیر
  `ndk/` هر مسیر محتمل SDK؛ فقط `source.properties` واقعی با
  `Pkg.Desc = Android NDK` پذیرفته می‌شود؛ چند نسخه‌ی هم‌زمان →
  `AmbiguousMultiple`. ۶ تست واحد.
- AAPT2: نه فقط وجود فایل — خواندن واقعی هدر ELF (`e_machine`) و مقایسه با
  معماری هاست (مشکل شناخته‌شده‌ی خودِ کاربر: aapt2 نصب‌شده از apt کالی
  گاهی x86_64 است روی هاست aarch64). مسیرهای محتمل: اول build-tools هر
  SDK محتمل، بعد PATH. فقط معماری غلط پیدا شود → `FoundButIncompatible`
  همراه آدرس‌دهی دقیق رفع (`android.aapt2FromMavenOverride`، طبق اصل ۱۱
  RULES.md). `EnvProbe` با `read_bytes` گسترش یافت. ۵ تست واحد.

اسکن موازی (`toolchain.rs`) هر پنج پروب را هم‌زمان با `tokio::task::JoinSet`
انجام می‌دهد؛ امضای `scan_toolchain` برای گرفتن `probe`، `host_kind`، و
`host_arch` از بیرون تغییر کرد (میزبان یک‌بار در `bana-ffi` تشخیص داده
می‌شود). `bana doctor` هر پنج گزارش را نشان می‌دهد.

**تأیید روی دستگاه واقعی تا commit `595ff9b`:** ۲۴ تست `bana-env-scanner`
سبز (۱۰ میزبان + ۳ JDK + ۵ SDK + ۶ NDK). SDK در `/usr/lib/android-sdk`
پیدا شد؛ NDK درست `NotFound` گزارش شد (منطبق با وضعیت شناخته‌شده‌ی همین
دستگاه طبق سند تشخیص اولیه‌ی بی‌مرز). AAPT2 هنوز روی دستگاه تست نشده —
منتظر نتیجه.

**تأیید روی دستگاه واقعی تا commit `2a1a6a5`:** ۲۹ تست `bana-env-scanner`
سبز (۱۰ میزبان + ۳ JDK + ۵ SDK + ۶ NDK + ۵ AAPT2). SDK در
`/usr/lib/android-sdk` پیدا شد؛ NDK درست `NotFound` گزارش شد (منطبق با
وضعیت شناخته‌شده‌ی همین دستگاه)؛ AAPT2 هم درست `Found` شد —
`Android Asset Packaging Tool (aapt) 2.19-debian`، `arch_mismatch: false`
— یعنی نسخه‌ی apt کالی روی این دستگاه الان واقعاً aarch64 است (برخلاف
یادداشت قدیمی‌تر کاربر در `topics/dev-environment.md` که override دستی
لازم بود؛ ظاهراً یا پکیج کالی به‌روز شده یا قبلاً پچ شده بود).

**Gradle wrapper (سطح پروژه، نه میزبان) هم اضافه شد**
(`crates/bana-env-scanner/src/gradle.rs`): چک واقعی سه فایل
(`gradlew`، `gradle/wrapper/gradle-wrapper.jar`،
`gradle/wrapper/gradle-wrapper.properties`) و پارس واقعی نسخه از
`distributionUrl`. برخلاف بقیه، این پروب `project_root` می‌گیرد، پس عمداً
بیرون از `AndroidToolchainReport` نگه داشته شد و از طریق تابع مستقل
sync (نه async) `bana-ffi::scan_gradle_wrapper` در دسترس است. `bana doctor`
حالا علاوه بر گزارش میزبان/توچین، یک گزارش سطح پروژه هم بر اساس مسیر کاری
فعلی نشان می‌دهد. ۵ تست واحد.

**فاز ۱ از نظر پیاده‌سازی هسته‌ای کامل است.** آیتم‌های باقی‌مانده
(`ScanReport` رسمی + نوع `Issue` یکپارچه) صرفاً ساختاری‌اند — آدرس‌دهی
دقیق رفع همین الان در متن هر پیام `bana doctor` وجود دارد؛ فقط جمع‌آوری
آن‌ها در یک نوع رسمی واحد به فاز CLI/`--explain` موکول شده.

**تأیید نهایی روی دستگاه واقعی (commit `1b463f7`):** هر ۳۴ تست
`bana-env-scanner` سبز (۱۰ میزبان + ۳ JDK + ۵ SDK + ۶ NDK + ۵ AAPT2 + ۵
Gradle). `bana doctor` هر سه بخش (میزبان، توچین، پروژه) را درست نمایش
داد — گزارش پروژه هم `NotFound` بود چون `~/bana/bana-py` خودش پروژه‌ی
اندروید نیست، دقیقاً رفتار مورد انتظار.

**نکته‌ی واقعی باقی‌مانده برای فاز build_driver (نه باگ):** زیر
`build-tools/` روی دستگاه کاربر یک پوشه‌ی غیراستاندارد به اسم `debian` هم
هست (از پکیج‌بندی apt کالی) که شماره‌نسخه نیست؛ هر منطق آینده که
«آخرین/بهترین build-tools» را انتخاب می‌کند باید فقط پوشه‌های شبیه
شماره‌نسخه را کاندید حساب کند.

**باگ سوم (پیدا و رفع‌شده — commit `b1d99f7`):** فارسی داخل رشته‌های خروجی
برنامه (`doctor.py` و `help=` در `cli/main.py`) — نقض صریح قانون «فارسی
فقط داخل کامنت». همه‌ی رشته‌های کاربرنهایی به انگلیسی اصلاح شدند و بند
مصداقی به بخش ۶ RULES.md اضافه شد تا این اشتباه تکرار نشود.

**باگ چهارم (پیدا و رفع‌شده — با داده‌ی واقعی از کاربر، بدون هیچ حدسی؛
commit `e4d783c`):** روی دستگاه واقعی، `bana doctor` محیط را به‌اشتباه
`NativeLinux` تشخیص داد، نه `KaliNetHunterProot`. علت تأییدشده: heuristic
فعلی فرض می‌کرد نشانه‌ی proot روی اندروید وجود `/system/build.prop` است،
ولی این مسیر روی این نوع Kali NetHunter proot اصلاً وجود ندارد
(`/etc/os-release` درست `ID=kali` را نشان می‌دهد، پس آن نیمه از heuristic
درست بود). خروجی واقعی `ls -la /` کاربر دو نشانه‌ی قابل‌اتکا را نشان داد:
پوشه‌ی `/termux` (حتی بدون هیچ مجوزی قابل‌تشخیص، چون فقط stat لازم است) و
`/sdcard`. heuristic با
`android_proot_signal = path_exists("/termux") || path_exists("/sdcard")`
بازنویسی شد.

---

**فاز ۲: شروع شده.** trait `PackageBackend` و ۶ پیاده‌سازی (apt، pacman،
pkg، winget، choco، brew) اضافه شدند
(`crates/bana-toolchain-mgr/src/backend.rs`). نکته‌ی طراحی کلیدی: انتخاب
backend بر اساس `is_available` واقعی روی `HostKind` است، نه فرض مستقیم —
چون هم Kali proot هم Arch لپ‌تاپ کاربر می‌توانند خانواده‌ی مشابه باشند ولی
package manager واقعی‌شان (apt در برابر pacman) فرق دارد؛ این دقیقاً همان
سناریویی است که تست `selects_pacman_when_only_pacman_available_on_linux`
پوشش می‌دهد. `ToolchainError` (typed، با thiserror) اضافه شد. ۶ تست واحد
با `MockRunner`. `bana-toolchain-mgr` حالا به `bana-env-scanner` وابسته
است تا `CommandRunner` را دوباره استفاده کند، نه بازتعریف.

باقی‌مانده‌ی فاز ۲: تعریف دقیق لایه‌ی Bundled Tier، منطق `bana setup`،
On-Demand Tier، کش content-addressed، پچ AAPT2 (اکنون که تشخیصش در فاز ۱
آماده است)، و idempotency.

**تأیید روی دستگاه واقعی (commit `5db316e`):** هر ۶ تست جدید
`bana-toolchain-mgr` سبز؛ ۳۴ تست `bana-env-scanner` هم دست‌نخورده و سبز
باقی ماندند.

فازهای ۳ تا ۱۰: **شروع‌نشده.**
