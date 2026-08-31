<div align="center">

# bana

**Build Automation for Native Android**
*(از فارسی «بنا» — بنا کردن / ساختن)*

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](#license--مجوز)
[![Rust](https://img.shields.io/badge/core-Rust-orange.svg)](#architecture--معماری)
[![Python](https://img.shields.io/badge/cli-Python%203.10%2B-blue.svg)](#architecture--معماری)
[![Platform](https://img.shields.io/badge/platform-Termux%20%7C%20Linux%20%7C%20macOS%20%7C%20Windows-lightgrey.svg)](#supported-hosts--میزبان‌های-پشتیبانی‌شده)

One command that detects, sets up, and builds an Android app —
no root, no manual SDK/NDK wrestling.

یک دستور که اپ اندرویدی را تشخیص می‌دهد، محیط را آماده می‌کند، و می‌سازد —
بدون روت، بدون دست‌وپنجه نرم‌کردن دستی با SDK/NDK.

**[🇬🇧 Read in English](#english)** &nbsp;·&nbsp; **[🇮🇷 مطالعه به فارسی](#فارسی)**

</div>

---

<a name="english"></a>

## English

### Table of Contents
- [What is bana?](#what-is-bana)
- [Why bana exists](#why-bana-exists)
- [Architecture](#architecture--معماری)
- [Supported hosts](#supported-hosts--میزبان‌های-پشتیبانی‌شده)
- [Supported project scenarios](#supported-project-scenarios)
- [Installation](#installation)
- [Commands](#commands)
- [Uninstalling](#uninstalling)
- [Project status](#project-status)
- [Contributing](#contributing)
- [License](#license--مجوز)

### What is bana?

**bana** is a CLI tool that takes the source code of an Android app —
pure Kotlin, or a hybrid app with a Rust/uniffi native layer — and
hands you back a finished APK/AAB, without ever making you touch the
Android SDK, NDK, Gradle wrapper, or AAPT2 layer by hand.

```
$ bana doctor   # full host + toolchain + project report
$ bana setup    # idempotently provisions whatever is missing
$ bana build    # runs the full pipeline, hands you a real APK
```

### Why bana exists

Most people never root their phone, so the real power of a modern
Android device — including its ability to *build* Android apps —
goes unused. bana repurposes Linux-only toolchain tricks (running a
full Linux userland via `proot`/`chroot` on Termux + Kali NetHunter,
with **no root required**) to close that gap. It deliberately targets
the exact environment that made Android app development from a phone
practically impossible before.

### Architecture

- **Rust core** — a Cargo workspace of 8 focused crates:
  `bana-types` (shared data), `bana-plugin-api` (the `ProjectScenario`
  extension point), `bana-env-scanner` (host/toolchain detection),
  `bana-toolchain-mgr` (package-manager backends + bundled-tool
  install), `bana-project-analyzer` (project-type detection),
  `bana-build-driver` (the actual build pipeline), `bana-output-validator`,
  and `bana-ffi` — the single PyO3 boundary; no other Rust crate
  knows Python exists.
- **Python CLI** (`bana-py`) — a thin [Typer](https://typer.tiangolo.com/)
  layer (`cli/`) that delegates all real work to `services/`, which
  in turn calls into the compiled Rust extension
  (`bana._bana_ffi`) via [PyO3](https://pyo3.rs)/[maturin](https://www.maturin.rs/).
- **Never guesses.** Every detection result is either confirmed with
  a real path/version, or explicitly reported as "not found." There
  is no ambiguous middle state.

### Supported hosts

| Host | Status |
|---|---|
| Kali NetHunter on Termux (proot, aarch64) | ✅ primary dev/test target |
| Arch Linux (native, `pacman`/`yay`) | 🧩 backend implemented, not yet build-tested |
| Other native Linux (`apt`) | 🧩 backend implemented |
| Windows (`choco`) | 🧩 backend implemented |
| macOS (`brew`) | 🧩 backend implemented |

bana deliberately has no hard dependency on Termux or Kali — it
detects whatever real Linux-like host it's running on.

### Supported project scenarios

| Scenario | Status |
|---|---|
| `hybrid-rust-uniffi` (root Cargo workspace + `android/` Gradle project, [uniffi](https://mozilla.github.io/uniffi-rs/)-generated bindings) | ✅ full pipeline implemented (v1 assumes the Gradle module is named `app`) |
| `pure-kotlin` | 🧩 detection implemented; build pipeline not yet wired |

### Installation

**Requirements:** Rust (via [rustup](https://rustup.rs)), Python 3.10+,
and [maturin](https://www.maturin.rs/) (`pip install maturin` or
`pipx install maturin`).

```bash
git clone https://github.com/msoleimani62/bana ~/bana
cd ~/bana/bana-py
maturin develop --release
```

This builds the Rust core and installs the `bana` command into your
active Python environment.

### Commands

| Command | What it does |
|---|---|
| `bana ping` | Health-checks the Rust↔Python wiring. |
| `bana doctor` | Full friendly report: host environment, JDK/SDK/NDK/AAPT2 toolchain, and (if run inside a project) the detected scenario + Gradle wrapper status. |
| `bana setup` | Idempotently provisions whatever's missing from the toolchain base layer (JDK, Android SDK for now). Already-satisfied tools are left untouched. |
| `bana build [--variant debug\|release]` | Runs the full build pipeline (scenario detection → Gradle wrapper → native build → uniffi bindings → AAPT2 patch → `gradlew`) and hands back a real APK path. |

Run `bana --help` or `bana <command> --help` at any time for full,
up-to-date flag documentation.

### Uninstalling

```bash
pip uninstall bana
rm -rf ~/bana                 # the cloned source, if you no longer need it
rm -rf ~/.bana                # install records written by `bana setup`
rm -rf ~/.cache/bana          # the content-addressed build cache
```

### Project status

bana is under active development. See [`docs/RULES.md`](docs/RULES.md)
for the project's binding architecture/principles document,
[`docs/AGENTS.md`](docs/AGENTS.md) for the live phase-by-phase
checklist and status log, [`docs/HANDOFF.md`](docs/HANDOFF.md) for a
full onboarding summary (useful if resuming work in a new AI chat
session), and [`CHANGELOG.md`](CHANGELOG.md) for a dated history of
every real change.

### Contributing

This is currently a solo/personal project without a formal
contribution process. Issues and pull requests on
[github.com/msoleimani62/bana](https://github.com/msoleimani62/bana)
are still welcome.

### License / مجوز

MIT — see [`LICENSE`](LICENSE) if present, or the license field in
`Cargo.toml`/`pyproject.toml`.

---

<a name="فارسی"></a>

## فارسی

### فهرست مطالب
- [بنا چیست؟](#بنا-چیست)
- [چرا بنا ساخته شد](#چرا-بنا-ساخته-شد)
- [معماری](#معماری--architecture)
- [میزبان‌های پشتیبانی‌شده](#میزبان‌های-پشتیبانی‌شده--supported-hosts)
- [سناریوهای پروژه‌ی پشتیبانی‌شده](#سناریوهای-پروژه‌ی-پشتیبانی‌شده)
- [نصب](#نصب)
- [دستورها](#دستورها)
- [حذف نصب](#حذف-نصب)
- [وضعیت پروژه](#وضعیت-پروژه)
- [مشارکت](#مشارکت)
- [مجوز](#مجوز--license)

### بنا چیست؟

**bana** یک ابزار خط‌فرمان است که سورس یک اپ اندروید — چه خالص Kotlin
باشد، چه هایبرید با لایه‌ی native نوشته‌شده به Rust از طریق uniffi —
را می‌گیرد و یک APK/AAB نهایی و کامل تحویل می‌دهد، بدون اینکه لازم
باشد کاربر حتی یک‌بار مستقیم با لایه‌ی Android SDK، NDK، Gradle
wrapper یا AAPT2 دست‌وپنجه نرم کند.

```
$ bana doctor   # گزارش کامل محیط میزبان + توچین + پروژه‌ی فعلی
$ bana setup    # فراهم‌کردن idempotent هرچیز لازم که کم است
$ bana build    # اجرای کامل pipeline و تحویل یک APK واقعی
```

### چرا بنا ساخته شد

اکثر کاربران گوشی خود را روت نمی‌کنند، پس ظرفیت واقعی گوشی‌های
امروزی — از جمله توانایی خودِ گوشی برای **ساختن** اپ اندروید — بلااستفاده
می‌ماند. bana ترفندهای مخصوص محیط‌های لینوکسی (اجرای کامل یک
userland لینوکسی از طریق `proot`/`chroot` روی Termux + Kali NetHunter،
**بدون نیاز به روت**) را به کار می‌گیرد تا این حفره را پر کند. هدف
این پروژه دقیقاً همان محیطی است که تا امروز توسعه‌ی اپ اندروید از
داخل گوشی را عملاً غیرممکن می‌کرد.

### معماری / Architecture

- **هسته‌ی Rust** — یک Cargo workspace شامل ۸ کریت مستقل:
  `bana-types` (داده‌ی مشترک بدون منطق)، `bana-plugin-api` (نقطه‌ی
  توسعه‌ی رسمی از طریق trait به نام `ProjectScenario`)،
  `bana-env-scanner` (تشخیص میزبان/توچین)، `bana-toolchain-mgr`
  (backendهای پکیج‌منیجر + نصب ابزار باندل‌شده)، `bana-project-analyzer`
  (تشخیص نوع پروژه)، `bana-build-driver` (خودِ pipeline ساخت)،
  `bana-output-validator`، و `bana-ffi` — تنها مرز PyO3 پروژه؛ هیچ
  کریت منطقی دیگری وجود پایتون را نمی‌بیند.
- **CLI پایتون** (`bana-py`) — یک لایه‌ی نازک با
  [Typer](https://typer.tiangolo.com/) (`cli/`) که کل کار واقعی را به
  `services/` می‌سپارد و آن‌جا از طریق
  [PyO3](https://pyo3.rs)/[maturin](https://www.maturin.rs/) به ماژول
  کامپایل‌شده‌ی Rust (`bana._bana_ffi`) وصل می‌شود.
- **هیچ‌وقت چیزی حدس زده نمی‌شود.** هر نتیجه‌ی تشخیص یا با مسیر/نسخه‌ی
  واقعی تأیید می‌شود، یا صریحاً «پیدا نشد» گزارش می‌شود؛ هیچ حالت
  میانیِ مبهمی وجود ندارد.

### میزبان‌های پشتیبانی‌شده / Supported hosts

| میزبان | وضعیت |
|---|---|
| Kali NetHunter روی Termux (proot، aarch64) | ✅ هدف اصلی توسعه و تست |
| Arch Linux (بومی، `pacman`/`yay`) | 🧩 backend پیاده شده، هنوز تست ساخت واقعی نشده |
| سایر لینوکس‌های بومی (`apt`) | 🧩 backend پیاده شده |
| ویندوز (`choco`) | 🧩 backend پیاده شده |
| macOS (`brew`) | 🧩 backend پیاده شده |

bana عمداً هیچ وابستگی سخت به Termux یا Kali ندارد — هر میزبان
لینوکسی واقعی که رویش اجرا شود را خودش تشخیص می‌دهد.

### سناریوهای پروژه‌ی پشتیبانی‌شده

| سناریو | وضعیت |
|---|---|
| `hybrid-rust-uniffi` (Cargo workspace در ریشه + پروژه‌ی Gradle در `android/`، بایندینگ‌های تولیدشده با [uniffi](https://mozilla.github.io/uniffi-rs/)) | ✅ کل pipeline پیاده و متصل (فرض نسخه‌ی v1: ماژول Gradle همیشه `app` نام دارد) |
| `pure-kotlin` | 🧩 تشخیص پیاده شده؛ pipeline ساخت هنوز متصل نشده |

### نصب

**پیش‌نیازها:** Rust (از طریق [rustup](https://rustup.rs))، پایتون
۳.۱۰ یا بالاتر، و [maturin](https://www.maturin.rs/)
(`pip install maturin` یا `pipx install maturin`).

```bash
git clone https://github.com/msoleimani62/bana ~/bana
cd ~/bana/bana-py
maturin develop --release
```

این دستور هسته‌ی Rust را می‌سازد و دستور `bana` را در محیط پایتونِ
فعال شما نصب می‌کند.

### دستورها

| دستور | کاری که انجام می‌دهد |
|---|---|
| `bana ping` | بررسی سلامت زنجیره‌ی Rust↔Python. |
| `bana doctor` | گزارش کامل و دوستانه: محیط میزبان، توچین JDK/SDK/NDK/AAPT2، و (اگر داخل یک پروژه اجرا شود) سناریوی تشخیص‌داده‌شده + وضعیت Gradle wrapper. |
| `bana setup` | فراهم‌کردن idempotent هرچه از لایه‌ی پایه‌ی توچین کم است (فعلاً JDK و Android SDK). ابزارهای از قبل موجود دست‌نخورده می‌مانند. |
| `bana build [--variant debug\|release]` | اجرای کامل pipeline ساخت (تشخیص سناریو ← Gradle wrapper ← build native ← بایندینگ‌های uniffi ← پچ AAPT2 ← `gradlew`) و تحویل مسیر یک APK واقعی. |

هر زمان با `bana --help` یا `bana <command> --help` مستندات کامل و
به‌روز هر پرچم را ببینید.

### حذف نصب

```bash
pip uninstall bana
rm -rf ~/bana                 # سورس کلون‌شده، اگر دیگر لازم ندارید
rm -rf ~/.bana                # رکوردهای نصب نوشته‌شده توسط `bana setup`
rm -rf ~/.cache/bana          # کش build با آدرس‌دهی محتوایی
```

### وضعیت پروژه

bana در حال توسعه‌ی فعال است. برای اصول و معماری الزام‌آور پروژه به
[`docs/RULES.md`](docs/RULES.md)، برای چک‌لیست زنده‌ی فاز‌به‌فاز و
لاگ وضعیت به [`docs/AGENTS.md`](docs/AGENTS.md)، برای خلاصه‌ی کامل
انتقال پروژه (مفید برای ادامه‌ی کار در یک چت/جلسه‌ی جدید با هر AI) به
[`docs/HANDOFF.md`](docs/HANDOFF.md)، و برای تاریخچه‌ی تاریخ‌دار هر
تغییر واقعی به [`CHANGELOG.md`](CHANGELOG.md) مراجعه کنید.

### مشارکت

این پروژه فعلاً یک پروژه‌ی شخصی/تک‌نفره بدون فرایند مشارکت رسمی است.
Issue و Pull Request در
[github.com/msoleimani62/bana](https://github.com/msoleimani62/bana)
هم‌چنان استقبال می‌شود.

### مجوز / License

MIT — فایل [`LICENSE`](LICENSE) را در صورت وجود، یا فیلد license در
`Cargo.toml`/`pyproject.toml` ببینید.

---

<div align="center">

**[⬆ back to top / بازگشت به بالا](#bana)**

</div>
