# نقطه‌ی ورود CLI؛ دستورات نازک هستند و کار واقعی را به services می‌سپارند.
# CLI entry point; commands stay thin and delegate real work to services.

import typer

from bana.services import doctor as doctor_service
from bana.services import wiring

app = typer.Typer(
    name="bana",
    help="Build Automation for Native Android.",
)


@app.callback()
def main() -> None:
    """
    این callback خالی است ولی حیاتی است: بدون آن typer با تک‌دستوری فعلی
    (ping) رفتار تک‌فرمانی می‌گیرد و اسم دستور را نمی‌پذیرد. با اضافه‌شدن
    دستورهای بعدی (setup, doctor, ...) این حالت طبیعتاً درست می‌شد، ولی در
    فاز ۰ باید صریح باشد.

    This callback is empty but essential: without it, typer collapses into
    single-command mode with only `ping` registered and rejects the command
    name. Adding more commands later would fix this naturally, but Phase 0
    needs it explicit.
    """


@app.command()
def ping() -> None:
    """
    بررسی سلامت زنجیره‌ی Rust↔Python؛ فقط برای فاز ۰.
    Health-checks the Rust↔Python chain; Phase 0 only.
    """
    typer.echo(wiring.ping())


@app.command()
def doctor() -> None:
    """
    گزارش کامل و دوستانه‌ی وضعیت میزبان + توچین (فاز ۱: میزبان کامل، فقط
    JDK از توچین؛ SDK/NDK/AAPT2/Gradle در ادامه‌ی همین فاز اضافه می‌شود).
    Full, friendly host + toolchain status report (Phase 1: host complete,
    JDK only from the toolchain so far; SDK/NDK/AAPT2/Gradle land later in
    this same phase).
    """
    # نکته‌ی مهم: نام تابع (doctor) و ماژول سرویس هر دو doctor بودند؛ بدون
    # نام مستعار، همین def اسم ماژول سرویس را در namespace سراسری بازنویسی
    # می‌کرد و فراخوانی زیر با AttributeError شکست می‌خورد.
    # Important: this function's name and the service module were both
    # `doctor`; without the alias, this very def would overwrite the
    # service module's name in the global namespace and the call below
    # would fail with AttributeError.
    host = doctor_service.scan_host()
    toolchain = doctor_service.scan_toolchain()
    typer.echo(doctor_service.render_host_report(host))
    typer.echo(doctor_service.render_toolchain_report(toolchain))


if __name__ == "__main__":
    app()
