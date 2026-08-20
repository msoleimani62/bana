# fixture: pure-kotlin-empty

پروژه‌ی اندروید مصنوعیِ حداقلی، فقط Kotlin/Gradle خالص، بدون هیچ لایه‌ی
native. برای تست `project_analyzer` (باید سناریوی `pure-kotlin` را تشخیص
دهد) و `build_driver` بدون وابستگی به بی‌مرز واقعی استفاده می‌شود.

A minimal synthetic Android project — pure Kotlin/Gradle, no native layer.
Used to test `project_analyzer` (must detect the `pure-kotlin` scenario) and
`build_driver` without depending on the real bimarz project.

**وضعیت / status:** اسکلت فاز ۰ — محتوای واقعی Gradle در فاز ۳/۴ اضافه
می‌شود. Phase 0 skeleton — real Gradle content added during Phase 3/4.
