# سرویس bana doctor — فعلاً فقط لایه‌ی HostEnvironment؛ لایه‌ی toolchain
# (JDK/SDK/NDK/AAPT2/Gradle) طبق ادامه‌ی فاز ۱ به همین‌جا اضافه می‌شود.
#
# The bana doctor service — currently only the HostEnvironment layer; the
# toolchain layer (JDK/SDK/NDK/AAPT2/Gradle) gets added here as Phase 1
# continues.

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
