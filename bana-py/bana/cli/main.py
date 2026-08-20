# نقطه‌ی ورود CLI؛ دستورات نازک هستند و کار واقعی را به services می‌سپارند.
# CLI entry point; commands stay thin and delegate real work to services.

import typer

from bana.services import wiring

app = typer.Typer(
    name="bana",
    help="Build Automation for Native Android — یک دستور، صفر درگیری.",
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


if __name__ == "__main__":
    app()
