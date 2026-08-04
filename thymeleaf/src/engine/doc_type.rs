use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io;
use std::sync::Arc;

use crate::model::{IDocType, IModelVisitor, ITemplateEvent};
use crate::util::{JavaWriter, Utf16String};

use super::{AbstractTemplateEvent, IEngineTemplateEvent, ITemplateHandler};

const DEFAULT_KEYWORD: &str = "DOCTYPE";
const DEFAULT_ELEMENT_NAME: &str = "html";
const DEFAULT_TYPE_PUBLIC: &str = "PUBLIC";
const DEFAULT_TYPE_SYSTEM: &str = "SYSTEM";

/// DOCTYPE 字段组合错误。
///
/// 对应 Java: `DocType#computeType(String,String)` 抛出的
/// `IllegalArgumentException`。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DocTypeError;

impl DocTypeError {
    /// 返回对应 Java 异常全限定名。
    #[must_use]
    pub const fn java_class_name(&self) -> &'static str {
        "java.lang.IllegalArgumentException"
    }
}

impl Display for DocTypeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("DOCTYPE clause cannot have a non-null PUBLIC ID and a null SYSTEM ID")
    }
}

impl Error for DocTypeError {}

/// 引擎内部的不可变 DOCTYPE 事件。
///
/// 对应 Java: `org.thymeleaf.engine.DocType`。
pub struct DocType {
    template_event: AbstractTemplateEvent,
    keyword: Option<Utf16String>,
    element_name: Option<Utf16String>,
    doc_type_kind: Option<Utf16String>,
    public_id: Option<Utf16String>,
    system_id: Option<Utf16String>,
    internal_subset: Option<Utf16String>,
    doc_type: Utf16String,
}

impl DocType {
    /// 创建默认 HTML DOCTYPE。
    ///
    /// 对应 Java: `DocType#DocType()`。
    pub fn new() -> Result<Self, DocTypeError> {
        Self::with_ids(None, None)
    }

    /// 使用默认 keyword/element name 与可空 PUBLIC/SYSTEM ID 创建事件。
    ///
    /// 对应 Java: `DocType#DocType(String,String)`。
    pub fn with_ids(
        public_id: Option<Utf16String>,
        system_id: Option<Utf16String>,
    ) -> Result<Self, DocTypeError> {
        Self::with_components(
            Some(Utf16String::from_rust_str(DEFAULT_KEYWORD)),
            Some(Utf16String::from_rust_str(DEFAULT_ELEMENT_NAME)),
            public_id,
            system_id,
            None,
        )
    }

    /// 从全部结构字段计算 DOCTYPE。
    ///
    /// 对应 Java:
    /// `DocType#DocType(String,String,String,String,String)`。
    pub fn with_components(
        keyword: Option<Utf16String>,
        element_name: Option<Utf16String>,
        public_id: Option<Utf16String>,
        system_id: Option<Utf16String>,
        internal_subset: Option<Utf16String>,
    ) -> Result<Self, DocTypeError> {
        let doc_type_kind = compute_type(public_id.as_ref(), system_id.as_ref())?;
        let doc_type = compute_doc_type(
            keyword.as_ref(),
            element_name.as_ref(),
            doc_type_kind.as_ref(),
            public_id.as_ref(),
            system_id.as_ref(),
            internal_subset.as_ref(),
        );
        Ok(Self {
            template_event: AbstractTemplateEvent::new(),
            keyword,
            element_name,
            doc_type_kind,
            public_id,
            system_id,
            internal_subset,
            doc_type,
        })
    }

    /// 使用 parser 完整文本、分解字段和位置创建事件。
    ///
    /// 对应 Java:
    /// `DocType#DocType(String,String,String,String,String,String,String,int,int)`。
    #[allow(clippy::too_many_arguments)]
    pub fn with_location(
        doc_type: Option<Utf16String>,
        keyword: Option<Utf16String>,
        element_name: Option<Utf16String>,
        public_id: Option<Utf16String>,
        system_id: Option<Utf16String>,
        internal_subset: Option<Utf16String>,
        template_name: Option<Utf16String>,
        line: i32,
        col: i32,
    ) -> Result<Self, DocTypeError> {
        let doc_type_kind = compute_type(public_id.as_ref(), system_id.as_ref())?;
        let doc_type = doc_type.unwrap_or_else(|| {
            compute_doc_type(
                keyword.as_ref(),
                element_name.as_ref(),
                doc_type_kind.as_ref(),
                public_id.as_ref(),
                system_id.as_ref(),
                internal_subset.as_ref(),
            )
        });
        Ok(Self {
            template_event: AbstractTemplateEvent::with_location(template_name, line, col),
            keyword,
            element_name,
            doc_type_kind,
            public_id,
            system_id,
            internal_subset,
            doc_type,
        })
    }
}

impl IDocType for DocType {
    fn get_keyword(&self) -> Option<&Utf16String> {
        self.keyword.as_ref()
    }

    fn get_element_name(&self) -> Option<&Utf16String> {
        self.element_name.as_ref()
    }

    fn get_type(&self) -> Option<&Utf16String> {
        self.doc_type_kind.as_ref()
    }

    fn get_public_id(&self) -> Option<&Utf16String> {
        self.public_id.as_ref()
    }

    fn get_system_id(&self) -> Option<&Utf16String> {
        self.system_id.as_ref()
    }

    fn get_internal_subset(&self) -> Option<&Utf16String> {
        self.internal_subset.as_ref()
    }

    fn get_doc_type(&self) -> Option<&Utf16String> {
        Some(&self.doc_type)
    }
}

impl ITemplateEvent for DocType {
    fn has_location(&self) -> bool {
        self.template_event.has_location()
    }

    fn get_template_name(&self) -> Option<&Utf16String> {
        self.template_event.get_template_name()
    }

    fn get_line(&self) -> i32 {
        self.template_event.get_line()
    }

    fn get_col(&self) -> i32 {
        self.template_event.get_col()
    }

    fn accept(&self, visitor: &mut dyn IModelVisitor) {
        visitor.visit_doc_type(self);
    }

    fn be_handled(
        self: Arc<Self>,
        handler: &mut dyn ITemplateHandler,
    ) -> Result<(), Box<dyn crate::exceptions::TemplateEngineException>> {
        handler.handle_doc_type(self)
    }

    fn write(&self, writer: &mut dyn JavaWriter) -> io::Result<()> {
        writer.write_utf16(self.doc_type.as_utf16())
    }
}

impl IEngineTemplateEvent for DocType {}

impl Display for DocType {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.doc_type.to_string_lossy())
    }
}

fn compute_type(
    public_id: Option<&Utf16String>,
    system_id: Option<&Utf16String>,
) -> Result<Option<Utf16String>, DocTypeError> {
    if public_id.is_some() && system_id.is_none() {
        return Err(DocTypeError);
    }
    if public_id.is_none() && system_id.is_none() {
        return Ok(None);
    }
    Ok(Some(Utf16String::from_rust_str(if public_id.is_some() {
        DEFAULT_TYPE_PUBLIC
    } else {
        DEFAULT_TYPE_SYSTEM
    })))
}

fn compute_doc_type(
    keyword: Option<&Utf16String>,
    element_name: Option<&Utf16String>,
    doc_type_kind: Option<&Utf16String>,
    public_id: Option<&Utf16String>,
    system_id: Option<&Utf16String>,
    internal_subset: Option<&Utf16String>,
) -> Utf16String {
    let mut result = Vec::with_capacity(120);
    result.extend("<!".encode_utf16());
    append_nullable(&mut result, keyword);
    result.push(u16::from(b' '));
    append_nullable(&mut result, element_name);
    if let Some(doc_type_kind) = doc_type_kind {
        result.push(u16::from(b' '));
        result.extend_from_slice(doc_type_kind.as_utf16());
        if let Some(public_id) = public_id {
            result.extend(" \"".encode_utf16());
            result.extend_from_slice(public_id.as_utf16());
            result.push(u16::from(b'"'));
        }
        result.extend(" \"".encode_utf16());
        append_nullable(&mut result, system_id);
        result.push(u16::from(b'"'));
    }
    if let Some(internal_subset) = internal_subset {
        result.extend(" [".encode_utf16());
        result.extend_from_slice(internal_subset.as_utf16());
        result.push(u16::from(b']'));
    }
    result.push(u16::from(b'>'));
    Utf16String::from_utf16(result)
}

fn append_nullable(result: &mut Vec<u16>, value: Option<&Utf16String>) {
    match value {
        Some(value) => result.extend_from_slice(value.as_utf16()),
        None => result.extend("null".encode_utf16()),
    }
}
