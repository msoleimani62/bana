# نقطه‌ی ورود CLI؛ دستورات نازک هستند و کار واقعی را به services می‌سپارند.
# CLI entry point; commands stay thin and delegate real work to services.

from pathlib import Path

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
    گزارش کامل و دوستانه‌ی وضعیت میزبان + توچین + پروژه‌ی فعلی (فاز ۱
    کامل: میزبان، JDK، SDK، NDK، AAPT2، و Gradle wrapper مسیر کاری فعلی).
    Full, friendly host + toolchain + current-project status report
    (Phase 1 complete: host, JDK, SDK, NDK, AAPT2, and the Gradle wrapper
    of the current working directory).
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

    # گزارش سطح پروژه اختیاری است: bana doctor باید بیرون از هر پروژه‌ای
    # هم بدون خطا کار کند (طبق اصل ۱۲ RULES.md — راهنمایی دوستانه، نه
    # شکست تیز).
    # The project-level report is best-effort: bana doctor must still work
    # cleanly outside any project (per RULES.md principle 12 — friendly
    # guidance, not a hard crash).
    project_root = str(Path.cwd())
    wrapper = doctor_service.scan_gradle_wrapper(project_root)
    typer.echo(doctor_service.render_project_report(project_root, wrapper))


if __name__ == "__main__":
    app()
