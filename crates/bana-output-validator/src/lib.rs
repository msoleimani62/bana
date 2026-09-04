//! اعتبارسنجی APK نهایی بعد از build — قبل از گزارش موفقیت به کاربر،
//! مطمئن می‌شویم فایل واقعاً یک APK سالم و کامل است، نه یک فایل خراب یا
//! نیمه‌کاره. عمداً هیچ‌وقت به aapt2 متکی نمی‌شود — طبق تجربه‌ی واقعی
//! این پروژه، خودِ نسخه/معماری aapt2 چندین بار منبع مستقل مشکل بوده
//! (رجوع به README اصلی، بخش «مشکلات شناخته‌شده»)؛ به‌جایش مستقیم
//! ساختار ZIP خودِ APK را می‌خواند، چون یک APK چیزی جز یک فایل ZIP
//! استاندارد نیست.
//! Validates the final APK after a build — before reporting success to
//! the user, confirms the file is genuinely a healthy, complete APK,
//! not a corrupt or partial one. Deliberately never depends on aapt2 —
//! per this project's own real experience, aapt2's version/architecture
//! has repeatedly been an independent source of problems (see the main
//! README's "Known Issues" section); reads the ZIP structure directly
//! instead, since an APK is nothing more than a standard ZIP file.

use std::fs::File;
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("APK not found at {path}")]
    NotFound { path: String },

    #[error("APK at {path} is empty (0 bytes)")]
    Empty { path: String },

    #[error("could not open {path} as a ZIP archive: {reason}")]
    NotAZip { path: String, reason: String },

    #[error("APK is missing required entry: {entry}")]
    MissingEntry { entry: String },

    #[error("APK is missing native library for ABI '{abi}' (expected under lib/{abi}/)")]
    MissingAbi { abi: String },
}

/// گزارش نتیجه‌ی اعتبارسنجی یک APK — هر فیلد از واقعیتی که مستقیم از
/// خودِ فایل ZIP خوانده شده می‌آید، هیچ‌چیز فرض نمی‌شود.
/// The validation result for one APK — every field comes from a fact
/// read directly from the ZIP file itself, nothing assumed.
#[derive(Debug, Clone)]
pub struct ApkReport {
    pub path: String,
    pub size_bytes: u64,
    pub has_manifest: bool,
    pub has_dex: bool,
    pub abis_present: Vec<String>,
    pub entry_count: usize,
}

/// اعتبارسنجی کامل یک APK: وجود فایل، سالم‌بودن ساختار ZIP، وجود
/// `AndroidManifest.xml` و حداقل یک `classes.dex`، و وجود یک `.so` برای
/// هر ABIای که در `expected_abis` خواسته شده. `expected_abis` خالی
/// یعنی بررسی ABI رد می‌شود (مثلاً برای سناریوی pure-kotlin که اصلاً
/// کد native ندارد).
/// Fully validates an APK: file existence, healthy ZIP structure,
/// presence of `AndroidManifest.xml` and at least one `classes.dex`,
/// and a native library present for every ABI listed in
/// `expected_abis`. An empty `expected_abis` skips the ABI check
/// (e.g. for a pure-kotlin scenario with no native code at all).
pub fn validate_apk(apk_path: &Path, expected_abis: &[&str]) -> Result<ApkReport, ValidationError> {
    let path_str = apk_path.to_string_lossy().to_string();

    let metadata = std::fs::metadata(apk_path).map_err(|_| ValidationError::NotFound {
        path: path_str.clone(),
    })?;

    if metadata.len() == 0 {
        return Err(ValidationError::Empty {
            path: path_str.clone(),
        });
    }

    let file = File::open(apk_path).map_err(|e| ValidationError::NotAZip {
        path: path_str.clone(),
        reason: e.to_string(),
    })?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| ValidationError::NotAZip {
        path: path_str.clone(),
        reason: e.to_string(),
    })?;

    let mut has_manifest = false;
    let mut has_dex = false;
    let mut abis_present: Vec<String> = Vec::new();

    for i in 0..archive.len() {
        let entry = archive.by_index(i).map_err(|e| ValidationError::NotAZip {
            path: path_str.clone(),
            reason: e.to_string(),
        })?;
        let name = entry.name();

        if name == "AndroidManifest.xml" {
            has_manifest = true;
        }
        if name == "classes.dex" || (name.starts_with("classes") && name.ends_with(".dex")) {
            has_dex = true;
        }
        if let Some(rest) = name.strip_prefix("lib/") {
            if let Some(slash) = rest.find('/') {
                let abi = &rest[..slash];
                if !abis_present.iter().any(|a| a == abi) {
                    abis_present.push(abi.to_string());
                }
            }
        }
    }

    if !has_manifest {
        return Err(ValidationError::MissingEntry {
            entry: "AndroidManifest.xml".to_string(),
        });
    }
    if !has_dex {
        return Err(ValidationError::MissingEntry {
            entry: "classes.dex".to_string(),
        });
    }
    for abi in expected_abis {
        if !abis_present.iter().any(|a| a == abi) {
            return Err(ValidationError::MissingAbi {
                abi: (*abi).to_string(),
            });
        }
    }

    Ok(ApkReport {
        path: path_str,
        size_bytes: metadata.len(),
        has_manifest,
        has_dex,
        abis_present,
        entry_count: archive.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// یک APK آزمایشی واقعی (فایل ZIP واقعی، نه شبیه‌سازی) با entryهای
    /// دلخواه می‌سازد — دقیقاً همان چیزی که تست باید ببیند.
    /// Builds a real test APK (an actual ZIP file, not a simulation)
    /// with the given entries — exactly what the test needs to see.
    fn write_test_apk(path: &Path, entries: &[(&str, &[u8])]) {
        let file = File::create(path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::FileOptions::default();
        for (name, content) in entries {
            zip.start_file(*name, options).unwrap();
            zip.write_all(content).unwrap();
        }
        zip.finish().unwrap();
    }

    #[test]
    fn reports_not_found_for_missing_file() {
        let result = validate_apk(Path::new("/tmp/does-not-exist-bana-test.apk"), &[]);
        assert!(matches!(result, Err(ValidationError::NotFound { .. })));
    }

    #[test]
    fn reports_empty_for_zero_byte_file() {
        let path = std::env::temp_dir().join("bana-test-empty.apk");
        File::create(&path).unwrap();
        let result = validate_apk(&path, &[]);
        assert!(matches!(result, Err(ValidationError::Empty { .. })));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn reports_missing_manifest_when_absent() {
        let path = std::env::temp_dir().join("bana-test-no-manifest.apk");
        write_test_apk(&path, &[("classes.dex", b"fake")]);
        let result = validate_apk(&path, &[]);
        assert!(matches!(
            result,
            Err(ValidationError::MissingEntry { entry }) if entry == "AndroidManifest.xml"
        ));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn reports_missing_dex_when_absent() {
        let path = std::env::temp_dir().join("bana-test-no-dex.apk");
        write_test_apk(&path, &[("AndroidManifest.xml", b"fake")]);
        let result = validate_apk(&path, &[]);
        assert!(matches!(
            result,
            Err(ValidationError::MissingEntry { entry }) if entry == "classes.dex"
        ));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn reports_missing_abi_when_requested_abi_absent() {
        let path = std::env::temp_dir().join("bana-test-missing-abi.apk");
        write_test_apk(
            &path,
            &[
                ("AndroidManifest.xml", b"fake"),
                ("classes.dex", b"fake"),
                ("lib/arm64-v8a/libmobile_core.so", b"fake"),
            ],
        );
        let result = validate_apk(&path, &["arm64-v8a", "x86_64"]);
        assert!(matches!(
            result,
            Err(ValidationError::MissingAbi { abi }) if abi == "x86_64"
        ));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn skips_abi_check_when_none_expected() {
        // برای سناریوی pure-kotlin (بدون کد native) — نباید هیچ ABIای
        // اجباری باشد.
        // For the pure-kotlin scenario (no native code) — no ABI
        // should ever be required.
        let path = std::env::temp_dir().join("bana-test-no-abi-required.apk");
        write_test_apk(
            &path,
            &[("AndroidManifest.xml", b"fake"), ("classes.dex", b"fake")],
        );
        let result = validate_apk(&path, &[]);
        assert!(result.is_ok());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn succeeds_and_reports_all_abis_present() {
        let path = std::env::temp_dir().join("bana-test-valid.apk");
        write_test_apk(
            &path,
            &[
                ("AndroidManifest.xml", b"fake"),
                ("classes.dex", b"fake"),
                ("lib/arm64-v8a/libmobile_core.so", b"fake"),
                ("lib/armeabi-v7a/libmobile_core.so", b"fake"),
                ("lib/x86_64/libmobile_core.so", b"fake"),
                ("lib/x86/libmobile_core.so", b"fake"),
            ],
        );
        let report = validate_apk(&path, &["arm64-v8a", "armeabi-v7a", "x86_64", "x86"]).unwrap();
        assert!(report.has_manifest);
        assert!(report.has_dex);
        assert_eq!(report.abis_present.len(), 4);
        assert_eq!(report.entry_count, 6);
        assert!(report.size_bytes > 0);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn reports_not_a_zip_for_garbage_content() {
        let path = std::env::temp_dir().join("bana-test-garbage.apk");
        let mut f = File::create(&path).unwrap();
        f.write_all(b"this is definitely not a zip file, just plain garbage bytes")
            .unwrap();
        let result = validate_apk(&path, &[]);
        assert!(matches!(result, Err(ValidationError::NotAZip { .. })));
        let _ = std::fs::remove_file(&path);
    }
}
