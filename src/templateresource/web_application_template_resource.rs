use std::io::{self, Read};
use std::sync::Arc;

use crate::web::IWebApplication;

use super::template_resource_reader::{is_java_empty_or_whitespace, transcoding_reader};
use super::template_resource_utils::TemplateResourceUtils;
use super::{ITemplateResource, TemplateResourceError};

/// 从 Web 应用根目录读取模板的资源。
///
/// 路径清理后强制以 `/` 开头；每次读取都向应用请求新流，并按可选 Java charset
/// 转码为 UTF-8。对应 Java:
/// `org.thymeleaf.templateresource.WebApplicationTemplateResource`。
pub struct WebApplicationTemplateResource {
    web_application: Arc<dyn IWebApplication>,
    path: String,
    character_encoding: Option<String>,
}

impl WebApplicationTemplateResource {
    /// 创建 Web 应用模板资源。
    ///
    /// # 参数
    ///
    /// - `web_application`：不可空应用对象。
    /// - `path`：不可空且非空白资源路径。
    /// - `character_encoding`：可空字符集名称。
    ///
    /// 对应 Java: `WebApplicationTemplateResource#WebApplicationTemplateResource`。
    pub fn new(
        web_application: Arc<dyn IWebApplication>,
        path: Option<&str>,
        character_encoding: Option<&str>,
    ) -> Result<Self, TemplateResourceError> {
        let path = path
            .filter(|value| !is_java_empty_or_whitespace(value))
            .ok_or_else(|| {
                TemplateResourceError::InvalidArgument(
                    "Resource Path cannot be null or empty".to_owned(),
                )
            })?;
        Ok(Self::from_validated_path(
            web_application,
            path,
            character_encoding,
        ))
    }

    fn from_validated_path(
        web_application: Arc<dyn IWebApplication>,
        path: &str,
        character_encoding: Option<&str>,
    ) -> Self {
        let clean_path =
            TemplateResourceUtils::clean_path(Some(path)).expect("validated path is non-null");
        let path = if clean_path.starts_with('/') {
            clean_path
        } else {
            format!("/{clean_path}")
        };
        Self {
            web_application,
            path,
            character_encoding: character_encoding.map(str::to_owned),
        }
    }
}

impl ITemplateResource for WebApplicationTemplateResource {
    fn get_description(&self) -> String {
        self.path.clone()
    }

    fn get_base_name(&self) -> Option<String> {
        TemplateResourceUtils::compute_base_name(Some(&self.path))
    }

    fn exists(&self) -> bool {
        self.web_application
            .resource_exists(Some(&crate::util::JavaString::from_rust_str(&self.path)))
    }

    fn reader(&self) -> io::Result<Box<dyn Read>> {
        let path = crate::util::JavaString::from_rust_str(&self.path);
        let input = self
            .web_application
            .get_resource_as_stream(Some(&path))
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Web Application resource \"{}\" does not exist", self.path),
                )
            })?;
        transcoding_reader(input, self.character_encoding.as_deref())
    }

    fn relative(
        &self,
        relative_location: Option<&str>,
    ) -> Result<Box<dyn ITemplateResource>, TemplateResourceError> {
        let relative_location = relative_location
            .filter(|value| !is_java_empty_or_whitespace(value))
            .ok_or_else(|| {
                TemplateResourceError::InvalidArgument(
                    "Relative Path cannot be null or empty".to_owned(),
                )
            })?;
        let full_relative_location =
            TemplateResourceUtils::compute_relative_location(&self.path, relative_location);
        Ok(Box::new(Self::from_validated_path(
            Arc::clone(&self.web_application),
            &full_relative_location,
            self.character_encoding.as_deref(),
        )))
    }
}
