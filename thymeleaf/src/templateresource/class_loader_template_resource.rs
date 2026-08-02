use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::PathBuf;

use super::template_resource_reader::{is_java_empty_or_whitespace, transcoding_reader};
use super::template_resource_utils::TemplateResourceUtils;
use super::{ITemplateResource, TemplateResourceError};
use crate::util::ResourceLoaderUtils;

/// 从 Rust 应用资源搜索路径读取类路径模板资源。
///
/// Java `ClassLoader` 没有 Rust 运行时等价物，因此该对象把 classpath 明确映射为有序
/// 搜索根目录：显式根目录优先；默认依次搜索可执行文件目录、crate manifest 目录和
/// 当前工作目录。资源路径仍完全遵循 Java 的清理、去除开头 `/`、相对定位与编码
/// 语义。
///
/// 对应 Java: `org.thymeleaf.templateresource.ClassLoaderTemplateResource`。
///
/// # 起始版本
/// 上游自 Thymeleaf 3.0.0 提供该对象；不指定具体加载器的构造方式自 3.0.3 起提供。
pub struct ClassLoaderTemplateResource {
    search_roots: Vec<PathBuf>,
    path: String,
    character_encoding: Option<String>,
}

impl ClassLoaderTemplateResource {
    /// 使用默认资源搜索顺序创建 classpath 资源。
    ///
    /// 未指定 Java `ClassLoader` 时，上游按线程上下文加载器、引擎加载器和系统加载器
    /// 的顺序查找。Rust 以 [`ResourceLoaderUtils::get_resource_roots`] 返回的有序根
    /// 目录承担相同职责。
    ///
    /// # 参数
    /// - `path`：模板资源路径；`None` 对应 Java `null`。
    /// - `character_encoding`：读取资源使用的字符编码；`None` 或全空白使用系统默认
    ///   UTF-8。
    ///
    /// # 返回值
    /// 返回清理路径并去除开头 `/` 的类路径资源。
    ///
    /// # 错误
    /// `path` 缺失、为空或仅含 Java 空白字符时返回参数错误。
    pub fn new(
        path: Option<&str>,
        character_encoding: Option<&str>,
    ) -> Result<Self, TemplateResourceError> {
        Self::with_search_roots(
            ResourceLoaderUtils::get_resource_roots(),
            path,
            character_encoding,
        )
    }

    /// 使用指定有序搜索根目录创建 classpath 资源。
    ///
    /// 该构造方式对应 Java 显式传入 `ClassLoader`：搜索根顺序由调用方固定，首个实际
    /// 文件即为该资源的解析结果。
    ///
    /// # 参数
    /// - `search_roots`：按优先级排列的资源根目录。
    /// - `path`：模板资源路径；`None` 对应 Java `null`。
    /// - `character_encoding`：读取资源使用的字符编码。
    ///
    /// # 返回值
    /// 返回绑定指定搜索根的类路径资源。
    ///
    /// # 错误
    /// `path` 缺失、为空或仅含 Java 空白字符时返回参数错误。
    pub fn with_search_roots(
        search_roots: Vec<PathBuf>,
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
        let clean_path =
            TemplateResourceUtils::clean_path(Some(path)).expect("validated path is non-null");
        Ok(Self {
            search_roots,
            path: clean_path
                .strip_prefix('/')
                .unwrap_or(&clean_path)
                .to_owned(),
            character_encoding: character_encoding.map(str::to_owned),
        })
    }

    fn resolved_path(&self) -> Option<PathBuf> {
        self.search_roots
            .iter()
            .map(|root| root.join(&self.path))
            .find(|candidate| candidate.is_file())
    }
}

impl ITemplateResource for ClassLoaderTemplateResource {
    fn get_description(&self) -> String {
        self.path.clone()
    }

    fn get_base_name(&self) -> Option<String> {
        TemplateResourceUtils::compute_base_name(Some(&self.path))
    }

    fn exists(&self) -> bool {
        self.resolved_path().is_some()
    }

    fn reader(&self) -> io::Result<Box<dyn Read>> {
        let resolved_path = self.resolved_path().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!(
                    "ClassLoader resource \"{}\" could not be resolved",
                    self.path
                ),
            )
        })?;
        transcoding_reader(
            Box::new(BufReader::new(File::open(resolved_path)?)),
            self.character_encoding.as_deref(),
        )
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
        Ok(Box::new(Self::with_search_roots(
            self.search_roots.clone(),
            Some(&full_relative_location),
            self.character_encoding.as_deref(),
        )?))
    }
}
