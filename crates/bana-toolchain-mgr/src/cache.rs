//! کش content-addressed زیر `~/.cache/bana/<hash>` — طبق بخش ۵ RULES.md.
//! مستقل از هر پروژه‌ی خاص است؛ اگر دو پروژه‌ی مختلف به دقیقاً همان
//! نسخه‌ی یک ابزار (مثلاً NDK ۲۶.۱) نیاز داشته باشند، فقط یک‌بار دانلود/
//! نصب می‌شود.
//! Content-addressed cache under `~/.cache/bana/<hash>` — per RULES.md
//! section 5. Independent of any specific project; if two different
//! projects need exactly the same tool version (e.g. NDK 26.1), it only
//! gets downloaded/installed once.

use std::path::{Path, PathBuf};

/// هش FNV-1a با ۶۴ بیت — بدون نیاز به وابستگی رمزنگاری سنگین (این کش
/// برای شناسایی است، نه امنیت)، ولی همیشه یکسان و قطعی است.
/// FNV-1a 64-bit hash — no need for a heavy cryptographic dependency
/// (this cache is for identification, not security), but always
/// deterministic and stable.
pub fn compute_cache_key(parts: &[&str]) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for part in parts {
        for byte in part.bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
        // جداکننده بین قطعات، تا [«ab»,«c»] با [«a»,«bc»] قاطی نشود.
        // Separator between parts, so [«ab»,«c»] and [«a»,«bc»] never collide.
        hash ^= 0xff;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

/// مسیر پیش‌فرض ریشه‌ی کش: `<home>/.cache/bana`.
/// The default cache root path: `<home>/.cache/bana`.
pub fn default_cache_root(home_dir: &Path) -> PathBuf {
    home_dir.join(".cache").join("bana")
}

/// مسیر کامل کش برای یک کلید مشخص (مثلاً `["ndk", "26.1.10909125"]`).
/// The full cache path for a specific key (e.g. `["ndk", "26.1.10909125"]`).
pub fn cache_dir_for(cache_root: &Path, key_parts: &[&str]) -> PathBuf {
    cache_root.join(compute_cache_key(key_parts))
}

/// اطمینان از وجود پوشه‌ی کش برای یک کلید؛ می‌گوید آیا از قبل وجود داشته
/// (یعنی از پروژه‌ی قبلی reuse شده) یا همین الان ساخته شده.
/// Ensures the cache folder for a key exists; reports whether it already
/// existed (i.e. reused from a previous project) or was just created.
pub fn ensure_cache_dir(cache_root: &Path, key_parts: &[&str]) -> std::io::Result<(PathBuf, bool)> {
    let dir = cache_dir_for(cache_root, key_parts);
    let already_existed = dir.exists();
    std::fs::create_dir_all(&dir)?;
    Ok((dir, already_existed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_key_parts_produce_the_same_hash() {
        let a = compute_cache_key(&["ndk", "26.1.10909125"]);
        let b = compute_cache_key(&["ndk", "26.1.10909125"]);
        assert_eq!(a, b);
    }

    #[test]
    fn different_key_parts_produce_different_hashes() {
        let a = compute_cache_key(&["ndk", "26.1.10909125"]);
        let b = compute_cache_key(&["ndk", "25.2.9519653"]);
        assert_ne!(a, b);
    }

    #[test]
    fn part_boundaries_do_not_collide() {
        // بدون جداکننده، ["ab", "c"] و ["a", "bc"] هش یکسان می‌گرفتند.
        // Without a separator, ["ab", "c"] and ["a", "bc"] would hash the same.
        let a = compute_cache_key(&["ab", "c"]);
        let b = compute_cache_key(&["a", "bc"]);
        assert_ne!(a, b);
    }

    #[test]
    fn cache_dir_for_builds_expected_path() {
        let root = PathBuf::from("/home/kali/.cache/bana");
        let key = compute_cache_key(&["ndk", "26.1.10909125"]);
        let expected = root.join(&key);
        assert_eq!(cache_dir_for(&root, &["ndk", "26.1.10909125"]), expected);
    }

    #[test]
    fn ensure_cache_dir_creates_and_then_reports_reuse() {
        let tmp_root = std::env::temp_dir().join(format!(
            "bana-cache-test-{}",
            crate::recorder::current_unix_timestamp()
        ));

        let (dir_first, existed_first) =
            ensure_cache_dir(&tmp_root, &["ndk", "26.1.10909125"]).unwrap();
        assert!(!existed_first);
        assert!(dir_first.exists());

        let (dir_second, existed_second) =
            ensure_cache_dir(&tmp_root, &["ndk", "26.1.10909125"]).unwrap();
        assert!(existed_second);
        assert_eq!(dir_first, dir_second);

        std::fs::remove_dir_all(&tmp_root).ok();
    }
}
