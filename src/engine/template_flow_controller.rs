/// 模板处理流水线的内部流控状态。
///
/// 两个标志均以 `false` 初始化，并由同一 `engine` 模块内的处理器直接更新。Java
/// 类型、构造器和字段均为 package-private，Rust 因此不公开构造器或字段，也不为
/// 测试额外增加 getter/setter。
///
/// 对应 Java: `org.thymeleaf.engine.TemplateFlowController`。
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "直接消费者随 engine handler 对象族在后续切片迁移")
)]
pub(crate) struct TemplateFlowController {
    pub(crate) stop_processing: bool,
    pub(crate) processor_template_handler_pending: bool,
}

impl TemplateFlowController {
    /// 创建两个流控标志均为 `false` 的状态。
    ///
    /// 对应 Java: `TemplateFlowController#TemplateFlowController()`。
    ///
    /// # 返回
    /// 独立的新流控对象。
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "直接消费者随 engine handler 对象族在后续切片迁移")
    )]
    pub(crate) const fn new() -> Self {
        Self {
            stop_processing: false,
            processor_template_handler_pending: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Write;

    use super::TemplateFlowController;

    const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
    const JAVA_GOLDEN: &str =
        include_str!("../../tests/fixtures/template_flow_controller_golden.txt");

    #[test]
    fn defaults_and_independent_package_state_match_java_golden() {
        let mut output = String::new();
        emit(&mut output, "baseline", JAVA_BASELINE);

        let mut first = TemplateFlowController::new();
        let second = TemplateFlowController::new();
        emit(
            &mut output,
            "default.first",
            format!(
                "{},{}",
                first.stop_processing, first.processor_template_handler_pending
            ),
        );
        emit(
            &mut output,
            "default.second",
            format!(
                "{},{}",
                second.stop_processing, second.processor_template_handler_pending
            ),
        );

        first.stop_processing = true;
        emit(
            &mut output,
            "mutate.stop",
            format!(
                "{},{}",
                first.stop_processing, first.processor_template_handler_pending
            ),
        );
        first.processor_template_handler_pending = true;
        emit(
            &mut output,
            "mutate.pending",
            format!(
                "{},{}",
                first.stop_processing, first.processor_template_handler_pending
            ),
        );
        emit(
            &mut output,
            "independent.second",
            format!(
                "{},{}",
                second.stop_processing, second.processor_template_handler_pending
            ),
        );

        assert_eq!(output, JAVA_GOLDEN);
    }

    fn emit(output: &mut String, key: &str, value: impl std::fmt::Display) {
        writeln!(output, "{key}={value}").expect("write to string");
    }
}
