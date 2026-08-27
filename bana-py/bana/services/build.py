# سرویس bana build — اجرای کامل pipeline (Gradle wrapper، build native،
# uniffi bindgen، پچ AAPT2، gradlew) و نمایش نتیجه‌ی واقعی.
#
# The bana build service — runs the full pipeline (Gradle wrapper, native
# build, uniffi bindgen, AAPT2 patch, gradlew) and shows the real result.

import json
from typing import Any

from bana import _bana_ffi


def run_build(repo_root: str, variant: str) -> dict[str, Any]:
    """
    فراخوانی اجرای واقعی کل pipeline ساخت.
    Calls the real execution of the full build pipeline.
    """
    return json.loads(_bana_ffi.run_build(repo_root, variant))


def render_build_result(result: dict[str, Any]) -> str:
    """
    تبدیل نتیجه‌ی خام build به متن خوانا و دوستانه.
    Turns the raw build result into readable, friendly text.
    """
    if result["success"]:
        return f"Build succeeded.\n  APK: {result['apk_path']}"
    return f"Build failed.\n\n{result['error']}"
