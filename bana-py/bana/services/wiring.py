# پوشش نازک روی تابع تست Rust؛ بعداً با سرویس‌های واقعی (اسکن، build) پر می‌شود.
# Thin wrapper over the Rust test function; will fill up with real services
# (scan, build, ...) as later phases land.

from bana import _bana_ffi


def ping() -> str:
    """
    فراخوانی تابع تست ping در bana-ffi برای تأیید صحت زنجیره‌ی ساخت.
    Calls the ping test function in bana-ffi to confirm the build chain works.
    """
    return _bana_ffi.ping()
