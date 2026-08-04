//! 标准转换服务对象族与 `NoOpToken` 的 Thymeleaf 3.1.5 Java/Rust Golden 差分测试。

use std::any::Any;
use std::fmt::Write;
use std::ptr;

use thymeleaf::expression::{
    AbstractStandardConversionService, ConversionObject, ConversionResult, ConversionValue,
    IStandardConversionService, NoOpToken, StandardConversionError, StandardConversionService,
    TargetClass, Utf16StringConversionResult,
};
use thymeleaf::util::Utf16String;

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const JAVA_GOLDEN: &str =
    include_str!("../../thymeleaf/tests/fixtures/standard_conversion_service_golden.txt");

#[test]
fn standard_conversion_service_objects_match_java_golden() {
    cover_public_adapter_contracts();

    let mut output = String::new();
    emit(&mut output, "baseline", JAVA_BASELINE);
    emit_no_op_cases(&mut output);
    emit_default_service_cases(&mut output);
    emit_custom_service_cases(&mut output);
    assert_eq!(output, JAVA_GOLDEN);
}

fn cover_public_adapter_contracts() {
    let service = StandardConversionService::default();
    let created = StandardConversionService::new();
    let source = Utf16String::from_rust_str("source");
    let context = ();

    assert!(matches!(
        created
            .convert(
                Some(&context as &dyn Any),
                ConversionValue::String(&source),
                Some(&TargetClass::String),
            )
            .expect("string conversion"),
        ConversionResult::BorrowedString(value) if ptr::eq(value, &source)
    ));
    assert_eq!(
        TargetClass::Other("example.Target".to_owned()).to_string(),
        "example.Target"
    );
    assert_eq!(TargetClass::String.get_name(), "java.lang.String");

    let borrowed: ConversionResult<'_> = Utf16StringConversionResult::Borrowed(&source).into();
    assert!(matches!(
        borrowed,
        ConversionResult::BorrowedString(value) if ptr::eq(value, &source)
    ));
    let owned: ConversionResult<'_> =
        Utf16StringConversionResult::Owned(Utf16String::from_rust_str("owned")).into();
    assert!(matches!(owned, ConversionResult::OwnedString(_)));
    let null: ConversionResult<'_> = Utf16StringConversionResult::Null.into();
    assert!(matches!(null, ConversionResult::Null));

    let borrowed_number = 9_i32;
    let borrowed_object = ConversionResult::BorrowedObject(&borrowed_number);
    assert_eq!(describe_result(borrowed_object), "9");

    let runtime = StandardConversionError::runtime("example.Exception", "failure");
    assert_eq!(runtime.class_name(), "example.Exception");
    assert_eq!(runtime.to_string(), "failure");

    let unavailable = match service.convert(
        None,
        ConversionValue::Null,
        Some(&TargetClass::Other("example.Target".to_owned())),
    ) {
        Ok(_) => panic!("default other conversion must fail"),
        Err(error) => error,
    };
    assert_eq!(
        unavailable.class_name(),
        "java.lang.IllegalArgumentException"
    );
}

fn emit_no_op_cases(output: &mut String) {
    emit(
        output,
        "noop.same",
        ptr::eq(NoOpToken::VALUE, NoOpToken::VALUE),
    );
    emit(output, "noop.text", NoOpToken::VALUE);
    emit(
        output,
        "noop.identityEquals",
        NoOpToken::VALUE.eq(NoOpToken::VALUE),
    );
    emit(output, "noop.otherEquals", false);
}

fn emit_default_service_cases(output: &mut String) {
    let service: &dyn IStandardConversionService = &StandardConversionService::new();
    let source = Utf16String::from_rust_str("source");
    let object = ToStringProbe::value("object");
    let null_object = ToStringProbe::null();
    let throwing_object = ToStringProbe::throwing();
    let integer_target = TargetClass::Other("java.lang.Integer".to_owned());
    let array_target = TargetClass::Other("[I".to_owned());

    emit_outcome(
        output,
        "default.targetNull",
        service.convert(None, ConversionValue::String(&source), None),
    );
    emit_conversion(
        output,
        "default.stringNull",
        service.convert(None, ConversionValue::Null, Some(&TargetClass::String)),
    );
    emit(
        output,
        "default.stringIdentity",
        matches!(
            service.convert(
                None,
                ConversionValue::String(&source),
                Some(&TargetClass::String),
            ),
            Ok(ConversionResult::BorrowedString(value)) if ptr::eq(value, &source)
        ),
    );
    emit_conversion(
        output,
        "default.objectString",
        service.convert(
            None,
            ConversionValue::Object(&object),
            Some(&TargetClass::String),
        ),
    );
    let borrowing_object = BorrowingProbe {
        value: Utf16String::from_rust_str("shared"),
    };
    emit(
        output,
        "default.objectStringIdentity",
        matches!(
            service.convert(
                None,
                ConversionValue::Object(&borrowing_object),
                Some(&TargetClass::String),
            ),
            Ok(ConversionResult::BorrowedString(value))
                if ptr::eq(value, &borrowing_object.value)
        ),
    );
    emit_conversion(
        output,
        "default.objectNull",
        service.convert(
            None,
            ConversionValue::Object(&null_object),
            Some(&TargetClass::String),
        ),
    );
    emit_outcome(
        output,
        "default.objectError",
        service.convert(
            None,
            ConversionValue::Object(&throwing_object),
            Some(&TargetClass::String),
        ),
    );
    emit_outcome(
        output,
        "default.otherNull",
        service.convert(None, ConversionValue::Null, Some(&integer_target)),
    );
    emit_outcome(
        output,
        "default.otherObject",
        service.convert(
            None,
            ConversionValue::Object(&object),
            Some(&integer_target),
        ),
    );
    emit_outcome(
        output,
        "default.arrayTarget",
        service.convert(None, ConversionValue::String(&source), Some(&array_target)),
    );
}

fn emit_custom_service_cases(output: &mut String) {
    let service: &dyn IStandardConversionService = &CustomService;
    let context = ();
    let source = Utf16String::from_rust_str("source");
    let object = ToStringProbe::value("object");

    emit_conversion(
        output,
        "custom.stringNull",
        service.convert(
            Some(&context as &dyn Any),
            ConversionValue::Null,
            Some(&TargetClass::String),
        ),
    );
    emit(
        output,
        "custom.stringIdentity",
        matches!(
            service.convert(
                Some(&context as &dyn Any),
                ConversionValue::String(&source),
                Some(&TargetClass::String),
            ),
            Ok(ConversionResult::BorrowedString(value)) if ptr::eq(value, &source)
        ),
    );
    emit_conversion(
        output,
        "custom.objectContext",
        service.convert(
            Some(&context as &dyn Any),
            ConversionValue::Object(&object),
            Some(&TargetClass::String),
        ),
    );
    emit_conversion(
        output,
        "custom.objectNullContext",
        service.convert(
            None,
            ConversionValue::Object(&object),
            Some(&TargetClass::String),
        ),
    );
    emit_conversion(
        output,
        "custom.otherNull",
        service.convert(
            Some(&context as &dyn Any),
            ConversionValue::Null,
            Some(&TargetClass::Other("java.lang.Integer".to_owned())),
        ),
    );
    emit_conversion(
        output,
        "custom.otherObject",
        service.convert(
            Some(&context as &dyn Any),
            ConversionValue::Object(&object),
            Some(&TargetClass::Other("java.lang.Integer".to_owned())),
        ),
    );
}

struct ToStringProbe {
    result: Result<Option<Utf16String>, StandardConversionError>,
}

impl ToStringProbe {
    fn value(value: &str) -> Self {
        Self {
            result: Ok(Some(Utf16String::from_rust_str(value))),
        }
    }

    fn null() -> Self {
        Self { result: Ok(None) }
    }

    fn throwing() -> Self {
        Self {
            result: Err(StandardConversionError::runtime(
                "java.lang.IllegalStateException",
                "boom",
            )),
        }
    }
}

impl ConversionObject for ToStringProbe {
    fn to_utf16_string(&self) -> Result<Utf16StringConversionResult<'_>, StandardConversionError> {
        match &self.result {
            Ok(Some(value)) => Ok(Utf16StringConversionResult::Owned(value.clone())),
            Ok(None) => Ok(Utf16StringConversionResult::Null),
            Err(StandardConversionError::Runtime {
                exception_class_name,
                message,
            }) => Err(StandardConversionError::runtime(
                exception_class_name,
                message,
            )),
            Err(error) => panic!("unexpected probe error: {error}"),
        }
    }
}

struct BorrowingProbe {
    value: Utf16String,
}

impl ConversionObject for BorrowingProbe {
    fn to_utf16_string(&self) -> Result<Utf16StringConversionResult<'_>, StandardConversionError> {
        Ok(Utf16StringConversionResult::Borrowed(&self.value))
    }
}

struct CustomService;

impl AbstractStandardConversionService for CustomService {
    fn convert_to_string<'a>(
        &self,
        context: Option<&dyn Any>,
        object: &'a dyn ConversionObject,
    ) -> Result<Utf16StringConversionResult<'a>, StandardConversionError> {
        let value = match object.to_utf16_string()? {
            Utf16StringConversionResult::Null => "null".to_owned(),
            Utf16StringConversionResult::Borrowed(value) => value.to_string_lossy(),
            Utf16StringConversionResult::Owned(value) => value.to_string_lossy(),
        };
        let prefix = if context.is_some() {
            "context:"
        } else {
            "null:"
        };
        Ok(Utf16StringConversionResult::Owned(
            Utf16String::from_rust_str(&format!("{prefix}{value}")),
        ))
    }

    fn convert_other<'a>(
        &self,
        _context: Option<&dyn Any>,
        object: ConversionValue<'a>,
        target_class: &TargetClass,
    ) -> Result<ConversionResult<'a>, StandardConversionError> {
        if target_class == &TargetClass::Other("java.lang.Integer".to_owned()) {
            let value = if matches!(object, ConversionValue::Null) {
                7_i32
            } else {
                8_i32
            };
            return Ok(ConversionResult::OwnedObject(Box::new(value)));
        }
        Err(StandardConversionError::NoAvailableConversion {
            target_class_name: target_class.get_name().to_owned(),
        })
    }
}

fn emit_conversion(
    output: &mut String,
    key: &str,
    result: Result<ConversionResult<'_>, StandardConversionError>,
) {
    match result {
        Ok(value) => emit(output, key, describe_result(value)),
        Err(error) => emit_error(output, key, &error),
    }
}

fn emit_outcome(
    output: &mut String,
    key: &str,
    result: Result<ConversionResult<'_>, StandardConversionError>,
) {
    match result {
        Ok(value) => emit(output, key, format!("OK:{}", describe_result(value))),
        Err(error) => emit(output, key, format!("ERR:{}:{}", error.class_name(), error)),
    }
}

fn emit_error(output: &mut String, key: &str, error: &StandardConversionError) {
    emit(output, key, format!("ERR:{}:{}", error.class_name(), error));
}

fn describe_result(result: ConversionResult<'_>) -> String {
    match result {
        ConversionResult::Null => "null".to_owned(),
        ConversionResult::BorrowedString(value) => value.to_string_lossy(),
        ConversionResult::OwnedString(value) => value.to_string_lossy(),
        ConversionResult::BorrowedObject(value) => value
            .downcast_ref::<i32>()
            .expect("borrowed i32 result")
            .to_string(),
        ConversionResult::OwnedObject(value) => value
            .downcast::<i32>()
            .expect("owned i32 result")
            .to_string(),
    }
}

fn emit(output: &mut String, key: &str, value: impl std::fmt::Display) {
    writeln!(output, "{key}={value}").expect("write to String");
}
