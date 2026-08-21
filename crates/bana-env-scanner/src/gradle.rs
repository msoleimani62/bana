//! تشخیص واقعی Gradle wrapper داخل یک پروژه‌ی مشخص. برخلاف بقیه‌ی پروب‌ها،
//! این یکی به محیط میزبان کاری ندارد — فقط به سه فایل واقعی داخل خودِ
//! پوشه‌ی پروژه: `gradlew`، `gradle/wrapper/gradle-wrapper.jar`، و
//! `gradle/wrapper/gradle-wrapper.properties`.
//! Real Gradle wrapper detection inside a specific project. Unlike the
//! other probes, this one doesn't care about the host environment — only
//! three real files inside the project folder itself: `gradlew`,
//! `gradle/wrapper/gradle-wrapper.jar`, and
//! `gradle/wrapper/gradle-wrapper.properties`.

use crate::host::EnvProbe;
use bana_types::{GradleWrapperInfo, ToolStatus};
use std::path::Path;

/// استخراج نسخه از `distributionUrl` واقعی، مثل:
/// `distributionUrl=https\://services.gradle.org/distributions/gradle-8.7-bin.zip`
/// Extracts the version from a real `distributionUrl` line, e.g. the one
/// shown above.
fn parse_distribution_version(properties: &str) -> Option<String> {
    let line = properties
        .lines()
        .find(|l| l.trim_start().starts_with("distributionUrl"))?;
    let url = line.splitn(2, '=').nth(1)?.trim();
    let file_name = url.rsplit('/').next()?;
    let without_prefix = file_name.strip_prefix("gradle-")?;
    let version = without_prefix.split('-').next()?;
    Some(version.to_string())
}

pub fn detect_gradle_wrapper(probe: &dyn EnvProbe, project_root: &Path) -> ToolStatus<GradleWrapperInfo> {
    let gradlew_present = probe.path_exists(&project_root.join("gradlew"));
    let wrapper_jar_present = probe.path_exists(
        &project_root
            .join("gradle")
            .join("wrapper")
            .join("gradle-wrapper.jar"),
    );
    let properties_content = probe.read_to_string(
        &project_root
            .join("gradle")
            .join("wrapper")
            .join("gradle-wrapper.properties"),
    );

    let Some(properties_content) = properties_content else {
        if !gradlew_present && !wrapper_jar_present {
            return ToolStatus::NotFound;
        }
        return ToolStatus::FoundButIncompatible {
            path: project_root.to_path_buf(),
            info: GradleWrapperInfo {
                gradlew_present,
                wrapper_jar_present,
                distribution_version: "unknown".to_string(),
            },
            reason: "gradle-wrapper.properties is missing".to_string(),
        };
    };

    let Some(distribution_version) = parse_distribution_version(&properties_content) else {
        return ToolStatus::FoundButIncompatible {
            path: project_root.to_path_buf(),
            info: GradleWrapperInfo {
                gradlew_present,
                wrapper_jar_present,
                distribution_version: "unknown".to_string(),
            },
            reason: "gradle-wrapper.properties found but distributionUrl could not be parsed"
                .to_string(),
        };
    };

    if !gradlew_present || !wrapper_jar_present {
        return ToolStatus::FoundButIncompatible {
            path: project_root.to_path_buf(),
            info: GradleWrapperInfo {
                gradlew_present,
                wrapper_jar_present,
                distribution_version,
            },
            reason: format!(
                "gradle wrapper is incomplete (gradlew present: {gradlew_present}, \
                 wrapper jar present: {wrapper_jar_present})"
            ),
        };
    }

    ToolStatus::Found {
        path: project_root.to_path_buf(),
        info: GradleWrapperInfo {
            gradlew_present: true,
            wrapper_jar_present: true,
            distribution_version,
        },
        verified: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[derive(Default)]
    struct MockProbe {
        existing_paths: Vec<PathBuf>,
        files: HashMap<PathBuf, String>,
    }

    impl EnvProbe for MockProbe {
        fn read_env(&self, _key: &str) -> Option<String> {
            None
        }
        fn path_exists(&self, path: &Path) -> bool {
            self.existing_paths.iter().any(|p| p == path)
        }
        fn read_to_string(&self, path: &Path) -> Option<String> {
            self.files.get(path).cloned()
        }
        fn list_dir(&self, _path: &Path) -> Vec<String> {
            Vec::new()
        }
        fn read_bytes(&self, _path: &Path, _max_len: usize) -> Option<Vec<u8>> {
            None
        }
    }

    const REAL_PROPS: &str = "distributionUrl=https\\://services.gradle.org/distributions/gradle-8.7-bin.zip\n";

    #[test]
    fn detects_complete_wrapper() {
        let root = PathBuf::from("/home/kali/bimarz/android");
        let mut probe = MockProbe::default();
        probe.existing_paths.push(root.join("gradlew"));
        probe
            .existing_paths
            .push(root.join("gradle/wrapper/gradle-wrapper.jar"));
        probe.files.insert(
            root.join("gradle/wrapper/gradle-wrapper.properties"),
            REAL_PROPS.to_string(),
        );

        match detect_gradle_wrapper(&probe, &root) {
            ToolStatus::Found { info, .. } => {
                assert!(info.gradlew_present);
                assert!(info.wrapper_jar_present);
                assert_eq!(info.distribution_version, "8.7");
            }
            other => panic!("expected Found, got {other:?}"),
        }
    }

    #[test]
    fn reports_not_found_when_nothing_present() {
        let root = PathBuf::from("/home/kali/some-empty-project");
        let probe = MockProbe::default();
        assert!(matches!(
            detect_gradle_wrapper(&probe, &root),
            ToolStatus::NotFound
        ));
    }

    #[test]
    fn reports_incompatible_when_gradlew_missing() {
        let root = PathBuf::from("/home/kali/bimarz/android");
        let mut probe = MockProbe::default();
        probe
            .existing_paths
            .push(root.join("gradle/wrapper/gradle-wrapper.jar"));
        probe.files.insert(
            root.join("gradle/wrapper/gradle-wrapper.properties"),
            REAL_PROPS.to_string(),
        );

        assert!(matches!(
            detect_gradle_wrapper(&probe, &root),
            ToolStatus::FoundButIncompatible { .. }
        ));
    }

    #[test]
    fn reports_incompatible_when_properties_missing_but_gradlew_present() {
        let root = PathBuf::from("/home/kali/bimarz/android");
        let mut probe = MockProbe::default();
        probe.existing_paths.push(root.join("gradlew"));

        assert!(matches!(
            detect_gradle_wrapper(&probe, &root),
            ToolStatus::FoundButIncompatible { .. }
        ));
    }

    #[test]
    fn reports_incompatible_when_distribution_url_unparseable() {
        let root = PathBuf::from("/home/kali/bimarz/android");
        let mut probe = MockProbe::default();
        probe.existing_paths.push(root.join("gradlew"));
        probe
            .existing_paths
            .push(root.join("gradle/wrapper/gradle-wrapper.jar"));
        probe.files.insert(
            root.join("gradle/wrapper/gradle-wrapper.properties"),
            "someOtherKey=someValue\n".to_string(),
        );

        assert!(matches!(
            detect_gradle_wrapper(&probe, &root),
            ToolStatus::FoundButIncompatible { .. }
        ));
    }
}
