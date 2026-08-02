//! `FastStringWriter` 与固定 Thymeleaf Java 基线的逐记录语义对照测试。

use std::fmt::Write;

use thymeleaf::util::{FastStringWriter, FastStringWriterError, JavaString};

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const JAVA_GOLDEN: &str =
    include_str!("../../thymeleaf/tests/fixtures/fast_string_writer_golden.txt");
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x100_0000_01b3;

#[test]
fn fast_string_writer_matches_java_golden() {
    let mut output = String::new();
    emit(&mut output, "baseline", JAVA_BASELINE);
    constructor_cases(&mut output);
    write_int_cases(&mut output);
    write_string_cases(&mut output);
    write_string_range_cases(&mut output);
    write_char_array_cases(&mut output);
    lifecycle_and_inherited_writer_cases(&mut output);
    exhaustive_cases(&mut output);

    assert_eq!(output, JAVA_GOLDEN);
}

fn constructor_cases(output: &mut String) {
    emit(
        output,
        "constructor.default",
        FastStringWriter::new().to_string().to_string_lossy(),
    );
    emit(
        output,
        "constructor.zero",
        FastStringWriter::with_initial_size(0)
            .expect("zero")
            .to_string()
            .to_string_lossy(),
    );
    emit(
        output,
        "constructor.one",
        FastStringWriter::with_initial_size(1)
            .expect("one")
            .to_string()
            .to_string_lossy(),
    );
    emit_result(
        output,
        "constructor.negative",
        FastStringWriter::with_initial_size(-1).map(|writer| writer.to_string()),
    );
    emit_result(
        output,
        "constructor.minimum",
        FastStringWriter::with_initial_size(i32::MIN).map(|writer| writer.to_string()),
    );
}

fn write_int_cases(output: &mut String) {
    let mut writer = FastStringWriter::new();
    writer.write_char(i32::from(b'A'));
    writer.write_char(-1);
    writer.write_char(0x1_0000);
    writer.write_char(0x1_F600);
    emit(output, "writeInt.utf16", to_utf16_hex(&writer.to_string()));
}

fn write_string_cases(output: &mut String) {
    let mut writer = FastStringWriter::new();
    writer.write_string(Some(&java("ab")));
    writer.write_string(None);
    writer.write_string(Some(&JavaString::from_utf16([0xD800])));
    emit(
        output,
        "writeString.utf16",
        to_utf16_hex(&writer.to_string()),
    );

    let empty = FastStringWriter::new();
    let empty_first = empty.to_string();
    let empty_second = empty.to_string();
    emit(
        output,
        "toString.emptyIdentity",
        std::ptr::eq(&empty_first, &empty_second),
    );
    writer.write_string(Some(&java("x")));
    let first = writer.to_string();
    let second = writer.to_string();
    emit(
        output,
        "toString.nonEmptyIdentity",
        std::ptr::eq(&first, &second),
    );
    let snapshot = writer.to_string();
    writer.write_string(Some(&java("tail")));
    emit(output, "toString.snapshot", to_utf16_hex(&snapshot));
    emit(
        output,
        "toString.current",
        to_utf16_hex(&writer.to_string()),
    );
}

fn write_string_range_cases(output: &mut String) {
    emit_string_range(output, "normal", Some(&java("abcdef")), 1, 3);
    emit_string_range(output, "emptyAtEnd", Some(&java("abcdef")), 6, 0);
    emit_string_range(output, "nullFull", None, 0, 4);
    emit_string_range(output, "nullMiddle", None, 1, 2);
    emit_string_range(output, "negativeOffset", Some(&java("abc")), -1, 1);
    emit_string_range(output, "negativeLength", Some(&java("abc")), 0, -1);
    emit_string_range(output, "offsetAfterEnd", Some(&java("abc")), 4, 0);
    emit_string_range(output, "endAfterEnd", Some(&java("abc")), 2, 2);
    emit_string_range(output, "overflow", Some(&java("abc")), i32::MAX, 1);
    emit_string_range(output, "nullAfterEnd", None, 0, 5);
}

fn write_char_array_cases(output: &mut String) {
    let chars = [0x0041, 0xD800, 0x0042, 0x0043];
    emit_char_range(output, "full", Some(&chars), 0, 4, true);
    emit_char_range(output, "middle", Some(&chars), 1, 2, false);
    emit_char_range(output, "emptyAtEnd", Some(&chars), 4, 0, false);
    emit_char_range(output, "negativeOffset", Some(&chars), -1, 1, false);
    emit_char_range(output, "negativeLength", Some(&chars), 0, -1, false);
    emit_char_range(output, "offsetAfterEnd", Some(&chars), 5, 0, false);
    emit_char_range(output, "endAfterEnd", Some(&chars), 3, 2, false);
    emit_char_range(output, "overflow", Some(&chars), 1, i32::MAX, false);
    emit_char_range(output, "nullFull", None, 0, 0, true);
    emit_char_range(output, "nullRange", None, 0, 0, false);
    emit_char_range(output, "nullNegativeOffset", None, -1, 0, false);
    emit_char_range(output, "nullNegativeLength", None, 0, -1, false);
}

fn lifecycle_and_inherited_writer_cases(output: &mut String) {
    let mut concrete = FastStringWriter::new();
    concrete.write_string(Some(&java("A")));
    concrete.flush();
    concrete.close();
    concrete.write_string(Some(&java("B")));
    concrete.close();
    concrete.flush();
    emit(
        output,
        "lifecycle.afterClose",
        concrete.to_string().to_string_lossy(),
    );

    let mut writer = FastStringWriter::new();
    let identity = std::ptr::from_ref(&writer);
    let append_null = std::ptr::from_mut(writer.append_sequence(None)).cast_const();
    emit(output, "append.nullIdentity", identity == append_null);
    let append_range = std::ptr::from_mut(
        writer
            .append_sequence_range(Some(&java("abcdef")), 1, 4)
            .expect("append range"),
    )
    .cast_const();
    emit(output, "append.rangeIdentity", identity == append_range);
    let append_char = std::ptr::from_mut(writer.append_char(0xD800)).cast_const();
    emit(output, "append.charIdentity", identity == append_char);
    emit(output, "append.utf16", to_utf16_hex(&writer.to_string()));

    emit_append_range(output, "nullMiddle", None, 1, 3);
    emit_append_range(output, "negativeStart", Some(&java("abc")), -1, 1);
    emit_append_range(output, "reversed", Some(&java("abc")), 2, 1);
    emit_append_range(output, "endAfterLength", Some(&java("abc")), 1, 4);
}

fn exhaustive_cases(output: &mut String) {
    let mut int_writer = FastStringWriter::with_initial_size(196_608).expect("capacity");
    for value in -65_536..=131_071 {
        int_writer.write_char(value);
    }
    emit(
        output,
        "exhaustive.writeIntHash",
        hex(hash_string(&int_writer.to_string())),
    );

    let string = JavaString::from_utf16([0x0041, 0xD800, 0x0042, 0x0043]);
    let mut string_range_hash = FNV_OFFSET;
    for offset in -2..=7 {
        for length in -2..=7 {
            let mut writer = FastStringWriter::new();
            match writer.write_string_range(Some(&string), offset, length) {
                Ok(()) => {
                    string_range_hash = mix(string_range_hash, 1);
                    string_range_hash = mix_string(string_range_hash, &writer.to_string());
                }
                Err(error) => {
                    string_range_hash = mix(string_range_hash, 0);
                    string_range_hash = mix_rust_str(string_range_hash, error.java_class_name());
                    string_range_hash =
                        mix_string(string_range_hash, &java_message_or_null(&error));
                }
            }
        }
    }
    emit(output, "exhaustive.stringRangeHash", hex(string_range_hash));

    let chars = [0x0041, 0xD800, 0x0042, 0x0043];
    let mut char_range_hash = FNV_OFFSET;
    for offset in -2..=7 {
        for length in -2..=7 {
            let mut writer = FastStringWriter::new();
            match writer.write_chars_range(Some(&chars), offset, length) {
                Ok(()) => {
                    char_range_hash = mix(char_range_hash, 1);
                    char_range_hash = mix_string(char_range_hash, &writer.to_string());
                }
                Err(error) => {
                    char_range_hash = mix(char_range_hash, 0);
                    char_range_hash = mix_rust_str(char_range_hash, error.java_class_name());
                    char_range_hash = mix_string(char_range_hash, &java_message_or_null(&error));
                }
            }
        }
    }
    emit(output, "exhaustive.charRangeHash", hex(char_range_hash));
}

fn emit_string_range(
    output: &mut String,
    key: &str,
    value: Option<&JavaString>,
    offset: i32,
    length: i32,
) {
    let mut writer = FastStringWriter::new();
    let result = writer
        .write_string_range(value, offset, length)
        .map(|()| writer.to_string());
    emit_result(output, &format!("writeStringRange.{key}"), result);
}

fn emit_char_range(
    output: &mut String,
    key: &str,
    value: Option<&[u16]>,
    offset: i32,
    length: i32,
    full_overload: bool,
) {
    let mut writer = FastStringWriter::new();
    let result = if full_overload {
        writer.write_chars(value)
    } else {
        writer.write_chars_range(value, offset, length)
    }
    .map(|()| writer.to_string());
    emit_result(output, &format!("writeChars.{key}"), result);
}

fn emit_append_range(
    output: &mut String,
    key: &str,
    value: Option<&JavaString>,
    start: i32,
    end: i32,
) {
    let mut writer = FastStringWriter::new();
    let result = writer
        .append_sequence_range(value, start, end)
        .map(|writer| writer.to_string());
    emit_result(output, &format!("appendRange.{key}"), result);
}

fn emit_result(output: &mut String, key: &str, result: Result<JavaString, FastStringWriterError>) {
    match result {
        Ok(value) => emit(output, key, format!("OK:{}", to_utf16_hex(&value))),
        Err(error) => emit(
            output,
            key,
            format!(
                "ERR:{}:{}",
                error.java_class_name(),
                to_utf16_hex(&java_message_or_null(&error))
            ),
        ),
    }
}

fn java_message_or_null(error: &FastStringWriterError) -> JavaString {
    error.message().unwrap_or_else(|| java("null"))
}

fn java(value: &str) -> JavaString {
    JavaString::from_rust_str(value)
}

fn to_utf16_hex(value: &JavaString) -> String {
    value
        .as_utf16()
        .iter()
        .map(|unit| format!("{unit:04x}"))
        .collect::<Vec<_>>()
        .join(",")
}

fn hash_string(value: &JavaString) -> u64 {
    mix_string(FNV_OFFSET, value)
}

fn mix_string(mut hash: u64, value: &JavaString) -> u64 {
    for unit in value.as_utf16() {
        hash = mix(hash, u64::from(unit & 0x00FF));
        hash = mix(hash, u64::from(unit >> 8));
    }
    hash
}

fn mix_rust_str(hash: u64, value: &str) -> u64 {
    mix_string(hash, &java(value))
}

fn mix(hash: u64, value: u64) -> u64 {
    (hash ^ value).wrapping_mul(FNV_PRIME)
}

fn hex(value: u64) -> String {
    format!("{value:016x}")
}

fn emit(output: &mut String, key: &str, value: impl std::fmt::Display) {
    writeln!(output, "{key}={value}").expect("write to string");
}
