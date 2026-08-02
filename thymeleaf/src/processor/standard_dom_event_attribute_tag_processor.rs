use std::io;
use std::sync::{Arc, Mutex};

use crate::TemplateMode;
use crate::element::{
    AbstractAttributeTagProcessor, IElementProcessor, IElementTagProcessor,
    IElementTagStructureHandler, MatchingAttributeName, MatchingElementName,
};
use crate::engine::EngineEventUtils;
use crate::exceptions::{TemplateEngineException, TemplateProcessingException};
use crate::expression::{StandardExpressionExecutionContext, TemplateValue};
use crate::model::IProcessableElementTag;
use crate::processor::IProcessor;
use crate::util::{EscapedAttributeUtils, JavaString, JavaWriter};

use super::{StandardAttributeCallback, expression_processing_error};

/// 安全处理 HTML DOM 事件属性的 Processor。
///
/// 可解析 Standard Expression 使用 `RESTRICTED_FORBID_UNSAFE_EXP_RESULTS`；普通
/// JavaScript 片段以 JAVASCRIPT 模式解析和处理后再写回。对应 Java:
/// `org.thymeleaf.standard.processor.StandardDOMEventAttributeTagProcessor`。
pub struct StandardDOMEventAttributeTagProcessor {
    processor: AbstractAttributeTagProcessor<StandardAttributeCallback>,
}

impl StandardDOMEventAttributeTagProcessor {
    /// Java precedence。
    pub const PRECEDENCE: i32 = 1000;
    /// StandardDialect 注册的 DOM 事件属性全集。
    pub const ATTR_NAMES: &'static [&'static str] = &[
        "onabort",
        "onafterprint",
        "onbeforeprint",
        "onbeforeunload",
        "onblur",
        "oncanplay",
        "oncanplaythrough",
        "onchange",
        "onclick",
        "oncontextmenu",
        "ondblclick",
        "ondrag",
        "ondragend",
        "ondragenter",
        "ondragleave",
        "ondragover",
        "ondragstart",
        "ondrop",
        "ondurationchange",
        "onemptied",
        "onended",
        "onerror",
        "onfocus",
        "onformchange",
        "onforminput",
        "onhashchange",
        "oninput",
        "oninvalid",
        "onkeydown",
        "onkeypress",
        "onkeyup",
        "onload",
        "onloadeddata",
        "onloadedmetadata",
        "onloadstart",
        "onmessage",
        "onmousedown",
        "onmousemove",
        "onmouseout",
        "onmouseover",
        "onmouseup",
        "onmousewheel",
        "onoffline",
        "ononline",
        "onpause",
        "onplay",
        "onplaying",
        "onpopstate",
        "onprogress",
        "onratechange",
        "onreadystatechange",
        "onredo",
        "onreset",
        "onresize",
        "onscroll",
        "onseeked",
        "onseeking",
        "onselect",
        "onshow",
        "onstalled",
        "onstorage",
        "onsubmit",
        "onsuspend",
        "ontimeupdate",
        "onundo",
        "onunload",
        "onvolumechange",
        "onwaiting",
    ];

    /// 创建指定 DOM 事件属性 Processor。
    pub fn new(
        dialect_prefix: Option<JavaString>,
        attr_name: JavaString,
    ) -> Result<Self, TemplateProcessingException> {
        let target_name = attr_name.clone();
        let callback: StandardAttributeCallback = Box::new(
            move |context, tag, attribute_name, attribute_value, structure_handler| {
                let expression_result = if let Some(attribute_value) = attribute_value.as_ref() {
                    match EngineEventUtils::compute_attribute_expression(
                        context,
                        tag,
                        attribute_name,
                        attribute_value,
                    ) {
                        Ok(expression) => expression
                            .execute_with_context(
                                context,
                                StandardExpressionExecutionContext::RESTRICTED_FORBID_UNSAFE_EXP_RESULTS,
                            )
                            .map_err(|error| {
                                expression_processing_error(
                                    "Could not execute DOM event expression",
                                    error,
                                )
                            })?,
                        Err(_) => Some(Arc::new(TemplateValue::String(Arc::new(
                            process_javascript_fragment(
                                context,
                                tag,
                                attribute_name,
                                attribute_value,
                            )?,
                        )))),
                    }
                } else {
                    None
                };
                if expression_result
                    .as_deref()
                    .is_some_and(|value| matches!(value, TemplateValue::NoOp))
                {
                    structure_handler.remove_attribute_with_prefix(
                        attribute_name.get_prefix().cloned(),
                        attribute_name.get_attribute_name().clone(),
                    );
                    return Ok(());
                }
                let raw = expression_result
                    .as_deref()
                    .and_then(TemplateValue::to_java_string);
                let escaped =
                    EscapedAttributeUtils::escape_attribute(Some(TemplateMode::HTML), raw.as_ref())
                        .map_err(|error| Box::new(error) as Box<dyn TemplateEngineException>)?;
                if escaped.as_ref().is_none_or(JavaString::is_empty) {
                    structure_handler.remove_attribute(target_name.clone());
                    structure_handler.remove_attribute_with_prefix(
                        attribute_name.get_prefix().cloned(),
                        attribute_name.get_attribute_name().clone(),
                    );
                } else {
                    structure_handler.set_attribute(target_name.clone(), escaped, None);
                    structure_handler.remove_attribute_with_prefix(
                        attribute_name.get_prefix().cloned(),
                        attribute_name.get_attribute_name().clone(),
                    );
                }
                Ok(())
            },
        );
        Ok(Self {
            processor: AbstractAttributeTagProcessor::new(
                Some(TemplateMode::HTML),
                dialect_prefix,
                None,
                false,
                Some(attr_name),
                true,
                Self::PRECEDENCE,
                false,
                "org.thymeleaf.standard.processor.StandardDOMEventAttributeTagProcessor",
                callback,
            )?,
        })
    }
}

impl IProcessor for StandardDOMEventAttributeTagProcessor {
    fn as_element_processor(&self) -> Option<&dyn IElementProcessor> {
        Some(self)
    }

    fn java_class_name(&self) -> &'static str {
        self.processor.java_class_name()
    }
    fn get_template_mode(&self) -> Option<TemplateMode> {
        self.processor.get_template_mode()
    }
    fn get_precedence(&self) -> i32 {
        self.processor.get_precedence()
    }
}

impl IElementProcessor for StandardDOMEventAttributeTagProcessor {
    fn as_element_tag_processor(&self) -> Option<&dyn IElementTagProcessor> {
        Some(self)
    }
    fn get_matching_element_name(&self) -> Option<&MatchingElementName> {
        self.processor.get_matching_element_name()
    }
    fn get_matching_attribute_name(&self) -> Option<&MatchingAttributeName> {
        self.processor.get_matching_attribute_name()
    }
}

impl IElementTagProcessor for StandardDOMEventAttributeTagProcessor {
    fn process(
        &self,
        context: &dyn crate::context::ITemplateContext,
        tag: &dyn IProcessableElementTag,
        structure_handler: &mut dyn IElementTagStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.processor.process(context, tag, structure_handler)
    }
}

fn process_javascript_fragment(
    context: &dyn crate::context::ITemplateContext,
    tag: &dyn IProcessableElementTag,
    attribute_name: &crate::engine::AttributeName,
    attribute_value: &JavaString,
) -> Result<JavaString, Box<dyn TemplateEngineException>> {
    let attribute = tag.get_attribute_by_name(attribute_name).ok_or_else(|| {
        Box::new(TemplateProcessingException::new(Some(
            "DOM event attribute is not present in the tag".to_owned(),
        ))) as Box<dyn TemplateEngineException>
    })?;
    let owner = context.get_template_data();
    let model = context
        .get_configuration()
        .get_template_manager()
        .parse_string(
            owner.as_ref(),
            attribute_value,
            attribute.get_line(),
            attribute.get_col(),
            Some(TemplateMode::JAVASCRIPT),
            true,
        )
        .map_err(|error| {
            Box::new(TemplateProcessingException::with_cause(
                Some("Could not parse DOM event JavaScript fragment".to_owned()),
                error,
            )) as Box<dyn TemplateEngineException>
        })?;
    let output = Arc::new(Mutex::new(Vec::new()));
    context
        .get_configuration()
        .get_template_manager()
        .process(
            model.as_ref(),
            context,
            Box::new(SharedWriter {
                output: Arc::clone(&output),
            }),
        )
        .map_err(|error| Box::new(error) as Box<dyn TemplateEngineException>)?;
    let units = output
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    Ok(JavaString::from_utf16(units))
}

struct SharedWriter {
    output: Arc<Mutex<Vec<u16>>>,
}

impl JavaWriter for SharedWriter {
    fn write_utf16(&mut self, characters: &[u16]) -> io::Result<()> {
        self.output
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .extend_from_slice(characters);
        Ok(())
    }
}
