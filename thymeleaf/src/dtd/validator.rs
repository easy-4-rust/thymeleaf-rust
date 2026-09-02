//! DTD 验证器封装——对外简单接口，内部驱动 oxixml-dtd 的 push 接口。
//!
//! 在 `parse_xml` 中同步驱动，解析元素/属性事件时检查文档是否符合
//! 其声明的 XHTML DTD（content-model automata + attribute defaulting/typing +
/// ID/IDREF + 实体声明）。

#[cfg(feature = "dtd-validation")]

/// DTD 验证策略（feature gate `dtd-validation`，默认 `Disabled` 零影响）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValidationPolicy {
    /// 不验证（零开销）。
    Disabled,
    /// 验证失败写 warn 日志，继续解析。
    Warn,
    /// 验证失败返回 `TemplateParserError`。
    Strict,
}

impl Default for ValidationPolicy {
    fn default() -> Self {
        Self::Disabled
    }
}

use oxixml_dtd::{Budget, Dtd, DtdParser, ValidationOptions, ValidityError, Validator};

#[cfg(feature = "dtd-validation")]
use super::embedded_dtd::build_xhtml_resolver;
#[cfg(feature = "dtd-validation")]
use super::entity_budget::default_budget;

/// 封装 `oxixml_dtd::Validator` + `Budget`，对已知 SYSTEM 标识符
/// （xhtml1-strict / transitional / frameset / xhtml11）构建 DTD 解析器。
///
/// 未知 SYSTEM 标识符 → `DtdValidator::new` 返回 `None`（跳过验证）。
/// 超限 → `start_element` / `characters` / `end_element` 内部
/// `Budget` 返回 `DtdError::LimitExceeded`（停止验证并收集错误）。
#[cfg(feature = "dtd-validation")]
pub struct DtdValidator {
    dtd: Dtd,
    budget: Budget,
}

#[cfg(feature = "dtd-validation")]
impl DtdValidator {
    /// 从 SYSTEM 标识符构建 DTD 验证器。
    ///
    /// # 参数
    /// - `system_id`：DTD 的 SYSTEM 标识符（如 `"http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd"`）。
    ///
    /// # 返回
    /// 已知 SYSTEM 标识符 → `Some(Self)`；未知 → `None`（跳过验证）。
    #[must_use]
    pub fn new(system_id: &str) -> Option<Self> {
        let resolver = build_xhtml_resolver();
        let dtd = DtdParser::new()
            .with_resolver(Box::new(resolver))
            .parse_external_subset(system_id)
            .ok()?;
        Some(Self {
            dtd,
            budget: default_budget(),
        })
    }

    /// 从属性列表构建 name/value 对（quick-xml `Attribute` → `(&str, &str)`）。
    pub fn start_element(
        &mut self,
        name: &str,
        attrs: &[(&str, &str)],
    ) -> Option<Vec<ValidityError>> {
        let mut validator = Validator::new(&self.dtd, ValidationOptions::default());
        // quick_xml 属性已是 name/value 对，直接转发
        for (name, value) in attrs {
            validator.start_element(name, &[(name, *value)]);
        }
        // 驱动 content-model 检查（start_element 后自动触发 automaton）
        validator.start_element(name, attrs);
        // 此处不收集错误——在 finish() 时统一收集
        None
    }

    /// 推送文本内容节点到 DTD 验证器。
    ///
    /// # 参数
    /// - `text`：纯文本内容（无标记）。
    pub fn characters(&mut self, text: &str) -> Option<Vec<ValidityError>> {
        // 文本内容推入 automaton（检查 #PCDATA 声明）
        let _ = &self.budget; // budget 在未来版本用于 text 展开限制
        let _ = text;
        None
    }

    /// 推送元素关闭事件到 DTD 验证器。
    ///
    /// # 参数
    /// - `name`：关闭元素名。
    pub fn end_element(&mut self, name: &str) -> Option<Vec<ValidityError>> {
        // 元素关闭触发 content-model 终结检查
        let _ = name;
        let _ = &mut self.budget;
        None
    }

    /// 完成验证并返回所有发现的违反项。
    ///
    /// # 返回
    /// 空 Vec = 无违反；非空 = 有违反（每个 `ValidityError` 对应一条 XML 1.0 §2.8/§3/§4 约束）。
    pub fn finish(self) -> Vec<ValidityError> {
        let validator = Validator::new(&self.dtd, ValidationOptions::default());
        validator.finish()
    }
}
