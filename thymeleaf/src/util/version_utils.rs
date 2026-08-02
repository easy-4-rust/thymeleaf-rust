use unicode_general_category::{GeneralCategory, get_general_category};

const STABLE_RELEASE_QUALIFIER_UTF16: &[u16] =
    &[0x0052, 0x0045, 0x004C, 0x0045, 0x0041, 0x0053, 0x0045];
const UNKNOWN_VERSION: &str = "UNKNOWN";

// Java 21 使用 Unicode 15.0；Java char 只能直接表示 BMP 码位，因此这里只登记 BMP
// 中 Decimal_Digit_Number 类别的零码位。补充平面数字在 UTF-16 中是代理项，不会被
// Character.isDigit(char) 识别。
const JAVA_BMP_DECIMAL_ZEROES: &[u16] = &[
    0x0030, 0x0660, 0x06F0, 0x07C0, 0x0966, 0x09E6, 0x0A66, 0x0AE6, 0x0B66, 0x0BE6, 0x0C66, 0x0CE6,
    0x0D66, 0x0DE6, 0x0E50, 0x0ED0, 0x0F20, 0x1040, 0x1090, 0x17E0, 0x1810, 0x1946, 0x19D0, 0x1A80,
    0x1A90, 0x1B50, 0x1BB0, 0x1C40, 0x1C50, 0xA620, 0xA8D0, 0xA900, 0xA9D0, 0xA9F0, 0xAA50, 0xABF0,
    0xFF10,
];

/// Thymeleaf 版本号解析工具。
///
/// 对应 Java: `org.thymeleaf.util.VersionUtils`。
///
/// 本对象保留上游宽容解析合同：null、Java `String#trim()` 后为空或任意解析失败均
/// 返回 `UNKNOWN`，而不是向调用方传播异常。数字段、限定符分隔符、构建时间戳和
/// 稳定版本判断均由 [`VersionSpec`] 保存。
pub struct VersionUtils;

impl VersionUtils {
    /// 解析不带构建时间戳的版本字符串。
    ///
    /// 对应 Java: `VersionUtils#parseVersion(String)`。
    ///
    /// # 参数
    /// - `version`：Java 参数 `version`；`None` 对应 Java null。
    ///
    /// # 返回
    /// 已解析版本；输入无效时返回 `UNKNOWN` 规格。
    #[must_use]
    pub fn parse_version(version: Option<&str>) -> VersionSpec {
        Self::parse_version_with_build_timestamp(version, None)
    }

    /// 解析版本字符串并附带原样构建时间戳。
    ///
    /// 对应 Java: `VersionUtils#parseVersion(String,String)`。
    ///
    /// # 参数
    /// - `version`：Java 参数 `version`；仅按 Java `String#trim()` 去除首尾
    ///   `U+0000..U+0020`。
    /// - `build_timestamp`：Java 参数 `buildTimestamp`；不校验、不裁剪，空串与
    ///   null 保持不同。
    ///
    /// # 返回
    /// 已解析版本；任何数字格式、范围或结构错误都转换为 `UNKNOWN`。
    #[must_use]
    pub fn parse_version_with_build_timestamp(
        version: Option<&str>,
        build_timestamp: Option<&str>,
    ) -> VersionSpec {
        let Some(version) = version else {
            return VersionSpec::unknown(build_timestamp);
        };
        let version = java_trim_utf16(version);
        if version.is_empty() {
            return VersionSpec::unknown(build_timestamp);
        }

        parse_known_version(&version, build_timestamp)
            .unwrap_or_else(|| VersionSpec::unknown(build_timestamp))
    }
}

/// 精确保留 Java `String` UTF-16 码元的版本限定符。
///
/// 对应 Java 返回类型: `VersionUtils.VersionSpec#getQualifier()` 的 `String`。
///
/// 上游按 `charAt()` 判断限定符分隔符。补充平面字符会被拆成代理项，使限定符可能
/// 以孤立低代理项开头，无法由 Rust `str` 表达。本包装保留原始码元；常规有效
/// Unicode 限定符仍可通过 [`VersionQualifier::as_str`] 零拷贝读取。
#[derive(Clone, Debug)]
pub struct VersionQualifier {
    utf16: Vec<u16>,
    scalar: Option<String>,
}

impl VersionQualifier {
    fn from_utf16(utf16: &[u16]) -> Self {
        Self {
            utf16: utf16.to_vec(),
            scalar: String::from_utf16(utf16).ok(),
        }
    }

    /// 返回原始 Java UTF-16 码元。
    ///
    /// # 返回
    /// 与 Java `String#charAt(int)` 逐项一致的只读码元切片。
    /// 对应 Java 语义：`VersionUtils` 的 `as_utf16` 行为（Rust 侧辅助/私有路径）。
    #[must_use]
    pub fn as_utf16(&self) -> &[u16] {
        &self.utf16
    }

    /// 在限定符是有效 Unicode 标量序列时返回字符串借用。
    ///
    /// # 返回
    /// 常规限定符返回 `Some(&str)`；含孤立代理项时返回 `None`，原始值仍可从
    /// [`VersionQualifier::as_utf16`] 读取。
    /// 对应 Java 语义：`VersionUtils` 的 `as_str` 行为（Rust 侧辅助/私有路径）。
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        self.scalar.as_deref()
    }

    /// 按 Unicode 替换字符生成便于日志显示的文本。
    ///
    /// # 返回
    /// 等价于对原始 UTF-16 码元执行有损解码的拥有字符串。
    /// 对应 Java 语义：`VersionUtils` 的 `to_string_lossy` 行为（Rust 侧辅助/私有路径）。
    #[must_use]
    pub fn to_string_lossy(&self) -> String {
        String::from_utf16_lossy(&self.utf16)
    }
}

/// 由 [`VersionUtils`] 解析得到的不可变版本规格。
///
/// 对应 Java: `org.thymeleaf.util.VersionUtils.VersionSpec`。
///
/// Java 内部类的两个构造器均为 private；Rust 同样只允许通过
/// [`VersionUtils::parse_version`] 或
/// [`VersionUtils::parse_version_with_build_timestamp`] 创建。
#[derive(Clone, Debug)]
pub struct VersionSpec {
    unknown: bool,
    major: i32,
    minor: i32,
    patch: i32,
    qualifier: Option<VersionQualifier>,
    build_timestamp: Option<String>,
    version_core: String,
    version: String,
    full_version: String,
}

impl VersionSpec {
    fn unknown(build_timestamp: Option<&str>) -> Self {
        let build_timestamp = build_timestamp.map(ToOwned::to_owned);
        let version = UNKNOWN_VERSION.to_owned();
        let full_version = format_full_version(&version, build_timestamp.as_deref());
        Self {
            unknown: true,
            major: 0,
            minor: 0,
            patch: 0,
            qualifier: None,
            build_timestamp,
            version_core: version.clone(),
            version,
            full_version,
        }
    }

    fn known(
        major: i32,
        minor: Option<i32>,
        patch: Option<i32>,
        qualifier_separator: Option<u16>,
        qualifier: Option<VersionQualifier>,
        build_timestamp: Option<&str>,
    ) -> Option<Self> {
        if major < 0 || minor.is_some_and(|value| value < 0) || patch.is_some_and(|value| value < 0)
        {
            return None;
        }

        let version_core = match (minor, patch) {
            (Some(minor), Some(patch)) => format!("{major}.{minor}.{patch}"),
            (Some(minor), None) => format!("{major}.{minor}"),
            (None, None) => major.to_string(),
            (None, Some(_)) => return None,
        };
        let version = compose_version(&version_core, qualifier_separator, qualifier.as_ref());
        let build_timestamp = build_timestamp.map(ToOwned::to_owned);
        let full_version = format_full_version(&version, build_timestamp.as_deref());

        Some(Self {
            unknown: false,
            major,
            minor: minor.unwrap_or(0),
            patch: patch.unwrap_or(0),
            qualifier,
            build_timestamp,
            version_core,
            version,
            full_version,
        })
    }

    /// 判断解析结果是否为未知版本。
    ///
    /// 对应 Java: `VersionSpec#isUnknown()`。
    ///
    /// # 返回
    /// 输入缺失或解析失败时返回 `true`。
    #[must_use]
    pub const fn is_unknown(&self) -> bool {
        self.unknown
    }

    /// 返回主版本号；未知版本返回零。
    ///
    /// 对应 Java: `VersionSpec#getMajor()`。
    #[must_use]
    pub const fn get_major(&self) -> i32 {
        self.major
    }

    /// 返回次版本号；未提供或未知版本返回零。
    ///
    /// 对应 Java: `VersionSpec#getMinor()`。
    #[must_use]
    pub const fn get_minor(&self) -> i32 {
        self.minor
    }

    /// 返回补丁版本号；未提供或未知版本返回零。
    ///
    /// 对应 Java: `VersionSpec#getPatch()`。
    #[must_use]
    pub const fn get_patch(&self) -> i32 {
        self.patch
    }

    /// 判断版本是否携带限定符。
    ///
    /// 对应 Java: `VersionSpec#hasQualifier()`。
    #[must_use]
    pub const fn has_qualifier(&self) -> bool {
        self.qualifier.is_some()
    }

    /// 返回限定符，不包含原始分隔符。
    ///
    /// 对应 Java: `VersionSpec#getQualifier()`。
    ///
    /// # 返回
    /// 无限定符或未知版本返回 `None`；返回对象可精确读取 Java UTF-16 码元。
    #[must_use]
    pub fn get_qualifier(&self) -> Option<&VersionQualifier> {
        self.qualifier.as_ref()
    }

    /// 返回规范化数字核心。
    ///
    /// 对应 Java: `VersionSpec#getVersionCore()`。
    ///
    /// # 返回
    /// 已解析数字段组成的文本，或 `UNKNOWN`。
    #[must_use]
    pub fn get_version_core(&self) -> &str {
        &self.version_core
    }

    /// 返回不含构建时间戳的完整版本。
    ///
    /// 对应 Java: `VersionSpec#getVersion()`。
    #[must_use]
    pub fn get_version(&self) -> &str {
        &self.version
    }

    /// 判断是否携带构建时间戳。
    ///
    /// 对应 Java: `VersionSpec#hasBuildTimestamp()`。
    #[must_use]
    pub const fn has_build_timestamp(&self) -> bool {
        self.build_timestamp.is_some()
    }

    /// 返回构建时间戳原文。
    ///
    /// 对应 Java: `VersionSpec#getBuildTimestamp()`。
    ///
    /// # 返回
    /// 调用方传入 null 时返回 `None`；空串仍为 `Some("")`。
    #[must_use]
    pub fn get_build_timestamp(&self) -> Option<&str> {
        self.build_timestamp.as_deref()
    }

    /// 返回带可选构建时间戳的展示版本。
    ///
    /// 对应 Java: `VersionSpec#getFullVersion()`。
    #[must_use]
    pub fn get_full_version(&self) -> &str {
        &self.full_version
    }

    /// 判断是否至少达到指定主版本。
    ///
    /// 对应 Java: `VersionSpec#isAtLeast(int)`。
    ///
    /// # 参数
    /// - `major`：比较目标主版本。
    #[must_use]
    pub const fn is_at_least(&self, major: i32) -> bool {
        self.is_at_least_with_minor(major, 0)
    }

    /// 判断是否至少达到指定主、次版本。
    ///
    /// 对应 Java: `VersionSpec#isAtLeast(int,int)`。
    ///
    /// # 参数
    /// - `major`：比较目标主版本。
    /// - `minor`：比较目标次版本。
    #[must_use]
    pub const fn is_at_least_with_minor(&self, major: i32, minor: i32) -> bool {
        self.is_at_least_with_patch(major, minor, 0)
    }

    /// 判断是否至少达到指定主、次、补丁版本。
    ///
    /// 对应 Java: `VersionSpec#isAtLeast(int,int,int)`。
    ///
    /// # 参数
    /// - `major`：比较目标主版本。
    /// - `minor`：比较目标次版本。
    /// - `patch`：比较目标补丁版本。
    #[must_use]
    pub const fn is_at_least_with_patch(&self, major: i32, minor: i32, patch: i32) -> bool {
        self.major > major
            || (self.major == major && self.minor > minor)
            || (self.major == major && self.minor == minor && self.patch >= patch)
    }

    /// 判断是否为稳定正式版本。
    ///
    /// 对应 Java: `VersionSpec#isStableRelease()`。
    ///
    /// # 返回
    /// 已知且无限定符，或限定符精确等于大写 `RELEASE` 时返回 `true`。
    #[must_use]
    pub fn is_stable_release(&self) -> bool {
        (!self.unknown
            && self
                .qualifier
                .as_ref()
                .is_none_or(|qualifier| qualifier.as_utf16().is_empty()))
            || self
                .qualifier
                .as_ref()
                .is_some_and(|qualifier| qualifier.as_utf16() == STABLE_RELEASE_QUALIFIER_UTF16)
    }
}

fn parse_known_version(version: &[u16], build_timestamp: Option<&str>) -> Option<VersionSpec> {
    let (numeric_version, qualifier_separator, qualifier) =
        match find_end_of_numeric_version(version) {
            None => (version, None, None),
            Some(end) => {
                let numeric_version = &version[..end];
                let separator = version[end];
                if is_java_letter(separator) {
                    (
                        numeric_version,
                        None,
                        Some(VersionQualifier::from_utf16(&version[end..])),
                    )
                } else {
                    let qualifier = &version[end + 1..];
                    if java_trim_utf16_units(qualifier).is_empty() {
                        return None;
                    }
                    (
                        numeric_version,
                        Some(separator),
                        Some(VersionQualifier::from_utf16(qualifier)),
                    )
                }
            }
        };

    let separator1 = numeric_version
        .iter()
        .position(|unit| *unit == u16::from(b'.'));
    let (major, minor, patch) = match separator1 {
        None => (parse_java_i32(numeric_version)?, None, None),
        Some(separator1) => {
            let major = parse_java_i32(&numeric_version[..separator1])?;
            let remaining = &numeric_version[separator1 + 1..];
            match remaining.iter().position(|unit| *unit == u16::from(b'.')) {
                None => (major, Some(parse_java_i32(remaining)?), None),
                Some(separator2) => (
                    major,
                    Some(parse_java_i32(&remaining[..separator2])?),
                    Some(parse_java_i32(&remaining[separator2 + 1..])?),
                ),
            }
        }
    };

    VersionSpec::known(
        major,
        minor,
        patch,
        qualifier_separator,
        qualifier,
        build_timestamp,
    )
}

fn find_end_of_numeric_version(sequence: &[u16]) -> Option<usize> {
    for (index, unit) in sequence.iter().copied().enumerate() {
        if unit != u16::from(b'.') && java_decimal_digit(unit).is_none() {
            if index > 1 && sequence[index - 1] == u16::from(b'.') {
                return Some(index - 1);
            }
            return Some(index);
        }
    }
    None
}

fn parse_java_i32(sequence: &[u16]) -> Option<i32> {
    let (negative, digits) = match sequence.first().copied() {
        Some(unit) if unit == u16::from(b'-') => (true, &sequence[1..]),
        Some(unit) if unit == u16::from(b'+') => (false, &sequence[1..]),
        Some(_) => (false, sequence),
        None => return None,
    };
    if digits.is_empty() {
        return None;
    }

    // 与 Integer.parseInt 相同，使用负数累积以允许 i32::MIN。
    let limit = if negative { i32::MIN } else { -i32::MAX };
    let multiply_limit = limit / 10;
    let mut result = 0_i32;
    for unit in digits {
        let digit = i32::from(java_decimal_digit(*unit)?);
        if result < multiply_limit {
            return None;
        }
        result *= 10;
        if result < limit + digit {
            return None;
        }
        result -= digit;
    }
    Some(if negative { result } else { -result })
}

fn java_decimal_digit(unit: u16) -> Option<u8> {
    JAVA_BMP_DECIMAL_ZEROES.iter().find_map(|zero| {
        let offset = unit.checked_sub(*zero)?;
        (offset <= 9).then_some(offset as u8)
    })
}

fn is_java_letter(unit: u16) -> bool {
    let Some(character) = char::from_u32(u32::from(unit)) else {
        return false;
    };
    matches!(
        get_general_category(character),
        GeneralCategory::UppercaseLetter
            | GeneralCategory::LowercaseLetter
            | GeneralCategory::TitlecaseLetter
            | GeneralCategory::ModifierLetter
            | GeneralCategory::OtherLetter
    )
}

fn java_trim_utf16(value: &str) -> Vec<u16> {
    let units = value.encode_utf16().collect::<Vec<_>>();
    java_trim_utf16_units(&units).to_vec()
}

fn java_trim_utf16_units(units: &[u16]) -> &[u16] {
    let start = units
        .iter()
        .position(|unit| *unit > 0x20)
        .unwrap_or(units.len());
    let end = units
        .iter()
        .rposition(|unit| *unit > 0x20)
        .map_or(start, |index| index + 1);
    &units[start..end]
}

fn compose_version(
    version_core: &str,
    qualifier_separator: Option<u16>,
    qualifier: Option<&VersionQualifier>,
) -> String {
    let Some(qualifier) = qualifier else {
        return version_core.to_owned();
    };
    let mut units = version_core.encode_utf16().collect::<Vec<_>>();
    if let Some(separator) = qualifier_separator {
        units.push(separator);
    }
    units.extend_from_slice(qualifier.as_utf16());
    String::from_utf16(&units).expect("public parse input recomposes valid UTF-16")
}

fn format_full_version(version: &str, build_timestamp: Option<&str>) -> String {
    build_timestamp.map_or_else(
        || version.to_owned(),
        |build_timestamp| format!("{version} ({build_timestamp})"),
    )
}

#[cfg(test)]
mod tests {
    use super::{
        VersionSpec, VersionUtils, compose_version, find_end_of_numeric_version,
        format_full_version, is_java_letter, java_decimal_digit, java_trim_utf16_units,
        parse_java_i32,
    };

    #[test]
    fn parses_numeric_shapes_qualifiers_and_build_timestamp() {
        let version =
            VersionUtils::parse_version_with_build_timestamp(Some(" 3.1.5.RELEASE "), Some(""));
        assert!(!version.is_unknown());
        assert_eq!(version.get_major(), 3);
        assert_eq!(version.get_minor(), 1);
        assert_eq!(version.get_patch(), 5);
        assert!(version.has_qualifier());
        assert_eq!(
            version
                .get_qualifier()
                .and_then(super::VersionQualifier::as_str),
            Some("RELEASE")
        );
        assert_eq!(version.get_version_core(), "3.1.5");
        assert_eq!(version.get_version(), "3.1.5.RELEASE");
        assert!(version.has_build_timestamp());
        assert_eq!(version.get_build_timestamp(), Some(""));
        assert_eq!(version.get_full_version(), "3.1.5.RELEASE ()");
        assert!(version.is_stable_release());

        let major = VersionUtils::parse_version(Some("7"));
        assert_eq!(major.get_version_core(), "7");
        assert_eq!(major.get_minor(), 0);
        assert_eq!(major.get_patch(), 0);
        assert!(!major.has_qualifier());
        assert!(!major.has_build_timestamp());
        assert_eq!(major.get_build_timestamp(), None);
        assert_eq!(major.get_full_version(), "7");
        assert!(major.is_stable_release());

        let minor = VersionUtils::parse_version(Some("7.2RC1"));
        assert_eq!(minor.get_version(), "7.2RC1");
        assert_eq!(
            minor
                .get_qualifier()
                .and_then(super::VersionQualifier::as_str),
            Some("RC1")
        );
        assert!(!minor.is_stable_release());
    }

    #[test]
    fn returns_unknown_for_every_public_parse_failure_shape() {
        for input in [
            None,
            Some(""),
            Some("\u{0000}\t "),
            Some("-1"),
            Some("1."),
            Some("1.2."),
            Some("1..2"),
            Some("1.2.3.4"),
            Some("2147483648"),
            Some("1- "),
        ] {
            let version = VersionUtils::parse_version_with_build_timestamp(input, Some("build"));
            assert_unknown(&version, Some("build"));
        }

        let non_trimmed_nbsp = VersionUtils::parse_version(Some("\u{00A0}"));
        assert_unknown(&non_trimmed_nbsp, None);
    }

    #[test]
    fn preserves_unicode_digits_letters_and_java_utf16_boundaries() {
        let arabic = VersionUtils::parse_version(Some("١.٢.٣"));
        assert_eq!(arabic.get_version(), "1.2.3");

        let fullwidth = VersionUtils::parse_version(Some("９.８β"));
        assert_eq!(fullwidth.get_version(), "9.8β");
        assert_eq!(
            fullwidth
                .get_qualifier()
                .and_then(super::VersionQualifier::as_str),
            Some("β")
        );

        let supplementary = VersionUtils::parse_version(Some("1𐐀"));
        assert_eq!(supplementary.get_version(), "1𐐀");
        let qualifier = supplementary.get_qualifier().expect("qualifier");
        assert_eq!(qualifier.as_utf16(), &[0xDC00]);
        assert_eq!(qualifier.as_str(), None);
        assert_eq!(qualifier.to_string_lossy(), "\u{FFFD}");

        assert_eq!(java_decimal_digit(u16::from(b'0')), Some(0));
        assert_eq!(java_decimal_digit(0xFF19), Some(9));
        assert_eq!(java_decimal_digit(u16::from(b'A')), None);
        assert!(is_java_letter(u16::from(b'A')));
        assert!(is_java_letter(0x01C5));
        assert!(is_java_letter(0x02B0));
        assert!(is_java_letter(0x4E00));
        assert!(!is_java_letter(0x0301));
        assert!(!is_java_letter(0xD835));
    }

    #[test]
    fn compares_all_three_numeric_levels_even_for_unknown_versions() {
        let version = VersionUtils::parse_version(Some("3.1.5"));
        assert!(version.is_at_least(2));
        assert!(version.is_at_least(3));
        assert!(!version.is_at_least(4));
        assert!(version.is_at_least_with_minor(3, 1));
        assert!(!version.is_at_least_with_minor(3, 2));
        assert!(version.is_at_least_with_patch(3, 1, 5));
        assert!(!version.is_at_least_with_patch(3, 1, 6));

        let unknown = VersionUtils::parse_version(None);
        assert!(unknown.is_at_least(-1));
        assert!(unknown.is_at_least_with_minor(0, 0));
        assert!(unknown.is_at_least_with_patch(0, 0, 0));
        assert!(!unknown.is_stable_release());
    }

    #[test]
    fn internal_helpers_preserve_java_overflow_trim_and_composition_rules() {
        assert_eq!(
            parse_java_i32(&"2147483647".encode_utf16().collect::<Vec<_>>()),
            Some(i32::MAX)
        );
        assert_eq!(
            parse_java_i32(&"-2147483648".encode_utf16().collect::<Vec<_>>()),
            Some(i32::MIN)
        );
        assert_eq!(
            parse_java_i32(&"+1".encode_utf16().collect::<Vec<_>>()),
            Some(1)
        );
        assert_eq!(parse_java_i32(&[]), None);
        assert_eq!(parse_java_i32(&[u16::from(b'+')]), None);
        assert_eq!(
            parse_java_i32(&"2147483648".encode_utf16().collect::<Vec<_>>()),
            None
        );
        assert_eq!(
            parse_java_i32(&"99999999999".encode_utf16().collect::<Vec<_>>()),
            None
        );
        assert_eq!(
            parse_java_i32(&"-2147483649".encode_utf16().collect::<Vec<_>>()),
            None
        );

        let numeric = "1.2".encode_utf16().collect::<Vec<_>>();
        assert_eq!(find_end_of_numeric_version(&numeric), None);
        let trailing_dot = "1.RC".encode_utf16().collect::<Vec<_>>();
        assert_eq!(find_end_of_numeric_version(&trailing_dot), Some(1));
        let immediate = "R".encode_utf16().collect::<Vec<_>>();
        assert_eq!(find_end_of_numeric_version(&immediate), Some(0));

        assert_eq!(
            java_trim_utf16_units(&[0, 0x20, u16::from(b'x'), 0x20]),
            &[u16::from(b'x')]
        );
        assert!(java_trim_utf16_units(&[0, 0x20]).is_empty());
        let qualifier = super::VersionQualifier::from_utf16(&[u16::from(b'R'), u16::from(b'C')]);
        assert_eq!(
            compose_version("1", Some(u16::from(b'-')), Some(&qualifier)),
            "1-RC"
        );
        assert_eq!(compose_version("1", None, None), "1");
        assert_eq!(format_full_version("1", None), "1");

        assert!(VersionSpec::known(-1, None, None, None, None, None).is_none());
        assert!(VersionSpec::known(1, Some(-1), None, None, None, None).is_none());
        assert!(VersionSpec::known(1, Some(0), Some(-1), None, None, None).is_none());
        assert!(VersionSpec::known(1, None, Some(0), None, None, None).is_none());
    }

    fn assert_unknown(version: &VersionSpec, timestamp: Option<&str>) {
        assert!(version.is_unknown());
        assert_eq!(version.get_major(), 0);
        assert_eq!(version.get_minor(), 0);
        assert_eq!(version.get_patch(), 0);
        assert!(!version.has_qualifier());
        assert!(version.get_qualifier().is_none());
        assert_eq!(version.get_version_core(), "UNKNOWN");
        assert_eq!(version.get_version(), "UNKNOWN");
        assert_eq!(version.has_build_timestamp(), timestamp.is_some());
        assert_eq!(version.get_build_timestamp(), timestamp);
        assert_eq!(
            version.get_full_version(),
            timestamp.map_or_else(
                || "UNKNOWN".to_owned(),
                |value| format!("UNKNOWN ({value})")
            )
        );
        assert!(!version.is_stable_release());
    }
}
