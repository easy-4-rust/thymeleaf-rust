use std::io::{self, Read};
use std::sync::Arc;

use crate::IEngineConfiguration;
use crate::TemplateMode;
use crate::engine::{ITemplateHandler, TemplateHandlerAdapterRawHandler};
use crate::exceptions::TemplateInputException;
use crate::raw::{RawParser, RawParserError, RawReader, RawStringReader};
use crate::templateresource::ITemplateResource;
use crate::util::JavaString;

use crate::templateparser::{ITemplateParser, TemplateParserError};

/// RAW 模式模板 Parser。
///
/// RAW 模式把完整输入作为一个 Text 事件，不允许模板选择器或解耦逻辑。对应 Java:
/// `org.thymeleaf.templateparser.raw.RawTemplateParser`。
pub struct RawTemplateParser {
    parser: RawParser,
}

impl RawTemplateParser {
    /// 创建具有指定 buffer 池大小和单 buffer 大小的 RAW Parser。
    ///
    /// 对应 Java: `RawTemplateParser#RawTemplateParser(int,int)`。
    #[must_use]
    pub fn new(buffer_pool_size: usize, buffer_size: usize) -> Self {
        Self {
            parser: RawParser::new(buffer_pool_size, buffer_size),
        }
    }

    fn parse_resource(
        &self,
        template_name: JavaString,
        resource: Arc<dyn ITemplateResource>,
        line_offset: i32,
        col_offset: i32,
        handler: Box<dyn ITemplateHandler>,
    ) -> Result<(), TemplateParserError> {
        let reader = resource.reader().map_err(|error| {
            TemplateInputException::with_template_and_cause(
                Some("An error happened during template parsing".to_owned()),
                Some(resource.get_description()),
                error,
            )
        })?;
        let mut reader = Utf8RawReader::new(reader);
        let mut adapter = TemplateHandlerAdapterRawHandler::new(
            Some(template_name),
            handler,
            line_offset,
            col_offset,
        );
        self.parser
            .parse_reader(Some(&mut reader), Some(&mut adapter))
            .map_err(|error| raw_error(error, Some(resource.get_description())).into())
    }
}

impl ITemplateParser for RawTemplateParser {
    fn parse_standalone(
        &self,
        _configuration: Arc<dyn IEngineConfiguration>,
        _owner_template: Option<&JavaString>,
        template: &JavaString,
        template_selectors: Option<&[JavaString]>,
        resource: Arc<dyn ITemplateResource>,
        template_mode: TemplateMode,
        use_decoupled_logic: bool,
        handler: Box<dyn ITemplateHandler>,
    ) -> Result<(), TemplateParserError> {
        if template_selectors.is_some_and(|selectors| !selectors.is_empty()) {
            return Err(TemplateParserError::IllegalArgument {
                message:
                    "Template selectors cannot be specified for a template using RAW template mode: \
                 template insertion operations must be always performed on whole template files, \
                 not fragments"
                        .to_owned(),
            });
        }
        if template_mode != TemplateMode::RAW {
            return Err(TemplateParserError::IllegalArgument {
                message: "Template Mode has to be RAW".to_owned(),
            });
        }
        if use_decoupled_logic {
            return Err(TemplateParserError::IllegalArgument {
                message: "Cannot use decoupled logic in template mode RAW".to_owned(),
            });
        }
        self.parse_resource(template.clone(), resource, 0, 0, handler)
    }

    fn parse_string(
        &self,
        _configuration: Arc<dyn IEngineConfiguration>,
        owner_template: &JavaString,
        template: &JavaString,
        line_offset: i32,
        col_offset: i32,
        template_mode: TemplateMode,
        handler: Box<dyn ITemplateHandler>,
    ) -> Result<(), TemplateParserError> {
        if template_mode != TemplateMode::RAW {
            return Err(TemplateParserError::IllegalArgument {
                message: "Template Mode has to be RAW".to_owned(),
            });
        }
        let mut reader = RawStringReader::new(template.clone());
        let mut adapter = TemplateHandlerAdapterRawHandler::new(
            Some(owner_template.clone()),
            handler,
            line_offset,
            col_offset,
        );
        self.parser
            .parse_reader(Some(&mut reader), Some(&mut adapter))
            .map_err(|error| raw_error(error, Some(template.to_string_lossy())).into())
    }
}

struct Utf8RawReader {
    value: JavaString,
    position: usize,
    initialization_error: Option<io::Error>,
    closed: bool,
}

impl Utf8RawReader {
    fn new(mut reader: Box<dyn Read>) -> Self {
        let mut bytes = Vec::new();
        let result = reader.read_to_end(&mut bytes);
        match result {
            Ok(_) => match String::from_utf8(bytes) {
                Ok(value) => Self {
                    value: JavaString::from_rust_str(&value),
                    position: 0,
                    initialization_error: None,
                    closed: false,
                },
                Err(error) => Self {
                    value: JavaString::from_utf16(Vec::new()),
                    position: 0,
                    initialization_error: Some(io::Error::new(io::ErrorKind::InvalidData, error)),
                    closed: false,
                },
            },
            Err(error) => Self {
                value: JavaString::from_utf16(Vec::new()),
                position: 0,
                initialization_error: Some(error),
                closed: false,
            },
        }
    }
}

impl RawReader for Utf8RawReader {
    fn read_utf16(&mut self, buffer: &mut [u16], offset: usize, length: usize) -> io::Result<i32> {
        if let Some(error) = self.initialization_error.take() {
            return Err(error);
        }
        if self.closed {
            return Err(io::Error::other("Stream closed"));
        }
        if self.position >= self.value.len() {
            return Ok(-1);
        }
        let count = length.min(self.value.len() - self.position);
        buffer[offset..offset + count]
            .copy_from_slice(&self.value.as_utf16()[self.position..self.position + count]);
        self.position += count;
        Ok(i32::try_from(count).unwrap_or(i32::MAX))
    }

    fn close(&mut self) -> io::Result<()> {
        self.closed = true;
        Ok(())
    }
}

fn raw_error(error: RawParserError, template_name: Option<String>) -> TemplateInputException {
    match &error {
        RawParserError::Parse(parse_error)
            if parse_error.get_line().is_some() && parse_error.get_col().is_some() =>
        {
            TemplateInputException::with_location_and_cause(
                Some("An error happened during template parsing".to_owned()),
                template_name,
                parse_error.get_line().expect("checked"),
                parse_error.get_col().expect("checked"),
                error,
            )
        }
        _ => TemplateInputException::with_template_and_cause(
            Some("An error happened during template parsing".to_owned()),
            template_name,
            error,
        ),
    }
}
