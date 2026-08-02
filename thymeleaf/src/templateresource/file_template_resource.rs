use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::{Path, PathBuf};

use super::template_resource_reader::{is_java_empty_or_whitespace, transcoding_reader};
use super::template_resource_utils::TemplateResourceUtils;
use super::{ITemplateResource, TemplateResourceError};

/// 文件系统中的模板资源。
///
/// 对应 Java: `org.thymeleaf.templateresource.FileTemplateResource`。
///
/// 对象分别保留 Thymeleaf 清理后的逻辑路径和文件系统路径：base name 与相对资源
/// 基于前者计算，描述、存在性和读取则基于后者。每次调用 `reader` 都重新打开文件，
/// 并将 Java 字符流语义转换为 UTF-8 字节流。显式 UTF 编码保留
/// `InputStreamReader` 的 BOM 行为；空白编码使用 Java 18 及以上规定的 UTF-8 默认值。
pub struct FileTemplateResource {
    path: String,
    file: PathBuf,
    character_encoding: Option<String>,
}

impl FileTemplateResource {
    /// 使用文件路径和可选字符集创建模板资源。
    ///
    /// 对应 Java: `FileTemplateResource#FileTemplateResource(String,String)`。
    ///
    /// # 参数
    /// - `path`：资源路径；`None` 对应 Java `null`。
    /// - `character_encoding`：Java 参数 `characterEncoding`；`None` 或全空白值使用
    ///   Java 18 及以上的系统默认 UTF-8 字符集。
    ///
    /// # 返回
    /// 保留逻辑路径和原始文件定位语义的新资源。
    ///
    /// # 错误
    /// 路径为 `None`、空字符串或仅含 Java 空白字符时返回与 Java 相同消息的参数错误。
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

        Ok(Self::from_validated_path(path, character_encoding))
    }

    /// 使用一个已有文件路径对象和可选字符集创建模板资源。
    ///
    /// 对应 Java: `FileTemplateResource#FileTemplateResource(File,String)`。
    ///
    /// 与字符串构造器不同，Java 只校验 `File` 对象本身非空，因此空路径和仅空白路径
    /// 均保持合法。Rust `Path` 还可能包含无法表示成 Java `String` 的非 UTF-8 字节，
    /// 此扩展边界会作为参数错误报告。
    ///
    /// # 参数
    /// - `file`：已有文件路径；`None` 对应 Java `null`。
    /// - `character_encoding`：Java 参数 `characterEncoding`。
    ///
    /// # 返回
    /// 指向同一词法文件位置的新资源。
    ///
    /// # 错误
    /// 文件路径缺失或不能表示为 UTF-8 时返回参数错误。
    pub fn from_file(
        file: Option<&Path>,
        character_encoding: Option<&str>,
    ) -> Result<Self, TemplateResourceError> {
        let file = file.ok_or_else(|| {
            TemplateResourceError::InvalidArgument("Resource File cannot be null".to_owned())
        })?;
        let Some(file_path) = file.to_str() else {
            return Err(TemplateResourceError::InvalidArgument(
                "Resource File path must be valid UTF-8".to_owned(),
            ));
        };
        let normalized_file_path = normalize_java_file_path(file_path);
        let cleaned_path = TemplateResourceUtils::clean_path(Some(&normalized_file_path))
            .expect("non-null path stays non-null");
        Ok(Self {
            path: cleaned_path,
            file: PathBuf::from(normalized_file_path),
            character_encoding: character_encoding.map(str::to_owned),
        })
    }

    fn from_validated_path(path: &str, character_encoding: Option<&str>) -> Self {
        let cleaned_path =
            TemplateResourceUtils::clean_path(Some(path)).expect("non-null path stays non-null");
        Self {
            path: cleaned_path,
            file: PathBuf::from(normalize_java_file_path(path)),
            character_encoding: character_encoding.map(str::to_owned),
        }
    }
}

impl ITemplateResource for FileTemplateResource {
    fn get_description(&self) -> String {
        let absolute = compute_absolute_file_path(&self.file, std::env::current_dir());
        absolute.to_string_lossy().into_owned()
    }

    fn get_base_name(&self) -> Option<String> {
        TemplateResourceUtils::compute_base_name(Some(&self.path))
    }

    fn exists(&self) -> bool {
        self.file.exists()
    }

    fn reader(&self) -> io::Result<Box<dyn Read>> {
        // Java 先打开 FileInputStream，再解析显式 charset；这一顺序决定组合失败时
        // FileNotFoundException 优先于 UnsupportedEncodingException。
        let input = BufReader::new(File::open(&self.file).map_err(|error| {
            io::Error::new(
                error.kind(),
                format!("{} ({error})", self.get_description()),
            )
        })?);
        transcoding_reader(Box::new(input), self.character_encoding.as_deref())
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
            &full_relative_location,
            self.character_encoding.as_deref(),
        )))
    }
}

fn compute_absolute_file_path(file: &Path, current_directory: io::Result<PathBuf>) -> PathBuf {
    if file.is_absolute() {
        return file.to_path_buf();
    }
    match current_directory {
        Ok(current_directory) if file.as_os_str().is_empty() => current_directory,
        Ok(current_directory) => current_directory.join(file),
        Err(_) => file.to_path_buf(),
    }
}

#[cfg(not(windows))]
fn normalize_java_file_path(path: &str) -> String {
    let mut normalized = String::with_capacity(path.len());
    let mut previous_was_separator = false;
    for character in path.chars() {
        if character == '/' {
            if !previous_was_separator {
                normalized.push(character);
            }
            previous_was_separator = true;
        } else {
            normalized.push(character);
            previous_was_separator = false;
        }
    }
    if normalized.len() > 1 && normalized.ends_with('/') {
        normalized.pop();
    }
    normalized
}

#[cfg(windows)]
fn normalize_java_file_path(path: &str) -> String {
    let mut normalized = String::with_capacity(path.len());
    let mut previous_was_separator = false;
    for character in path.chars() {
        if character == '/' || character == '\\' {
            if !previous_was_separator {
                normalized.push('\\');
            }
            previous_was_separator = true;
        } else {
            normalized.push(character);
            previous_was_separator = false;
        }
    }
    if normalized.len() > 1 && normalized.ends_with('\\') {
        normalized.pop();
    }
    normalized
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::{Cursor, Read};
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{
        FileTemplateResource, ITemplateResource, compute_absolute_file_path,
        normalize_java_file_path,
    };
    use crate::templateresource::java_charset_decoder::JavaCharsetDecoder;
    use crate::templateresource::template_resource_reader::is_java_empty_or_whitespace;
    use crate::templateresource::transcoding_reader::TranscodingReader;

    #[test]
    fn validates_string_and_file_constructor_boundaries() {
        for invalid in [
            None,
            Some(""),
            Some(" \t\n"),
            Some("\u{001C}"),
            Some("\u{1680}"),
            Some("\u{2000}"),
            Some("\u{2008}"),
            Some("\u{2028}"),
            Some("\u{2029}"),
            Some("\u{205F}"),
            Some("\u{3000}"),
        ] {
            let error = FileTemplateResource::new(invalid, None)
                .err()
                .expect("empty Java string path must fail");
            assert_eq!(error.to_string(), "Resource Path cannot be null or empty");
        }
        assert_eq!(
            FileTemplateResource::from_file(None, None)
                .err()
                .expect("null file must fail")
                .to_string(),
            "Resource File cannot be null"
        );

        let empty_file = FileTemplateResource::from_file(Some(Path::new("")), None)
            .expect("empty File is valid");
        assert_eq!(
            empty_file.get_description(),
            std::env::current_dir()
                .expect("current directory")
                .to_string_lossy()
        );
        let whitespace_file = FileTemplateResource::from_file(Some(Path::new("   ")), None)
            .expect("whitespace File is valid");
        assert_eq!(whitespace_file.get_base_name(), Some("   ".to_owned()));
    }

    #[cfg(unix)]
    #[test]
    fn rejects_non_utf8_rust_paths_outside_the_java_file_domain() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        let path = Path::new(OsStr::from_bytes(&[0xFF]));
        assert_eq!(
            FileTemplateResource::from_file(Some(path), None)
                .err()
                .expect("non-UTF-8 path must fail")
                .to_string(),
            "Resource File path must be valid UTF-8"
        );
    }

    #[test]
    fn computes_absolute_paths_when_the_current_directory_is_unavailable() {
        let failure = std::io::Error::other("current directory unavailable");
        assert_eq!(
            compute_absolute_file_path(Path::new("relative.html"), Err(failure)),
            PathBuf::from("relative.html")
        );
        assert_eq!(
            compute_absolute_file_path(
                Path::new("/absolute.html"),
                Err(std::io::Error::other("unused"))
            ),
            PathBuf::from("/absolute.html")
        );
        assert!(!is_java_empty_or_whitespace("\u{00A0}"));
    }

    #[test]
    fn separates_file_description_from_cleaned_base_name() {
        let resource =
            FileTemplateResource::new(Some("something/else/../more.html"), Some("UTF-8"))
                .expect("valid resource");
        assert!(
            resource
                .get_description()
                .ends_with("something/else/../more.html")
        );
        assert_eq!(resource.get_base_name(), Some("more".to_owned()));

        let duplicate =
            FileTemplateResource::new(Some("//something//else"), None).expect("valid resource");
        assert_eq!(duplicate.get_description(), "/something/else");
        assert_eq!(duplicate.get_base_name(), Some("else".to_owned()));
        assert_eq!(normalize_java_file_path("a//b/"), "a/b");
    }

    #[test]
    fn reports_exists_and_opens_a_fresh_reader_each_time() {
        let path = temporary_path("exists");
        fs::write(&path, "first\n第二 😀").expect("write fixture");
        let resource = FileTemplateResource::from_file(Some(&path), Some("UTF-8"))
            .expect("valid file resource");
        assert!(resource.exists());

        let mut first = resource.reader().expect("first reader");
        let mut prefix = [0_u8; 5];
        first.read_exact(&mut prefix).expect("read prefix");
        assert_eq!(&prefix, b"first");

        let mut second = resource.reader().expect("second reader");
        let mut complete = String::new();
        second.read_to_string(&mut complete).expect("read content");
        assert_eq!(complete, "first\n第二 😀");
        fs::remove_file(path).expect("remove fixture");
    }

    #[test]
    fn preserves_file_open_failure_before_charset_failure() {
        let missing = temporary_path("missing");
        let resource = FileTemplateResource::from_file(Some(&missing), Some("not-a-charset"))
            .expect("resource construction is lazy");
        let error = resource
            .reader()
            .err()
            .expect("missing file must fail first");
        assert_eq!(error.kind(), std::io::ErrorKind::NotFound);

        let path = temporary_path("unsupported-charset");
        fs::write(&path, b"content").expect("write fixture");
        let resource = FileTemplateResource::from_file(Some(&path), Some(" UTF-8 "))
            .expect("charset validation is lazy");
        let error = resource.reader().err().expect("invalid charset must fail");
        assert_eq!(error.kind(), std::io::ErrorKind::Unsupported);
        assert_eq!(error.to_string(), " UTF-8 ");

        let resource = FileTemplateResource::from_file(Some(&path), Some("not-a-charset"))
            .expect("charset validation is lazy");
        let error = resource.reader().err().expect("unknown charset must fail");
        assert_eq!(error.kind(), std::io::ErrorKind::Unsupported);
        assert_eq!(error.to_string(), "not-a-charset");
        fs::remove_file(path).expect("remove fixture");
    }

    #[test]
    fn derives_relative_resources_and_validates_java_whitespace() {
        let resource =
            FileTemplateResource::new(Some("something/else/more.html"), Some("ISO-8859-1"))
                .expect("valid resource");
        for invalid in [None, Some(""), Some("\t \u{3000}")] {
            assert_eq!(
                resource
                    .relative(invalid)
                    .err()
                    .expect("invalid relative path")
                    .to_string(),
                "Relative Path cannot be null or empty"
            );
        }

        let relative = resource
            .relative(Some("../more_es.properties"))
            .expect("relative resource");
        assert!(
            relative
                .get_description()
                .ends_with("something/else/../more_es.properties")
        );
        assert_eq!(relative.get_base_name(), Some("more_es".to_owned()));
    }

    #[test]
    fn decodes_java_required_charsets_and_bom_rules() {
        let cases: [(&str, &[u8], &str); 10] = [
            ("UTF-8", b"hello", "hello"),
            ("UTF8", &[0xEF, 0xBB, 0xBF, b'a'], "\u{FEFF}a"),
            ("US-ASCII", &[b'a', 0x80, b'b'], "a\u{FFFD}b"),
            ("ISO-8859-1", &[b'a', 0x80, 0xFF], "a\u{0080}\u{00FF}"),
            ("UTF-16", &[0xFE, 0xFF, 0x00, 0x61], "a"),
            ("Unicode", &[0xFF, 0xFE, 0x61, 0x00], "a"),
            ("UTF-16", &[0x00, 0x61], "a"),
            ("UTF-16BE", &[0xFE, 0xFF, 0x00, 0x61], "\u{FEFF}a"),
            ("UTF-16LE", &[0xFF, 0xFE, 0x61, 0x00], "\u{FEFF}a"),
            ("GBK", &[0xC4, 0xE3, 0xBA, 0xC3], "你好"),
        ];

        for (charset, input, expected) in cases {
            let decoder =
                JavaCharsetDecoder::for_name(Some(charset)).expect("supported Java charset");
            let mut reader = TranscodingReader::new(Box::new(Cursor::new(input)), decoder);
            let mut actual = String::new();
            reader.read_to_string(&mut actual).expect("decoded UTF-8");
            assert_eq!(actual, expected, "charset {charset}");
        }
    }

    #[test]
    fn streams_multibyte_input_through_small_caller_buffers() {
        let input = "甲😀乙".repeat(INPUT_BUFFER_BOUNDARY_REPETITIONS);
        let decoder = JavaCharsetDecoder::for_name(Some("UTF-8")).expect("UTF-8");
        let mut reader =
            TranscodingReader::new(Box::new(Cursor::new(input.as_bytes().to_vec())), decoder);
        let mut output = Vec::new();
        let mut one_byte = [0_u8; 1];
        loop {
            let read = reader.read(&mut one_byte).expect("stream byte");
            if read == 0 {
                break;
            }
            output.extend_from_slice(&one_byte[..read]);
        }
        assert_eq!(String::from_utf8(output).expect("valid UTF-8"), input);
    }

    #[test]
    fn handles_zero_length_reads_and_blank_default_encoding() {
        let decoder = JavaCharsetDecoder::for_name(Some("\t \u{3000}")).expect("default charset");
        let mut reader = TranscodingReader::new(Box::new(Cursor::new("默认".as_bytes())), decoder);
        assert_eq!(reader.read(&mut []).expect("zero length read"), 0);
        let mut content = String::new();
        reader.read_to_string(&mut content).expect("default UTF-8");
        assert_eq!(content, "默认");
    }

    #[test]
    fn propagates_underlying_reader_failures() {
        struct FailingReader;

        impl Read for FailingReader {
            fn read(&mut self, _output: &mut [u8]) -> std::io::Result<usize> {
                Err(std::io::Error::other("synthetic read failure"))
            }
        }

        let decoder = JavaCharsetDecoder::for_name(Some("UTF-8")).expect("UTF-8");
        let mut reader = TranscodingReader::new(Box::new(FailingReader), decoder);
        let mut output = [0_u8; 4];
        let error = reader.read(&mut output).expect_err("read must fail");
        assert_eq!(error.to_string(), "synthetic read failure");
    }

    #[test]
    fn maps_every_windows_1252_byte_with_java_undefined_byte_replacement() {
        let bytes = (0_u8..=u8::MAX).collect::<Vec<_>>();
        let decoder = JavaCharsetDecoder::for_name(Some("cp1252")).expect("Windows-1252");
        let mut reader = TranscodingReader::new(Box::new(Cursor::new(bytes)), decoder);
        let mut content = String::new();
        reader
            .read_to_string(&mut content)
            .expect("decode Windows-1252");

        assert_eq!(content.chars().count(), 256);
        assert_eq!(content.chars().nth(0x7F), Some('\u{007F}'));
        assert_eq!(content.chars().nth(0x80), Some('\u{20AC}'));
        assert_eq!(content.chars().nth(0x81), Some('\u{FFFD}'));
        assert_eq!(content.chars().nth(0x9F), Some('\u{0178}'));
        assert_eq!(content.chars().nth(0xA0), Some('\u{00A0}'));
        assert_eq!(content.chars().nth(0xFF), Some('\u{00FF}'));
    }

    const INPUT_BUFFER_BOUNDARY_REPETITIONS: usize = 2_000;

    fn temporary_path(label: &str) -> std::path::PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "thymeleaf-rust-file-resource-{}-{label}-{nonce}.tmp",
            std::process::id()
        ))
    }
}
