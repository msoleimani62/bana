# سرویس bana doctor — فعلاً لایه‌ی HostEnvironment و JDK؛ بقیه‌ی لایه‌ی
# toolchain (SDK/NDK/AAPT2/Gradle) طبق ادامه‌ی فاز ۱ به همین‌جا اضافه می‌شود.
#
# The bana doctor service — currently the HostEnvironment and JDK layers;
# the rest of the toolchain layer (SDK/NDK/AAPT2/Gradle) gets added here as
# Phase 1 continues.

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
    فراخوانی تشخیص واقعی توچین (فعلاً فقط JDK) و بازگرداندن آن به‌صورت dict.
    Calls the real toolchain detection (JDK only for now) and returns it as
    a dict.
    """
    return json.loads(_bana_ffi.scan_toolchain())


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


def render_toolchain_report(toolchain: dict[str, Any]) -> str:
    """
    تبدیل گزارش خام توچین به متن خوانا؛ فعلاً فقط خط JDK.
    Turns the raw toolchain report into readable text; JDK line only for now.
    """
    return "\n".join(["", "bana doctor -- toolchain report", "", _render_jdk(toolchain["jdk"])])
