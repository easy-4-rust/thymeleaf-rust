//! `IEngineProcessable` 的 Thymeleaf 3.1.5 Java/Rust Golden 差分测试。

use std::fmt::Write;
use std::ptr;

use thymeleaf::engine::IEngineProcessable;

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const JAVA_GOLDEN: &str = include_str!("fixtures/engine_processable_golden.txt");

#[test]
fn engine_processable_dynamic_contract_matches_java_golden() {
    let mut output = String::new();
    emit(&mut output, "baseline", JAVA_BASELINE);

    let mut concrete = AlternatingProcessable { calls: 0 };
    let concrete_pointer: *mut AlternatingProcessable = &mut concrete;
    let dynamic_pointer = {
        let dynamic: &mut dyn IEngineProcessable = &mut concrete;
        emit(
            &mut output,
            "process.1",
            dynamic.process().expect("process 1"),
        );
        emit(
            &mut output,
            "process.2",
            dynamic.process().expect("process 2"),
        );
        emit(
            &mut output,
            "process.3",
            dynamic.process().expect("process 3"),
        );
        emit(
            &mut output,
            "process.4",
            dynamic.process().expect("process 4"),
        );
        dynamic as *mut dyn IEngineProcessable as *mut ()
    };
    emit(&mut output, "process.calls", concrete.calls);
    emit(
        &mut output,
        "process.sameDynamicObject",
        ptr::eq(dynamic_pointer, concrete_pointer.cast()),
    );

    assert_eq!(output, JAVA_GOLDEN);
}

struct AlternatingProcessable {
    calls: usize,
}

impl IEngineProcessable for AlternatingProcessable {
    fn process(&mut self) -> Result<bool, Box<dyn thymeleaf::exceptions::TemplateEngineException>> {
        self.calls += 1;
        Ok(self.calls % 2 == 0)
    }
}

fn emit(output: &mut String, key: &str, value: impl std::fmt::Display) {
    writeln!(output, "{key}={value}").expect("write to string");
}
