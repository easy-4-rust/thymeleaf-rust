use std::sync::OnceLock;

use crate::util::{VersionQualifier, VersionSpec, VersionUtils};

const VERSION: &str = "3.1.5.RELEASE";
const BUILD_TIMESTAMP: &str = "2026-04-21T20:38:36+0000";
static VERSION_SPEC: OnceLock<VersionSpec> = OnceLock::new();

/// Thymeleaf 兼容基线的版本与构建元数据入口。
///
/// 对应 Java: `org.thymeleaf.Thymeleaf`。
///
/// Java 实现从正式制品中的 `org/thymeleaf/thymeleaf.properties` 读取并解析版本；
/// Rust 制品在编译时固化同一 3.1.5.RELEASE 制品的已过滤属性，避免运行时类加载器
/// 差异。私有字段阻止外部构造，与 Java 私有构造器保持一致。
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Thymeleaf {
    _private: (),
}

impl Thymeleaf {
    /// 返回当前兼容的 Thymeleaf 完整版本。
    ///
    /// 对应 Java: `Thymeleaf#getVersion()`。
    ///
    /// # 返回
    /// 固定上游制品的版本 `3.1.5.RELEASE`。
    #[must_use]
    pub fn get_version() -> &'static str {
        version_spec().get_version()
    }

    /// 返回固定上游制品的构建时间戳。
    ///
    /// 对应 Java: `Thymeleaf#getBuildTimestamp()`。
    ///
    /// # 返回
    /// Maven Central 正式制品中 `build.date` 的值；使用 `Option` 保留 Java
    /// 返回类型可为 `null` 的合同。
    #[must_use]
    pub fn get_build_timestamp() -> Option<&'static str> {
        version_spec().get_build_timestamp()
    }

    /// 返回兼容版本的主版本号。
    ///
    /// 对应 Java: `Thymeleaf#getVersionMajor()`。
    ///
    /// # 返回
    /// 主版本号 `3`。
    #[must_use]
    pub fn get_version_major() -> i32 {
        version_spec().get_major()
    }

    /// 返回兼容版本的次版本号。
    ///
    /// 对应 Java: `Thymeleaf#getVersionMinor()`。
    ///
    /// # 返回
    /// 次版本号 `1`。
    #[must_use]
    pub fn get_version_minor() -> i32 {
        version_spec().get_minor()
    }

    /// 返回兼容版本的补丁版本号。
    ///
    /// 对应 Java: `Thymeleaf#getVersionPatch()`。
    ///
    /// # 返回
    /// 补丁版本号 `5`。
    #[must_use]
    pub fn get_version_patch() -> i32 {
        version_spec().get_patch()
    }

    /// 返回兼容版本的限定符。
    ///
    /// 对应 Java: `Thymeleaf#getVersionQualifier()`。
    ///
    /// # 返回
    /// 正式发布限定符 `RELEASE`；使用 `Option` 保留 Java 可空合同。
    #[must_use]
    pub fn get_version_qualifier() -> Option<&'static str> {
        version_spec()
            .get_qualifier()
            .and_then(VersionQualifier::as_str)
    }

    /// 判断兼容版本是否为稳定正式发布。
    ///
    /// 对应 Java: `Thymeleaf#isVersionStableRelease()`。
    ///
    /// # 返回
    /// 3.1.5.RELEASE 的限定符为 `RELEASE`，因此始终返回 `true`。
    #[must_use]
    pub fn is_version_stable_release() -> bool {
        version_spec().is_stable_release()
    }
}

fn version_spec() -> &'static VersionSpec {
    VERSION_SPEC.get_or_init(|| {
        VersionUtils::parse_version_with_build_timestamp(Some(VERSION), Some(BUILD_TIMESTAMP))
    })
}

#[cfg(test)]
mod tests {
    use super::Thymeleaf;

    #[test]
    fn exposes_exact_filtered_release_metadata() {
        assert_eq!(Thymeleaf::get_version(), "3.1.5.RELEASE");
        assert_eq!(
            Thymeleaf::get_build_timestamp(),
            Some("2026-04-21T20:38:36+0000")
        );
        assert_eq!(Thymeleaf::get_version_major(), 3);
        assert_eq!(Thymeleaf::get_version_minor(), 1);
        assert_eq!(Thymeleaf::get_version_patch(), 5);
        assert_eq!(Thymeleaf::get_version_qualifier(), Some("RELEASE"));
        assert!(Thymeleaf::is_version_stable_release());
    }
}
