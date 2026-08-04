//! `ContentTypeUtils` 的 Thymeleaf 3.1.5 Java/Rust Golden 差分测试。

use std::fmt::{Display, Write};

use thymeleaf::util::{Charset, CharsetError, ContentTypeError, ContentTypeUtils};

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const JAVA_GOLDEN: &str =
    include_str!("../../thymeleaf/tests/fixtures/content_type_utils_golden.txt");

#[test]
fn content_type_utils_matches_java_golden() {
    let mut output = String::new();
    emit(&mut output, "baseline", JAVA_BASELINE);
    emit_content_types(&mut output);
    emit_template_names(&mut output);
    emit_request_paths(&mut output);
    emit_charsets(&mut output);
    emit_combinations(&mut output);
    if output != JAVA_GOLDEN {
        let mismatch = output
            .lines()
            .zip(JAVA_GOLDEN.lines())
            .enumerate()
            .find(|(_, (rust, java))| rust != java);
        panic!("first Java/Rust mismatch: {mismatch:?}");
    }
}

fn emit_content_types(output: &mut String) {
    let values = [
        None,
        Some(""),
        Some(" \t"),
        Some("text/html"),
        Some(" APPLICATION/XHTML+XML ; q=1"),
        Some("application/xml"),
        Some("text/xml"),
        Some("application/rss+xml"),
        Some("application/atom+xml"),
        Some("application/javascript"),
        Some("application/x-javascript"),
        Some("application/ecmascript"),
        Some("text/javascript"),
        Some("text/ecmascript"),
        Some("application/json"),
        Some("text/css"),
        Some("text/plain"),
        Some("text/event-stream"),
        Some("application/octet-stream"),
        Some("; TEXT/HTML ;; Charset=UTF-8"),
        Some(";;;"),
    ];
    for (index, value) in values.into_iter().enumerate() {
        let key = format!("content.{index}");
        emit_bool_result(
            output,
            &format!("{key}.html"),
            ContentTypeUtils::is_content_type_html(value),
        );
        emit_bool_result(
            output,
            &format!("{key}.xml"),
            ContentTypeUtils::is_content_type_xml(value),
        );
        emit_bool_result(
            output,
            &format!("{key}.rss"),
            ContentTypeUtils::is_content_type_rss(value),
        );
        emit_bool_result(
            output,
            &format!("{key}.atom"),
            ContentTypeUtils::is_content_type_atom(value),
        );
        emit_bool_result(
            output,
            &format!("{key}.javascript"),
            ContentTypeUtils::is_content_type_java_script(value),
        );
        emit_bool_result(
            output,
            &format!("{key}.json"),
            ContentTypeUtils::is_content_type_json(value),
        );
        emit_bool_result(
            output,
            &format!("{key}.css"),
            ContentTypeUtils::is_content_type_css(value),
        );
        emit_bool_result(
            output,
            &format!("{key}.text"),
            ContentTypeUtils::is_content_type_text(value),
        );
        emit_bool_result(
            output,
            &format!("{key}.sse"),
            ContentTypeUtils::is_content_type_sse(value),
        );
        emit_option_result(
            output,
            &format!("{key}.mode"),
            ContentTypeUtils::compute_template_mode_for_content_type(value),
        );
    }
}

fn emit_template_names(output: &mut String) {
    let values = [
        None,
        Some(""),
        Some(" \t"),
        Some("index"),
        Some("index."),
        Some("view.html"),
        Some("view.HTML "),
        Some("archive.tar.xml"),
        Some(".rss"),
        Some("script.js"),
        Some("data.json"),
        Some("style.css"),
        Some("plain.txt"),
        Some("feed.atom"),
        Some("unknown.bin"),
        Some("name.xhtml"),
    ];
    let utf8 = Charset::for_name("UTF-8").unwrap();
    for (index, value) in values.into_iter().enumerate() {
        let key = format!("template.{index}");
        emit_ok_option(
            output,
            &format!("{key}.mode"),
            ContentTypeUtils::compute_template_mode_for_template_name(value),
        );
        emit_ok_option(
            output,
            &format!("{key}.recognized"),
            Some(ContentTypeUtils::has_recognized_file_extension(value)),
        );
        emit_ok_option(
            output,
            &format!("{key}.plain"),
            ContentTypeUtils::compute_content_type_for_template_name(value, None),
        );
        emit_ok_option(
            output,
            &format!("{key}.utf8"),
            ContentTypeUtils::compute_content_type_for_template_name(value, Some(&utf8)),
        );
    }
}

fn emit_request_paths(output: &mut String) {
    let values = [
        None,
        Some(""),
        Some("/"),
        Some("/index"),
        Some("/view.html"),
        Some("/INDEX.HTML"),
        Some("/asset/app.js?x=.css#part;v=1"),
        Some("/style.css;v=2?x=1#part"),
        Some("/feed.atom#fragment"),
        Some("/data.json?x=1"),
        Some("/plain.txt "),
        Some("relative.xml"),
        Some("/dir.with.dot/file"),
        Some("/dir/.rss"),
    ];
    let latin1 = Charset::for_name("ISO-8859-1").unwrap();
    for (index, value) in values.into_iter().enumerate() {
        let key = format!("request.{index}");
        emit_option_result(
            output,
            &format!("{key}.mode"),
            ContentTypeUtils::compute_template_mode_for_request_path(value),
        );
        emit_option_result(
            output,
            &format!("{key}.plain"),
            ContentTypeUtils::compute_content_type_for_request_path(value, None),
        );
        emit_option_result(
            output,
            &format!("{key}.latin1"),
            ContentTypeUtils::compute_content_type_for_request_path(value, Some(&latin1)),
        );
    }
}

fn emit_charsets(output: &mut String) {
    let values = [
        None,
        Some(""),
        Some("text/html"),
        Some("text/html;charset=UTF-8"),
        Some("text/html;CHARSET=latin1"),
        Some("text/html;charset=UTF-16LE"),
        Some("text/html;charset=UTF-32BE"),
        Some("text/html;charset=windows-1252"),
        Some("text/html;charset=Shift_JIS"),
        Some("text/html;charset=x-no-such-charset"),
        Some("text/html;charset=replacement"),
        Some("text/html;charset=US-ASCII"),
        Some("text/html;charset=ascii"),
        Some("text/html;charset=iso646-us"),
        Some("text/html;charset=646"),
        Some("text/html;charset=iso-8859-1"),
        Some("text/html;charset=iso_8859-1"),
        Some("text/html;charset=l1"),
        Some("text/html;charset=ibm819"),
        Some("text/html;charset=cp819"),
        Some("text/html;charset=utf8"),
        Some("text/html;charset=unicode-1-1-utf-8"),
        Some("text/html;charset=utf-16"),
        Some("text/html;charset=utf16"),
        Some("text/html;charset=unicode"),
        Some("text/html;charset=utf-16be"),
        Some("text/html;charset=utf_16be"),
        Some("text/html;charset=unicodebigunmarked"),
        Some("text/html;charset=utf_16le"),
        Some("text/html;charset=unicodelittleunmarked"),
        Some("text/html;charset=utf-32"),
        Some("text/html;charset=utf32"),
        Some("text/html;charset=utf_32be"),
        Some("text/html;charset=utf-32le"),
        Some("text/html;charset=utf_32le"),
        Some("text/html;charset=csiso2022kr"),
        Some("text/html;charset=hz-gb-2312"),
        Some("text/html;charset=iso-2022-cn"),
        Some("text/html;charset=iso-2022-cn-ext"),
        Some("text/html;charset=iso-2022-kr"),
        Some("text/html;charset=\"utf-8\""),
        Some("text/html;charset"),
        Some("text/html;charset=;q=1"),
        Some(";;;"),
    ];
    for (index, value) in values.into_iter().enumerate() {
        emit_option_result(
            output,
            &format!("charset.{index}"),
            ContentTypeUtils::compute_charset_from_content_type(value),
        );
    }
}

fn emit_combinations(output: &mut String) {
    let values = [
        None,
        Some(""),
        Some(" \t"),
        Some("text/html"),
        Some("TEXT/HTML;CHARSET=us-ascii;q=1"),
        Some(" Text/HTML ; Foo = A ; flag ; foo=B "),
        Some(";;;"),
    ];
    let utf16 = Charset::for_name("Unicode").unwrap();
    for (index, value) in values.into_iter().enumerate() {
        let key = format!("combine.{index}");
        emit_option_result(
            output,
            &format!("{key}.null"),
            ContentTypeUtils::combine_content_type_and_charset(value, None),
        );
        emit_option_result(
            output,
            &format!("{key}.utf16"),
            ContentTypeUtils::combine_content_type_and_charset(value, Some(&utf16)),
        );
    }
}

fn emit_bool_result(output: &mut String, key: &str, result: Result<bool, ContentTypeError>) {
    emit_option_result(output, key, result.map(Some));
}

fn emit_option_result<T: Display>(
    output: &mut String,
    key: &str,
    result: Result<Option<T>, ContentTypeError>,
) {
    match result {
        Ok(value) => emit_ok_option(output, key, value),
        Err(error) => {
            let (class_name, message) = error_parts(&error);
            emit(
                output,
                key,
                format!("error:{class_name}:{}", encode(Some(message))),
            );
        }
    }
}

fn emit_ok_option<T: Display>(output: &mut String, key: &str, value: Option<T>) {
    emit(
        output,
        key,
        format!(
            "ok:{}",
            encode(value.as_ref().map(ToString::to_string).as_deref())
        ),
    );
}

fn error_parts(error: &ContentTypeError) -> (&'static str, &str) {
    match error {
        ContentTypeError::MissingMimeType => (
            "ArrayIndexOutOfBoundsException",
            "Index 0 out of bounds for length 0",
        ),
        ContentTypeError::NullRequestPath => (
            "NullPointerException",
            "Cannot invoke \"String.indexOf(int)\" because \"<local1>\" is null",
        ),
        ContentTypeError::InvalidCharset(CharsetError::Illegal { charset_name }) => {
            ("IllegalCharsetNameException", charset_name)
        }
        ContentTypeError::InvalidCharset(CharsetError::Unsupported { charset_name }) => {
            ("UnsupportedCharsetException", charset_name)
        }
    }
}

fn encode(value: Option<&str>) -> String {
    let Some(value) = value else {
        return "<null>".to_owned();
    };
    let mut encoded = String::new();
    for (index, code_unit) in value.encode_utf16().enumerate() {
        if index > 0 {
            encoded.push(',');
        }
        write!(encoded, "{code_unit:04X}").unwrap();
    }
    encoded
}

fn emit(output: &mut String, key: &str, value: impl Display) {
    writeln!(output, "{key}={value}").unwrap();
}
