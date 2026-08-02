//! 标准转换服务对象族与 `NoOpToken` 的 Thymeleaf 3.1.5 Java/Rust Golden 差分测试。

use std::any::Any;
use std::fmt::Write;
use std::ptr;

use thymeleaf::expression::{
    AbstractStandardConversionService, IStandardConversionService, JavaConversionObject,
    JavaConversionResult, JavaConversionValue, JavaStringConversionResult, JavaTargetClass,
    NoOpToken, StandardConversionError, StandardConversionService,
};
use thymeleaf::util::JavaString;

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
    let source = JavaString::from_rust_str("source");
    let context = ();

    assert!(matches!(
        created
            .convert(
                Some(&context as &dyn Any),
                JavaConversionValue::String(&source),
                Some(&JavaTargetClass::String),
            )
            .expect("string conversion"),
        JavaConversionResult::BorrowedString(value) if ptr::eq(value, &source)
    ));
    assert_eq!(
        JavaTargetClass::Other("example.Target".to_owned()).to_string(),
        "example.Target"
    );
    assert_eq!(JavaTargetClass::String.get_name(), "java.lang.String");

    let borrowed: JavaConversionResult<'_> = JavaStringConversionResult::Borrowed(&source).into();
    assert!(matches!(
        borrowed,
        JavaConversionResult::BorrowedString(value) if ptr::eq(value, &source)
    ));
    let owned: JavaConversionResult<'_> =
        JavaStringConversionResult::Owned(JavaString::from_rust_str("owned")).into();
    assert!(matches!(owned, JavaConversionResult::OwnedString(_)));
    let null: JavaConversionResult<'_> = JavaStringConversionResult::Null.into();
    assert!(matches!(null, JavaConversionResult::Null));

    let borrowed_number = 9_i32;
    let borrowed_object = JavaConversionResult::BorrowedObject(&borrowed_number);
    assert_eq!(describe_result(borrowed_object), "9");

    let runtime = StandardConversionError::runtime("example.Exception", "failure");
    assert_eq!(runtime.java_class_name(), "example.Exception");
    assert_eq!(runtime.to_string(), "failure");

    let unavailable = match service.convert(
        None,
        JavaConversionValue::Null,
        Some(&JavaTargetClass::Other("example.Target".to_owned())),
    ) {
        Ok(_) => panic!("default other conversion must fail"),
        Err(error) => error,
    };
    assert_eq!(
        unavailable.java_class_name(),
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
    let source = JavaString::from_rust_str("source");
    let object = ToStringProbe::value("object");
    let null_object = ToStringProbe::null();
    let throwing_object = ToStringProbe::throwing();
    let integer_target = JavaTargetClass::Other("java.lang.Integer".to_owned());
    let array_target = JavaTargetClass::Other("[I".to_owned());

    emit_outcome(
        output,
        "default.targetNull",
        service.convert(None, JavaConversionValue::String(&source), None),
    );
    emit_conversion(
        output,
        "default.stringNull",
        service.convert(
            None,
            JavaConversionValue::Null,
            Some(&JavaTargetClass::String),
        ),
    );
    emit(
        output,
        "default.stringIdentity",
        matches!(
            service.convert(
                None,
                JavaConversionValue::String(&source),
                Some(&JavaTargetClass::String),
            ),
            Ok(JavaConversionResult::BorrowedString(value)) if ptr::eq(value, &source)
        ),
    );
    emit_conversion(
        output,
        "default.objectString",
        service.convert(
            None,
            JavaConversionValue::Object(&object),
            Some(&JavaTargetClass::String),
        ),
    );
    let borrowing_object = BorrowingProbe {
        value: JavaString::from_rust_str("shared"),
    };
    emit(
        output,
        "default.objectStringIdentity",
        matches!(
            service.convert(
                None,
                JavaConversionValue::Object(&borrowing_object),
                Some(&JavaTargetClass::String),
            ),
            Ok(JavaConversionResult::BorrowedString(value))
                if ptr::eq(value, &borrowing_object.value)
        ),
    );
    emit_conversion(
        output,
        "default.objectNull",
        service.convert(
            None,
            JavaConversionValue::Object(&null_object),
            Some(&JavaTargetClass::String),
        ),
    );
    emit_outcome(
        output,
        "default.objectError",
        service.convert(
            None,
            JavaConversionValue::Object(&throwing_object),
            Some(&JavaTargetClass::String),
        ),
    );
    emit_outcome(
        output,
        "default.otherNull",
        service.convert(None, JavaConversionValue::Null, Some(&integer_target)),
    );
    emit_outcome(
        output,
        "default.otherObject",
        service.convert(
            None,
            JavaConversionValue::Object(&object),
            Some(&integer_target),
        ),
    );
    emit_outcome(
        output,
        "default.arrayTarget",
        service.convert(
            None,
            JavaConversionValue::String(&source),
            Some(&array_target),
        ),
    );
}

fn emit_custom_service_cases(output: &mut String) {
    let service: &dyn IStandardConversionService = &CustomService;
    let context = ();
    let source = JavaString::from_rust_str("source");
    let object = ToStringProbe::value("object");

    emit_conversion(
        output,
        "custom.stringNull",
        service.convert(
            Some(&context as &dyn Any),
            JavaConversionValue::Null,
            Some(&JavaTargetClass::String),
        ),
    );
    emit(
        output,
        "custom.stringIdentity",
        matches!(
            service.convert(
                Some(&context as &dyn Any),
                JavaConversionValue::String(&source),
                Some(&JavaTargetClass::String),
            ),
            Ok(JavaConversionResult::BorrowedString(value)) if ptr::eq(value, &source)
        ),
    );
    emit_conversion(
        output,
        "custom.objectContext",
        service.convert(
            Some(&context as &dyn Any),
            JavaConversionValue::Object(&object),
            Some(&JavaTargetClass::String),
        ),
    );
    emit_conversion(
        output,
        "custom.objectNullContext",
        service.convert(
            None,
            JavaConversionValue::Object(&object),
            Some(&JavaTargetClass::String),
        ),
    );
    emit_conversion(
        output,
        "custom.otherNull",
        service.convert(
            Some(&context as &dyn Any),
            JavaConversionValue::Null,
            Some(&JavaTargetClass::Other("java.lang.Integer".to_owned())),
        ),
    );
    emit_conversion(
        output,
        "custom.otherObject",
        service.convert(
            Some(&context as &dyn Any),
            JavaConversionValue::Object(&object),
            Some(&JavaTargetClass::Other("java.lang.Integer".to_owned())),
        ),
    );
}

struct ToStringProbe {
    result: Result<Option<JavaString>, StandardConversionError>,
}

impl ToStringProbe {
    fn value(value: &str) -> Self {
        Self {
            result: Ok(Some(JavaString::from_rust_str(value))),
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

impl JavaConversionObject for ToStringProbe {
    fn java_to_string(&self) -> Result<JavaStringConversionResult<'_>, StandardConversionError> {
        match &self.result {
            Ok(Some(value)) => Ok(JavaStringConversionResult::Owned(value.clone())),
            Ok(None) => Ok(JavaStringConversionResult::Null),
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
    value: JavaString,
}

impl JavaConversionObject for BorrowingProbe {
    fn java_to_string(&self) -> Result<JavaStringConversionResult<'_>, StandardConversionError> {
        Ok(JavaStringConversionResult::Borrowed(&self.value))
    }
}

struct CustomService;

impl AbstractStandardConversionService for CustomService {
    fn convert_to_string<'a>(
        &self,
        context: Option<&dyn Any>,
        object: &'a dyn JavaConversionObject,
    ) -> Result<JavaStringConversionResult<'a>, StandardConversionError> {
        let value = match object.java_to_string()? {
            JavaStringConversionResult::Null => "null".to_owned(),
            JavaStringConversionResult::Borrowed(value) => value.to_string_lossy(),
            JavaStringConversionResult::Owned(value) => value.to_string_lossy(),
        };
        let prefix = if context.is_some() {
            "context:"
        } else {
            "null:"
        };
        Ok(JavaStringConversionResult::Owned(
            JavaString::from_rust_str(&format!("{prefix}{value}")),
        ))
    }

    fn convert_other<'a>(
        &self,
        _context: Option<&dyn Any>,
        object: JavaConversionValue<'a>,
        target_class: &JavaTargetClass,
    ) -> Result<JavaConversionResult<'a>, StandardConversionError> {
        if target_class == &JavaTargetClass::Other("java.lang.Integer".to_owned()) {
            let value = if matches!(object, JavaConversionValue::Null) {
                7_i32
            } else {
                8_i32
            };
            return Ok(JavaConversionResult::OwnedObject(Box::new(value)));
        }
        Err(StandardConversionError::NoAvailableConversion {
            target_class_name: target_class.get_name().to_owned(),
        })
    }
}

fn emit_conversion(
    output: &mut String,
    key: &str,
    result: Result<JavaConversionResult<'_>, StandardConversionError>,
) {
    match result {
        Ok(value) => emit(output, key, describe_result(value)),
        Err(error) => emit_error(output, key, &error),
    }
}

fn emit_outcome(
    output: &mut String,
    key: &str,
    result: Result<JavaConversionResult<'_>, StandardConversionError>,
) {
    match result {
        Ok(value) => emit(output, key, format!("OK:{}", describe_result(value))),
        Err(error) => emit(
            output,
            key,
            format!("ERR:{}:{}", error.java_class_name(), error),
        ),
    }
}

fn emit_error(output: &mut String, key: &str, error: &StandardConversionError) {
    emit(
        output,
        key,
        format!("ERR:{}:{}", error.java_class_name(), error),
    );
}

fn describe_result(result: JavaConversionResult<'_>) -> String {
    match result {
        JavaConversionResult::Null => "null".to_owned(),
        JavaConversionResult::BorrowedString(value) => value.to_string_lossy(),
        JavaConversionResult::OwnedString(value) => value.to_string_lossy(),
        JavaConversionResult::BorrowedObject(value) => value
            .downcast_ref::<i32>()
            .expect("borrowed i32 result")
            .to_string(),
        JavaConversionResult::OwnedObject(value) => value
            .downcast::<i32>()
            .expect("owned i32 result")
            .to_string(),
    }
}

fn emit(output: &mut String, key: &str, value: impl std::fmt::Display) {
    writeln!(output, "{key}={value}").expect("write to String");
}
