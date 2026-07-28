use std::fmt::{Display, Formatter};
use std::io::Read;

use crate::TemplateInputException;

/// 模板解析器读取真实模板内容的资源契约。
///
/// 对应 Java: `org.thymeleaf.templateresource.ITemplateResource`。
///
/// 模板 Resolver 创建资源对象，用它抽象字符串、文件、URL 或宿主资源中的模板内容。
/// 资源对象本身存在不表示底层资源一定存在；调用 `exists` 可能实际访问文件或远程资源，
/// 因而可能带来一次额外 I/O。`reader` 每次返回新的可关闭读取器，调用方负责在消费后
/// 释放它。
///
/// 上游明确说明实现未必线程安全，因此本 trait 不强制 `Send + Sync`。具体实现可以
/// 根据自身所有权与宿主约束增加这些能力。
pub trait ITemplateResource {
    /// 返回用于日志和诊断的资源描述。
    ///
    /// 对应 Java: `ITemplateResource#getDescription()`。
    ///
    /// 描述不保证足够简短或唯一，不能直接当作稳定资源标识。
    ///
    /// # 返回
    /// 永不缺失的资源描述。
    fn get_description(&self) -> String;

    /// 返回可用于派生伴随资源名称的 base name。
    ///
    /// 对应 Java: `ITemplateResource#getBaseName()`。
    ///
    /// 例如 `/home/user/template/main.html` 的 base name 通常为 `main`。
    ///
    /// # 返回
    /// 可计算时返回 base name；`None` 对应 Java `null`。
    fn get_base_name(&self) -> Option<String>;

    /// 判断当前对象表示的底层资源是否真实存在。
    ///
    /// 对应 Java: `ITemplateResource#exists()`。
    ///
    /// # 返回
    /// 底层资源存在时返回 `true`。实现可以为此执行文件或网络访问。
    fn exists(&self) -> bool;

    /// 创建一个用于消费模板内容的新读取器。
    ///
    /// 对应 Java: `ITemplateResource#reader()`。
    ///
    /// Java `Reader` 是解码后的字符流；Rust 使用标准 `Read` 返回该字符流的 UTF-8
    /// 表示。文件和网络实现仍须在构造读取器时按配置 charset 完成解码。
    ///
    /// # 返回
    /// 新的非空读取器；每次调用都必须可以从资源起点重新读取。
    ///
    /// # 错误
    /// 底层资源不存在、无法访问或读取器初始化失败时返回 `std::io::Error`。
    fn reader(&self) -> std::io::Result<Box<dyn Read>>;

    /// 创建一个位于当前资源相对位置的新资源。
    ///
    /// 对应 Java: `ITemplateResource#relative(String)`。
    ///
    /// 部分资源类型不支持相对定位，此时返回对应的输入异常。
    ///
    /// # 参数
    /// - `relative_location`：Java 参数 `relativeLocation`；`None` 表示 Java `null`。
    ///
    /// # 返回
    /// 成功时返回通常具有相同具体实现类型的新资源。
    ///
    /// # 错误
    /// 参数校验失败或当前资源类型不支持相对资源时返回保留 Java 异常类别的错误。
    fn relative(
        &self,
        relative_location: Option<&str>,
    ) -> Result<Box<dyn ITemplateResource>, TemplateResourceError>;
}

/// 模板资源操作的错误类别。
///
/// 对应 Java `ITemplateResource` 各实现可能抛出的 `IllegalArgumentException` 和
/// `TemplateInputException`。这是 Rust 类型化错误扩展，不计入 Java 对象迁移分子。
#[derive(Debug)]
pub enum TemplateResourceError {
    /// 参数违反具体资源实现的前置条件。
    InvalidArgument(String),
    /// URL 文本无法构造成资源地址。
    MalformedUrl {
        /// 无法解析的原始 URL 位置。
        location: String,
        /// Rust URL 解析器报告的底层原因。
        source: url::ParseError,
    },
    /// 模板输入资源无法创建或定位。
    Input(TemplateInputException),
}

impl Display for TemplateResourceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArgument(message) => formatter.write_str(message),
            Self::MalformedUrl { location, source } => {
                write!(formatter, "Malformed URL \"{location}\": {source}")
            }
            Self::Input(error) => Display::fmt(error, formatter),
        }
    }
}

impl std::error::Error for TemplateResourceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidArgument(_) => None,
            Self::MalformedUrl { source, .. } => Some(source),
            Self::Input(error) => error.source(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::io::{self, Cursor};

    use super::{ITemplateResource, TemplateResourceError};
    use crate::TemplateInputException;

    struct CustomResource;

    impl ITemplateResource for CustomResource {
        fn get_description(&self) -> String {
            "custom".to_owned()
        }

        fn get_base_name(&self) -> Option<String> {
            Some("base".to_owned())
        }

        fn exists(&self) -> bool {
            false
        }

        fn reader(&self) -> io::Result<Box<dyn io::Read>> {
            Ok(Box::new(Cursor::new(b"custom".to_vec())))
        }

        fn relative(
            &self,
            relative_location: Option<&str>,
        ) -> Result<Box<dyn ITemplateResource>, TemplateResourceError> {
            match relative_location {
                Some("child") => Ok(Box::new(Self)),
                Some(_) | None => Err(TemplateResourceError::InvalidArgument(
                    "invalid relative location".to_owned(),
                )),
            }
        }
    }

    #[test]
    fn supports_dynamic_resources_without_forcing_thread_safety() {
        let resource: &dyn ITemplateResource = &CustomResource;
        assert_eq!(resource.get_description(), "custom");
        assert_eq!(resource.get_base_name(), Some("base".to_owned()));
        assert!(!resource.exists());
        assert!(resource.reader().is_ok());
        assert_eq!(
            resource
                .relative(Some("child"))
                .expect("relative resource")
                .get_description(),
            "custom"
        );
    }

    #[test]
    fn preserves_resource_error_categories_messages_and_sources() {
        let invalid = CustomResource
            .relative(None)
            .err()
            .expect("null location is invalid");
        assert_eq!(invalid.to_string(), "invalid relative location");
        assert!(invalid.source().is_none());

        let input = TemplateResourceError::Input(TemplateInputException::new(Some(
            "input failure".to_owned(),
        )));
        assert_eq!(input.to_string(), "input failure");
        assert!(input.source().is_none());

        let malformed_source =
            url::Url::parse("relative").expect_err("relative URL without a base must fail");
        let malformed = TemplateResourceError::MalformedUrl {
            location: "relative".to_owned(),
            source: malformed_source,
        };
        assert!(malformed.to_string().contains("relative"));
        assert!(malformed.source().is_some());
    }
}
