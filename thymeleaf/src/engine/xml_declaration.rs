use std::fmt::{Display, Formatter};
use std::io;
use std::sync::Arc;

use crate::model::{IModelVisitor, ITemplateEvent, IXMLDeclaration};
use crate::util::{JavaString, JavaWriter};

use super::{AbstractTemplateEvent, IEngineTemplateEvent, ITemplateHandler};

const DEFAULT_KEYWORD: &str = "xml";
const DEFAULT_VERSION: &str = "1.0";
const ATTRIBUTE_NAME_VERSION: &str = "version";
const ATTRIBUTE_NAME_ENCODING: &str = "encoding";
const ATTRIBUTE_NAME_STANDALONE: &str = "standalone";

/// 引擎内部的不可变 XML declaration 事件。
///
/// 对应 Java: `org.thymeleaf.engine.XMLDeclaration`。
pub struct XMLDeclaration {
    template_event: AbstractTemplateEvent,
    keyword: Option<JavaString>,
    version: Option<JavaString>,
    encoding: Option<JavaString>,
    standalone: Option<JavaString>,
    xml_declaration: JavaString,
}

impl XMLDeclaration {
    /// 使用默认 `xml`/`1.0` 与指定编码创建声明。
    ///
    /// 对应 Java: `XMLDeclaration#XMLDeclaration(String)`。
    #[must_use]
    pub fn new(encoding: Option<JavaString>) -> Self {
        Self::with_components(
            Some(JavaString::from_rust_str(DEFAULT_KEYWORD)),
            Some(JavaString::from_rust_str(DEFAULT_VERSION)),
            encoding,
            None,
        )
    }

    /// 从 keyword、version、encoding 和 standalone 计算声明。
    ///
    /// 对应 Java:
    /// `XMLDeclaration#XMLDeclaration(String,String,String,String)`。
    #[must_use]
    pub fn with_components(
        keyword: Option<JavaString>,
        version: Option<JavaString>,
        encoding: Option<JavaString>,
        standalone: Option<JavaString>,
    ) -> Self {
        let xml_declaration = compute_xml_declaration(
            keyword.as_ref(),
            version.as_ref(),
            encoding.as_ref(),
            standalone.as_ref(),
        );
        Self {
            template_event: AbstractTemplateEvent::new(),
            keyword,
            version,
            encoding,
            standalone,
            xml_declaration,
        }
    }

    /// 使用 parser 完整文本、分解字段和位置创建声明。
    ///
    /// 对应 Java:
    /// `XMLDeclaration#XMLDeclaration(String,String,String,String,String,String,int,int)`。
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn with_location(
        xml_declaration: Option<JavaString>,
        keyword: Option<JavaString>,
        version: Option<JavaString>,
        encoding: Option<JavaString>,
        standalone: Option<JavaString>,
        template_name: Option<JavaString>,
        line: i32,
        col: i32,
    ) -> Self {
        let xml_declaration = xml_declaration.unwrap_or_else(|| {
            compute_xml_declaration(
                keyword.as_ref(),
                version.as_ref(),
                encoding.as_ref(),
                standalone.as_ref(),
            )
        });
        Self {
            template_event: AbstractTemplateEvent::with_location(template_name, line, col),
            keyword,
            version,
            encoding,
            standalone,
            xml_declaration,
        }
    }
}

impl IXMLDeclaration for XMLDeclaration {
    fn get_keyword(&self) -> Option<&JavaString> {
        self.keyword.as_ref()
    }

    fn get_version(&self) -> Option<&JavaString> {
        self.version.as_ref()
    }

    fn get_encoding(&self) -> Option<&JavaString> {
        self.encoding.as_ref()
    }

    fn get_standalone(&self) -> Option<&JavaString> {
        self.standalone.as_ref()
    }

    fn get_xml_declaration(&self) -> Option<&JavaString> {
        Some(&self.xml_declaration)
    }
}

impl ITemplateEvent for XMLDeclaration {
    fn has_location(&self) -> bool {
        self.template_event.has_location()
    }

    fn get_template_name(&self) -> Option<&JavaString> {
        self.template_event.get_template_name()
    }

    fn get_line(&self) -> i32 {
        self.template_event.get_line()
    }

    fn get_col(&self) -> i32 {
        self.template_event.get_col()
    }

    fn accept(&self, visitor: &mut dyn IModelVisitor) {
        visitor.visit_xml_declaration(self);
    }

    fn be_handled(
        self: Arc<Self>,
        handler: &mut dyn ITemplateHandler,
    ) -> Result<(), Box<dyn crate::exceptions::TemplateEngineException>> {
        handler.handle_xml_declaration(self)
    }

    fn write(&self, writer: &mut dyn JavaWriter) -> io::Result<()> {
        writer.write_utf16(self.xml_declaration.as_utf16())
    }
}

impl IEngineTemplateEvent for XMLDeclaration {}

impl Display for XMLDeclaration {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.xml_declaration.to_string_lossy())
    }
}

fn compute_xml_declaration(
    keyword: Option<&JavaString>,
    version: Option<&JavaString>,
    encoding: Option<&JavaString>,
    standalone: Option<&JavaString>,
) -> JavaString {
    let mut result = Vec::with_capacity(40);
    result.extend("<?".encode_utf16());
    append_nullable(&mut result, keyword);
    append_attribute(&mut result, ATTRIBUTE_NAME_VERSION, version);
    append_attribute(&mut result, ATTRIBUTE_NAME_ENCODING, encoding);
    append_attribute(&mut result, ATTRIBUTE_NAME_STANDALONE, standalone);
    result.extend("?>".encode_utf16());
    JavaString::from_utf16(result)
}

fn append_attribute(result: &mut Vec<u16>, name: &str, value: Option<&JavaString>) {
    let Some(value) = value else {
        return;
    };
    result.push(u16::from(b' '));
    result.extend(name.encode_utf16());
    result.extend("=\"".encode_utf16());
    result.extend_from_slice(value.as_utf16());
    result.push(u16::from(b'"'));
}

fn append_nullable(result: &mut Vec<u16>, value: Option<&JavaString>) {
    match value {
        Some(value) => result.extend_from_slice(value.as_utf16()),
        None => result.extend("null".encode_utf16()),
    }
}
