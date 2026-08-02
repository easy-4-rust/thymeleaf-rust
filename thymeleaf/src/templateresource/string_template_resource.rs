use std::io::{Cursor, Read};

use super::{ITemplateResource, TemplateResourceError};
use crate::TemplateInputException;

/// 完整保存在内存字符串中的模板资源。
///
/// 对应 Java: `org.thymeleaf.templateresource.StringTemplateResource`。
///
/// 本对象通常由 `StringTemplateResolver` 创建。资源内容不可变，每次 `reader` 调用都
/// 返回从头开始的新读取器；空字符串是合法且存在的模板。字符串资源没有可派生名称，
/// 也不支持创建相对资源。
pub struct StringTemplateResource {
    resource: String,
}

impl StringTemplateResource {
    /// 使用完整模板字符串创建内存资源。
    ///
    /// 对应 Java: `StringTemplateResource#StringTemplateResource(String)`。
    ///
    /// Java 的校验消息写作 “null or empty”，但实际只调用 `Validate.notNull`，因此
    /// 空字符串必须接受。
    ///
    /// # 参数
    /// - `resource`：完整模板内容；`None` 对应 Java `null`。
    ///
    /// # 错误
    /// `resource` 为 `None` 时返回与 Java 完全相同消息的参数错误。
    pub fn new(resource: Option<&str>) -> Result<Self, TemplateResourceError> {
        let resource = resource.ok_or_else(|| {
            TemplateResourceError::InvalidArgument("Resource cannot be null or empty".to_owned())
        })?;
        Ok(Self {
            resource: resource.to_owned(),
        })
    }
}

impl ITemplateResource for StringTemplateResource {
    fn get_description(&self) -> String {
        self.resource.clone()
    }

    fn get_base_name(&self) -> Option<String> {
        None
    }

    fn exists(&self) -> bool {
        true
    }

    fn reader(&self) -> std::io::Result<Box<dyn Read>> {
        Ok(Box::new(Cursor::new(self.resource.as_bytes().to_owned())))
    }

    fn relative(
        &self,
        relative_location: Option<&str>,
    ) -> Result<Box<dyn ITemplateResource>, TemplateResourceError> {
        let message = match relative_location {
            Some(_) | None => format!(
                "Cannot create a relative resource for String resource  \"{}\"",
                self.resource
            ),
        };
        Err(TemplateResourceError::Input(TemplateInputException::new(
            Some(message),
        )))
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use super::StringTemplateResource;
    use crate::templateresource::{ITemplateResource, TemplateResourceError};

    #[test]
    fn validates_only_null_and_accepts_empty_or_unicode_content() {
        let error = StringTemplateResource::new(None)
            .err()
            .expect("null resource must fail");
        assert_eq!(error.to_string(), "Resource cannot be null or empty");

        let empty = StringTemplateResource::new(Some("")).expect("empty resource is legal");
        assert_eq!(empty.get_description(), "");
        assert!(empty.exists());
        assert_eq!(empty.get_base_name(), None);

        let unicode =
            StringTemplateResource::new(Some("你好 😀\n")).expect("unicode resource is legal");
        assert_eq!(unicode.get_description(), "你好 😀\n");
    }

    #[test]
    fn creates_independent_readers_from_the_resource_start() {
        let resource = StringTemplateResource::new(Some("first\n第二 😀")).expect("valid resource");
        let mut first = resource.reader().expect("first reader");
        let mut prefix = [0_u8; 5];
        first.read_exact(&mut prefix).expect("read prefix");
        assert_eq!(&prefix, b"first");

        let mut second = resource.reader().expect("second reader");
        let mut complete = String::new();
        second
            .read_to_string(&mut complete)
            .expect("read complete resource");
        assert_eq!(complete, "first\n第二 😀");
    }

    #[test]
    fn rejects_every_relative_location_with_the_exact_input_exception() {
        let resource =
            StringTemplateResource::new(Some("line1\n\"line2\"")).expect("valid resource");

        for relative_location in [None, Some(""), Some("child.html")] {
            let error = resource
                .relative(relative_location)
                .err()
                .expect("string resources never support relatives");
            assert_eq!(
                error.to_string(),
                "Cannot create a relative resource for String resource  \"line1\n\"line2\"\""
            );
            let invalid_argument = TemplateResourceError::InvalidArgument(String::new());
            assert_ne!(
                std::mem::discriminant(&error),
                std::mem::discriminant(&invalid_argument)
            );
        }
    }
}
