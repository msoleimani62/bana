# پوشش نازک روی تابع تست Rust؛ فقط برای بررسی سلامت زنجیره‌ی ساخت است.
# Thin wrapper over the Rust test function; only for build-chain health checks.

from bana import _bana_ffi


def ping() -> str:
    """
    فراخوانی تابع تست ping در bana-ffi برای تأیید صحت زنجیره‌ی ساخت.
    Calls the ping test function in bana-ffi to confirm the build chain works.
    """
    return _bana_ffi.ping()
