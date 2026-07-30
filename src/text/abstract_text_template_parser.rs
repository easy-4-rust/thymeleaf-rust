use std::io::Read;
use std::sync::Arc;

use crate::engine::{ITemplateHandler, TemplateHandlerAdapterTextHandler};
use crate::exceptions::TemplateInputException;
use crate::reader::{ParserLevelCommentTextReader, PrototypeOnlyCommentTextReader};
use crate::templateparser::{ITemplateParser, TemplateParserError};
use crate::templateresource::ITemplateResource;
use crate::util::JavaString;
use crate::{IEngineConfiguration, TemplateMode};

use super::{
    ITextHandler, InlinedOutputExpressionTextHandler, TextParseException, TextParser,
    TextParserReader, TextParserReaderError,
};

/// TEXT、JAVASCRIPT 与 CSS 模式共享的高层模板 Parser。
///
/// 本对象负责参数约束、资源读取、parser-level/prototype-only 注释 Reader 链、
/// Parser→Engine 事件适配以及 `TextParseException` 到 `TemplateInputException`
/// 的位置和原因转换。
///
/// 对应 Java: `org.thymeleaf.templateparser.text.AbstractTextTemplateParser`。
pub struct AbstractTextTemplateParser {
    parser: TextParser,
}

impl AbstractTextTemplateParser {
    /// 创建文本模式高层 Parser。
    ///
    /// 对应 Java:
    /// `AbstractTextTemplateParser#AbstractTextTemplateParser(int,int,boolean,boolean)`。
    #[must_use]
    pub(crate) fn new(
        buffer_pool_size: i32,
        buffer_size: i32,
        process_comments_and_literals: bool,
        standard_dialect_present: bool,
    ) -> Self {
        Self {
            parser: TextParser::new(
                buffer_pool_size,
                buffer_size,
                process_comments_and_literals,
                standard_dialect_present,
            ),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn parse_internal(
        &self,
        configuration: Arc<dyn IEngineConfiguration>,
        owner_template: Option<&JavaString>,
        template: &JavaString,
        resource: Option<Arc<dyn ITemplateResource>>,
        line_offset: i32,
        col_offset: i32,
        template_mode: TemplateMode,
        handler: Box<dyn ITemplateHandler>,
    ) -> Result<(), TemplateParserError> {
        let template_name = if resource.is_some() {
            template.clone()
        } else {
            owner_template
                .expect("parseString validates owner template")
                .clone()
        };
        let description = resource.as_ref().map_or_else(
            || template.to_string_lossy(),
            |value| value.get_description(),
        );

        let reader: Box<dyn TextParserReader> = if let Some(resource) = resource {
            Box::new(Utf8TextParserReader::from_reader(
                resource.reader().map_err(|error| {
                    TemplateInputException::with_template_and_cause(
                        Some("An error happened during template parsing".to_owned()),
                        Some(description.clone()),
                        error,
                    )
                })?,
            )?)
        } else {
            Box::new(Utf8TextParserReader::from_java_string(template))
        };
        let reader: Box<dyn TextParserReader> = if template_mode == TemplateMode::TEXT {
            Box::new(ParserLevelCommentTextReader::new(reader))
        } else {
            Box::new(ParserLevelCommentTextReader::new(Box::new(
                PrototypeOnlyCommentTextReader::new(reader),
            )))
        };

        let adapter = TemplateHandlerAdapterTextHandler::new(
            Some(template_name),
            handler,
            configuration.clone(),
            template_mode,
            line_offset,
            col_offset,
        );
        let mut parser_handler: Box<dyn ITextHandler> = Box::new(adapter);
        if is_model_reshapeable(configuration.as_ref(), template_mode) {
            parser_handler = Box::new(
                InlinedOutputExpressionTextHandler::new(
                    configuration.as_ref(),
                    template_mode,
                    configuration.get_standard_dialect_prefix(),
                    parser_handler,
                )
                .map_err(|error| TemplateInputException::new(Some(error.to_string())))?,
            );
        }
        self.parser
            .parse_reader(Some(reader), Some(parser_handler))
            .map_err(|error| text_parse_error(*error, description).into())
    }
}

fn is_model_reshapeable(
    configuration: &dyn IEngineConfiguration,
    template_mode: TemplateMode,
) -> bool {
    configuration.is_standard_dialect_present()
        && configuration.get_text_processors(template_mode).len() <= 1
        && (!template_mode.is_markup()
            || (configuration.get_comment_processors(template_mode).len()
                <= if template_mode == TemplateMode::HTML {
                    2
                } else {
                    1
                }
                && configuration
                    .get_cdata_section_processors(template_mode)
                    .len()
                    <= 1))
        && configuration.get_pre_processors(template_mode).is_empty()
        && configuration.get_post_processors(template_mode).is_empty()
}

impl ITemplateParser for AbstractTextTemplateParser {
    fn parse_standalone(
        &self,
        configuration: Arc<dyn IEngineConfiguration>,
        owner_template: Option<&JavaString>,
        template: &JavaString,
        template_selectors: Option<&[JavaString]>,
        resource: Arc<dyn ITemplateResource>,
        template_mode: TemplateMode,
        use_decoupled_logic: bool,
        handler: Box<dyn ITemplateHandler>,
    ) -> Result<(), TemplateParserError> {
        if template_selectors.is_some_and(|selectors| !selectors.is_empty()) {
            return Err(TemplateParserError::IllegalArgument {
                message: "Template selectors cannot be specified for a template using a TEXT template mode: template insertion operations must be always performed on whole template files, not fragments".to_owned(),
            });
        }
        if !template_mode.is_text() {
            return Err(TemplateParserError::IllegalArgument {
                message: "Template Mode has to be a text template mode".to_owned(),
            });
        }
        if use_decoupled_logic {
            return Err(TemplateParserError::IllegalArgument {
                message: format!("Cannot use decoupled logic in template mode {template_mode}"),
            });
        }
        self.parse_internal(
            configuration,
            owner_template,
            template,
            Some(resource),
            0,
            0,
            template_mode,
            handler,
        )
    }

    fn parse_string(
        &self,
        configuration: Arc<dyn IEngineConfiguration>,
        owner_template: &JavaString,
        template: &JavaString,
        line_offset: i32,
        col_offset: i32,
        template_mode: TemplateMode,
        handler: Box<dyn ITemplateHandler>,
    ) -> Result<(), TemplateParserError> {
        if !template_mode.is_text() {
            return Err(TemplateParserError::IllegalArgument {
                message: "Template Mode has to be a text template mode".to_owned(),
            });
        }
        self.parse_internal(
            configuration,
            Some(owner_template),
            template,
            None,
            line_offset,
            col_offset,
            template_mode,
            handler,
        )
    }
}

struct Utf8TextParserReader {
    input: Vec<u16>,
    position: usize,
    closed: bool,
}

impl Utf8TextParserReader {
    fn from_reader(mut reader: Box<dyn Read>) -> Result<Self, TemplateParserError> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).map_err(|error| {
            TemplateInputException::with_cause(
                Some("An error happened during template parsing".to_owned()),
                error,
            )
        })?;
        let input = String::from_utf8(bytes).map_err(|error| {
            TemplateInputException::with_cause(
                Some("An error happened during template parsing".to_owned()),
                error,
            )
        })?;
        Ok(Self {
            input: input.encode_utf16().collect(),
            position: 0,
            closed: false,
        })
    }

    fn from_java_string(input: &JavaString) -> Self {
        Self {
            input: input.as_utf16().to_vec(),
            position: 0,
            closed: false,
        }
    }
}

impl TextParserReader for Utf8TextParserReader {
    fn read_range(
        &mut self,
        buffer: &mut [u16],
        offset: i32,
        len: i32,
    ) -> Result<i32, TextParserReaderError> {
        if self.closed {
            return Err(TextParserReaderError::io("Stream closed"));
        }
        if len == 0 {
            return Ok(0);
        }
        if self.position >= self.input.len() {
            return Ok(-1);
        }
        let offset = usize::try_from(offset)
            .map_err(|_| TextParserReaderError::io("Invalid destination offset"))?;
        let len = usize::try_from(len)
            .map_err(|_| TextParserReaderError::io("Invalid destination length"))?;
        let copied = len.min(self.input.len() - self.position);
        let end = offset
            .checked_add(copied)
            .filter(|end| *end <= buffer.len())
            .ok_or_else(|| TextParserReaderError::io("Invalid destination range"))?;
        buffer[offset..end].copy_from_slice(&self.input[self.position..self.position + copied]);
        self.position += copied;
        Ok(i32::try_from(copied).unwrap_or(i32::MAX))
    }

    fn close(&mut self) -> Result<(), TextParserReaderError> {
        self.closed = true;
        Ok(())
    }
}

fn text_parse_error(error: TextParseException, description: String) -> TemplateInputException {
    if let (Some(line), Some(col)) = (error.get_line(), error.get_col()) {
        TemplateInputException::with_location_and_cause(
            Some("An error happened during template parsing".to_owned()),
            Some(description),
            line,
            col,
            error,
        )
    } else {
        TemplateInputException::with_template_and_cause(
            Some("An error happened during template parsing".to_owned()),
            Some(description),
            error,
        )
    }
}
