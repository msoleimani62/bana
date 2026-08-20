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
- [ ] پیاده‌سازی تشخیص Android SDK root + platformها + build-toolsها
- [ ] پیاده‌سازی تشخیص NDK (گشتن دنبال `source.properties`، نه مسیر ثابت)
- [ ] پیاده‌سازی تست واقعی AAPT2 (اجرای واقعی + بررسی تطابق معماری باینری با
      هاست، نه فقط `command -v`)
- [ ] پیاده‌سازی تشخیص Gradle wrapper موجود در پروژه
- [ ] ترکیب همه در `AndroidToolchainReport`
- [ ] تولید `ScanReport` نهایی + سریالایز JSON
- [ ] هر `Issue` گزارش‌شده شامل «آدرس‌دهی دقیق رفع» باشد (لینک دانلود رسمی/
      دستور نصب دقیق برای backend تشخیص‌داده‌شده) — طبق اصل ۱۱ RULES.md
- [ ] تست‌های واحد برای هر تشخیص روی محیط واقعی فعلی (Kali proot + Arch)؛
      تست واحد Windows/macOS حداقل در سطح mock/unit (بدون دستگاه واقعی)
- [x] دستور `bana doctor` که این گزارش را انسانی و دوستانه نمایش می‌دهد
      (طبق اصل ۱۲ — قابل‌فهم برای کاربر آماتور) — فعلاً میزبان کامل + فقط
      JDK از توچین؛ بقیه‌ی توچین با ادامه‌ی فاز ۱ اضافه می‌شود

---

## فاز ۲ — toolchain_mgr

**هدف:** فراهم‌کردن/پچ‌کردن ابزارهای گمشده، بدون حدس.

- [ ] تعریف لایه‌ی Bundled Tier (چه چیزهایی، چه نسخه‌هایی — طبق بخش ۵ قانون)
- [ ] تعریف trait `PackageBackend` و پیاده‌سازی‌های v1: `TermuxPkgBackend`,
      `AptBackend`, `PacmanBackend` (طبق بخش ۵.۱ RULES.md — هیچ فراخوانی
      مستقیم pkg/apt/pacman خارج از این پیاده‌سازی‌ها مجاز نیست)
- [ ] پیاده‌سازی‌های چندسکویی: `WingetBackend`/`ChocoBackend` (ویندوز),
      `HomebrewBackend` (macOS) — همان الگوی trait، بدون تغییر در بقیه‌ی
      toolchain_mgr
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

**فاز ۱: در حال پیشرفت.** `HostEnvironment` واقعی کامل شد (تشخیص
Termux/KaliNetHunterProot/NativeLinux/Windows/MacOs، معماری، systemd
استاب‌شده)، همراه ۱۰ تست واحد با `MockEnvProbe`. تشخیص واقعی JDK اضافه شد
(`crates/bana-env-scanner/src/jdk.rs`، پشت `trait CommandRunner`، ۳ تست
واحد) و اسکن موازی با `tokio::task::JoinSet` راه‌اندازی شد
(`toolchain.rs`، با `enum ProbeResult` برای هماهنگ‌سازی پروب‌های ناهمگون).
دستور `bana doctor` هر دو گزارش (میزبان + JDK) را نمایش می‌دهد. یک باگ
واقعی (سریالایز `NotFound` به‌صورت رشته‌ی خام، نه dict — چک substring
اشتباه در پایتون) قبل از رسیدن به کاربر پیدا و رفع شد.

باقی‌مانده‌ی فاز ۱: تشخیص واقعی Android SDK root + platformها + build-tools،
NDK (با گشتن دنبال `source.properties`)، تست واقعی AAPT2 (شامل تطابق
معماری باینری)، Gradle wrapper موجود در پروژه، ترکیب همه در
`AndroidToolchainReport` کامل، و افزودن «آدرس‌دهی دقیق رفع» به هر `Issue`.

**تأیید روی دستگاه واقعی (commit `23dce1c`):** ۱۳ تست `bana-env-scanner`
سبز (۱۰ میزبان + ۳ JDK). `bana doctor` هم محیط را درست تشخیص داد هم JDK
واقعی را پیدا کرد: `found, version 21.0.11-ea`.

**باگ سوم (پیدا و رفع‌شده — commit `b1d99f7`):** فارسی داخل رشته‌های خروجی
برنامه (`doctor.py` و `help=` در `cli/main.py`) — نقض صریح قانون «فارسی
فقط داخل کامنت». همه‌ی رشته‌های کاربرنهایی به انگلیسی اصلاح شدند و بند
مصداقی به بخش ۶ RULES.md اضافه شد تا این اشتباه تکرار نشود.

**باگ چهارم (پیدا و رفع‌شده — با داده‌ی واقعی از کاربر، بدون هیچ حدسی؛
commit `e4d783c`):**
روی دستگاه واقعی، `bana doctor` محیط را به‌اشتباه `NativeLinux` تشخیص داد،
نه `KaliNetHunterProot`. علت تأییدشده: heuristic فعلی فرض می‌کرد نشانه‌ی
proot روی اندروید وجود `/system/build.prop` است، ولی این مسیر روی این نوع
Kali NetHunter proot اصلاً وجود ندارد (`/etc/os-release` درست `ID=kali` را
نشان می‌دهد، پس آن نیمه از heuristic درست بود). خروجی واقعی `ls -la /`
کاربر دو نشانه‌ی قابل‌اتکا را نشان داد: پوشه‌ی `/termux` (حتی بدون هیچ
مجوزی قابل‌تشخیص، چون فقط stat لازم است) و `/sdcard`. heuristic با
`android_proot_signal = path_exists("/termux") || path_exists("/sdcard")`
بازنویسی شد؛ ۲ تست جدید و ۱ تست منفی اضافه شد (مجموع تست‌های `host.rs`: ۱۰،
همه سبز روی دستگاه واقعی). `bana doctor` روی دستگاه واقعی این‌بار درست
`Kali NetHunter (inside proot on Android)` را گزارش داد.

فازهای ۲ تا ۱۰: **شروع‌نشده.**
