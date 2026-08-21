# سرویس bana doctor — لایه‌ی میزبان (HostEnvironment)، لایه‌ی توچین کامل
# (JDK/SDK/NDK/AAPT2)، و لایه‌ی سطح پروژه (Gradle wrapper).
#
# The bana doctor service — the host layer (HostEnvironment), the full
# toolchain layer (JDK/SDK/NDK/AAPT2), and the project-level layer
# (Gradle wrapper).

import json
from typing import Any

from bana import _bana_ffi

# نگاشت مقادیر خام enum به توضیح دوستانه‌ی انگلیسی؛ طبق قانون کاربر هیچ
# فارسی‌ای داخل کد/خروجی برنامه مجاز نیست، فقط کامنت‌ها دوزبانه‌اند.
# Maps raw enum values to a friendly English description; per the user's
# rule, no Persian is allowed inside code/program output — only comments
# are bilingual.
_HOST_KIND_LABELS: dict[str, str] = {
    "Termux": "Termux (no extra proot layer)",
    "KaliNetHunterProot": "Kali NetHunter (inside proot on Android)",
    "NativeLinux": "native Linux",
    "Windows": "Windows",
    "MacOs": "macOS",
    "Unknown": "unrecognized — please open an issue on GitHub",
}


def scan_host() -> dict[str, Any]:
    """
    فراخوانی تشخیص واقعی محیط میزبان و بازگرداندن آن به‌صورت dict پایتونی.
    Calls the real host-environment detection and returns it as a Python dict.
    """
    return json.loads(_bana_ffi.scan_host())


def scan_toolchain() -> dict[str, Any]:
    """
    فراخوانی تشخیص واقعی توچین (JDK/SDK/NDK/AAPT2) و بازگرداندن آن به‌صورت dict.
    Calls the real toolchain detection (JDK/SDK/NDK/AAPT2) and returns it
    as a dict.
    """
    return json.loads(_bana_ffi.scan_toolchain())


def scan_gradle_wrapper(project_root: str) -> Any:
    """
    فراخوانی تشخیص واقعی Gradle wrapper داخل یک پروژه‌ی مشخص.
    Calls the real Gradle wrapper detection inside a specific project.
    """
    return json.loads(_bana_ffi.scan_gradle_wrapper(project_root))


def _render_gradle_wrapper(wrapper: Any) -> str:
    """
    تبدیل ToolStatus<GradleWrapperInfo> خام به یک خط خوانا؛ همان نکته‌ی
    `NotFound` درباره‌ی بقیه این‌جا هم صادق است.
    Turns a raw ToolStatus<GradleWrapperInfo> into one readable line; the
    same `NotFound`-is-a-plain-string note from the others applies here too.
    """
    if wrapper == "NotFound":
        return (
            "  Gradle      : no wrapper found in this directory. Run `gradle wrapper` "
            "inside your Android project root, or run `bana doctor` from inside one."
        )
    if "Found" in wrapper:
        version = wrapper["Found"]["info"]["distribution_version"]
        return f"  Gradle      : wrapper found, targets version {version}"
    if "FoundButIncompatible" in wrapper:
        reason = wrapper["FoundButIncompatible"]["reason"]
        return f"  Gradle      : wrapper incomplete ({reason})"
    if "AmbiguousMultiple" in wrapper:
        return "  Gradle      : ambiguous wrapper state, please open an issue on GitHub"
    return "  Gradle      : unrecognized status — please open an issue on GitHub"


def render_project_report(project_root: str, wrapper: Any) -> str:
    """
    گزارش سطح پروژه (نه میزبان) — فعلاً فقط Gradle wrapper.
    Project-level (not host-level) report — Gradle wrapper only for now.
    """
    return "\n".join(
        [
            "",
            f"bana doctor -- project report ({project_root})",
            "",
            _render_gradle_wrapper(wrapper),
        ]
    )


def _render_jdk(jdk: Any) -> str:
    """
    تبدیل ToolStatus<JdkInfo> خام به یک خط خوانا، طبق آدرس‌دهی دقیق اصل ۱۱.
    نکته‌ی مهم: `NotFound` (واریانت بدون داده) به‌صورت رشته‌ی خام "NotFound"
    سریالایز می‌شود، نه dict — باید جدا از واریانت‌های دیگر بررسی شود، وگرنه
    چک `"Found" in jdk` روی خودِ رشته هم به‌اشتباه True برمی‌گرداند، چون
    "Found" زیررشته‌ی "NotFound" هم هست.

    Turns a raw ToolStatus<JdkInfo> into one readable line, per the precise
    remediation-addressing rule (principle 11). Important: `NotFound` (the
    data-less variant) serializes as the plain string "NotFound", not a
    dict — it must be checked separately from the other variants, or the
    `"Found" in jdk` check would wrongly return True on the string itself,
    since "Found" is also a substring of "NotFound".
    """
    if jdk == "NotFound":
        return (
            "  JDK         : not found. Install a JDK (e.g. `apt install openjdk-17-jdk` "
            "on Debian/Kali, `pacman -S jdk-openjdk` on Arch) and run `bana doctor` again."
        )
    if "Found" in jdk:
        return f"  JDK         : found, version {jdk['Found']['info']['version']}"
    if "FoundButIncompatible" in jdk:
        reason = jdk["FoundButIncompatible"]["reason"]
        return f"  JDK         : found but unusable ({reason})"
    if "AmbiguousMultiple" in jdk:
        count = len(jdk["AmbiguousMultiple"]["candidates"])
        return f"  JDK         : {count} versions found at once, pick one manually for now"
    return "  JDK         : unrecognized status — please open an issue on GitHub"


def _render_sdk(sdk: Any) -> str:
    """
    تبدیل ToolStatus<SdkInfo> خام به یک خط خوانا؛ همان نکته‌ی `NotFound`
    درباره‌ی JDK این‌جا هم صادق است.
    Turns a raw ToolStatus<SdkInfo> into one readable line; the same
    `NotFound`-is-a-plain-string note from JDK applies here too.
    """
    if sdk == "NotFound":
        return (
            "  SDK         : not found. Set ANDROID_HOME to your SDK root, or install "
            "one (e.g. `apt install android-sdk` on Debian/Kali) and run `bana doctor` again."
        )
    if "Found" in sdk:
        info = sdk["Found"]["info"]
        platforms = ", ".join(info["installed_platforms"]) or "none"
        build_tools = ", ".join(info["installed_build_tools"]) or "none"
        return (
            f"  SDK         : found at {sdk['Found']['path']}\n"
            f"                platforms: {platforms}\n"
            f"                build-tools: {build_tools}"
        )
    if "FoundButIncompatible" in sdk:
        reason = sdk["FoundButIncompatible"]["reason"]
        return f"  SDK         : found but unusable ({reason})"
    if "AmbiguousMultiple" in sdk:
        count = len(sdk["AmbiguousMultiple"]["candidates"])
        return f"  SDK         : {count} candidate roots found at once, pick one manually for now"
    return "  SDK         : unrecognized status — please open an issue on GitHub"


def render_host_report(host: dict[str, Any]) -> str:
    """
    تبدیل گزارش خام میزبان به متن خوانا و دوستانه برای کاربر آماتور.
    Turns the raw host report into readable, friendly text for amateur users.
    """
    kind_label = _HOST_KIND_LABELS.get(host["kind"], host["kind"])
    lines = [
        "bana doctor -- host report",
        "",
        f"  Environment : {kind_label}",
        f"  Arch        : {host['arch']}",
        f"  Shell       : {host['shell']}",
        f"  Home        : {host['home_dir']}",
    ]
    if host["systemd_stubbed"]:
        lines.append(
            "  Note: systemd looks stubbed. This is expected on proot "
            "environments and needs no action."
        )
    return "\n".join(lines)


def _render_ndk(ndk: Any) -> str:
    """
    تبدیل ToolStatus<NdkInfo> خام به یک خط خوانا؛ همان نکته‌ی `NotFound`
    درباره‌ی JDK/SDK این‌جا هم صادق است.
    Turns a raw ToolStatus<NdkInfo> into one readable line; the same
    `NotFound`-is-a-plain-string note from JDK/SDK applies here too.
    """
    if ndk == "NotFound":
        return (
            "  NDK         : not found. Install one via `sdkmanager --install "
            '"ndk;26.1.10909125"` (adjust the version as needed) and run `bana doctor` again.'
        )
    if "Found" in ndk:
        return f"  NDK         : found, version {ndk['Found']['info']['version']}"
    if "FoundButIncompatible" in ndk:
        reason = ndk["FoundButIncompatible"]["reason"]
        return f"  NDK         : found but unusable ({reason})"
    if "AmbiguousMultiple" in ndk:
        count = len(ndk["AmbiguousMultiple"]["candidates"])
        return f"  NDK         : {count} versions found at once, pick one manually for now"
    return "  NDK         : unrecognized status — please open an issue on GitHub"


def _render_aapt2(aapt2: Any) -> str:
    """
    تبدیل ToolStatus<Aapt2Info> خام به یک خط خوانا؛ همان نکته‌ی `NotFound`
    درباره‌ی بقیه این‌جا هم صادق است.
    Turns a raw ToolStatus<Aapt2Info> into one readable line; the same
    `NotFound`-is-a-plain-string note from the others applies here too.
    """
    if aapt2 == "NotFound":
        return (
            "  AAPT2       : not found. It ships inside SDK build-tools; "
            "install one via `sdkmanager --install \"build-tools;34.0.0\"` "
            "and run `bana doctor` again."
        )
    if "Found" in aapt2:
        return f"  AAPT2       : found, version {aapt2['Found']['info']['version']}"
    if "FoundButIncompatible" in aapt2:
        reason = aapt2["FoundButIncompatible"]["reason"]
        return f"  AAPT2       : found but unusable ({reason})"
    if "AmbiguousMultiple" in aapt2:
        count = len(aapt2["AmbiguousMultiple"]["candidates"])
        return f"  AAPT2       : {count} binaries found at once, pick one manually for now"
    return "  AAPT2       : unrecognized status — please open an issue on GitHub"


def render_toolchain_report(toolchain: dict[str, Any]) -> str:
    """
    تبدیل گزارش خام توچین به متن خوانا؛ فعلاً خط JDK، SDK، NDK، و AAPT2.
    Turns the raw toolchain report into readable text; JDK, SDK, NDK, and
    AAPT2 lines for now.
    """
    return "\n".join(
        [
            "",
            "bana doctor -- toolchain report",
            "",
            _render_jdk(toolchain["jdk"]),
            _render_sdk(toolchain["sdk"]),
            _render_ndk(toolchain["ndk"]),
            _render_aapt2(toolchain["aapt2"]),
        ]
    )
