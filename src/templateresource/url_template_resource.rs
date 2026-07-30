use std::fs::File;
use std::io::{self, BufReader, Read};
use std::sync::{Arc, OnceLock};

use ureq::Agent;
use url::Url;

use super::template_resource_reader::{is_java_empty_or_whitespace, transcoding_reader};
use super::template_resource_utils::TemplateResourceUtils;
use super::{ITemplateResource, TemplateResourceError};
use crate::TemplateInputException;

/// Rust 宿主为 JAR、FTP 或自定义 URL 协议提供的连接工厂。
///
/// Java `URL` 把协议处理器保存在 `URLStreamHandler`/`URLConnection` 中；Rust
/// 没有对应的进程级标准注册表，因此把同一能力作为资源实例依赖显式注入。
pub type UrlResourceConnectionHandler =
    dyn Fn(&Url) -> io::Result<Box<dyn Read>> + Send + Sync + 'static;

/// 通过本地或远程 URL 访问的模板资源。
///
/// 对应 Java: `org.thymeleaf.templateresource.UrlTemplateResource`。
///
/// 资源保留 Java `URL#toString()` 可观察到的原始描述，同时使用解析后的 URL 执行
/// 相对地址解析、文件访问以及 HTTP/HTTPS 连接。`reader` 每次创建新的连接或文件句柄，
/// `exists` 对 HTTP/HTTPS 使用 `HEAD`，避免下载正文。字符集解码与文件资源共享同一套
/// Java `InputStreamReader` 兼容规则。
///
/// 上游自 Thymeleaf 3.0.0 提供该对象，通常由 `UrlTemplateResolver` 创建。
pub struct UrlTemplateResource {
    url: Url,
    description: String,
    path: String,
    character_encoding: Option<String>,
    connection_handler: Option<Arc<UrlResourceConnectionHandler>>,
}

impl UrlTemplateResource {
    /// 使用 URL 文本和可选字符集创建模板资源。
    ///
    /// 对应 Java: `UrlTemplateResource#UrlTemplateResource(String,String)`。
    ///
    /// # 参数
    /// - `path`：Java 参数 `path`；`None` 对应 Java `null`。
    /// - `character_encoding`：Java 参数 `characterEncoding`；缺失或全空白时使用
    ///   JDK 21 Oracle 的默认 UTF-8。
    ///
    /// # 返回
    /// 保留原始 URL 描述的新资源。
    ///
    /// # 错误
    /// 路径缺失、为空或仅含 Java 空白时返回参数错误；URL 语法无效时返回
    /// `MalformedUrl`，对应 Java `MalformedURLException`。
    pub fn new(
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
        let url = Url::parse(path).map_err(|source| TemplateResourceError::MalformedUrl {
            location: path.to_owned(),
            source,
        })?;
        Ok(Self {
            path: extract_java_url_path(path),
            url,
            description: path.to_owned(),
            character_encoding: character_encoding.map(str::to_owned),
            connection_handler: None,
        })
    }

    /// 使用已解析 URL 和可选字符集创建模板资源。
    ///
    /// 对应 Java: `UrlTemplateResource#UrlTemplateResource(URL,String)`。
    ///
    /// # 参数
    /// - `url`：已解析 URL；`None` 对应 Java `null`。
    /// - `character_encoding`：Java 参数 `characterEncoding`。
    ///
    /// # 返回
    /// 使用 URL 标准文本作为描述的新资源。
    ///
    /// # 错误
    /// URL 缺失时返回消息与 Java 一致的参数错误。
    pub fn from_url(
        url: Option<Url>,
        character_encoding: Option<&str>,
    ) -> Result<Self, TemplateResourceError> {
        let url = url.ok_or_else(|| {
            TemplateResourceError::InvalidArgument("Resource URL cannot be null".to_owned())
        })?;
        let description = url.to_string();
        let path = url.path().to_owned();
        Ok(Self {
            url,
            description,
            path,
            character_encoding: character_encoding.map(str::to_owned),
            connection_handler: None,
        })
    }

    /// 使用宿主 URL 连接处理器创建模板资源。
    ///
    /// 对应 Java `URL#openConnection()` 通过 `URLStreamHandler` 支持任意已注册协议的
    /// 能力。file/http/https 仍使用内置实现；其他协议由 `connection_handler`
    /// 打开，并在 `reader()` 与 `exists()` 中遵循同一连接合同。
    ///
    /// # 参数
    /// - `path`：完整 URL 文本；
    /// - `character_encoding`：可选 Java 字符集名称；
    /// - `connection_handler`：为非内置协议创建新输入流的线程安全工厂。
    ///
    /// # 错误
    /// URL 参数校验与解析规则和 [`Self::new`] 相同。
    pub fn with_connection_handler(
        path: Option<&str>,
        character_encoding: Option<&str>,
        connection_handler: Arc<UrlResourceConnectionHandler>,
    ) -> Result<Self, TemplateResourceError> {
        let mut resource = Self::new(path, character_encoding)?;
        resource.connection_handler = Some(connection_handler);
        Ok(resource)
    }

    fn from_resolved_url(
        url: Url,
        character_encoding: Option<&str>,
        connection_handler: Option<Arc<UrlResourceConnectionHandler>>,
    ) -> Self {
        let description = url.to_string();
        let path = url.path().to_owned();
        Self {
            url,
            description,
            path,
            character_encoding: character_encoding.map(str::to_owned),
            connection_handler,
        }
    }

    fn input_stream(&self) -> io::Result<Box<dyn Read>> {
        match self.url.scheme() {
            "file" => {
                let path = self.url.to_file_path().map_err(|()| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("URL cannot be converted to a file: {}", self.description),
                    )
                })?;
                let file = File::open(path)?;
                Ok(Box::new(BufReader::new(file)))
            }
            "http" | "https" => {
                let response = reader_agent()
                    .get(self.url.as_str())
                    .call()
                    .map_err(ureq_to_io_error)?;
                let (_, body) = response.into_parts();
                Ok(Box::new(body.into_reader()))
            }
            protocol => self.connection_handler.as_ref().map_or_else(
                || {
                    Err(io::Error::new(
                        io::ErrorKind::Unsupported,
                        format!("No URL resource connection handler for protocol \"{protocol}\""),
                    ))
                },
                |handler| handler(&self.url),
            ),
        }
    }

    fn http_exists(&self) -> bool {
        let Ok(response) = existence_agent().head(self.url.as_str()).call() else {
            return false;
        };
        let status = response.status().as_u16();
        if status == 200 {
            return true;
        }
        if status == 404 {
            return false;
        }
        response
            .headers()
            .get("content-length")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<i64>().ok())
            .is_some_and(|length| length >= 0)
    }
}

impl ITemplateResource for UrlTemplateResource {
    fn get_description(&self) -> String {
        self.description.clone()
    }

    fn get_base_name(&self) -> Option<String> {
        let cleaned = TemplateResourceUtils::clean_path(Some(&self.path));
        TemplateResourceUtils::compute_base_name(cleaned.as_deref())
    }

    fn exists(&self) -> bool {
        match self.url.scheme() {
            "file" => self
                .url
                .to_file_path()
                .is_ok_and(|path| path.as_path().exists()),
            "http" | "https" => self.http_exists(),
            _ => self.input_stream().is_ok(),
        }
    }

    fn reader(&self) -> io::Result<Box<dyn Read>> {
        let input = self.input_stream()?;
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
        let relative_spec = relative_location
            .strip_prefix('/')
            .unwrap_or(relative_location);

        // Java 的 `new URL(context, "")` 保留 context 的外部形式，包括主机 URL
        // 是否显式带有尾部 `/`，而 WHATWG URL 会统一补 `/`，因此空 spec 直接复制。
        if relative_spec.is_empty() {
            return Ok(Box::new(Self {
                url: self.url.clone(),
                description: self.description.clone(),
                path: self.path.clone(),
                character_encoding: self.character_encoding.clone(),
                connection_handler: self.connection_handler.clone(),
            }));
        }

        let relative_url = self.url.join(relative_spec).map_err(|source| {
            let message = format!(
                "Could not create relative URL resource for resource \"{}\" and relative location \"{}\"",
                self.get_description(),
                relative_location
            );
            TemplateResourceError::Input(TemplateInputException::with_cause(
                Some(message),
                source,
            ))
        })?;
        Ok(Box::new(Self::from_resolved_url(
            relative_url,
            self.character_encoding.as_deref(),
            self.connection_handler.clone(),
        )))
    }
}

fn reader_agent() -> &'static Agent {
    static AGENT: OnceLock<Agent> = OnceLock::new();
    AGENT.get_or_init(|| Agent::config_builder().proxy(None).build().into())
}

fn existence_agent() -> &'static Agent {
    static AGENT: OnceLock<Agent> = OnceLock::new();
    AGENT.get_or_init(|| {
        Agent::config_builder()
            .http_status_as_error(false)
            .proxy(None)
            .build()
            .into()
    })
}

fn ureq_to_io_error(error: ureq::Error) -> io::Error {
    let kind = match error {
        ureq::Error::StatusCode(404) => io::ErrorKind::NotFound,
        ureq::Error::Timeout(_) => io::ErrorKind::TimedOut,
        ureq::Error::Io(ref error) => error.kind(),
        _ => io::ErrorKind::Other,
    };
    io::Error::new(kind, error)
}

fn extract_java_url_path(location: &str) -> String {
    let Some(scheme_end) = location.find(':') else {
        return String::new();
    };
    let after_scheme = &location[scheme_end + 1..];
    let path = if let Some(authority) = after_scheme.strip_prefix("//") {
        authority
            .find('/')
            .map_or("", |path_start| &authority[path_start..])
    } else {
        after_scheme
    };
    let end = path
        .char_indices()
        .find_map(|(index, character)| matches!(character, '?' | '#').then_some(index))
        .unwrap_or(path.len());
    path[..end].to_owned()
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs;
    use std::io::{self, BufRead, BufReader, Write};
    use std::net::TcpListener;
    use std::thread;

    use super::{ITemplateResource, UrlTemplateResource, extract_java_url_path, ureq_to_io_error};
    use url::Url;

    #[test]
    fn rejects_unconvertible_file_urls_and_protocols_without_a_rust_handler() {
        let remote_file = UrlTemplateResource::new(Some("file://example.com/template.html"), None)
            .expect("syntactically valid file URL");
        #[cfg(not(windows))]
        assert_eq!(
            remote_file
                .reader()
                .err()
                .expect("remote file authority")
                .kind(),
            io::ErrorKind::InvalidInput
        );

        let ftp = UrlTemplateResource::new(Some("ftp://example.com/template.html"), None)
            .expect("syntactically valid FTP URL");
        assert_eq!(
            ftp.reader().err().expect("missing FTP handler").kind(),
            io::ErrorKind::Unsupported
        );
        assert!(!ftp.exists());
    }

    #[test]
    fn preserves_url_parse_sources_and_maps_transport_error_kinds() {
        let malformed = UrlTemplateResource::new(Some("relative"), None)
            .err()
            .expect("relative URL without a base");
        assert!(malformed.source().is_some());

        assert_eq!(
            ureq_to_io_error(ureq::Error::Timeout(ureq::Timeout::Global)).kind(),
            io::ErrorKind::TimedOut
        );
        assert_eq!(
            ureq_to_io_error(ureq::Error::Io(io::Error::new(
                io::ErrorKind::ConnectionReset,
                "reset",
            )))
            .kind(),
            io::ErrorKind::ConnectionReset
        );
        assert_eq!(
            ureq_to_io_error(ureq::Error::ConnectionFailed).kind(),
            io::ErrorKind::Other
        );
    }

    #[test]
    fn extracts_java_url_paths_without_whatwg_normalization() {
        assert_eq!(extract_java_url_path("relative"), "");
        assert_eq!(
            extract_java_url_path("file:/tmp/a/../b.html?query#fragment"),
            "/tmp/a/../b.html"
        );
        assert_eq!(
            extract_java_url_path("http://example.com/a/./b.html?query"),
            "/a/./b.html"
        );
        assert_eq!(extract_java_url_path("http://example.com?query"), "");
    }

    #[test]
    fn exercises_file_http_validation_and_relative_public_contracts() {
        assert!(UrlTemplateResource::new(None, None).is_err());
        assert!(UrlTemplateResource::from_url(None, None).is_err());

        let directory =
            std::env::temp_dir().join(format!("thymeleaf-url-unit-{}", std::process::id()));
        fs::create_dir_all(&directory).expect("create URL unit directory");
        let file = directory.join("main.html");
        fs::write(&file, b"file").expect("write URL unit file");
        let file_url = Url::from_file_path(&file).expect("file URL");
        let file_resource =
            UrlTemplateResource::from_url(Some(file_url), Some("UTF-8")).expect("URL resource");
        assert!(file_resource.exists());
        assert!(file_resource.reader().is_ok());
        assert_eq!(file_resource.get_base_name().as_deref(), Some("main"));
        assert!(file_resource.relative(None).is_err());
        assert!(file_resource.relative(Some("http://[")).is_err());
        assert_eq!(
            file_resource
                .relative(Some("child.html"))
                .expect("relative URL")
                .get_base_name()
                .as_deref(),
            Some("child")
        );
        let missing_file_url =
            Url::from_file_path(directory.join("missing.html")).expect("missing file URL");
        let missing_file_resource =
            UrlTemplateResource::from_url(Some(missing_file_url), None).expect("file URL resource");
        assert_eq!(
            missing_file_resource
                .reader()
                .err()
                .expect("missing file URL must fail")
                .kind(),
            io::ErrorKind::NotFound
        );

        let listener = TcpListener::bind("127.0.0.1:0").expect("bind URL unit HTTP server");
        let port = listener.local_addr().expect("HTTP address").port();
        let handle = thread::spawn(move || {
            for _ in 0..5 {
                let (mut stream, _) = listener.accept().expect("accept HTTP request");
                let mut input = BufReader::new(stream.try_clone().expect("clone stream"));
                let mut request_line = String::new();
                input.read_line(&mut request_line).expect("request line");
                let mut line = String::new();
                loop {
                    line.clear();
                    input.read_line(&mut line).expect("request header");
                    if line == "\r\n" || line.is_empty() {
                        break;
                    }
                }
                let status = if request_line.contains(" /not-found ") {
                    "404 Not Found"
                } else if request_line.starts_with("HEAD /other ") {
                    "500 Server Error"
                } else {
                    "200 OK"
                };
                write!(
                    stream,
                    "HTTP/1.1 {status}\r\nContent-Length: 2\r\nConnection: close\r\n\r\n"
                )
                .expect("HTTP response headers");
                if request_line.starts_with("GET ") {
                    stream.write_all(b"ok").expect("HTTP response body");
                }
                stream.flush().expect("flush HTTP response");
            }
        });
        let http_url = format!("http://127.0.0.1:{port}/ok");
        let http_resource =
            UrlTemplateResource::new(Some(&http_url), Some("UTF-8")).expect("HTTP resource");
        assert!(http_resource.exists());
        let not_found_url = format!("http://127.0.0.1:{port}/not-found");
        assert!(
            !UrlTemplateResource::new(Some(&not_found_url), None)
                .expect("404 HTTP resource")
                .exists()
        );
        assert_eq!(
            UrlTemplateResource::new(Some(&not_found_url), None)
                .expect("404 HTTP reader resource")
                .reader()
                .err()
                .expect("404 reader must fail")
                .kind(),
            io::ErrorKind::NotFound
        );
        let other_url = format!("http://127.0.0.1:{port}/other");
        assert!(
            UrlTemplateResource::new(Some(&other_url), None)
                .expect("other HTTP resource")
                .exists()
        );
        assert!(http_resource.reader().is_ok());
        handle.join().expect("HTTP unit server");

        // HTTPS 与 HTTP 共用 Java URLConnection 分派合同；使用已关闭的本地端口验证
        // TLS 建连失败时 reader/exists 分别传播与吞掉 I/O 异常。
        let unavailable = TcpListener::bind("127.0.0.1:0").expect("reserve HTTPS unit port");
        let unavailable_port = unavailable.local_addr().expect("HTTPS unit address").port();
        drop(unavailable);
        let https_url = format!("https://127.0.0.1:{unavailable_port}/unavailable");
        let https_resource =
            UrlTemplateResource::new(Some(&https_url), None).expect("HTTPS resource");
        assert!(!https_resource.exists());
        assert!(https_resource.reader().is_err());

        fs::remove_dir_all(directory).expect("remove URL unit directory");
    }
}
