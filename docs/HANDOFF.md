# HANDOFF.md — سند انتقال کامل پروژه‌ی bana

> این سند برای شروع در یک چت/صفحه‌ی جدید نوشته شده — چه با کلود، چه هر
> AI دیگری. اول این فایل را کامل بخوان، بعد `docs/RULES.md` (قانون
> اساسی) و `docs/AGENTS.md` (چک‌لیست دقیق فاز‌به‌فاز + لاگ وضعیت). این سه
> فایل با هم باید کافی باشند تا دقیقاً از همین‌جا ادامه پیدا کند، بدون
> نیاز به خواندن تاریخچه‌ی طولانی چت قبلی.
>
> نکته: اگه این چت جدید هم با کلود و همون حساب کاربری باشد، حافظه‌ی
> داخلی کلود (`/areas/android-build-orchestrator.md` و مشابه) هم به‌طور
> خودکار در دسترس است و باید با همین سند هم‌راستا باشد — ولی این سند
> باید مستقل از آن هم کامل و قابل‌اتکا بماند، چون ممکن است AI یا حساب
> دیگری استفاده شود.

---

## ۱. پروژه چیست

**bana** (از فارسی «بنا» — بنا کردن/ساختن) — **Build Automation for
Native Android**. یک ابزار CLI که سورس یک اپ اندروید (خالص Kotlin یا
Hybrid با لایه‌ی Rust/uniffi) را می‌گیرد و APK/AAB نهایی تحویل می‌دهد،
بدون این‌که کاربر مجبور باشد با لایه‌ی SDK/NDK/Gradle/AAPT2 دست‌وپنجه نرم
کند. انگیزه: چون اکثر کاربران گوشی خود را روت نمی‌کنند، bana ترفندهای
لینوکسی (proot/chroot روی Termux+Kali NetHunter، بدون نیاز به روت) را
برای پر کردن این حفره در ابزارهای موبایل به کار می‌گیرد.

- مخزن: `https://github.com/msoleimani62/bana`
- مسیر محلی روی گوشی کاربر: `~/bana`
- زبان‌ها: هسته Rust (workspace با ۸ کریت)، orchestration/CLI پایتون
  (`bana-py`، از طریق PyO3)
- معماری کامل، فلسفه، و اصول ثابت: `docs/RULES.md`
- چک‌لیست دقیق فاز‌به‌فاز + لاگ زنده‌ی وضعیت/تست/commit: `docs/AGENTS.md`
- تاریخچه‌ی تغییرات: `CHANGELOG.md`

---

## ۲. محیط واقعی کاربر (برای هر دستوری که می‌دهی، این‌ها را در نظر بگیر)

- **دستگاه اصلی تست:** Redmi Note 8 Pro شیائومی (۶ گیگ رم، ۱۲۸ گیگ
  حافظه، aarch64) با **Kali NetHunter روی Termux proot**. تقریباً همه‌ی
  تست‌های واقعی این پروژه روی همین دستگاه انجام شده.
  - `HostKind::KaliNetHunterProot` تشخیص داده می‌شود (نشانه‌ها: پوشه‌ی
    `/termux` یا `/sdcard` در ریشه + `ID=kali` در `/etc/os-release`).
  - پوشه‌ی Download واقعی گوشی از داخل proot در `/sdcard/Download` است.
  - shell: zsh با starship.
- **دستگاه دوم:** Dell Inspiron 1525 با **Arch Linux** (XFCE). از
  `pacman`/`yay` استفاده می‌کند (`yay` را برای پکیج‌های AUR ترجیح
  می‌دهد). هنوز هیچ تست واقعی bana روی این دستگاه انجام نشده — فقط
  معماری (`YayBackend`، `PacmanBackend`) از قبل برایش آماده است.
- **گیت‌هاب:** یوزرنیم `msoleimani62`، ایمیل
  `msoleimaniphone@gmail.com`.

---

## ۳. پروژه‌ی مرجع واقعی: بی‌مرز (`~/bimarz`)

تنها پروژه‌ی واقعی که برای تست Hybrid استفاده شده. ساختار دقیق (تأییدشده
با داده‌ی واقعی، نه حدس):

```
~/bimarz/
├── Cargo.toml          ← [workspace], members = ["engine-core", "mobile-core"]
├── engine-core/         (بدون uniffi)
├── mobile-core/         Cargo.toml شامل وابستگی uniffi است — این عضوی
│                        است که bana برای cargo-ndk/uniffi-bindgen انتخاب می‌کند
└── android/              ← ریشه‌ی واقعی پروژه‌ی Gradle
    ├── settings.gradle.kts
    ├── gradlew            ← وجود دارد (Gradle 8.7)
    ├── gradle/wrapper/...
    └── app/                ← فرض صریح v1: نام ماژول همیشه "app"
```

وضعیت توچین روی دستگاه (تأییدشده با `bana doctor`):

| ابزار | وضعیت |
|---|---|
| JDK | `Found`, نسخه‌ی `21.0.11-ea` |
| Android SDK | `Found` در `/usr/lib/android-sdk`، platforms: `android-35, android-34`، build-tools: `debian, 34.0.0` (نکته: پوشه‌ی `debian` واقعی است ولی شماره‌نسخه نیست — هر منطق آینده که «بهترین build-tools» را انتخاب می‌کند باید این را فیلتر کند) |
| NDK | **`NotFound`** — هنوز هیچ نسخه‌ای نصب نیست. **این یک مانع واقعی برای اجرای کامل `bana build` است** چون native build (`cargo ndk`) به NDK نیاز دارد |
| AAPT2 | `Found`، `2.19-debian`، معماری match (aarch64) — نیازی به پچ ندارد فعلاً |
| Gradle wrapper | `Found` زیر `android/` (نه ریشه‌ی مخزن) |
| Scenario | `hybrid-rust-uniffi`، اطمینان `0.95` |

---

## ۴. وضعیت دقیق پروژه در لحظه‌ی نوشتن این سند

- **فاز ۰ (اسکلت مخزن):** کامل ✅
- **فاز ۱ (`env_scanner`):** کامل ✅ — تشخیص میزبان، JDK، SDK، NDK،
  AAPT2 (با تطابق معماری واقعی ELF)، Gradle wrapper. ۳۴ تست.
- **فاز ۲ (`toolchain_mgr`، سطح میزبان):** کامل ✅ — `PackageBackend` (۷
  backend: apt/pacman/yay/pkg/winget/choco/brew)، کاتالوگ Bundled
  (JDK+Android SDK با نام پکیج واقعی و منبع‌دار)، `InstallRecorder`
  (رکورد JSON برای هر نصب زیر `~/.bana/installs/`)، کش
  content-addressed زیر `~/.cache/bana/<hash>`، دستور واقعی
  `bana setup` (idempotent). ۲۱ تست.
- **فاز ۳ (`project_analyzer`):** کامل ✅ — `PureKotlinScenario`،
  `HybridRustUniffiScenario`، رجیستری انتخاب بالاترین‌اطمینان. تست شده
  روی مخزن واقعی بی‌مرز. ۱۲ تست.
- **فاز ۴ (`build_driver`):** همه‌ی ۵ قطعه پیاده و تک‌تک تست شده‌اند
  (native build با cargo-ndk، uniffi bindgen library-mode، Gradle
  wrapper idempotent، پچ AAPT2، اجرای gradlew با دسته‌بندی خطا) و همه‌ی
  ۵ تا در `pipeline.rs::build_hybrid_project` زنجیره شده‌اند + دستور
  واقعی CLI `bana build [--variant debug|release]`.
  **⚠️ همین الان (آخرین قدم قبل از این سند) یک باگ واقعی در fixture
  تست pipeline پیدا و رفع شد** (`existing_paths`/`files` در Mock تست
  هماهنگ نبودند) — **این رفع هنوز روی دستگاه واقعی تست نشده**. اولین
  کاری که در چت جدید باید انجام شود: گرفتن نتیجه‌ی
  `cargo test --workspace` بعد از این رفع، و بعد commit/push.
  **بعد از آن، `bana build` هنوز هیچ‌وقت واقعاً روی `~/bimarz` اجرا
  نشده** — و طبق جدول بالا، چون NDK نصب نیست، به‌احتمال زیاد در قدم
  build native شکست می‌خورد. این باید یا قبلش با `sdkmanager` دستی حل
  شود، یا (بهتر) به فاز On-Demand Tier وصل شود که هنوز کامل نشده.
- **فاز ۵ تا ۱۰:** شروع‌نشده.

جمع تست‌های پروژه تا آخرین تأیید روی دستگاه: **۸۸ تست** (بعد از رفع باگ
fixture، باید همین عدد یا نزدیکش بماند — رفع فقط یک اصلاح Mock بود، نه
افزودن تست جدید).

---

## ۵. قوانین کاری که در طول این پروژه محکم رعایت شده (حتماً ادامه بده)

1. **هیچ‌وقت حدس نزن.** نه اسم پکیج، نه سینتکس CLI، نه رفتار یک ابزار.
   یا با سرچ وب و ذکر منبع تأیید کن، یا از کاربر بخواه دستور تشخیصی
   واقعی روی دستگاهش اجرا کند و نتیجه را برایت بفرستد. چند باگ واقعی
   همین‌جوری پیدا شدند (heuristic غلط تشخیص proot، اسم پکیج NDK/SDK،
   مسیر اشتباه Gradle wrapper برای Hybrid).
2. **بعد از هر تغییر، هر سه سند را به‌روز کن** — `RULES.md` (فقط برای
   تصمیمات معماری/اصول جدید)، `AGENTS.md` (چک‌لیست + لاگ وضعیت با شماره‌ی
   commit و تعداد تست)، `CHANGELOG.md` (فرمت Keep a Changelog، هر ورودی
   هم فارسی هم انگلیسی). این کار را **قبل از رفتن به قدم بعدی** انجام
   بده، نه در پایان.
3. **همیشه دستورات کامل و کپی‌پیست‌شدنی بده**، نه تکه‌تکه. الگوی
   استاندارد تحویل کد:
   ```
   mv /sdcard/Download/<file>.zip ~/<file>.zip
   rm -f /sdcard/Download/<file>.zip
   unzip -o ~/<file>.zip -d ~/bana
   rm -f ~/<file>.zip
   cd ~/bana
   cargo build --workspace 2>&1 | grep -i warning
   cargo test --workspace
   git add -A
   git commit -m "..."
   git push
   ```
   وقتی تغییر پایتون هم دارد، قبل از commit این‌ها هم اضافه شود:
   ```
   cd bana-py
   source .venv/bin/activate
   maturin develop
   cd ~/bana
   ```
4. **فایل‌های تغییریافته را در یک zip واحد بسته‌بندی کن** (`present_files`)،
   نه جدا جدا.
5. **کامنت‌ها همیشه یک خط فارسی + یک خط انگلیسی.** هیچ فارسی‌ای داخل
   خودِ کد، شناسه‌ها، یا هر رشته‌ای که مستقیم در ترمینال/خروجی برنامه
   چاپ می‌شود (این قانون یک‌بار نقض و فوراً رفع شد — `doctor.py`).
6. **تست‌پذیری کامل:** هر لایه‌ی I/O (فایل‌سیستم، اجرای دستور، نصب،
   نوشتن properties) پشت یک trait تست‌پذیر می‌رود
   (`EnvProbe`/`CommandRunner`/`InstallRecorder`/`PropertiesWriter`)، با
   یک پیاده‌سازی `Real*` برای production و Mock/Fake برای تست. هیچ منطقی
   مستقیم `std::fs`/`std::process::Command` صدا نمی‌زند.
7. **Mockهای تست همیشه `Mutex`، نه `RefCell`** — چون traitهای اصلی
   `Send + Sync` هستند (برای `tokio::task::JoinSet`). این باگ یک‌بار
   واقعاً کامپایل را شکست داد.
8. **همیشه به فارسی پاسخ بده** (این خواسته‌ی صریح و ثابت کاربر است).
   لحن: صریح، بدون تعارف و تعریف، بازخورد فنی صادقانه — حتی وقتی چیزی
   اشتباه است یا ایده‌ای منطقی نیست، محکم بگو، نه فقط تأیید کن.
9. **پیش از پیشنهاد چیزی که سیستم واقعی را تغییر می‌دهد** (نصب،
   overwrite فایل، اجرای build واقعی)، وضعیت را برای کاربر روشن کن —
   مثلاً وقتی گفتیم `bana setup` را تست کن، صریح گفتیم چون JDK/SDK از
   قبل پیدا شده‌اند نباید هیچ نصب واقعی‌ای اتفاق بیفتد.

---

## ۶. نقشه‌ی معماری خیلی فشرده (برای جزئیات کامل، `docs/RULES.md` بخش ۳)

```
bana-types          → فقط داده (بدون منطق)؛ همه به آن وابسته‌اند
bana-plugin-api      → trait ProjectScenario (نقطه‌ی توسعه)
bana-env-scanner     → تشخیص میزبان + JDK/SDK/NDK/AAPT2/Gradle wrapper
                       (host.rs, jdk.rs, sdk.rs, ndk.rs, aapt2.rs,
                       gradle.rs, command.rs, toolchain.rs)
bana-toolchain-mgr   → نصب/کش/رکورد (backend.rs, bundled.rs,
                       recorder.rs, cache.rs)
bana-project-analyzer → تشخیص سناریو (pure_kotlin.rs,
                       hybrid_rust_uniffi.rs, registry.rs)
bana-build-driver    → اجرای واقعی build (native.rs, bindgen.rs,
                       wrapper.rs, aapt2_patch.rs, gradlew.rs,
                       pipeline.rs)
bana-output-validator → هنوز شروع‌نشده (فاز ۵)
bana-ffi              → تنها مرز PyO3
bana-py                → CLI (typer): ping, doctor, setup, build
```

---

## ۷. قدم بعدی فوری (وقتی چت جدید شروع شد)

۱. زیپ حاوی رفع باگ fixture (`pipeline.rs`) را که همین الان تحویل داده
   شده، اعمال کن و `cargo test --workspace` را روی دستگاه بگیر.
۲. اگه سبز شد، commit/push کن و مستندات را با تأیید نهایی به‌روز کن.
۳. بعد، تصمیم بگیر: قبل از اولین اجرای واقعی `bana build` روی
   `~/bimarz`، اول باید NDK نصب شود (دستی با `sdkmanager --install
   "ndk;<version>"` یا وصل‌کردن On-Demand Tier). این مکالمه را با کاربر
   باز کن، حدس نزن کدوم مسیر را ترجیح می‌دهد.
