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
- [Known Issues & Troubleshooting](#known-issues--troubleshooting)
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
  `bana-build-driver` (the actual build pipeline), `bana-output-validator`
  (post-build APK health validation), and `bana-ffi` — the single
  PyO3 boundary; no other Rust crate knows Python exists.
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
| `hybrid-rust-uniffi` (root Cargo workspace + `android/` Gradle project, [uniffi](https://mozilla.github.io/uniffi-rs/)-generated bindings) | ✅ full pipeline implemented and confirmed producing a real installable APK on a real device (v1 assumes the Gradle module is named `app`) |
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

### Known Issues & Troubleshooting

This section documents, in full, every real problem hit during
bana's **first successful end-to-end real build** (a
`hybrid-rust-uniffi` project, on a Redmi Note 8 Pro running Kali
NetHunter in proot, aarch64). Nothing here is theoretical — every
item was actually hit, actually diagnosed on the real device, and
actually fixed or worked around. Four of these were real bugs fixed
in bana's own code (already in this release); the rest are
environment-specific gotchas that are documented here because
they're likely to hit anyone building on a similar aarch64
Linux-userland host, but aren't (yet) automated into bana itself.

#### 1. NDK: official Google NDK doesn't run on an aarch64 host (environment, not a bana bug)

**Symptom:** `bana build` fails at the native-compile step with a
bare `exit status: 1` and no useful diagnostic — the linker step
never even produces an error message.

**Cause:** the official Android NDK's `clang` toolchain only ships
`linux-x86_64` prebuilt binaries. On a genuine aarch64 Linux host
(not emulated), those binaries simply cannot execute — and some
tool wrappers swallow the resulting exec error instead of
surfacing it.

**Fix:** install a community aarch64-native NDK rebuild instead of
the official one — [`lzhiyong/termux-ndk`](https://github.com/lzhiyong/termux-ndk)
(release tag `android-ndk`) provides one. Get the real download URL
first (don't guess it):

```bash
curl -s https://api.github.com/repos/lzhiyong/termux-ndk/releases/tags/android-ndk | grep browser_download_url
```

Download, extract, and point `ANDROID_NDK_HOME` at it:

```bash
mkdir -p ~/toolchains && cd ~/toolchains
curl -L -o ndk.tar.xz "<the URL from above>"
tar -xJf ndk.tar.xz && rm ndk.tar.xz
export ANDROID_NDK_HOME=~/toolchains/<extracted-folder-name>
echo 'export ANDROID_NDK_HOME=~/toolchains/<extracted-folder-name>' >> ~/.zshrc
```

Verify with `file -L $ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-aarch64/bin/clang`
before trusting it — it must say `ARM aarch64`, not `x86-64`.

#### 2. False "2 NDK versions found" — real bana bug, fixed

**Symptom:** `bana doctor` reports `NDK: 2 versions found at once,
pick one manually for now` even though only one NDK is actually
installed.

**Cause:** `sdk.rs::candidate_paths` pushed both `ANDROID_HOME` and
`ANDROID_SDK_ROOT` without deduplicating — when both env vars point
to the identical directory (very common), the one real SDK/NDK got
counted twice.

**Fix:** already fixed in this release — `candidate_paths` now
dedupes while keeping first-seen order. If you're on an older
version, either upgrade, or unset one of the two duplicate env vars
as a workaround.

#### 3. AAPT2: architecture-correct but version-too-old (environment, plus a real gap in bana's patch logic)

**Symptom:** `bana`'s automatic aapt2 patch succeeds (finds an
arch-matching aapt2, writes the override), but the build still
fails with:

```
ERROR: AAPT: LoadedArsc.cpp:94 RES_TABLE_TYPE_TYPE entry offsets overlap actual entry data.
Failed to load resources table in APK '.../platforms/android-35/android.jar'.
```

**Cause:** bana's aapt2 auto-detection (`aapt2_patch.rs` /
`env-scanner/aapt2.rs`) only checks **architecture** compatibility,
never **version** compatibility. On this device the only
arch-matching aapt2 it could find was an old distro package
(`aapt2 2.19-debian`) — architecturally fine, but far too old to
parse the newer resource-table format SDK 35's `android.jar` uses
(a widely-reported issue independent of bana). The *correct-version*
aapt2 (the one bundled in the official `build-tools/35.0.1`) exists
on the device too, but it's an `x86_64` binary that can't execute on
this aarch64 host — so bana correctly rejects it and falls back to
the old-but-running one.

**This is a real, documented gap in bana's current patch logic** —
it should ideally prefer a correct-version binary running under
emulation over a wrong-version binary running natively, but doesn't
yet. Tracked for a future fix (see `docs/AGENTS.md`).

**Workaround:** install `box64` (an x86_64-on-aarch64 Linux
emulation layer, available via `apt` on Kali) and run the official,
correct-version aapt2 through it:

```bash
sudo apt-get install -y box64
cp ~/android-sdk/build-tools/35.0.1/aapt2 ~/android-sdk/build-tools/35.0.1/aapt2.orig-x86_64
mkdir -p ~/android-sdk/build-tools/35.0.1-box64
printf '%s\n' \
  '#!/bin/sh' \
  'exec box64 "'"$HOME"'/android-sdk/build-tools/35.0.1/aapt2.orig-x86_64" "$@"' \
  > ~/android-sdk/build-tools/35.0.1-box64/aapt2
chmod +x ~/android-sdk/build-tools/35.0.1-box64/aapt2
```

Because this wrapper is a shell script (not a real ELF), bana's own
detector never picks it automatically — you currently have to set
the override manually before every `gradlew` invocation:

```bash
grep -v aapt2FromMavenOverride android/gradle.properties > /tmp/gp.new
printf 'android.aapt2FromMavenOverride=%s/android-sdk/build-tools/35.0.1-box64/aapt2\n' "$HOME" >> /tmp/gp.new
mv /tmp/gp.new android/gradle.properties
```

**Important caveat:** `bana build` re-runs its own aapt2
auto-detection on *every* invocation and will overwrite this manual
override right back to the old distro aapt2 (that's deliberate,
tested behavior — see `aapt2_patch.rs`'s
`replaces_stale_override_instead_of_duplicating` test). Until a
future bana release can be told to respect a manual override, the
practical workflow is: let `bana build` run the earlier pipeline
stages (native build, bindgen, AAPT2 patch — even though it'll fail
at the very end), then re-apply the override above and run
`./gradlew assembleDebug` directly for the final resource-link +
compile + package steps.

#### 4. uniffi-bindgen silently generates nothing from an Android `.so` — real bana bug, fixed

**Symptom:** the pipeline reports success at the bindgen step
(exit code 0), but no Kotlin files ever appear under
`app/src/main/kotlin/uniffi/`, and the later Gradle build fails
with `Unresolved reference 'uniffi'` everywhere the generated
bindings should be.

**Cause:** `uniffi-bindgen` in "library mode" does a real host
`dlopen()` to read the compiled library's embedded metadata.
`pipeline.rs` was pointing it at the **Android/Bionic**
cross-compiled `.so` from `jniLibs/` — which a Linux/glibc `dlopen`
simply cannot load. This particular `uniffi-bindgen` version didn't
surface that with any error message; it just silently produced
nothing.

**Fix:** already fixed in this release. `native::build_host_library`
now does a separate, plain `cargo build -p <package>` (no
`cargo-ndk`) to produce a genuine host-native `.so`, used only for
metadata extraction — never shipped in the APK.

#### 5. Even a host-native `.so` gives empty uniffi metadata if built with an aggressive release profile — real bana bug, fixed

**Symptom:** even after fixing #4 (host-native `.so`, confirmed via
`file -L` to be a real Linux ELF), `uniffi-bindgen print-repr
<library>` still returned an empty `[]` — no crates, no methods,
nothing — despite `nm -D <library> | grep UNIFFI_META` clearly
showing the metadata symbols present in the dynamic symbol table.

**Cause:** this project's `[profile.release]` uses `lto = true`,
`codegen-units = 1`, and `strip = "symbols"`. These settings
corrupt the *contents* of uniffi's metadata sections during
link-time optimization/reordering — the symbol table entries still
exist (hence `nm -D` sees them), but the buffers they point to no
longer contain valid data by the time `uniffi-bindgen` tries to
parse them. Confirmed by direct A/B comparison: a plain `cargo
build` (debug profile, none of those settings) produced a `.so`
whose `print-repr` output was the full, correct interface — every
method, record, enum, and function.

**Fix:** already fixed in this release. `build_host_library` builds
with the plain `debug` profile — no `--release`, so none of the
metadata-corrupting settings apply — and always uses the
`target/debug/` output for bindgen, regardless of what profile the
real Android `.so` files (built separately, via `cargo-ndk`, with
the full release settings) use.

#### 6. Generated Kotlin uses the wrong package unless `uniffi.toml` exists (app-level, not a bana issue)

**Symptom:** bindgen succeeds and Kotlin files are generated, but
compilation fails with `Unresolved reference 'uniffi'` /
`Unresolved reference 'MobileEngineClient'` in hand-written Kotlin
files that `import ir.bimarz.app.uniffi.*` — while the *generated*
file declares `package uniffi.mobile_core` (uniffi's default naming:
`uniffi.<crate_name>`).

**Cause:** nothing to do with bana — this is a missing
project-level config file. `uniffi-bindgen` automatically picks up
a `uniffi.toml` next to the crate if one exists, merging its
`package_name` setting; without that file, it falls back to the
default `uniffi.<crate_name>` naming, which won't match hand-written
imports that assume a custom package.

**Fix (app-level):**

```bash
printf '%s\n' \
  '[bindings.kotlin]' \
  'package_name = "your.app.package.uniffi"' \
  > <path-to-the-uniffi-crate>/uniffi.toml
```

Then re-run the native build + bindgen step so the Kotlin file gets
regenerated with the corrected package declaration.

#### 7. Generated Kotlin error classes fight with `Throwable.message` (app-level, not a bana issue)

**Symptom:** Kotlin compilation of the *generated* bindings file
itself fails with `Conflicting declarations: val message: String`
and `'message' hides member of supertype 'Throwable' and needs an
'override' modifier`.

**Cause:** uniffi maps a Rust error `enum` (marked
`#[derive(uniffi::Error)]`) to a Kotlin exception class hierarchy
that extends `Throwable`. If any variant of that enum has a field
literally named `message`, it collides with `Throwable`'s own
built-in `message` property — this is a known interaction between
uniffi and Kotlin, not specific to any one project.

**Fix (app-level):** rename the conflicting field in the Rust enum
definition to anything other than `message` (e.g. `detail`, `reason`)
— in every variant that has it, in the enum definition itself, and
in any `From<...>` conversion code that constructs those variants.
Then rebuild from scratch so the generated Kotlin picks up the
rename.

#### 8. `java.lang.Process.pid()` doesn't exist on Android, at any API level (app-level, not a bana issue)

**Symptom:** Kotlin code calling `process.pid()` (the standard JDK
9+ API) fails to compile with `Unresolved reference 'pid'` —
regardless of `minSdk`, `compileSdk`, or whether core library
desugaring is enabled.

**Cause:** confirmed directly by decompiling the actual
`android.jar` stub (`javap` on `java/lang/Process.class`) — even at
`compileSdk 35`, Android's `Process` class stub simply never
declares a `pid()` method at all. This isn't an API-level gate that
desugaring can fix; the method was never backported to Android's
core library in the first place.

**Fix (app-level):** read the process ID via reflection instead —
the standard, widely-used Android workaround:

```kotlin
private fun Process.pidCompat(): Long =
    try {
        val field = javaClass.getDeclaredField("pid")
        field.isAccessible = true
        field.getLong(this)
    } catch (e: Exception) {
        -1L
    }
```

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
- [مشکلات شناخته‌شده و راه‌حل‌ها](#مشکلات-شناخته‌شده-و-راه‌حل‌ها--known-issues--troubleshooting)
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
  `bana-output-validator` (اعتبارسنجی سلامت APK بعد از build)، و
  `bana-ffi` — تنها مرز PyO3 پروژه؛ هیچ کریت منطقی دیگری وجود پایتون
  را نمی‌بیند.
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
| `hybrid-rust-uniffi` (Cargo workspace در ریشه + پروژه‌ی Gradle در `android/`، بایندینگ‌های تولیدشده با [uniffi](https://mozilla.github.io/uniffi-rs/)) | ✅ کل pipeline پیاده و متصل، و تولید یک APK واقعی و نصب‌شدنی روی یک دستگاه واقعی تأیید شد (فرض نسخه‌ی v1: ماژول Gradle همیشه `app` نام دارد) |
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

### مشکلات شناخته‌شده و راه‌حل‌ها / Known Issues & Troubleshooting

این بخش، کامل و دقیق، هر مشکل واقعی‌ای که در **اولین build کامل و
موفق واقعی** بانا (یک پروژه‌ی `hybrid-rust-uniffi`، روی Redmi Note 8
Pro با Kali NetHunter در proot، aarch64) پیش اومد رو مستند می‌کنه.
هیچ‌کدوم فرضی نیستن — همه واقعاً پیش اومدن، واقعاً روی دستگاه واقعی
تشخیص داده شدن، و واقعاً رفع یا دورزده شدن. چهارتاشون باگ واقعی توی
خودِ کد بانا بودن (همین نسخه رفعشون کرده)؛ بقیه مشکلات مخصوص محیط
هستن که مستندشون کردیم چون احتمالاً هر کسی که روی یه هاست لینوکسی
aarch64 مشابه build بگیره باهاشون مواجه می‌شه، ولی هنوز (فعلاً) توی
خودِ بانا خودکار نشدن.

#### ۱. NDK: نسخه‌ی رسمی گوگل روی هاست aarch64 اجرا نمی‌شه (مشکل محیط، نه باگ بانا)

**نشونه:** `bana build` توی مرحله‌ی کامپایل native با یک `exit
status: 1` خشک و بدون هیچ توضیحی fail می‌شه — حتی مرحله‌ی لینکر هم
هیچ پیام خطایی چاپ نمی‌کنه.

**علت:** توچین `clang` مربوط به NDK رسمی اندروید فقط باینری‌های
`linux-x86_64` منتشر می‌کنه. روی یک هاست aarch64 واقعی (نه شبیه‌سازی‌شده)
این باینری‌ها اصلاً قابل‌اجرا نیستن — و بعضی wrapperها هم خطای exec
رو به‌جای نشون‌دادن، قورت می‌دن.

**راه‌حل:** به‌جای NDK رسمی، یک بازکامپایل بومی aarch64 از جامعه نصب
کنید — [`lzhiyong/termux-ndk`](https://github.com/lzhiyong/termux-ndk)
(تگ ریلیز `android-ndk`) یکی از این‌ها رو داره. اول لینک واقعی دانلود
رو بگیرید (حدس نزنید):

```bash
curl -s https://api.github.com/repos/lzhiyong/termux-ndk/releases/tags/android-ndk | grep browser_download_url
```

دانلود، استخراج، و تنظیم `ANDROID_NDK_HOME`:

```bash
mkdir -p ~/toolchains && cd ~/toolchains
curl -L -o ndk.tar.xz "<لینک بالا>"
tar -xJf ndk.tar.xz && rm ndk.tar.xz
export ANDROID_NDK_HOME=~/toolchains/<نام-پوشه‌ی-استخراج‌شده>
echo 'export ANDROID_NDK_HOME=~/toolchains/<نام-پوشه‌ی-استخراج‌شده>' >> ~/.zshrc
```

قبل از اعتماد بهش با `file -L $ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-aarch64/bin/clang`
تأیید کنید — باید `ARM aarch64` بگه، نه `x86-64`.

#### ۲. گزارش کاذب «۲ نسخه‌ی NDK پیدا شد» — باگ واقعی بانا، رفع شد

**نشونه:** `bana doctor` می‌گه `NDK: 2 versions found at once, pick
one manually for now` با اینکه فقط یک NDK واقعی نصب هست.

**علت:** `sdk.rs::candidate_paths` مسیرهای `ANDROID_HOME` و
`ANDROID_SDK_ROOT` رو بدون حذف تکراری اضافه می‌کرد — وقتی این دو env
var دقیقاً یک مسیر باشن (خیلی رایج)، همون یک SDK/NDK واقعی دوبار
شمرده می‌شد.

**راه‌حل:** توی همین نسخه رفع شده — `candidate_paths` حالا با حفظ
ترتیب اولین‌دیده‌شدن، تکراری‌ها رو حذف می‌کنه. اگه نسخه‌ی قدیمی‌تری
دارید، یا آپدیت کنید، یا به‌عنوان راه‌حل موقت یکی از دو env var
تکراری رو unset کنید.

#### ۳. AAPT2: معماری درست ولی نسخه‌ی خیلی قدیمی (مشکل محیط + یک گپ واقعی در منطق پچ بانا)

**نشونه:** پچ خودکار aapt2 بانا موفق می‌شه (یک aapt2 هم‌معماری پیدا
می‌کنه، override رو می‌نویسه)، ولی build بازم fail می‌شه:

```
ERROR: AAPT: LoadedArsc.cpp:94 RES_TABLE_TYPE_TYPE entry offsets overlap actual entry data.
Failed to load resources table in APK '.../platforms/android-35/android.jar'.
```

**علت:** تشخیص خودکار aapt2 بانا (`aapt2_patch.rs` /
`env-scanner/aapt2.rs`) فقط سازگاری **معماری** رو چک می‌کنه، هیچ‌وقت
سازگاری **نسخه** رو نه. روی این دستگاه تنها aapt2 هم‌معماری‌ای که
پیدا شد یک پکیج قدیمی توزیع (`aapt2 2.19-debian`) بود — از نظر
معماری خوب، ولی خیلی قدیمی‌تر از اونی که بتونه فرمت جدول منابع جدید
`android.jar` نسخه‌ی SDK 35 رو بخونه (یک مشکل شناخته‌شده و
گزارش‌شده‌ی گسترده، مستقل از بانا). نسخه‌ی *درست* aapt2 (همونی که
داخل `build-tools/35.0.1` رسمی باندل شده) هم روی دستگاه هست، ولی یک
باینری `x86_64`ه که روی این هاست aarch64 اجرا نمی‌شه — پس بانا درست
ردش می‌کنه و برمی‌گرده سراغ همون قدیمیِ کارکن.

**این یک گپ واقعی و مستندشده در منطق فعلی پچ بانا است** — در حالت
ایده‌آل باید یک باینری با نسخه‌ی درست که زیر شبیه‌سازی اجرا می‌شه رو
به یک باینری با نسخه‌ی غلط که بومی اجرا می‌شه ترجیح بده، ولی فعلاً
این کار رو نمی‌کنه. برای رفع آینده یادداشت شده (`docs/AGENTS.md`).

**راه‌حل موقت:** `box64` (یک لایه‌ی شبیه‌سازی x86_64-روی-aarch64
لینوکس، از طریق `apt` روی Kali در دسترس) رو نصب کنید و aapt2 رسمی و
نسخه‌درست رو از طریقش اجرا کنید:

```bash
sudo apt-get install -y box64
cp ~/android-sdk/build-tools/35.0.1/aapt2 ~/android-sdk/build-tools/35.0.1/aapt2.orig-x86_64
mkdir -p ~/android-sdk/build-tools/35.0.1-box64
printf '%s\n' \
  '#!/bin/sh' \
  'exec box64 "'"$HOME"'/android-sdk/build-tools/35.0.1/aapt2.orig-x86_64" "$@"' \
  > ~/android-sdk/build-tools/35.0.1-box64/aapt2
chmod +x ~/android-sdk/build-tools/35.0.1-box64/aapt2
```

چون این wrapper یک shell script است (نه یک ELF واقعی)، تشخیص خودکار
بانا هیچ‌وقت خودکار پیداش نمی‌کنه — فعلاً باید قبل از هر اجرای
`gradlew` دستی override رو ست کنید:

```bash
grep -v aapt2FromMavenOverride android/gradle.properties > /tmp/gp.new
printf 'android.aapt2FromMavenOverride=%s/android-sdk/build-tools/35.0.1-box64/aapt2\n' "$HOME" >> /tmp/gp.new
mv /tmp/gp.new android/gradle.properties
```

**نکته‌ی مهم:** `bana build` تشخیص خودکار خودش رو در **هر** اجرا از
نو انجام می‌ده و همین override دستی رو دوباره به aapt2 قدیمی توزیع
برمی‌گردونه (این رفتار عمدی و تست‌شده‌ست — تست
`replaces_stale_override_instead_of_duplicating` در
`aapt2_patch.rs`). تا زمانی که نسخه‌ی آینده‌ی بانا بتونه یاد بگیره
یک override دستی رو محترم بشمره، روش عملی فعلی اینه: بذارید `bana
build` مراحل اولیه‌ی pipeline (native build، bindgen، پچ AAPT2 —
حتی اگه آخرش fail بشه) رو انجام بده، بعد override بالا رو دوباره
بذارید و `./gradlew assembleDebug` رو مستقیم برای مراحل نهایی
(لینک منابع + کامپایل + بسته‌بندی) بزنید.

#### ۴. uniffi-bindgen ساکت از یک `.so`ی اندرویدی هیچی تولید نمی‌کنه — باگ واقعی بانا، رفع شد

**نشونه:** pipeline توی مرحله‌ی bindgen موفقیت گزارش می‌ده (exit code
صفر)، ولی هیچ فایل Kotlinی زیر `app/src/main/kotlin/uniffi/` ظاهر
نمی‌شه، و build بعدی Gradle همه‌جا با `Unresolved reference 'uniffi'`
fail می‌شه.

**علت:** `uniffi-bindgen` در «library mode» یک `dlopen()` واقعی روی
هاست انجام می‌ده تا متادیتای embedded کتابخانه رو بخونه.
`pipeline.rs` مسیر `.so`ی کراس‌کامپایل‌شده برای **اندروید/Bionic**
(از `jniLibs/`) رو بهش می‌داد — که `dlopen` لینوکسی/glibc اصلاً
نمی‌تونه لودش کنه. این نسخه‌ی خاص `uniffi-bindgen` این خطا رو با
هیچ پیامی نشون نمی‌داد؛ فقط ساکت هیچی تولید نمی‌کرد.

**راه‌حل:** توی همین نسخه رفع شده. `native::build_host_library` حالا
یک `cargo build -p <package>` ساده و جدا (بدون `cargo-ndk`) می‌زنه
تا یک `.so`ی واقعاً هاست‌بومی بسازه، فقط برای استخراج متادیتا — هرگز
در APK توزیع نمی‌شه.

#### ۵. حتی یک `.so`ی هاست‌بومی هم اگه با پروفایل release سنگین ساخته بشه متادیتای خالی می‌ده — باگ واقعی بانا، رفع شد

**نشونه:** حتی بعد از رفع مشکل ۴ (یک `.so`ی هاست‌بومی که با `file
-L` تأیید شده یک ELF لینوکسی واقعیه)، `uniffi-bindgen print-repr
<library>` بازم یک `[]` خالی برمی‌گردوند — نه کریتی، نه متدی، هیچی —
با اینکه `nm -D <library> | grep UNIFFI_META` سمبل‌های متادیتا رو
واضح توی جدول سمبل دینامیک نشون می‌داد.

**علت:** `[profile.release]` این پروژه از `lto = true`،
`codegen-units = 1`، و `strip = "symbols"` استفاده می‌کنه. این
تنظیمات **محتوای** سکشن‌های متادیتای uniffi رو در حین
LTO/بازآرایی‌های link-time خراب می‌کنن — ورودی‌های جدول سمبل هنوز
وجود دارن (برای همین `nm -D` می‌بینتشون)، ولی بافرهایی که بهشون
اشاره می‌کنن دیگه، وقتی `uniffi-bindgen` می‌خواد تجزیه‌شون کنه، داده‌ی
معتبری ندارن. با مقایسه‌ی مستقیم A/B تأیید شد: یک `cargo build` ساده
(پروفایل debug، بدون هیچ‌کدوم از این تنظیمات) یک `.so` تولید کرد که
خروجی `print-repr`ش کل رابط (هر متد، رکورد، enum، و تابع) رو کامل و
درست نشون داد.

**راه‌حل:** توی همین نسخه رفع شده. `build_host_library` با پروفایل
`debug` ساده build می‌کنه — بدون `--release`، پس هیچ‌کدوم از تنظیمات
خرابکننده‌ی متادیتا اعمال نمی‌شن — و همیشه از خروجی `target/debug/`
برای bindgen استفاده می‌کنه، مهم نیست فایل‌های واقعی `.so` اندروید
(که جدا، از طریق `cargo-ndk`، با تنظیمات کامل release ساخته می‌شن)
چه پروفایلی دارن.

#### ۶. Kotlin تولیدشده پکیج غلط داره مگر `uniffi.toml` وجود داشته باشه (سطح اپلیکیشن، نه مشکل بانا)

**نشونه:** bindgen موفق می‌شه و فایل‌های Kotlin تولید می‌شن، ولی
کامپایل با `Unresolved reference 'uniffi'` / `Unresolved reference
'MobileEngineClient'` توی فایل‌های Kotlin دستی که `import
ir.bimarz.app.uniffi.*` دارن fail می‌شه — درحالی‌که فایل *تولیدشده*
`package uniffi.mobile_core` اعلام می‌کنه (نام‌گذاری پیش‌فرض uniffi:
`uniffi.<نام-کریت>`).

**علت:** ربطی به بانا نداره — یک فایل کانفیگ سطح‌پروژه‌ی گم‌شده است.
`uniffi-bindgen` خودکار یک `uniffi.toml` کنار کریت رو (اگه وجود
داشته باشه) پیدا و `package_name`ش رو اعمال می‌کنه؛ بدون این فایل،
به نام‌گذاری پیش‌فرض `uniffi.<نام-کریت>` برمی‌گرده که با importهای
دستی که یک پکیج سفارشی فرض کردن جور در نمی‌آد.

**راه‌حل (سطح اپلیکیشن):**

```bash
printf '%s\n' \
  '[bindings.kotlin]' \
  'package_name = "your.app.package.uniffi"' \
  > <مسیر-کریت-uniffi>/uniffi.toml
```

بعد مرحله‌ی native build + bindgen رو دوباره بزنید تا فایل Kotlin با
اعلام پکیج درست بازتولید بشه.

#### ۷. کلاس‌های خطای تولیدشده‌ی Kotlin با `Throwable.message` تداخل دارن (سطح اپلیکیشن، نه مشکل بانا)

**نشونه:** کامپایل خودِ فایل بایندینگ‌های *تولیدشده* با
`Conflicting declarations: val message: String` و «`'message' hides
member of supertype 'Throwable' and needs an 'override' modifier`»
fail می‌شه.

**علت:** uniffi یک `enum` خطای Rust (با `#[derive(uniffi::Error)]`)
رو به یک سلسله‌مراتب کلاس exception در Kotlin نگاشت می‌کنه که از
`Throwable` ارث می‌بره. اگه هر variant این enum فیلدی دقیقاً به اسم
`message` داشته باشه، با پراپرتی داخلی `message` خودِ `Throwable`
تداخل پیدا می‌کنه — این یک تعامل شناخته‌شده بین uniffi و Kotlin است،
مخصوص هیچ پروژه‌ی خاصی نیست.

**راه‌حل (سطح اپلیکیشن):** فیلد متداخل رو توی تعریف enum سمت Rust به
هر اسم دیگه‌ای غیر از `message` تغییر بدید (مثلاً `detail`, `reason`)
— توی هر variant که داره، توی خودِ تعریف enum، و توی هر کد
`From<...>`ای که این variantها رو می‌سازه. بعد از صفر rebuild کنید
تا Kotlin تولیدشده این تغییر اسم رو بگیره.

#### ۸. `java.lang.Process.pid()` روی اندروید، در هیچ سطح APIای وجود نداره (سطح اپلیکیشن، نه مشکل بانا)

**نشونه:** کد Kotlin که `process.pid()` (API استاندارد JDK 9+) رو
صدا می‌زنه با `Unresolved reference 'pid'` کامپایل نمی‌شه — مهم
نیست `minSdk`، `compileSdk`، یا فعال‌بودن core library desugaring
چی باشه.

**علت:** با decompile مستقیم خودِ stub واقعی `android.jar` (اجرای
`javap` روی `java/lang/Process.class`) تأیید شد — حتی توی `compileSdk
35`، کلاس `Process` اندروید اصلاً متد `pid()` رو تعریف نمی‌کنه. این
یک gate سطح-API نیست که desugaring بتونه حلش کنه؛ این متد از اول
هیچ‌وقت به core library اندروید بازپورت نشده.

**راه‌حل (سطح اپلیکیشن):** شناسه‌ی فرآیند رو با reflection بخونید —
راه‌حل استاندارد و رایج جامعه‌ی اندروید:

```kotlin
private fun Process.pidCompat(): Long =
    try {
        val field = javaClass.getDeclaredField("pid")
        field.isAccessible = true
        field.getLong(this)
    } catch (e: Exception) {
        -1L
    }
```

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
