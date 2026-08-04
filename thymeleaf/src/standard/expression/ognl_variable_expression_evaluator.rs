//! 占位 stub：对应 Java `standard/expression/OGNLVariableExpressionEvaluator.java`，当前未移植或评估为 N/A。
//!
//! 当前状态：未移植。理由见模块顶部列表。
//!
//! 修复：调用方已使用等价替代路径（见 layout_approvals.json 与各模块 doc 注释）。

#[allow(dead_code)]
struct ognl_variable_expression_evaluator;

#[cfg(test)]
mod tests {
    use super::ognl_variable_expression_evaluator;
    #[test]
    fn placeholder_compiles() {
        // 存在性桩——保证目录 1:1 与 Java 上游镜像
        let _ = ognl_variable_expression_evaluator;
    }
}
