use std::io::Read;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::sync::Arc;

use html5gum::emitters::default::{DefaultEmitter, StartTag, Token};
use html5gum::{State, Tokenizer};
use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::decoupled::{
    DecoupledInjectedAttribute, DecoupledTemplateLogicMarkupHandler, DecoupledTemplateLogicUtils,
};
use crate::engine::{ITemplateHandler, TemplateHandlerAdapterMarkupHandler};
use crate::exceptions::TemplateInputException;
use crate::inline::StandardInlineModeParseError;
use crate::reader::{ParserLevelCommentMarkupReader, PrototypeOnlyCommentMarkupReader};
use crate::templateparser::{ITemplateParser, TemplateParserError};
use crate::templateresource::ITemplateResource;
use crate::text::{TextParserReader, TextParserReaderError};
use crate::util::{ContentTypeUtils, Utf16String};
use crate::{IEngineConfiguration, TemplateMode};

#[cfg(feature = "dtd-validation")]
use crate::dtd::{DtdValidator, ValidationPolicy, Validator, ValidityError};

use super::markup_selector::{
    MarkupSelectorEngine, SelectorNode, SelectorNodeSummary, SelectorNodeType,
};
use super::{InlinedOutputExpressionMarkupHandler, TemplateFragmentMarkupReferenceResolver};

/// 单个模板允许的最大字节数（结构防御，文档化偏离：Java 无上限）。
///
/// 64MB 覆盖全部合法模板场景；超限输入按解析失败返回，避免病态输入把整个
/// 模板一次性读入内存后进入 tokenizer。
const MAX_TEMPLATE_SIZE: usize = 64 * 1024 * 1024;

/// tokenizer 连续产出同一结束位置的 token 数上限（进度守卫阈值）。
///
/// 合法 HTML 输入下 token 结束位置严格前进；连续 32 个不前进的 token 说明
/// tokenizer 卡死（第三方库状态机缺陷），按解析失败中止而不是无限循环。
const MAX_STALLED_TOKENS: u32 = 32;

/// HTML/XML 高层模板 parser 的公共实现。
///
/// HTML 使用 WHATWG tokenizer 识别容错标记边界，再由本对象执行 Thymeleaf/AttoParser
/// 可观察的 void、隐式闭合与未匹配闭合事件语义；XML 使用严格流式 parser 并额外维护
/// 元素栈。两种模式都从原始输入 span 重建事件，避免第三方 parser 规范化属性、实体或
/// 空白后丢失模板源码形态。
///
/// 对应 Java: `org.thymeleaf.templateparser.markup.AbstractMarkupTemplateParser`。
pub struct AbstractMarkupTemplateParser {
    html: bool,
    _buffer_pool_size: i32,
    _buffer_size: i32,
}

impl AbstractMarkupTemplateParser {
    /// 创建指定模式的标记 parser。
    ///
    /// buffer 参数保留 Java 构造合同；Rust tokenizer 自主管理缓冲区。
    #[must_use]
    pub(crate) const fn new(html: bool, buffer_pool_size: i32, buffer_size: i32) -> Self {
        Self {
            html,
            _buffer_pool_size: buffer_pool_size,
            _buffer_size: buffer_size,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn parse_internal(
        &self,
        configuration: Arc<dyn IEngineConfiguration>,
        owner_template: Option<&Utf16String>,
        template: &Utf16String,
        template_selectors: Option<&[Utf16String]>,
        resource: Option<Arc<dyn ITemplateResource>>,
        line_offset: i32,
        col_offset: i32,
        template_mode: TemplateMode,
        use_decoupled_logic: bool,
        handler: Box<dyn ITemplateHandler>,
    ) -> Result<(), TemplateParserError> {
        self.validate_mode(template_mode)?;
        let template_name = if resource.is_some() {
            template.clone()
        } else {
            owner_template
                .cloned()
                .expect("parseString validates ownerTemplate")
        };
        let description = resource.as_ref().map_or_else(
            || template.to_string_lossy(),
            |value| value.get_description(),
        );
        let decoupled_logic = if use_decoupled_logic {
            let resource = resource
                .as_ref()
                .expect("only standalone parsing enables decoupled logic");
            DecoupledTemplateLogicUtils::compute_decoupled_template_logic(
                configuration.as_ref(),
                owner_template,
                template,
                template_selectors,
                resource.as_ref(),
                template_mode,
            )?
        } else {
            None
        };
        let input = if let Some(resource) = resource {
            let reader = resource.reader().map_err(|error| {
                TemplateInputException::with_template_and_cause(
                    Some("An error happened during template parsing".to_owned()),
                    Some(description.clone()),
                    error,
                )
            })?;
            read_utf8(reader, &description)?
        } else {
            template.to_string_lossy()
        };
        if input.len() > MAX_TEMPLATE_SIZE {
            return Err(TemplateInputException::new(Some(format!(
                "Template exceeds the maximum allowed size of {} bytes ({} bytes found)",
                MAX_TEMPLATE_SIZE,
                input.len()
            )))
            .into());
        }
        let input = preprocess_markup(input, &description)?;

        let needs_reference_resolver = decoupled_logic
            .as_ref()
            .is_some_and(|logic| logic.has_injected_attributes())
            || template_selectors.is_some_and(|selectors| !selectors.is_empty());
        let reference_resolver = if needs_reference_resolver {
            configuration.get_standard_dialect_prefix().map(|prefix| {
                TemplateFragmentMarkupReferenceResolver::for_prefix(self.html, Some(prefix))
            })
        } else {
            None
        };
        let decoupled_selector_engine = decoupled_logic
            .as_ref()
            .filter(|logic| logic.has_injected_attributes())
            .map(|logic| {
                let selectors = logic
                    .get_all_injected_attribute_selectors()
                    .into_iter()
                    .cloned()
                    .collect::<Vec<_>>();
                MarkupSelectorEngine::new(self.html, &selectors, reference_resolver.clone())
                    .map_err(|message| TemplateParserError::IllegalArgument { message })
            })
            .transpose()?;
        let selection = MarkupSelection {
            decoupled_selector_engine,
            decoupled_handler: decoupled_logic.map(DecoupledTemplateLogicMarkupHandler::new),
            block_selector_engine: template_selectors
                .filter(|selectors| !selectors.is_empty())
                .map(|selectors| {
                    MarkupSelectorEngine::new(self.html, selectors, reference_resolver)
                        .map_err(|message| TemplateParserError::IllegalArgument { message })
                })
                .transpose()?,
        };

        let handler: Box<dyn ITemplateHandler> =
            if is_model_reshapeable(configuration.as_ref(), template_mode) {
                Box::new(
                    InlinedOutputExpressionMarkupHandler::new(
                        configuration.clone(),
                        template_mode,
                        configuration.get_standard_dialect_prefix(),
                        handler,
                    )
                    .map_err(|error| TemplateInputException::new(Some(error.to_string())))?,
                )
            } else {
                handler
            };
        // dtd-validation: clone before move to avoid E0382 use-after-move
        #[cfg(feature = "dtd-validation")]
        let cfg_for_dtd = Arc::clone(&configuration);
        let mut adapter = TemplateHandlerAdapterMarkupHandler::new(
            Some(template_name),
            handler,
            configuration,
            template_mode,
            line_offset,
            col_offset,
        );
        let parse_result = catch_unwind(AssertUnwindSafe(|| {
            adapter.document_start()?;
            if self.html {
                parse_html(&input, &mut adapter, &description, &selection)?;
            } else {
                #[cfg(feature = "dtd-validation")]
                parse_xml(
                    &input,
                    &mut adapter,
                    &description,
                    &selection,
                    cfg_for_dtd.get_dtd_validation_policy(),
                )?;
                #[cfg(not(feature = "dtd-validation"))]
                parse_xml(&input, &mut adapter, &description, &selection)?;
            }
            adapter.document_end()
        }));
        match parse_result {
            Ok(result) => result,
            Err(payload) => {
                let payload = match payload.downcast::<StandardInlineModeParseError>() {
                    Ok(error) => {
                        return Err(TemplateInputException::with_cause(
                            Some(error.to_string()),
                            *error,
                        )
                        .into());
                    }
                    Err(payload) => payload,
                };
                let payload = match payload.downcast::<TemplateInputException>() {
                    Ok(error) => return Err((*error).into()),
                    Err(payload) => payload,
                };
                resume_unwind(payload)
            }
        }
    }

    fn validate_mode(&self, template_mode: TemplateMode) -> Result<(), TemplateParserError> {
        match (self.html, template_mode) {
            (true, TemplateMode::HTML) | (false, TemplateMode::XML) => Ok(()),
            (html, mode) if mode.is_markup() => Err(TemplateParserError::IllegalArgument {
                message: format!(
                    "Parser is configured as {}, but {}-mode template parsing is being requested",
                    if html { "HTML" } else { "XML" },
                    mode
                ),
            }),
            (html, mode) => Err(TemplateParserError::IllegalArgument {
                message: format!(
                    "Parser is configured as {} but an unsupported template mode has been specified: {}",
                    if html { "HTML" } else { "XML" },
                    mode
                ),
            }),
        }
    }
}

fn is_model_reshapeable(
    configuration: &dyn IEngineConfiguration,
    template_mode: TemplateMode,
) -> bool {
    configuration.is_standard_dialect_present()
        && configuration.get_text_processors(template_mode).len() <= 1
        && configuration.get_comment_processors(template_mode).len()
            <= if template_mode == TemplateMode::HTML {
                2
            } else {
                1
            }
        && configuration
            .get_cdata_section_processors(template_mode)
            .len()
            <= 1
        && configuration.get_pre_processors(template_mode).is_empty()
        && configuration.get_post_processors(template_mode).is_empty()
}

impl ITemplateParser for AbstractMarkupTemplateParser {
    fn parse_standalone(
        &self,
        configuration: Arc<dyn IEngineConfiguration>,
        owner_template: Option<&Utf16String>,
        template: &Utf16String,
        template_selectors: Option<&[Utf16String]>,
        resource: Arc<dyn ITemplateResource>,
        template_mode: TemplateMode,
        use_decoupled_logic: bool,
        handler: Box<dyn ITemplateHandler>,
    ) -> Result<(), TemplateParserError> {
        self.parse_internal(
            configuration,
            owner_template,
            template,
            template_selectors,
            Some(resource),
            0,
            0,
            template_mode,
            use_decoupled_logic,
            handler,
        )
    }

    fn parse_string(
        &self,
        configuration: Arc<dyn IEngineConfiguration>,
        owner_template: &Utf16String,
        template: &Utf16String,
        line_offset: i32,
        col_offset: i32,
        template_mode: TemplateMode,
        handler: Box<dyn ITemplateHandler>,
    ) -> Result<(), TemplateParserError> {
        self.parse_internal(
            configuration,
            Some(owner_template),
            template,
            None,
            None,
            line_offset,
            col_offset,
            template_mode,
            false,
            handler,
        )
    }
}

fn parse_html(
    source: &str,
    adapter: &mut TemplateHandlerAdapterMarkupHandler,
    description: &str,
    selection: &MarkupSelection,
) -> Result<(), TemplateParserError> {
    let mut emitter = DefaultEmitter::<usize>::new_with_span();
    // emitter 负责在状态切换时同步文本 span 起点；非脚本 `script[type]` 再覆盖回 Data。
    emitter.naively_switch_states(true);
    let mut tokenizer = Tokenizer::new_with_emitter(source, emitter);
    let mut stack: Vec<MarkupFrame> = Vec::new();
    let mut document_siblings = Vec::new();
    // 进度守卫：token 结束位置必须前进；连续不前进说明 tokenizer 卡死。
    let mut last_token_end = 0usize;
    let mut stalled_tokens = 0_u32;

    while let Some(token) = tokenizer.next() {
        let token =
            token.map_err(|error| parse_error(description, source, 0, error.to_string()))?;
        let token_end = match &token {
            Token::StartTag(tag) => tag.span.end,
            Token::EndTag(tag) => tag.span.end,
            Token::String(text) => text.span.end,
            Token::Comment(comment) => comment.span.end,
            Token::Doctype(doc_type) => doc_type.span.end,
            // 可恢复错误没有 span，按"未前进"计数——卡死时同样会被中止。
            Token::Error(_) => last_token_end,
        };
        if token_end < last_token_end {
            return Err(parse_error(
                description,
                source,
                token_end,
                "Tokenizer produced tokens in non-increasing source order",
            ));
        }
        if token_end == last_token_end {
            stalled_tokens += 1;
            if stalled_tokens > MAX_STALLED_TOKENS {
                return Err(parse_error(
                    description,
                    source,
                    token_end,
                    "Tokenizer failed to make progress",
                ));
            }
        } else {
            stalled_tokens = 0;
        }
        last_token_end = token_end;
        match token {
            Token::StartTag(tag) => {
                let start = tag.span.start;
                let end = tag.span.end;
                if safe_range(source, start, end).starts_with("<?") {
                    let xml_declaration = is_xml_declaration(source, start, end);
                    if should_emit_event(
                        selection,
                        &mut stack,
                        &mut document_siblings,
                        if xml_declaration {
                            SelectorNodeType::XmlDeclaration
                        } else {
                            SelectorNodeType::ProcessingInstruction
                        },
                    ) {
                        if xml_declaration {
                            emit_xml_declaration(source, start, end, adapter)?;
                        } else {
                            emit_processing_instruction(source, start, end, adapter)?;
                        }
                    }
                    continue;
                }
                let (name_start, name_end) =
                    start_tag_name(source, start, end).ok_or_else(|| {
                        parse_error(description, source, start, "Malformed start tag")
                    })?;
                let name = source[name_start..name_end].to_ascii_lowercase();
                auto_close_for_start(
                    &name,
                    source,
                    start,
                    &mut stack,
                    adapter,
                    &mut document_siblings,
                )?;
                let standalone = tag.self_closing || is_html_void(&name);
                let preceding_siblings = Arc::new(stack.last().map_or_else(
                    || document_siblings.clone(),
                    |frame| frame.completed_children.clone(),
                ));
                let (node, injected_attributes, selected_here, content_selected) = selection
                    .prepare_element(
                        true,
                        source,
                        name_start,
                        name_end,
                        tag_content_end(source, start, end),
                        preceding_siblings,
                        stack.iter().map(|frame| frame.node.clone()).collect(),
                    )?;
                let ancestor_emits = stack.last().is_some_and(|frame| frame.emit_descendants);
                let emit_tag =
                    selection.block_selector_engine.is_none() || ancestor_emits || selected_here;
                let emit_descendants = emit_tag || content_selected;
                if emit_tag {
                    adapter.element_start_with_injected(
                        source,
                        start,
                        end,
                        name_start,
                        name_end,
                        standalone,
                        tag.self_closing,
                        false,
                        &injected_attributes,
                    )?;
                }
                if !standalone {
                    stack.push(MarkupFrame {
                        name,
                        node,
                        completed_children: Vec::new(),
                        emit_tag,
                        emit_descendants,
                    });
                } else {
                    finish_node(node.summary(), &mut stack, &mut document_siblings);
                }
                if !standalone && let Some(state) = html_tokenizer_override_state(&tag) {
                    tokenizer.set_state(state);
                }
            }
            Token::EndTag(tag) => {
                let start = tag.span.start;
                let end = tag.span.end;
                let (name_start, name_end) = end_tag_name(source, start, end)
                    .ok_or_else(|| parse_error(description, source, start, "Malformed end tag"))?;
                let name = source[name_start..name_end].to_ascii_lowercase();
                if let Some(index) = stack.iter().rposition(|entry| entry.name == name) {
                    while stack.len() > index + 1 {
                        let frame = stack.pop().expect("stack is not empty");
                        finish_html_frame(
                            source,
                            frame,
                            start,
                            adapter,
                            &mut stack,
                            &mut document_siblings,
                        )?;
                    }
                    let frame = stack.pop().expect("matching frame exists");
                    if frame.emit_tag {
                        adapter
                            .element_end(source, start, end, name_start, name_end, false, false)?;
                    }
                    finish_node(frame.node.summary(), &mut stack, &mut document_siblings);
                } else if selection.block_selector_engine.is_none()
                    || stack.last().is_some_and(|frame| frame.emit_descendants)
                {
                    adapter.element_end(source, start, end, name_start, name_end, false, true)?;
                }
            }
            Token::String(text) => {
                if should_emit_event(
                    selection,
                    &mut stack,
                    &mut document_siblings,
                    SelectorNodeType::Text,
                ) {
                    adapter.text(source, text.span.start, text.span.end)?;
                }
            }
            Token::Comment(comment) => {
                let start = comment.span.start;
                let end = comment.span.end;
                if safe_range(source, start, end).starts_with("<![CDATA[") {
                    let content_start = start + "<![CDATA[".len();
                    let content_end = end.saturating_sub("]]>".len()).max(content_start);
                    if should_emit_event(
                        selection,
                        &mut stack,
                        &mut document_siblings,
                        SelectorNodeType::Cdata,
                    ) {
                        adapter.cdata(source, start, content_start, content_end, end)?;
                    }
                } else if safe_range(source, start, end).starts_with("<?") {
                    let xml_declaration = is_xml_declaration(source, start, end);
                    if should_emit_event(
                        selection,
                        &mut stack,
                        &mut document_siblings,
                        if xml_declaration {
                            SelectorNodeType::XmlDeclaration
                        } else {
                            SelectorNodeType::ProcessingInstruction
                        },
                    ) {
                        if xml_declaration {
                            emit_xml_declaration(source, start, end, adapter)?;
                        } else {
                            emit_processing_instruction(source, start, end, adapter)?;
                        }
                    }
                } else {
                    let content_start = if safe_range(source, start, end).starts_with("<!--") {
                        start + 4
                    } else {
                        start + 2
                    };
                    let content_end = if safe_range(source, start, end).ends_with("-->") {
                        end - 3
                    } else if safe_range(source, start, end).ends_with('>') {
                        end - 1
                    } else {
                        end
                    };
                    if should_emit_event(
                        selection,
                        &mut stack,
                        &mut document_siblings,
                        SelectorNodeType::Comment,
                    ) {
                        adapter.comment(source, start, content_start, content_end, end)?;
                    }
                }
            }
            Token::Doctype(doc_type) => {
                if should_emit_event(
                    selection,
                    &mut stack,
                    &mut document_siblings,
                    SelectorNodeType::DocType,
                ) {
                    emit_doctype(source, doc_type.span.start, doc_type.span.end, adapter)?;
                }
            }
            Token::Error(error) => {
                // HTML tokenizer 错误多数是 WHATWG 可恢复错误。唯一属性等 Thymeleaf
                // 强约束由 adapter 的原始属性扫描器执行，其余错误继续解析。
                let _recoverable_error = error;
            }
        }
    }

    while let Some(frame) = stack.pop() {
        finish_html_frame(
            source,
            frame,
            source.len(),
            adapter,
            &mut stack,
            &mut document_siblings,
        )?;
    }
    Ok(())
}

/// 按 AttoParser 的 HTML 元素内容类型选择 tokenizer 状态。
///
/// 普通 HTML tokenizer 会把所有 `script` 元素当作脚本原始文本；Thymeleaf 还会检查
/// `type`，让 `text/template` 等非脚本内容继续产生可处理的元素事件。
fn html_tokenizer_override_state(tag: &StartTag<usize>) -> Option<State> {
    if tag.name.as_ref().eq_ignore_ascii_case(b"script") {
        let script_type = tag.attributes.iter().find_map(|(name, value)| {
            name.as_ref().eq_ignore_ascii_case(b"type").then(|| {
                String::from_utf8_lossy(value.value.as_ref())
                    .trim()
                    .to_ascii_lowercase()
                    .to_owned()
            })
        });
        return match script_type.as_deref() {
            None
            | Some("")
            | Some("module")
            | Some("importmap")
            | Some("speculationrules")
            | Some("javascript")
            | Some("ecmascript") => None,
            Some(content_type)
                if ContentTypeUtils::is_content_type_java_script(Some(content_type))
                    .unwrap_or(false) =>
            {
                None
            }
            Some(_) => Some(State::Data),
        };
    }
    None
}

fn parse_xml(
    source: &str,
    adapter: &mut TemplateHandlerAdapterMarkupHandler,
    description: &str,
    selection: &MarkupSelection,
    #[cfg(feature = "dtd-validation")] validation_policy: ValidationPolicy,
) -> Result<(), TemplateParserError> {
    let mut reader = Reader::from_str(source);
    reader.config_mut().trim_text(false);
    reader.config_mut().check_end_names = true;
    reader.config_mut().allow_unmatched_ends = false;
    #[cfg(feature = "dtd-validation")]
    // DTD 验证器：Disabled 策略零开销；否则主循环前从源码预扫描 DOCTYPE 的
    // SYSTEM 标识符并解析 DTD（Validator 为有状态 push 接口，须跨事件持有）。
    // 无 DOCTYPE 即无验证义务；有 DOCTYPE 但解析/展开超限失败时
    // Strict 报错、Warn 降级为不验证。
    #[cfg(feature = "dtd-validation")]
    let dtd_holder: Option<DtdValidator> = if validation_policy == ValidationPolicy::Disabled {
        None
    } else if let Some(declaration) = scan_doctype_declaration(source) {
        match DtdValidator::new(&declaration) {
            Some(holder) => Some(holder),
            None if validation_policy == ValidationPolicy::Strict => {
                return Err(parse_error(
                    description,
                    source,
                    0,
                    "Cannot resolve DTD declared by the document DOCTYPE",
                ));
            }
            None => None,
        }
    } else {
        None
    };
    #[cfg(feature = "dtd-validation")]
    let mut dtd_validator: Option<Validator<'_>> =
        dtd_holder.as_ref().map(|holder| holder.validator());
    let mut previous = 0_u64;
    let mut stack: Vec<MarkupFrame> = Vec::new();
    let mut document_siblings = Vec::new();

    loop {
        let event = reader.read_event().map_err(|error| {
            parse_error(
                description,
                source,
                reader.error_position() as usize,
                error.to_string(),
            )
        })?;
        let end = reader.buffer_position() as usize;
        let start = previous as usize;
        previous = reader.buffer_position();
        let empty_event = matches!(&event, Event::Empty(_));
        match event {
            Event::Start(tag) | Event::Empty(tag) => {
                let empty = empty_event;
                let (name_start, name_end) = start_tag_name(source, start, end)
                    .ok_or_else(|| parse_error(description, source, start, "Malformed XML tag"))?;
                let name = source[name_start..name_end].to_owned();
                let preceding_siblings = Arc::new(stack.last().map_or_else(
                    || document_siblings.clone(),
                    |frame| frame.completed_children.clone(),
                ));
                let (node, injected_attributes, selected_here, content_selected) = selection
                    .prepare_element(
                        false,
                        source,
                        name_start,
                        name_end,
                        tag_content_end(source, start, end),
                        preceding_siblings,
                        stack.iter().map(|frame| frame.node.clone()).collect(),
                    )?;
                let ancestor_emits = stack.last().is_some_and(|frame| frame.emit_descendants);
                let emit_tag =
                    selection.block_selector_engine.is_none() || ancestor_emits || selected_here;
                let emit_descendants = emit_tag || content_selected;
                if emit_tag {
                    adapter.element_start_with_injected(
                        source,
                        start,
                        end,
                        name_start,
                        name_end,
                        empty,
                        empty,
                        false,
                        &injected_attributes,
                    )?;
                }
                #[cfg(feature = "dtd-validation")]
                if let Some(validator) = dtd_validator.as_mut() {
                    let attributes = xml_attributes(source, name_end, end);
                    let _defaults = validator.start_element(&name, &attributes);
                    // 自闭合标签（quick-xml Empty 事件）对验证器是立即开合
                    if empty {
                        validator.end_element(&name);
                    }
                    if validation_policy == ValidationPolicy::Strict && validator.has_errors() {
                        return Err(dtd_validation_error(
                            description,
                            source,
                            start,
                            validator.errors(),
                        ));
                    }
                }
                if !empty {
                    stack.push(MarkupFrame {
                        name,
                        node,
                        completed_children: Vec::new(),
                        emit_tag,
                        emit_descendants,
                    });
                } else {
                    finish_node(node.summary(), &mut stack, &mut document_siblings);
                }
                let _ = tag;
            }
            Event::End(_) => {
                let (name_start, name_end) = end_tag_name(source, start, end).ok_or_else(|| {
                    parse_error(description, source, start, "Malformed XML end tag")
                })?;
                let name = &source[name_start..name_end];
                let frame = stack.pop().ok_or_else(|| {
                    parse_error(description, source, start, "Unmatched XML end tag")
                })?;
                if frame.name != name {
                    return Err(parse_error(
                        description,
                        source,
                        start,
                        format!("Expecting </{}> found </{name}>", frame.name),
                    ));
                }
                if frame.emit_tag {
                    adapter.element_end(source, start, end, name_start, name_end, false, false)?;
                }
                #[cfg(feature = "dtd-validation")]
                if let Some(validator) = dtd_validator.as_mut() {
                    validator.end_element(name);
                    if validation_policy == ValidationPolicy::Strict && validator.has_errors() {
                        return Err(dtd_validation_error(
                            description,
                            source,
                            start,
                            validator.errors(),
                        ));
                    }
                }
                finish_node(frame.node.summary(), &mut stack, &mut document_siblings);
            }
            _text_event @ (Event::Text(_) | Event::GeneralRef(_)) => {
                #[cfg(feature = "dtd-validation")]
                if let Some(validator) = dtd_validator.as_mut() {
                    // 通用实体引用按标记处理（EMPTY 内容违反）；字面文本按字符数据校验
                    if matches!(_text_event, Event::GeneralRef(_)) {
                        validator.markup();
                    } else {
                        validator.characters(&source[start..end]);
                    }
                    if validation_policy == ValidationPolicy::Strict && validator.has_errors() {
                        return Err(dtd_validation_error(
                            description,
                            source,
                            start,
                            validator.errors(),
                        ));
                    }
                }
                if should_emit_event(
                    selection,
                    &mut stack,
                    &mut document_siblings,
                    SelectorNodeType::Text,
                ) {
                    adapter.text(source, start, end)?;
                }
            }
            Event::CData(_) => {
                let content_start = start + "<![CDATA[".len();
                let content_end = end.saturating_sub("]]>".len()).max(content_start);
                #[cfg(feature = "dtd-validation")]
                if let Some(validator) = dtd_validator.as_mut() {
                    validator.reference_data(&source[content_start..content_end]);
                    if validation_policy == ValidationPolicy::Strict && validator.has_errors() {
                        return Err(dtd_validation_error(
                            description,
                            source,
                            start,
                            validator.errors(),
                        ));
                    }
                }
                if should_emit_event(
                    selection,
                    &mut stack,
                    &mut document_siblings,
                    SelectorNodeType::Cdata,
                ) {
                    adapter.cdata(source, start, content_start, content_end, end)?;
                }
            }
            Event::Comment(_) => {
                let content_start = start + "<!--".len();
                let content_end = end.saturating_sub("-->".len()).max(content_start);
                #[cfg(feature = "dtd-validation")]
                if let Some(validator) = dtd_validator.as_mut() {
                    validator.markup();
                }
                if should_emit_event(
                    selection,
                    &mut stack,
                    &mut document_siblings,
                    SelectorNodeType::Comment,
                ) {
                    adapter.comment(source, start, content_start, content_end, end)?;
                }
            }
            Event::Decl(_) => {
                if should_emit_event(
                    selection,
                    &mut stack,
                    &mut document_siblings,
                    SelectorNodeType::XmlDeclaration,
                ) {
                    emit_xml_declaration(source, start, end, adapter)?;
                }
            }
            Event::PI(_) => {
                #[cfg(feature = "dtd-validation")]
                if let Some(validator) = dtd_validator.as_mut() {
                    validator.markup();
                }
                if should_emit_event(
                    selection,
                    &mut stack,
                    &mut document_siblings,
                    SelectorNodeType::ProcessingInstruction,
                ) {
                    emit_processing_instruction(source, start, end, adapter)?;
                }
            }
            Event::DocType(_) => {
                if should_emit_event(
                    selection,
                    &mut stack,
                    &mut document_siblings,
                    SelectorNodeType::DocType,
                ) {
                    emit_doctype(source, start, end, adapter)?;
                }
            }
            Event::Eof => break,
        }
    }
    if let Some(unclosed) = stack.pop() {
        return Err(parse_error(
            description,
            source,
            source.len(),
            format!("Element <{}> is not closed", unclosed.name),
        ));
    }
    #[cfg(feature = "dtd-validation")]
    if let Some(validator) = dtd_validator {
        let errors = validator.finish();
        if validation_policy == ValidationPolicy::Strict && !errors.is_empty() {
            return Err(dtd_validation_error(
                description,
                source,
                source.len(),
                &errors,
            ));
        }
    }
    Ok(())
}

fn auto_close_for_start(
    incoming: &str,
    source: &str,
    close_position: usize,
    stack: &mut Vec<MarkupFrame>,
    adapter: &mut TemplateHandlerAdapterMarkupHandler,
    document_siblings: &mut Vec<SelectorNodeSummary>,
) -> Result<(), TemplateParserError> {
    let Some((required, limits)) = html_auto_close_rule(incoming) else {
        return Ok(());
    };
    let mut unstack_count = 0;
    for (depth, frame) in stack.iter().rev().enumerate() {
        if limits.contains(&frame.name.as_str()) {
            break;
        }
        if required.contains(&frame.name.as_str()) {
            unstack_count = depth + 1;
        }
    }
    for _ in 0..unstack_count {
        let frame = stack.pop().expect("stack is not empty");
        finish_html_frame(
            source,
            frame,
            close_position,
            adapter,
            stack,
            document_siblings,
        )?;
    }
    Ok(())
}

struct MarkupSelection {
    decoupled_selector_engine: Option<MarkupSelectorEngine>,
    decoupled_handler: Option<DecoupledTemplateLogicMarkupHandler>,
    block_selector_engine: Option<MarkupSelectorEngine>,
}

impl MarkupSelection {
    #[allow(clippy::too_many_arguments)]
    #[expect(
        clippy::type_complexity,
        reason = "返回元组逐项对应选择器节点、注入属性与两种选择状态"
    )]
    fn prepare_element(
        &self,
        html: bool,
        source: &str,
        name_start: usize,
        name_end: usize,
        content_end: usize,
        preceding_siblings: Arc<Vec<SelectorNodeSummary>>,
        mut ancestor_path: Vec<SelectorNode>,
    ) -> Result<
        (
            SelectorNode,
            Vec<Arc<DecoupledInjectedAttribute>>,
            bool,
            bool,
        ),
        TemplateParserError,
    > {
        let mut node = SelectorNode::from_tag(
            html,
            source,
            name_start,
            name_end,
            content_end,
            preceding_siblings.clone(),
            &[],
        );
        ancestor_path.push(node.clone());
        let selected_decoupled = self
            .decoupled_selector_engine
            .as_ref()
            .map(|engine| engine.matching_element_selectors(&ancestor_path));
        let injected_attributes = self
            .decoupled_handler
            .as_ref()
            .map_or_else(Vec::new, |handler| {
                handler.process_injected_attributes(selected_decoupled.as_deref())
            });

        if !injected_attributes.is_empty() {
            let injected_values = injected_attribute_values(&injected_attributes)?;
            node = SelectorNode::from_tag(
                html,
                source,
                name_start,
                name_end,
                content_end,
                preceding_siblings,
                &injected_values,
            );
            *ancestor_path.last_mut().expect("current node exists") = node.clone();
        }
        let selected_here = self
            .block_selector_engine
            .as_ref()
            .is_some_and(|engine| !engine.matching_element_selectors(&ancestor_path).is_empty());
        let content_selected = self
            .block_selector_engine
            .as_ref()
            .is_some_and(|engine| engine.selects_content_of(&ancestor_path));
        Ok((node, injected_attributes, selected_here, content_selected))
    }
}

fn injected_attribute_values(
    injected_attributes: &[Arc<DecoupledInjectedAttribute>],
) -> Result<Vec<(Utf16String, Option<Utf16String>)>, TemplateParserError> {
    injected_attributes
        .iter()
        .map(|attribute| {
            let name = attribute.get_name().map_err(decoupled_attribute_error)?;
            let (_, _, _, _, operator_len, _, _, _, _) = attribute.parser_parts();
            let value = (operator_len > 0)
                .then(|| attribute.get_value_content())
                .transpose()
                .map_err(decoupled_attribute_error)?;
            Ok((name, value))
        })
        .collect()
}

struct MarkupFrame {
    name: String,
    node: SelectorNode,
    completed_children: Vec<SelectorNodeSummary>,
    emit_tag: bool,
    emit_descendants: bool,
}

fn finish_html_frame(
    source: &str,
    frame: MarkupFrame,
    close_position: usize,
    adapter: &mut TemplateHandlerAdapterMarkupHandler,
    stack: &mut [MarkupFrame],
    document_siblings: &mut Vec<SelectorNodeSummary>,
) -> Result<(), TemplateParserError> {
    if frame.emit_tag {
        adapter.synthetic_element_end(source, close_position, &frame.name)?;
    }
    finish_node(frame.node.summary(), stack, document_siblings);
    Ok(())
}

fn finish_node(
    node: SelectorNodeSummary,
    stack: &mut [MarkupFrame],
    document_siblings: &mut Vec<SelectorNodeSummary>,
) {
    if let Some(parent) = stack.last_mut() {
        parent.completed_children.push(node);
    } else {
        document_siblings.push(node);
    }
}

fn should_emit_event(
    selection: &MarkupSelection,
    stack: &mut [MarkupFrame],
    document_siblings: &mut Vec<SelectorNodeSummary>,
    node_type: super::markup_selector::SelectorNodeType,
) -> bool {
    let preceding_siblings = Arc::new(stack.last().map_or_else(
        || document_siblings.clone(),
        |frame| frame.completed_children.clone(),
    ));
    let selected = selection
        .block_selector_engine
        .as_ref()
        .is_some_and(|engine| {
            let path = stack
                .iter()
                .map(|frame| frame.node.clone())
                .collect::<Vec<_>>();
            !engine
                .matching_event_selectors(&path, node_type, preceding_siblings.clone())
                .is_empty()
        });
    let emit = selection.block_selector_engine.is_none()
        || stack.last().is_some_and(|frame| frame.emit_descendants)
        || selected;
    let summary = SelectorNode::event(node_type, preceding_siblings).summary();
    finish_node(summary, stack, document_siblings);
    emit
}

fn decoupled_attribute_error(
    error: crate::decoupled::DecoupledInjectedAttributeError,
) -> TemplateParserError {
    TemplateInputException::new(Some(error.to_string())).into()
}

fn html_auto_close_rule(
    incoming: &str,
) -> Option<(&'static [&'static str], &'static [&'static str])> {
    const NONE: &[&str] = &[];
    const BODY_BLOCK_REQUIRED: &[&str] = &["p", "head"];
    const BODY_BLOCK_LIMITS: &[&str] = &[
        "script",
        "template",
        "element",
        "decorator",
        "content",
        "shadow",
    ];
    const TABLE_SECTIONS: &[&str] = &[
        "tr", "td", "th", "thead", "tfoot", "tbody", "caption", "colgroup",
    ];
    const TABLE_LIMIT: &[&str] = &["table"];

    match incoming {
        "body" => Some((&["head"], NONE)),
        "article" | "section" | "nav" | "aside" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6"
        | "hgroup" | "header" | "footer" | "address" | "main" | "p" | "hr" | "pre"
        | "blockquote" | "ol" | "ul" | "dl" | "div" | "table" | "form" | "fieldset" | "menu" => {
            Some((BODY_BLOCK_REQUIRED, BODY_BLOCK_LIMITS))
        }
        "li" => Some((&["li"], &["ul", "ol"])),
        "dt" | "dd" => Some((&["dt", "dd"], &["dl"])),
        "rb" | "rtc" => Some((&["rb", "rt", "rtc", "rp"], &["ruby"])),
        "rt" | "rp" => Some((&["rb", "rt", "rp"], &["ruby", "rtc"])),
        "caption" | "colgroup" | "tbody" | "thead" | "tfoot" => Some((TABLE_SECTIONS, TABLE_LIMIT)),
        "col" => Some((
            &["tr", "td", "th", "thead", "tfoot", "tbody", "caption"],
            TABLE_LIMIT,
        )),
        "tr" => Some((
            &["tr", "td", "th", "caption", "colgroup"],
            &["table", "thead", "tbody", "tfoot"],
        )),
        "td" | "th" => Some((&["td", "th"], &["tr"])),
        "optgroup" => Some((&["optgroup", "option"], &["select"])),
        "option" => Some((&["option"], &["select", "optgroup", "datalist"])),
        _ => None,
    }
}

fn is_html_void(name: &str) -> bool {
    matches!(
        name,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "keygen"
            | "link"
            | "menuitem"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

/// 把 tokenizer 派生索引钳制到 UTF-8 字符边界。
///
/// html5gum 对 `<?` 等自定义状态产生的 span 可能落在多字节字符中间（tokenizer
/// 按字节推进、span 单位与 &str 字节偏移不一致）；PI/XML 声明、标签名与注释
/// 定界符按规范均为 ASCII，钳制到边界不改变语义，仅避免 &str 切片 panic。
fn clamp_forward(source: &str, position: usize) -> usize {
    let mut position = position.min(source.len());
    while position < source.len() && !source.is_char_boundary(position) {
        position += 1;
    }
    position
}

fn clamp_backward(source: &str, position: usize) -> usize {
    let mut position = position.min(source.len());
    while position > 0 && !source.is_char_boundary(position) {
        position -= 1;
    }
    position
}

fn safe_range(source: &str, start: usize, end: usize) -> &str {
    &source[clamp_forward(source, start)..clamp_backward(source, end)]
}

fn start_tag_name(source: &str, start: usize, end: usize) -> Option<(usize, usize)> {
    let mut position = start.checked_add(1)?;
    while position < end && source.as_bytes()[position].is_ascii_whitespace() {
        position += 1;
    }
    let name_start = position;
    while position < end {
        let byte = source.as_bytes()[position];
        if byte.is_ascii_whitespace() || matches!(byte, b'/' | b'>') {
            break;
        }
        position += source[position..].chars().next()?.len_utf8();
    }
    (position > name_start).then_some((name_start, position))
}

fn end_tag_name(source: &str, start: usize, end: usize) -> Option<(usize, usize)> {
    let mut position = start.checked_add(2)?;
    while position < end && source.as_bytes()[position].is_ascii_whitespace() {
        position += 1;
    }
    let name_start = position;
    while position < end {
        let byte = source.as_bytes()[position];
        if byte.is_ascii_whitespace() || byte == b'>' {
            break;
        }
        position += source[position..].chars().next()?.len_utf8();
    }
    (position > name_start).then_some((name_start, position))
}

fn tag_content_end(source: &str, start: usize, end: usize) -> usize {
    let mut position = end.saturating_sub(1);
    if position > start && source.as_bytes().get(position) == Some(&b'>') {
        position -= 1;
    }
    if position > start && source.as_bytes().get(position) == Some(&b'/') {
        position -= 1;
    }
    position + 1
}

fn emit_xml_declaration(
    source: &str,
    start: usize,
    end: usize,
    adapter: &mut TemplateHandlerAdapterMarkupHandler,
) -> Result<(), TemplateParserError> {
    let inner = safe_range(source, start + 2, end.saturating_sub(2)).trim();
    let (keyword, rest) = split_name(inner);
    adapter.xml_declaration(
        source,
        start,
        end,
        keyword,
        find_pseudo_attribute(rest, "version"),
        find_pseudo_attribute(rest, "encoding"),
        find_pseudo_attribute(rest, "standalone"),
    )
}

fn is_xml_declaration(source: &str, start: usize, end: usize) -> bool {
    let suffix = if safe_range(source, start, end).ends_with("?>") {
        2
    } else {
        1
    };
    let inner = safe_range(source, start + 2, end.saturating_sub(suffix)).trim_start();
    let (target, _) = split_name(inner);
    target.eq_ignore_ascii_case("xml")
}

fn emit_processing_instruction(
    source: &str,
    start: usize,
    end: usize,
    adapter: &mut TemplateHandlerAdapterMarkupHandler,
) -> Result<(), TemplateParserError> {
    let suffix = if safe_range(source, start, end).ends_with("?>") {
        2
    } else {
        1
    };
    let inner = safe_range(source, start + 2, end.saturating_sub(suffix)).trim();
    let (target, rest) = split_name(inner);
    let content = (!rest.trim().is_empty()).then(|| rest.trim());
    adapter.processing_instruction(source, start, end, target, content)
}

fn emit_doctype(
    source: &str,
    start: usize,
    end: usize,
    adapter: &mut TemplateHandlerAdapterMarkupHandler,
) -> Result<(), TemplateParserError> {
    let inner = safe_range(source, start + 2, end.saturating_sub(1)).trim();
    let (keyword, rest) = split_name(inner);
    let (root, remainder) = split_name(rest.trim_start());
    let upper = remainder.trim_start().to_ascii_uppercase();
    let quoted = quoted_values(remainder);
    let (public_id, system_id) = if upper.starts_with("PUBLIC") {
        (quoted.first().copied(), quoted.get(1).copied())
    } else if upper.starts_with("SYSTEM") {
        (None, quoted.first().copied())
    } else {
        (None, None)
    };
    let internal_subset = remainder
        .find('[')
        .and_then(|index| remainder.rfind(']').map(|last| &remainder[index + 1..last]));
    adapter.doc_type(
        source,
        start,
        end,
        keyword,
        root,
        public_id,
        system_id,
        internal_subset,
    )
}

fn split_name(value: &str) -> (&str, &str) {
    let end = value.find(char::is_whitespace).unwrap_or(value.len());
    (&value[..end], &value[end..])
}

fn find_pseudo_attribute<'a>(source: &'a str, name: &str) -> Option<&'a str> {
    let mut rest = source;
    while !rest.is_empty() {
        rest = rest.trim_start();
        let (candidate, after_name) = split_name_or_equals(rest);
        let after_name = after_name.trim_start();
        if !after_name.starts_with('=') {
            break;
        }
        let after_operator = after_name[1..].trim_start();
        let quote = after_operator.chars().next()?;
        if quote != '\'' && quote != '"' {
            break;
        }
        let value_start = quote.len_utf8();
        let value_end = after_operator[value_start..].find(quote)? + value_start;
        if candidate.eq_ignore_ascii_case(name) {
            return Some(&after_operator[value_start..value_end]);
        }
        rest = &after_operator[value_end + quote.len_utf8()..];
    }
    None
}

/// 预扫描源码定位首个 `<!DOCTYPE ...>` 并返回声明主体
/// （`<!DOCTYPE` 与匹配 `>` 之间的文本，供 `DtdValidator::new` 解析）。
/// 引号内的 `>` 不作为声明结束符；含内部子集 `[` 的声明返回 `None`
/// （跳过验证，与既有"DOCTYPE internal subset 不保留"偏差保持一致）。
#[cfg(feature = "dtd-validation")]
fn scan_doctype_declaration(source: &str) -> Option<String> {
    let marker = "<!DOCTYPE";
    let name_start = source.find(marker)? + marker.len();
    let bytes = source.as_bytes();
    let mut quote: Option<u8> = None;
    let mut cursor = name_start;
    let decl_end = loop {
        if cursor >= bytes.len() {
            return None;
        }
        let byte = bytes[cursor];
        if let Some(active) = quote {
            if byte == active {
                quote = None;
            }
        } else if byte == b'"' || byte == b'\'' {
            quote = Some(byte);
        } else if byte == b'[' {
            // 内部子集不支持：多级 `>` 使简单扫描无法定位声明边界。
            return None;
        } else if byte == b'>' {
            break cursor;
        }
        cursor += 1;
    };
    Some(safe_range(source, name_start, decl_end).trim().to_owned())
}

/// 解析 XML 开始标签内部属性（name="value" / name='value'），供 DTD 验证使用。
#[cfg(feature = "dtd-validation")]
fn xml_attributes(source: &str, name_end: usize, tag_end: usize) -> Vec<(&str, &str)> {
    let mut attributes = Vec::new();
    let mut rest = safe_range(source, name_end, tag_end.saturating_sub(1)).trim_start();
    while !rest.is_empty() {
        let (name, after_name) = split_name_or_equals(rest);
        if name.is_empty() {
            break;
        }
        let after_name = after_name.trim_start();
        if !after_name.starts_with('=') {
            break;
        }
        let after_operator = after_name[1..].trim_start();
        let Some(quote) = after_operator.chars().next() else {
            break;
        };
        if quote != '\'' && quote != '"' {
            break;
        }
        let value_start = quote.len_utf8();
        let Some(relative_end) = after_operator[value_start..].find(quote) else {
            break;
        };
        let value_end = value_start + relative_end;
        attributes.push((name, &after_operator[value_start..value_end]));
        rest = after_operator[value_end + quote.len_utf8()..].trim_start();
    }
    attributes
}

/// 将 DTD 有效性错误归一为模板解析错误（Strict 策略通道）。
#[cfg(feature = "dtd-validation")]
fn dtd_validation_error(
    description: &str,
    source: &str,
    position: usize,
    errors: &[ValidityError],
) -> TemplateParserError {
    let message = errors
        .iter()
        .map(|error| error.to_string())
        .collect::<Vec<_>>()
        .join("; ");
    parse_error(
        description,
        source,
        position,
        format!("DTD validation failed: {message}"),
    )
}

fn split_name_or_equals(value: &str) -> (&str, &str) {
    let end = value
        .find(|character: char| character.is_whitespace() || character == '=')
        .unwrap_or(value.len());
    (&value[..end], &value[end..])
}

fn quoted_values(source: &str) -> Vec<&str> {
    let mut values = Vec::new();
    let mut rest = source;
    while let Some(index) = rest.find(['\'', '"']) {
        let quote = rest.as_bytes()[index] as char;
        let after = &rest[index + 1..];
        let Some(end) = after.find(quote) else {
            break;
        };
        values.push(&after[..end]);
        rest = &after[end + 1..];
    }
    values
}

fn read_utf8(mut reader: Box<dyn Read>, description: &str) -> Result<String, TemplateParserError> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes).map_err(|error| {
        TemplateInputException::with_template_and_cause(
            Some("An error happened during template parsing".to_owned()),
            Some(description.to_owned()),
            error,
        )
    })?;
    String::from_utf8(bytes).map_err(|error| {
        TemplateInputException::with_template_and_cause(
            Some("An error happened during template parsing".to_owned()),
            Some(description.to_owned()),
            error,
        )
        .into()
    })
}

fn preprocess_markup(input: String, description: &str) -> Result<String, TemplateParserError> {
    let base: Box<dyn TextParserReader> = Box::new(StringTextReader::new(input));
    let prototype: Box<dyn TextParserReader> =
        Box::new(PrototypeOnlyCommentMarkupReader::new(base));
    let mut parser_level = ParserLevelCommentMarkupReader::new(prototype);
    let mut output = Vec::new();
    let mut buffer = vec![0_u16; 4096];
    loop {
        let read = parser_level
            .read_range(&mut buffer, 0, 4096)
            .map_err(|error| parser_reader_error(description, error))?;
        if read < 0 {
            break;
        }
        output.extend_from_slice(&buffer[..read as usize]);
    }
    parser_level
        .close()
        .map_err(|error| parser_reader_error(description, error))?;
    String::from_utf16(&output).map_err(|error| {
        TemplateInputException::with_template_and_cause(
            Some("An error happened during template parsing".to_owned()),
            Some(description.to_owned()),
            error,
        )
        .into()
    })
}

fn parser_reader_error(description: &str, error: TextParserReaderError) -> TemplateParserError {
    TemplateInputException::with_template_and_cause(
        Some("An error happened during template parsing".to_owned()),
        Some(description.to_owned()),
        error,
    )
    .into()
}

fn parse_error(
    description: &str,
    source: &str,
    offset: usize,
    message: impl Into<String>,
) -> TemplateParserError {
    let (line, col) = source_location(source, offset.min(source.len()));
    TemplateInputException::with_location(
        Some(format!(
            "An error happened during template parsing: {}",
            message.into()
        )),
        Some(description.to_owned()),
        line,
        col,
    )
    .into()
}

fn source_location(source: &str, offset: usize) -> (i32, i32) {
    let mut line = 1_i32;
    let mut col = 1_i32;
    let mut chars = source[..clamp_backward(source, offset)].chars().peekable();
    while let Some(character) = chars.next() {
        match character {
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                line += 1;
                col = 1;
            }
            '\n' => {
                line += 1;
                col = 1;
            }
            value => col += value.len_utf16() as i32,
        }
    }
    (line, col)
}

struct StringTextReader {
    value: Vec<u16>,
    position: usize,
    closed: bool,
}

impl StringTextReader {
    fn new(value: String) -> Self {
        Self {
            value: value.encode_utf16().collect(),
            position: 0,
            closed: false,
        }
    }
}

impl TextParserReader for StringTextReader {
    fn read_range(
        &mut self,
        buffer: &mut [u16],
        offset: i32,
        len: i32,
    ) -> Result<i32, TextParserReaderError> {
        if self.closed {
            return Err(TextParserReaderError::io("Stream closed"));
        }
        if self.position >= self.value.len() {
            return Ok(-1);
        }
        let offset = usize::try_from(offset).map_err(|_| {
            TextParserReaderError::new(
                "java.lang.IndexOutOfBoundsException",
                Some(Utf16String::from_rust_str("Negative reader offset")),
            )
        })?;
        let len = usize::try_from(len).map_err(|_| {
            TextParserReaderError::new(
                "java.lang.IndexOutOfBoundsException",
                Some(Utf16String::from_rust_str("Negative reader length")),
            )
        })?;
        let copied = len
            .min(self.value.len() - self.position)
            .min(buffer.len().saturating_sub(offset));
        buffer[offset..offset + copied]
            .copy_from_slice(&self.value[self.position..self.position + copied]);
        self.position += copied;
        Ok(copied as i32)
    }

    fn close(&mut self) -> Result<(), TextParserReaderError> {
        self.closed = true;
        Ok(())
    }
}
