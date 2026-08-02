use std::io::{self, Read};
use std::sync::Arc;

use crate::web::IWebApplication;

use super::template_resource_reader::{is_java_empty_or_whitespace, transcoding_reader};
use super::template_resource_utils::TemplateResourceUtils;
use super::{ITemplateResource, TemplateResourceError};

/// 从 Web 应用根目录读取模板的资源。
///
/// 资源路径从 Web 应用文件根开始；Servlet 宿主中通常对应 `/WEB-INF`。路径清理后
/// 强制以 `/` 开头，每次读取都向应用请求新流，并按可选 Java charset 转码为 UTF-8。
/// 该对象通常由 `WebApplicationTemplateResolver` 创建。
///
/// 对应 Java: `org.thymeleaf.templateresource.WebApplicationTemplateResource`。
///
/// # 起始版本
/// 上游自 Thymeleaf 3.1.0 提供该对象。
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
    /// - `web_application`：应用对象；`None` 对应 Java `null`。
    /// - `path`：不可空且非空白资源路径。
    /// - `character_encoding`：可空字符集名称。
    ///
    /// # 返回值
    /// 返回路径已清理且以 `/` 开头的 Web 应用资源。
    ///
    /// # 错误
    /// 先校验应用对象，再校验资源路径；缺失值返回与 Java 构造器一致的参数错误。
    ///
    /// 对应 Java:
    /// `WebApplicationTemplateResource#WebApplicationTemplateResource(IWebApplication,String,String)`。
    pub fn new(
        web_application: Option<Arc<dyn IWebApplication>>,
        path: Option<&str>,
        character_encoding: Option<&str>,
    ) -> Result<Self, TemplateResourceError> {
        let web_application = web_application.ok_or_else(|| {
            TemplateResourceError::InvalidArgument(
                "Web Application object cannot be null".to_owned(),
            )
        })?;
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
