# سرویس bana setup — نصب idempotent لایه‌ی Bundled Tier (فعلاً JDK و
# Android SDK). هر ابزار از قبل واقعاً پیدا شده باشد، هیچ نصبی انجام
# نمی‌شود.
#
# The bana setup service — idempotent Bundled Tier install (JDK and
# Android SDK for now). If a tool is already really found, no install
# happens.

import json
from typing import Any

from bana import _bana_ffi

_OUTCOME_LABELS: dict[str, str] = {
    "already_satisfied": "already satisfied, nothing to do",
    "installed": "installed",
    "failed": "install failed",
    "no_backend": "no supported package manager found",
}


def run_setup() -> list[dict[str, Any]]:
    """
    فراخوانی نصب واقعی و idempotent لایه‌ی Bundled Tier.
    Calls the real, idempotent Bundled Tier install.
    """
    return json.loads(_bana_ffi.setup_bundled_tools())


def render_setup_report(actions: list[dict[str, Any]]) -> str:
    """
    تبدیل نتیجه‌ی خام setup به متن خوانا و دوستانه.
    Turns the raw setup result into readable, friendly text.
    """
    lines = ["bana setup -- result", ""]
    for action in actions:
        label = _OUTCOME_LABELS.get(action["outcome"], action["outcome"])
        line = f"  {action['tool_id']:<12}: {label}"
        if action.get("detail"):
            line += f" ({action['detail']})"
        lines.append(line)
    return "\n".join(lines)
