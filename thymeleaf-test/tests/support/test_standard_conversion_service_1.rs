use std::any::Any;

use thymeleaf::expression::{
    AbstractStandardConversionService, ConversionObject, StandardConversionError,
    Utf16StringConversionResult,
};
use thymeleaf::util::Utf16String;

/// 在默认字符串转换结果外增加方括号的测试转换服务。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.conversion.conversion1.TestStandardConversionService1`。
pub struct TestStandardConversionService1;

impl AbstractStandardConversionService for TestStandardConversionService1 {
    /// 调用 Java 默认 `toString()` 语义，并在结果外增加方括号。
    ///
    /// 对应 Java: `TestStandardConversionService1#convertToString`。
    fn convert_to_string<'a>(
        &self,
        _context: Option<&dyn Any>,
        object: &'a dyn ConversionObject,
    ) -> Result<Utf16StringConversionResult<'a>, StandardConversionError> {
        let converted = match object.java_to_string()? {
            Utf16StringConversionResult::Null => "null".to_owned(),
            Utf16StringConversionResult::Borrowed(value) => value.to_string_lossy(),
            Utf16StringConversionResult::Owned(value) => value.to_string_lossy(),
        };
        Ok(Utf16StringConversionResult::Owned(
            Utf16String::from_rust_str(&format!("[{converted}]")),
        ))
    }
}
