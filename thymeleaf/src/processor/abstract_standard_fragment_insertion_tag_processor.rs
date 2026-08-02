use std::io;
use std::sync::{Arc, Mutex, RwLock, RwLockReadGuard};

use crate::TemplateMode;
use crate::element::{
    AbstractAttributeTagProcessor, IElementProcessor, IElementTagProcessor,
    IElementTagStructureHandler, MatchingAttributeName, MatchingElementName,
};
use crate::exceptions::{
    TemplateEngineException, TemplateInputException, TemplateProcessingException,
};
use crate::expression::{
    Fragment, FragmentExpression, FragmentParameterMap, FragmentSignatureUtils,
    StandardExpressions, TemplateObject, TemplateValue,
};
use crate::model::{IModel, IProcessableElementTag};
use crate::util::{EscapedAttributeUtils, JavaString, JavaWriter};

use super::{
    IProcessor, StandardAttributeCallback, expression_processing_error, is_empty_or_java_whitespace,
};

/// 解析 Fragment 表达式并执行 insert/include/replace 的抽象 Processor。
///
/// 保留旧式未包裹表达式、包含深度限制、Fragment 签名参数重整、合成参数校验、
/// 跨模板模式嵌套处理、TemplateData 切换及 include 去外壳语义。对应 Java:
/// `org.thymeleaf.standard.processor.AbstractStandardFragmentInsertionTagProcessor`。
pub struct AbstractStandardFragmentInsertionTagProcessor {
    processor: AbstractAttributeTagProcessor<StandardAttributeCallback>,
}

impl AbstractStandardFragmentInsertionTagProcessor {
    /// 最大 Fragment 递归包含深度。
    pub const MAX_FRAGMENT_INCLUSION_DEPTH: usize = 100;
    const FRAGMENT_ATTR_NAME: &'static str = "fragment";

    /// 创建 insert/replace 处理器。
    #[allow(clippy::too_many_arguments)]
    /// 对应 Java 语义：`AbstractStandardFragmentInsertionTagProcessor` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(
        template_mode: TemplateMode,
        dialect_prefix: Option<JavaString>,
        attr_name: JavaString,
        precedence: i32,
        replace_host: bool,
        processor_class_name: &'static str,
    ) -> Result<Self, TemplateProcessingException> {
        Self::with_insert_only_contents(
            template_mode,
            dialect_prefix,
            attr_name,
            precedence,
            replace_host,
            false,
            processor_class_name,
        )
    }

    /// 创建仅插入 Fragment 内容的处理器；该模式只供 deprecated `th:include`。
    #[allow(clippy::too_many_arguments)]
    /// 对应 Java 语义：`AbstractStandardFragmentInsertionTagProcessor` 的 `with_insert_only_contents` 行为（Rust 侧辅助/私有路径）。
    pub fn with_insert_only_contents(
        template_mode: TemplateMode,
        dialect_prefix: Option<JavaString>,
        attr_name: JavaString,
        precedence: i32,
        replace_host: bool,
        insert_only_contents: bool,
        processor_class_name: &'static str,
    ) -> Result<Self, TemplateProcessingException> {
        let callback: StandardAttributeCallback = Box::new(
            move |context, tag, attribute_name, attribute_value, structure_handler| {
                let attribute_value = attribute_value.as_ref();
                if is_empty_or_java_whitespace(attribute_value) {
                    return Err(Box::new(TemplateProcessingException::new(Some(
                        "Fragment specifications cannot be empty".to_owned(),
                    ))));
                }
                let attribute_value = attribute_value.expect("fragment specification was checked");
                let current_depth = context.get_template_stack().len();
                if current_depth >= Self::MAX_FRAGMENT_INCLUSION_DEPTH {
                    return Err(Box::new(TemplateProcessingException::new(Some(format!(
                        "Fragment inclusion depth ({current_depth}) exceeded the allowed maximum of {}. This is most likely caused by recursive template inclusion. Current template: \"{}\". Fragment expression: \"{}\"",
                        Self::MAX_FRAGMENT_INCLUSION_DEPTH,
                        context
                            .get_template_data()
                            .get_template()
                            .map_or_else(|| "null".to_owned(), JavaString::to_string_lossy),
                        attribute_value.to_string_lossy()
                    )))));
                }

                let attribute = tag.get_attribute_by_name(attribute_name).ok_or_else(|| {
                    Box::new(TemplateProcessingException::new(Some(
                        "Fragment insertion attribute is not present in the tag".to_owned(),
                    ))) as Box<dyn TemplateEngineException>
                })?;
                let computed = compute_fragment(context, attribute_value).map_err(|error| {
                    enrich_fragment_error(
                        error,
                        attribute.get_template_name(),
                        attribute.get_line(),
                        attribute.get_col(),
                    )
                })?;
                let fragment_object = match computed {
                    ComputedFragment::Null => {
                        return Err(Box::new(TemplateInputException::new(Some(format!(
                            "Error resolving fragment: \"{}\": template or fragment could not be resolved",
                            attribute_value.to_string_lossy()
                        )))));
                    }
                    ComputedFragment::NoOp => return Ok(()),
                    ComputedFragment::Fragment(fragment) => fragment,
                };
                let fragment = fragment_object
                    .as_any()
                    .downcast_ref::<Fragment>()
                    .ok_or_else(|| invalid_fragment(attribute_value))?;
                let Some(fragment_model) = fragment.get_template_model_arc() else {
                    if replace_host {
                        structure_handler.remove_element();
                    } else {
                        structure_handler.remove_body();
                    }
                    return Ok(());
                };

                let mut fragment_parameters = fragment.get_parameters_arc();
                let mut signature_applied = false;
                if fragment_model.size() > 2 {
                    let first_event = fragment_model.get(1);
                    if let Some(fragment_holder) = first_event.into_processable_element_tag() {
                        let prefix = attribute_name.get_prefix();
                        let fragment_name = JavaString::from_rust_str(Self::FRAGMENT_ATTR_NAME);
                        if fragment_holder
                            .has_attribute_with_prefix(prefix, &fragment_name)
                            .map_err(attribute_error)?
                        {
                            let signature_value = fragment_holder
                                .get_attribute_value_with_prefix(prefix, &fragment_name)
                                .map_err(attribute_error)?;
                            let signature_spec = EscapedAttributeUtils::unescape_attribute(
                                Some(fragment_model.get_template_mode()),
                                signature_value,
                            )
                            .map_err(|error| Box::new(error) as Box<dyn TemplateEngineException>)?;
                            if !is_empty_or_java_whitespace(signature_spec.as_ref()) {
                                let signature = FragmentSignatureUtils::parse_fragment_signature(
                                    Some(context.get_configuration()),
                                    signature_spec.as_ref(),
                                )
                                .map_err(|error| {
                                    expression_processing_error(
                                        "Could not parse Fragment signature",
                                        error,
                                    )
                                })?;
                                fragment_parameters = FragmentSignatureUtils::process_parameters(
                                    Some(signature.as_ref()),
                                    fragment_parameters,
                                    fragment.has_synthetic_parameters(),
                                )
                                .map_err(|error| {
                                    expression_processing_error(
                                        "Could not apply Fragment parameters",
                                        error,
                                    )
                                })?;
                                signature_applied = true;
                            }
                        }
                    }
                }
                if !signature_applied && fragment.has_synthetic_parameters() {
                    return Err(Box::new(TemplateProcessingException::new(Some(format!(
                        "Fragment '{}' specifies synthetic (unnamed) parameters, but the resolved fragment does not match a fragment signature (th:fragment,data-th-fragment) which could apply names to the specified parameters.",
                        attribute_value.to_string_lossy()
                    )))));
                }

                if context.get_template_mode() != fragment_model.get_template_mode() {
                    if insert_only_contents {
                        return Err(Box::new(TemplateProcessingException::new(Some(format!(
                            "Template being processed uses template mode {}, inserted fragment \"{}\" uses template mode {}. Cross-template-mode fragment insertion is not allowed using the {} attribute, which is no longer recommended for use as of Thymeleaf 3.0. Use {{th:insert,data-th-insert}} or {{th:replace,data-th-replace}} instead, which do not remove the container element from the fragment being inserted.",
                            context.get_template_mode(),
                            attribute_value.to_string_lossy(),
                            fragment_model.get_template_mode(),
                            attribute_name
                                .to_java_string()
                                .map_or_else(|_| String::new(), |value| value.to_string_lossy())
                        )))));
                    }
                    if let Some(parameters) = fragment_parameters.as_ref()
                        && !read_parameters(parameters).is_empty()
                    {
                        let engine_context =
                            context.as_engine_context().ok_or_else(|| {
                                Box::new(TemplateProcessingException::new(Some(
                                    "Parameterized fragment insertion is not supported because local variable support is DISABLED."
                                        .to_owned(),
                                ))) as Box<dyn TemplateEngineException>
                            })?;
                        let guard = read_parameters(parameters);
                        engine_context.set_variables(&guard);
                    }
                    let rendered = process_fragment_to_string(context, fragment_model.as_ref())?;
                    if replace_host {
                        structure_handler.replace_with_text(rendered, false);
                    } else {
                        structure_handler.set_body_text(rendered, false);
                    }
                    return Ok(());
                }

                let fragment_template_data =
                    fragment_model.get_template_data_arc().ok_or_else(|| {
                        Box::new(TemplateProcessingException::new(Some(
                            "Resolved Fragment model is not a TemplateModel".to_owned(),
                        ))) as Box<dyn TemplateEngineException>
                    })?;
                structure_handler.set_template_data(Arc::clone(&fragment_template_data));
                if let Some(parameters) = fragment_parameters.as_ref() {
                    let parameters = read_parameters(parameters);
                    for (name, value) in parameters.iter() {
                        let name = name.clone().ok_or_else(|| {
                            Box::new(TemplateProcessingException::new(Some(
                                "Fragment parameter name cannot be null".to_owned(),
                            ))) as Box<dyn TemplateEngineException>
                        })?;
                        structure_handler.set_local_variable(name, value.clone());
                    }
                }

                if insert_only_contents && fragment_template_data.has_template_selectors() {
                    let mut model = fragment_model.clone_model();
                    remove_fragment_envelopes(model.as_mut())?;
                    let model: Arc<dyn IModel> = Arc::from(model);
                    if replace_host {
                        structure_handler.replace_with_model(model, true);
                    } else {
                        structure_handler.set_body_model(model, true);
                    }
                    return Ok(());
                }

                if replace_host {
                    structure_handler.replace_with_model(fragment_model, true);
                } else {
                    structure_handler.set_body_model(fragment_model, true);
                }
                Ok(())
            },
        );
        Ok(Self {
            processor: AbstractAttributeTagProcessor::new(
                Some(template_mode),
                dialect_prefix,
                None,
                false,
                Some(attr_name),
                true,
                precedence,
                true,
                processor_class_name,
                callback,
            )?,
        })
    }

    /// 判断旧式输入是否应包裹为 `~{...}` Fragment 表达式。
    #[must_use]
    /// 对应 Java: `AbstractStandardFragmentInsertionTagProcessor#shouldBeWrappedAsFragmentExpression()`。
    pub fn should_be_wrapped_as_fragment_expression(input: &JavaString) -> bool {
        let input = input.as_utf16();
        if input.len() > 2
            && input[0] == FragmentExpression::SELECTOR
            && input[1] == u16::from(b'{')
        {
            return false;
        }
        let mut bracket_level = 0_i32;
        let mut parameter_level = 0_i32;
        let mut in_literal = false;
        let mut index = 0;
        while index < input.len() {
            let character = input[index];
            if (u16::from(b'a')..=u16::from(b'z')).contains(&character)
                || character == u16::from(b' ')
            {
                index += 1;
                continue;
            }
            if character == u16::from(b'\'') {
                in_literal = !in_literal;
            } else if !in_literal {
                if character == u16::from(b'{') {
                    bracket_level += 1;
                } else if character == u16::from(b'}') {
                    bracket_level -= 1;
                } else if bracket_level == 0 {
                    if character == u16::from(b'(') {
                        parameter_level += 1;
                    } else if character == u16::from(b')') {
                        parameter_level -= 1;
                    } else if character == u16::from(b'=') && parameter_level == 1 {
                        return true;
                    } else if character == FragmentExpression::SELECTOR
                        && input.get(index + 1) == Some(&u16::from(b'{'))
                    {
                        return false;
                    } else if character == u16::from(b':')
                        && input.get(index + 1) == Some(&u16::from(b':'))
                    {
                        return true;
                    }
                }
            }
            index += 1;
        }
        true
    }
}

impl IProcessor for AbstractStandardFragmentInsertionTagProcessor {
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

impl IElementProcessor for AbstractStandardFragmentInsertionTagProcessor {
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

impl IElementTagProcessor for AbstractStandardFragmentInsertionTagProcessor {
    fn process(
        &self,
        context: &dyn crate::context::ITemplateContext,
        tag: &dyn IProcessableElementTag,
        structure_handler: &mut dyn IElementTagStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.processor.process(context, tag, structure_handler)
    }
}

enum ComputedFragment {
    Null,
    NoOp,
    Fragment(Arc<dyn TemplateObject>),
}

fn compute_fragment(
    context: &dyn crate::context::ITemplateContext,
    input: &JavaString,
) -> Result<ComputedFragment, Box<dyn TemplateEngineException>> {
    let parser = StandardExpressions::get_expression_parser(context.get_configuration()).map_err(
        |error| expression_processing_error("Could not obtain Standard Expression parser", error),
    )?;
    let trimmed = java_trim(input);
    if AbstractStandardFragmentInsertionTagProcessor::should_be_wrapped_as_fragment_expression(
        &trimmed,
    ) {
        let mut units = vec![u16::from(b'~'), u16::from(b'{')];
        units.extend_from_slice(trimmed.as_utf16());
        units.push(u16::from(b'}'));
        let wrapped = JavaString::from_utf16(units);
        let expression = parser
            .parse_expression(context, Some(&wrapped))
            .map_err(|error| {
                expression_processing_error("Could not parse wrapped Fragment expression", error)
            })?;
        let fragment_expression = expression.as_fragment_expression().ok_or_else(|| {
            Box::new(TemplateProcessingException::new(Some(
                "Wrapped fragment specification did not parse as FragmentExpression".to_owned(),
            ))) as Box<dyn TemplateEngineException>
        })?;
        let executed =
            FragmentExpression::create_executed_fragment_expression(context, fragment_expression)
                .map_err(|error| {
                expression_processing_error("Could not execute wrapped Fragment expression", error)
            })?;
        if executed.get_fragment_selector_expression_result().is_none()
            && executed.get_fragment_parameters().is_none()
            && let Some(template_name_result) = executed.get_template_name_expression_result_arc()
        {
            match template_name_result.as_ref() {
                TemplateValue::Object(object) if object.as_any().is::<Fragment>() => {
                    return Ok(ComputedFragment::Fragment(Arc::clone(object)));
                }
                TemplateValue::NoOp => return Ok(ComputedFragment::NoOp),
                _ => {}
            }
        }
        return resolved_to_computed(
            FragmentExpression::resolve_executed_fragment_expression(context, &executed, true)
                .map_err(|error| {
                    expression_processing_error(
                        "Could not resolve wrapped Fragment expression",
                        error,
                    )
                })?,
        );
    }

    let expression = parser
        .parse_expression(context, Some(&trimmed))
        .map_err(|error| {
            expression_processing_error("Could not parse Fragment specification", error)
        })?;
    if let Some(fragment_expression) = expression.as_fragment_expression() {
        let executed =
            FragmentExpression::create_executed_fragment_expression(context, fragment_expression)
                .map_err(|error| {
                expression_processing_error("Could not execute Fragment expression", error)
            })?;
        return resolved_to_computed(
            FragmentExpression::resolve_executed_fragment_expression(context, &executed, true)
                .map_err(|error| {
                    expression_processing_error("Could not resolve Fragment expression", error)
                })?,
        );
    }
    let result = expression.execute(context).map_err(|error| {
        expression_processing_error("Could not execute Fragment specification", error)
    })?;
    match result.as_deref() {
        None | Some(TemplateValue::Null) => Ok(ComputedFragment::Null),
        Some(TemplateValue::NoOp) => Ok(ComputedFragment::NoOp),
        Some(TemplateValue::Object(object)) if object.as_any().is::<Fragment>() => {
            Ok(ComputedFragment::Fragment(Arc::clone(object)))
        }
        _ => Err(invalid_fragment(input)),
    }
}

fn resolved_to_computed(
    fragment: Option<Arc<Fragment>>,
) -> Result<ComputedFragment, Box<dyn TemplateEngineException>> {
    Ok(match fragment {
        None => ComputedFragment::Null,
        Some(fragment) => {
            let object: Arc<dyn TemplateObject> = fragment;
            ComputedFragment::Fragment(object)
        }
    })
}

fn process_fragment_to_string(
    context: &dyn crate::context::ITemplateContext,
    fragment_model: &dyn IModel,
) -> Result<JavaString, Box<dyn TemplateEngineException>> {
    let output = Arc::new(Mutex::new(Vec::new()));
    context
        .get_configuration()
        .get_template_manager()
        .process(
            fragment_model,
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

fn remove_fragment_envelopes(
    model: &mut dyn IModel,
) -> Result<(), Box<dyn TemplateEngineException>> {
    let mut model_level = 0_i32;
    for position in (0..model.size()).rev() {
        let event = model.get(position);
        if let Some(close) = event.as_close_element_tag() {
            if close.is_unmatched() {
                continue;
            }
            if model_level <= 0 {
                model.remove(position).map_err(model_error)?;
            }
            model_level += 1;
            continue;
        }
        if event.as_open_element_tag().is_some() {
            model_level -= 1;
            if model_level <= 0 {
                model.remove(position).map_err(model_error)?;
            }
            continue;
        }
        if model_level <= 0 {
            model.remove(position).map_err(model_error)?;
        }
    }
    Ok(())
}

fn model_error(error: crate::model::IModelError) -> Box<dyn TemplateEngineException> {
    Box::new(TemplateProcessingException::with_cause(
        Some("Could not remove Fragment envelope events".to_owned()),
        error,
    ))
}

fn invalid_fragment(input: &JavaString) -> Box<dyn TemplateEngineException> {
    Box::new(TemplateProcessingException::new(Some(format!(
        "Invalid fragment specification: \"{}\": expression does not return a Fragment object",
        input.to_string_lossy()
    ))))
}

fn attribute_error(error: crate::engine::AttributesError) -> Box<dyn TemplateEngineException> {
    Box::new(TemplateProcessingException::with_cause(
        Some("Could not inspect Fragment holder attributes".to_owned()),
        error,
    ))
}

fn java_trim(input: &JavaString) -> JavaString {
    let units = input.as_utf16();
    let start = units
        .iter()
        .position(|unit| *unit > 0x20)
        .unwrap_or(units.len());
    let end = units
        .iter()
        .rposition(|unit| *unit > 0x20)
        .map_or(start, |position| position + 1);
    JavaString::from_utf16(units[start..end].to_vec())
}

fn read_parameters(
    parameters: &RwLock<FragmentParameterMap>,
) -> RwLockReadGuard<'_, FragmentParameterMap> {
    parameters
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn enrich_fragment_error(
    mut error: Box<dyn TemplateEngineException>,
    template_name: Option<&JavaString>,
    line: i32,
    col: i32,
) -> Box<dyn TemplateEngineException> {
    if let Some(processing) = error.as_processing_exception_mut() {
        if !processing.has_template_name() {
            processing.set_template_name(template_name.map(JavaString::to_string_lossy));
        }
        if !processing.has_line_and_col() {
            processing.set_line_and_col(line, col);
        }
    }
    error
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
