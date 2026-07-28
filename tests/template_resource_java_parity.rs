//! 模板资源接口、字符串资源和文件资源的 Thymeleaf 3.1.5 Java/Rust Golden 差分测试。

use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{SystemTime, UNIX_EPOCH};

use thymeleaf::{
    FileTemplateResource, ITemplateResource, StringTemplateResource, TemplateResourceError,
    UrlTemplateResource,
};
use url::Url;

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const RUST_BASELINE: &str = "eca81ffdc14b721e60cbfc812cb701ffb8fae7ba";
const JAVA_GOLDEN: &str = include_str!("fixtures/template_resource_golden.txt");

#[test]
fn template_resource_objects_match_java_golden() {
    let mut output = String::new();
    emit(&mut output, "java_baseline", Some(JAVA_BASELINE));
    emit(&mut output, "rust_baseline", Some(RUST_BASELINE));
    export_null_constructor(&mut output);
    export_empty_resource(&mut output);
    export_unicode_resource(&mut output);
    export_relative_failures(&mut output);
    export_file_validation(&mut output);
    export_file_paths(&mut output);
    export_file_readers(&mut output);
    export_url_validation(&mut output);
    export_url_paths(&mut output);
    export_url_files(&mut output);
    export_url_http(&mut output);
    assert_eq!(output, JAVA_GOLDEN);
}

fn export_null_constructor(output: &mut String) {
    let error = StringTemplateResource::new(None)
        .err()
        .expect("null resource must fail");
    emit(
        output,
        "string.null.type",
        Some("java.lang.IllegalArgumentException"),
    );
    emit(output, "string.null.message", Some(&error.to_string()));
}

fn export_empty_resource(output: &mut String) {
    let resource = StringTemplateResource::new(Some("")).expect("empty resource is legal");
    let resource: &dyn ITemplateResource = &resource;
    emit(
        output,
        "string.empty.description",
        Some(&resource.get_description()),
    );
    emit(
        output,
        "string.empty.base_name",
        resource.get_base_name().as_deref(),
    );
    emit_bool(output, "string.empty.exists", resource.exists());
    emit(
        output,
        "string.empty.reader",
        Some(&read_all(resource.reader().expect("empty reader"))),
    );
    let first = resource.reader().expect("first reader");
    let second = resource.reader().expect("second reader");
    emit_bool(
        output,
        "string.empty.fresh_readers",
        !std::ptr::eq::<dyn Read>(&*first, &*second),
    );
}

fn export_unicode_resource(output: &mut String) {
    let contents = "<p>你好 😀</p>\r\n\0tail";
    let resource = StringTemplateResource::new(Some(contents)).expect("valid resource");
    let resource: &dyn ITemplateResource = &resource;
    emit(
        output,
        "string.unicode.description",
        Some(&resource.get_description()),
    );
    emit(
        output,
        "string.unicode.base_name",
        resource.get_base_name().as_deref(),
    );
    emit_bool(output, "string.unicode.exists", resource.exists());

    let mut first = resource.reader().expect("first reader");
    let mut prefix = [0_u8; 3];
    let prefix_count = first.read(&mut prefix).expect("read prefix");
    emit(
        output,
        "string.unicode.prefix_count",
        Some(&prefix_count.to_string()),
    );
    emit(
        output,
        "string.unicode.prefix",
        Some(std::str::from_utf8(&prefix).expect("utf-8 prefix")),
    );
    emit(
        output,
        "string.unicode.second_full",
        Some(&read_all(resource.reader().expect("second reader"))),
    );
    emit(
        output,
        "string.unicode.first_remaining",
        Some(&read_all(first)),
    );
}

fn export_relative_failures(output: &mut String) {
    let resource = StringTemplateResource::new(Some("line1\n\"line2\"")).expect("valid resource");
    let resource: &dyn ITemplateResource = &resource;
    for (name, relative_location) in [
        ("null", None),
        ("empty", Some("")),
        ("child", Some("child.html")),
    ] {
        let error = resource
            .relative(relative_location)
            .err()
            .expect("relative string resource must fail");
        let error_type = match &error {
            TemplateResourceError::Input(_) => "org.thymeleaf.exceptions.TemplateInputException",
            TemplateResourceError::InvalidArgument(_) => "java.lang.IllegalArgumentException",
            TemplateResourceError::MalformedUrl { .. } => "java.net.MalformedURLException",
        };
        emit(
            output,
            &format!("string.relative.{name}.type"),
            Some(error_type),
        );
        emit(
            output,
            &format!("string.relative.{name}.message"),
            Some(&error.to_string()),
        );
    }
}

fn export_file_validation(output: &mut String) {
    for (name, path) in [
        ("null", None),
        ("empty", Some("")),
        ("whitespace", Some("\t \u{3000}")),
    ] {
        let error = FileTemplateResource::new(path, None)
            .err()
            .expect("invalid string path");
        emit(
            output,
            &format!("file.path.{name}.type"),
            Some("java.lang.IllegalArgumentException"),
        );
        emit(
            output,
            &format!("file.path.{name}.message"),
            Some(&error.to_string()),
        );
    }

    let error = FileTemplateResource::from_file(None, None)
        .err()
        .expect("null file must fail");
    emit(
        output,
        "file.null_file.type",
        Some("java.lang.IllegalArgumentException"),
    );
    emit(output, "file.null_file.message", Some(&error.to_string()));

    #[cfg(unix)]
    {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        let non_utf8 = Path::new(OsStr::from_bytes(&[0xFF]));
        assert!(
            FileTemplateResource::from_file(Some(non_utf8), None).is_err(),
            "a Java File cannot represent a non-UTF-8 Rust path"
        );
    }

    let empty_file =
        FileTemplateResource::from_file(Some(Path::new("")), None).expect("empty File is valid");
    emit_bool(
        output,
        "file.empty_file.description_is_user_dir",
        empty_file.get_description()
            == std::env::current_dir()
                .expect("current directory")
                .to_string_lossy(),
    );
    emit(
        output,
        "file.empty_file.base_name",
        empty_file.get_base_name().as_deref(),
    );
    emit_bool(output, "file.empty_file.exists", empty_file.exists());
}

fn export_file_paths(output: &mut String) {
    let resource =
        FileTemplateResource::new(Some("something/else/../more.html"), Some("ISO-8859-1"))
            .expect("valid file resource");
    emit_bool(
        output,
        "file.path.description_suffix",
        slash(&resource.get_description()).ends_with("/something/else/../more.html"),
    );
    emit(
        output,
        "file.path.base_name",
        resource.get_base_name().as_deref(),
    );
    emit_bool(output, "file.path.exists", resource.exists());

    let duplicate =
        FileTemplateResource::new(Some("//something//else"), None).expect("valid file resource");
    emit(
        output,
        "file.path.duplicate.description",
        Some(&slash(&duplicate.get_description())),
    );
    emit(
        output,
        "file.path.duplicate.base_name",
        duplicate.get_base_name().as_deref(),
    );

    for (name, relative_location) in [
        ("null", None),
        ("empty", Some("")),
        ("whitespace", Some("\t \u{3000}")),
    ] {
        let error = resource
            .relative(relative_location)
            .err()
            .expect("invalid relative path");
        emit(
            output,
            &format!("file.relative.{name}.type"),
            Some("java.lang.IllegalArgumentException"),
        );
        emit(
            output,
            &format!("file.relative.{name}.message"),
            Some(&error.to_string()),
        );
    }

    let relative = resource
        .relative(Some("../more_es.properties"))
        .expect("valid relative resource");
    emit_bool(
        output,
        "file.relative.valid.description_suffix",
        slash(&relative.get_description()).ends_with("/something/../more_es.properties"),
    );
    emit(
        output,
        "file.relative.valid.base_name",
        relative.get_base_name().as_deref(),
    );
    emit_bool(output, "file.relative.valid.exists", relative.exists());
}

fn export_file_readers(output: &mut String) {
    let directory = temporary_directory();
    fs::create_dir(&directory).expect("create temporary directory");

    export_decode(output, &directory, "default", None, "默认😀".as_bytes());
    export_decode(
        output,
        &directory,
        "blank",
        Some("\t \u{3000}"),
        "默认😀".as_bytes(),
    );
    export_decode(
        output,
        &directory,
        "utf8_bom",
        Some("UTF8"),
        &[0xEF, 0xBB, 0xBF, b'a'],
    );
    export_decode(
        output,
        &directory,
        "utf8_malformed",
        Some("UTF-8"),
        &[b'a', 0xC0, 0xAF, 0xE2, 0x82, b'b'],
    );
    export_decode(
        output,
        &directory,
        "ascii",
        Some("ASCII"),
        &[b'a', 0x80, 0x81, b'b'],
    );
    export_decode(
        output,
        &directory,
        "latin1",
        Some("ISO8859_1"),
        &[b'a', 0x80, 0xFF],
    );
    export_decode(
        output,
        &directory,
        "utf16_bom_be",
        Some("UTF-16"),
        &[0xFE, 0xFF, 0x00, b'a'],
    );
    export_decode(
        output,
        &directory,
        "utf16_bom_le",
        Some("Unicode"),
        &[0xFF, 0xFE, b'a', 0x00],
    );
    export_decode(
        output,
        &directory,
        "utf16_no_bom",
        Some("UTF-16"),
        &[0x00, b'a'],
    );
    export_decode(
        output,
        &directory,
        "utf16be_explicit_bom",
        Some("UnicodeBigUnmarked"),
        &[0xFE, 0xFF, 0x00, b'a'],
    );
    export_decode(
        output,
        &directory,
        "utf16le_explicit_bom",
        Some("UnicodeLittleUnmarked"),
        &[0xFF, 0xFE, b'a', 0x00],
    );
    export_decode(
        output,
        &directory,
        "windows1252",
        Some("windows-1252"),
        &[b'a', 0x80, 0x81],
    );
    export_decode(
        output,
        &directory,
        "gbk",
        Some("GBK"),
        &[0xC4, 0xE3, 0xBA, 0xC3],
    );

    let fresh_path = directory.join("fresh.txt");
    fs::write(&fresh_path, b"fresh-reader").expect("write fresh reader fixture");
    let fresh = FileTemplateResource::from_file(Some(&fresh_path), Some("UTF-8"))
        .expect("valid file resource");
    emit_bool(output, "file.reader.exists", fresh.exists());
    let first_identity = fresh.reader().expect("first identity reader");
    let second_identity = fresh.reader().expect("second identity reader");
    emit_bool(
        output,
        "file.reader.fresh",
        !std::ptr::eq::<dyn Read>(&*first_identity, &*second_identity),
    );
    emit(
        output,
        "file.reader.first",
        Some(&read_all(fresh.reader().expect("first reader"))),
    );
    emit(
        output,
        "file.reader.second",
        Some(&read_all(fresh.reader().expect("second reader"))),
    );

    let unsupported_path = directory.join("unsupported.txt");
    fs::write(&unsupported_path, b"a").expect("write unsupported charset fixture");
    let unsupported = FileTemplateResource::from_file(Some(&unsupported_path), Some(" UTF-8 "))
        .expect("resource construction is lazy");
    let unsupported_error = unsupported
        .reader()
        .err()
        .expect("unsupported charset must fail");
    emit(
        output,
        "file.reader.unsupported.type",
        Some("java.io.UnsupportedEncodingException"),
    );
    emit(
        output,
        "file.reader.unsupported.message",
        Some(&code_points(&unsupported_error.to_string())),
    );

    let unknown = FileTemplateResource::from_file(Some(&unsupported_path), Some("not-a-charset"))
        .expect("resource construction is lazy");
    let unknown_error = unknown.reader().err().expect("unknown charset must fail");
    emit(
        output,
        "file.reader.unknown.type",
        Some("java.io.UnsupportedEncodingException"),
    );
    emit(
        output,
        "file.reader.unknown.message",
        Some(&code_points(&unknown_error.to_string())),
    );

    let missing_path = directory.join("missing.txt");
    let missing = FileTemplateResource::from_file(Some(&missing_path), Some("not-a-charset"))
        .expect("resource construction is lazy");
    let missing_error = missing
        .reader()
        .err()
        .expect("missing file must fail before charset");
    emit(
        output,
        "file.reader.missing_precedes_charset.type",
        Some("java.io.FileNotFoundException"),
    );
    emit_bool(
        output,
        "file.reader.missing_precedes_charset.message_mentions_file",
        slash(&missing_error.to_string()).contains(&slash(&missing_path.to_string_lossy())),
    );

    fs::remove_dir_all(directory).expect("remove temporary directory");
}

fn export_url_validation(output: &mut String) {
    for (name, path) in [
        ("null", None),
        ("empty", Some("")),
        ("whitespace", Some("\t \u{3000}")),
    ] {
        let error = UrlTemplateResource::new(path, None)
            .err()
            .expect("invalid URL path");
        emit(
            output,
            &format!("url.path.{name}.type"),
            Some("java.lang.IllegalArgumentException"),
        );
        emit(
            output,
            &format!("url.path.{name}.message"),
            Some(&error.to_string()),
        );
    }

    let null_url = UrlTemplateResource::from_url(None, None)
        .err()
        .expect("null URL must fail");
    emit(
        output,
        "url.null_url.type",
        Some("java.lang.IllegalArgumentException"),
    );
    emit(output, "url.null_url.message", Some(&null_url.to_string()));

    let malformed = UrlTemplateResource::new(Some("not-a-url"), None)
        .err()
        .expect("relative URL without a base must fail");
    emit(
        output,
        "url.malformed.type",
        Some("java.net.MalformedURLException"),
    );
    emit_bool(
        output,
        "url.malformed.message_nonempty",
        !malformed.to_string().is_empty(),
    );
}

fn export_url_paths(output: &mut String) {
    let descriptions = [
        "http://www.thymeleaf.org/",
        "http://www.thymeleaf.org",
        "http://www.thymeleaf.org/something",
        "http://www.thymeleaf.org/something/",
        "http://www.thymeleaf.org/something/else",
        "http://www.thymeleaf.org/something/else.html",
        "http://www.thymeleaf.org/something/./else.html",
        "http://www.thymeleaf.org/something/more/../else.html",
        "http://www.thymeleaf.org/something/./more/../else.html",
    ];
    for (index, description) in descriptions.iter().enumerate() {
        let resource =
            UrlTemplateResource::new(Some(description), None).expect("valid URL resource");
        emit(
            output,
            &format!("url.description.{index}"),
            Some(&resource.get_description()),
        );
    }

    let relatives = [
        ("http://www.thymeleaf.org/", "/"),
        ("http://www.thymeleaf.org", "/"),
        ("http://www.thymeleaf.org", "/something"),
        ("http://www.thymeleaf.org", "something"),
        ("http://www.thymeleaf.org/more", "something"),
        ("http://www.thymeleaf.org/more/", "something"),
        ("http://www.thymeleaf.org/something/else", "more"),
        ("http://www.thymeleaf.org/something/else.html", "more.html"),
        (
            "http://www.thymeleaf.org/something/else.html",
            "../more.html",
        ),
        (
            "http://www.thymeleaf.org/something/more/../else.html",
            "../less.html",
        ),
        (
            "http://www.thymeleaf.org/something/more/../else.html",
            "../even/less.html",
        ),
        (
            "http://www.thymeleaf.org/something/./more/../else.html",
            "../even/./less.html",
        ),
    ];
    for (index, (base, relative_location)) in relatives.iter().enumerate() {
        let resource =
            UrlTemplateResource::new(Some(base), Some("ISO-8859-1")).expect("valid URL resource");
        let relative = resource
            .relative(Some(relative_location))
            .expect("valid relative URL");
        emit(
            output,
            &format!("url.relative.{index}"),
            Some(&relative.get_description()),
        );
    }

    let base_names = [
        "http://www.thymeleaf.org/",
        "http://www.thymeleaf.org",
        "http://www.thymeleaf.org/more",
        "http://www.thymeleaf.org/more/",
        "http://www.thymeleaf.org/something/else",
        "http://www.thymeleaf.org/something/else.html",
        "http://www.thymeleaf.org/something/more/../else.html",
        "http://www.thymeleaf.org/something/more/../else.html/",
        "http://www.thymeleaf.org/something/more/../else.html/a/..",
        "http://www.thymeleaf.org/something/./more/../else.html",
        "http://www.thymeleaf.org/something/./more/../else.html?param=a",
    ];
    for (index, base) in base_names.iter().enumerate() {
        let resource = UrlTemplateResource::new(Some(base), None).expect("valid URL resource");
        emit(
            output,
            &format!("url.base_name.{index}"),
            resource.get_base_name().as_deref(),
        );
    }

    let failure =
        UrlTemplateResource::new(Some("http://www.thymeleaf.org/base.html"), None).expect("URL");
    for (name, relative_location) in [
        ("null", None),
        ("empty", Some("")),
        ("whitespace", Some("\t \u{3000}")),
    ] {
        let error = failure
            .relative(relative_location)
            .err()
            .expect("invalid relative URL");
        emit(
            output,
            &format!("url.relative_failure.{name}.type"),
            Some("java.lang.IllegalArgumentException"),
        );
        emit(
            output,
            &format!("url.relative_failure.{name}.message"),
            Some(&error.to_string()),
        );
    }

    let malformed = failure
        .relative(Some("http://["))
        .err()
        .expect("malformed relative URL");
    emit(
        output,
        "url.relative_failure.malformed.type",
        Some("org.thymeleaf.exceptions.TemplateInputException"),
    );
    emit(
        output,
        "url.relative_failure.malformed.message",
        Some(&malformed.to_string()),
    );
    emit(
        output,
        "url.relative_failure.malformed.cause_type",
        Some("java.net.MalformedURLException"),
    );
}

fn export_url_files(output: &mut String) {
    let directory = temporary_directory().with_file_name(format!(
        "thymeleaf-url-golden-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos()
    ));
    let parent = directory.join("space dir");
    fs::create_dir_all(&parent).expect("create URL fixture directory");
    let primary = parent.join("main.html");
    let sibling = parent.join("child.html");
    fs::write(&primary, [b'a', 0xE9]).expect("write primary URL fixture");
    fs::write(&sibling, [b'b', 0xE9]).expect("write sibling URL fixture");

    let url = Url::from_file_path(&primary).expect("absolute file URL");
    let resource =
        UrlTemplateResource::from_url(Some(url), Some("ISO-8859-1")).expect("valid URL resource");
    emit_bool(
        output,
        "url.file.description_scheme",
        resource.get_description().starts_with("file:"),
    );
    emit_bool(
        output,
        "url.file.description_has_escaped_space",
        resource.get_description().contains("space%20dir"),
    );
    emit(
        output,
        "url.file.base_name",
        resource.get_base_name().as_deref(),
    );
    emit_bool(output, "url.file.exists", resource.exists());
    emit(
        output,
        "url.file.reader",
        Some(&code_points(&read_all(
            resource.reader().expect("file URL reader"),
        ))),
    );
    let first = resource.reader().expect("first file URL reader");
    let second = resource.reader().expect("second file URL reader");
    emit_bool(
        output,
        "url.file.reader_fresh",
        !std::ptr::eq::<dyn Read>(&*first, &*second),
    );

    let relative = resource
        .relative(Some("child.html"))
        .expect("relative file URL");
    emit(
        output,
        "url.file.relative.base_name",
        relative.get_base_name().as_deref(),
    );
    emit_bool(output, "url.file.relative.exists", relative.exists());
    emit(
        output,
        "url.file.relative.reader",
        Some(&code_points(&read_all(
            relative.reader().expect("relative file URL reader"),
        ))),
    );

    let missing = resource
        .relative(Some("missing.html"))
        .expect("missing file URL object");
    emit_bool(output, "url.file.missing.exists", missing.exists());
    let error = missing
        .reader()
        .err()
        .expect("missing URL reader must fail");
    emit(
        output,
        "url.file.missing.reader.type",
        Some(if error.kind() == std::io::ErrorKind::NotFound {
            "java.io.FileNotFoundException"
        } else {
            "java.io.IOException"
        }),
    );

    #[cfg(not(windows))]
    {
        let remote_file = UrlTemplateResource::new(Some("file://example.com/template.html"), None)
            .expect("syntactically valid remote file URL");
        assert_eq!(
            remote_file
                .reader()
                .err()
                .expect("remote file authority is not local")
                .kind(),
            std::io::ErrorKind::InvalidInput
        );
    }

    fs::remove_dir_all(directory).expect("remove URL fixture directory");
}

fn export_url_http(output: &mut String) {
    let mut server = LocalHttpServer::new(10);
    emit_bool(
        output,
        "url.http.exists.ok",
        UrlTemplateResource::new(Some(&server.url("/ok")), None)
            .expect("HTTP URL")
            .exists(),
    );
    emit_bool(
        output,
        "url.http.exists.not_found",
        UrlTemplateResource::new(Some(&server.url("/not-found")), None)
            .expect("HTTP URL")
            .exists(),
    );
    emit_bool(
        output,
        "url.http.exists.other_with_length",
        UrlTemplateResource::new(Some(&server.url("/other-with-length")), None)
            .expect("HTTP URL")
            .exists(),
    );
    emit_bool(
        output,
        "url.http.exists.other_without_length",
        UrlTemplateResource::new(Some(&server.url("/other-without-length")), None)
            .expect("HTTP URL")
            .exists(),
    );

    let ok = UrlTemplateResource::new(Some(&server.url("/ok")), Some("UTF-8")).expect("HTTP URL");
    emit(
        output,
        "url.http.reader.ok",
        Some(&read_all(ok.reader().expect("HTTP reader"))),
    );
    let latin1 = UrlTemplateResource::new(Some(&server.url("/latin1")), Some("ISO-8859-1"))
        .expect("HTTP URL");
    emit(
        output,
        "url.http.reader.latin1",
        Some(&code_points(&read_all(
            latin1.reader().expect("Latin-1 HTTP reader"),
        ))),
    );

    let unsupported = UrlTemplateResource::new(Some(&server.url("/ok")), Some("not-a-charset"))
        .expect("HTTP URL")
        .reader()
        .err()
        .expect("unsupported charset");
    emit(
        output,
        "url.http.reader.unsupported.type",
        Some(if unsupported.kind() == std::io::ErrorKind::Unsupported {
            "java.io.UnsupportedEncodingException"
        } else {
            "java.io.IOException"
        }),
    );

    let not_found = UrlTemplateResource::new(Some(&server.url("/not-found")), Some("UTF-8"))
        .expect("HTTP URL")
        .reader()
        .err()
        .expect("HTTP 404");
    emit(
        output,
        "url.http.reader.not_found.type",
        Some(if not_found.kind() == std::io::ErrorKind::NotFound {
            "java.io.FileNotFoundException"
        } else {
            "java.io.IOException"
        }),
    );

    let fresh =
        UrlTemplateResource::new(Some(&server.url("/ok")), Some("UTF-8")).expect("HTTP URL");
    let first = fresh.reader().expect("first HTTP reader");
    let second = fresh.reader().expect("second HTTP reader");
    emit_bool(
        output,
        "url.http.reader.fresh",
        !std::ptr::eq::<dyn Read>(&*first, &*second),
    );

    server.await_requests();
    emit(
        output,
        "url.http.server.head_count",
        Some(&server.count("HEAD").to_string()),
    );
    emit(
        output,
        "url.http.server.get_count",
        Some(&server.count("GET").to_string()),
    );

    let unavailable = TcpListener::bind("127.0.0.1:0").expect("reserve unavailable port");
    let unavailable_port = unavailable.local_addr().expect("listener address").port();
    drop(unavailable);
    let unavailable_url = format!("http://127.0.0.1:{unavailable_port}/unavailable");
    emit_bool(
        output,
        "url.http.exists.connection_refused",
        UrlTemplateResource::new(Some(&unavailable_url), None)
            .expect("HTTP URL")
            .exists(),
    );

    // 穿过与 Java URLConnection 对应的 HTTPS 分派和传输失败路径；这些断言不产生
    // Golden 记录，避免把平台 TLS 错误文本写入跨语言基线。
    let unavailable_https_url = format!("https://127.0.0.1:{unavailable_port}/unavailable");
    let unavailable_https =
        UrlTemplateResource::new(Some(&unavailable_https_url), None).expect("HTTPS URL");
    assert!(!unavailable_https.exists());
    assert!(unavailable_https.reader().is_err());

    // Java 会把未知协议交给 URLStreamHandler；当前内建传输层没有 FTP handler，
    // 因而必须稳定返回 Unsupported，并让 exists() 吞掉该 I/O 失败。
    let unsupported_protocol =
        UrlTemplateResource::new(Some("ftp://example.com/template.html"), None).expect("FTP URL");
    assert!(!unsupported_protocol.exists());
    assert_eq!(
        unsupported_protocol
            .reader()
            .err()
            .expect("FTP handler is unavailable")
            .kind(),
        std::io::ErrorKind::Unsupported
    );
}

struct LocalHttpServer {
    port: u16,
    methods: Arc<Mutex<Vec<String>>>,
    handle: Option<JoinHandle<()>>,
}

impl LocalHttpServer {
    fn new(expected_requests: usize) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind local HTTP server");
        let port = listener.local_addr().expect("HTTP server address").port();
        let methods = Arc::new(Mutex::new(Vec::new()));
        let observed_methods = Arc::clone(&methods);
        let handle = thread::spawn(move || {
            for _ in 0..expected_requests {
                let (mut stream, _) = listener.accept().expect("accept HTTP request");
                handle_http_request(&mut stream, &observed_methods);
            }
        });
        Self {
            port,
            methods,
            handle: Some(handle),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("http://127.0.0.1:{}{path}", self.port)
    }

    fn count(&self, method: &str) -> usize {
        self.methods
            .lock()
            .expect("HTTP method log")
            .iter()
            .filter(|observed| observed.as_str() == method)
            .count()
    }

    fn await_requests(&mut self) {
        self.handle
            .take()
            .expect("HTTP server handle")
            .join()
            .expect("HTTP server thread");
    }
}

fn handle_http_request(stream: &mut TcpStream, methods: &Arc<Mutex<Vec<String>>>) {
    let mut input = BufReader::new(stream.try_clone().expect("clone HTTP stream"));
    let mut request_line = String::new();
    input
        .read_line(&mut request_line)
        .expect("read HTTP request line");
    let mut line = String::new();
    loop {
        line.clear();
        input.read_line(&mut line).expect("read HTTP header");
        if line == "\r\n" || line.is_empty() {
            break;
        }
    }

    let mut request = request_line.split_whitespace();
    let method = request.next().expect("HTTP method");
    let path = request.next().expect("HTTP path");
    methods
        .lock()
        .expect("HTTP method log")
        .push(method.to_owned());

    let (status, reason, body, include_length): (u16, &str, &[u8], bool) = match path {
        "/ok" => (200, "OK", b"ok", true),
        "/latin1" => (200, "OK", &[b'a', 0xE9], true),
        "/not-found" => (404, "Not Found", b"", true),
        "/other-with-length" => (500, "Server Error", b"er", true),
        _ => (204, "No Content", b"", false),
    };
    write!(stream, "HTTP/1.1 {status} {reason}\r\n").expect("write HTTP status");
    if include_length {
        write!(stream, "Content-Length: {}\r\n", body.len()).expect("write content length");
    }
    write!(stream, "Connection: close\r\n\r\n").expect("finish HTTP headers");
    if method != "HEAD" {
        stream.write_all(body).expect("write HTTP body");
    }
    stream.flush().expect("flush HTTP response");
}

fn export_decode(
    output: &mut String,
    directory: &Path,
    name: &str,
    character_encoding: Option<&str>,
    bytes: &[u8],
) {
    let file = directory.join(format!("{name}.txt"));
    fs::write(&file, bytes).expect("write decoding fixture");
    let resource = FileTemplateResource::from_file(Some(&file), character_encoding)
        .expect("valid decoding resource");
    let decoded = read_all(resource.reader().expect("decoded reader"));
    emit(
        output,
        &format!("file.decode.{name}"),
        Some(&code_points(&decoded)),
    );
}

fn code_points(value: &str) -> String {
    value
        .chars()
        .map(|character| format!("{:04X}", character as u32))
        .collect::<Vec<_>>()
        .join(",")
}

fn slash(value: &str) -> String {
    value.replace('\\', "/")
}

fn temporary_directory() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "thymeleaf-file-golden-{}-{nonce}",
        std::process::id()
    ))
}

fn read_all(mut reader: Box<dyn Read>) -> String {
    let mut result = String::new();
    reader
        .read_to_string(&mut result)
        .expect("read template resource");
    result
}

fn emit_bool(output: &mut String, key: &str, value: bool) {
    emit(output, key, Some(&value.to_string()));
}

fn emit(output: &mut String, key: &str, value: Option<&str>) {
    output.push_str(key);
    output.push('=');
    output.push_str(&escape(value));
    output.push('\n');
}

fn escape(value: Option<&str>) -> String {
    let Some(value) = value else {
        return "null".to_owned();
    };
    value
        .replace('\\', "\\\\")
        .replace('\r', "\\r")
        .replace('\n', "\\n")
        .replace('\0', "\\0")
}
