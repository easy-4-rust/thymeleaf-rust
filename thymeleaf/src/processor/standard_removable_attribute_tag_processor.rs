use crate::TemplateMode;
use crate::util::Utf16String;

use super::{
    AbstractStandardAttributeModifierTagProcessor, delegate_standard_element_tag_processor,
};

/// 处理空表达式结果时应删除的标准 HTML 属性 Processor。
/// 对应 Java: `org.thymeleaf.standard.processor.StandardRemovableAttributeTagProcessor`。
pub struct StandardRemovableAttributeTagProcessor {
    processor: AbstractStandardAttributeModifierTagProcessor,
}

impl StandardRemovableAttributeTagProcessor {
    /// Java precedence。
    pub const PRECEDENCE: i32 = 1000;
    /// StandardDialect 注册的可移除 HTML 属性全集。
    pub const ATTR_NAMES: &'static [&'static str] = &[
        "abbr",
        "accept",
        "accept-charset",
        "accesskey",
        "align",
        "alt",
        "archive",
        "audio",
        "autocomplete",
        "axis",
        "background",
        "bgcolor",
        "border",
        "cellpadding",
        "cellspacing",
        "challenge",
        "charset",
        "cite",
        "class",
        "classid",
        "codebase",
        "codetype",
        "cols",
        "colspan",
        "compact",
        "content",
        "contenteditable",
        "contextmenu",
        "data",
        "datetime",
        "dir",
        "draggable",
        "dropzone",
        "enctype",
        "for",
        "form",
        "formaction",
        "formenctype",
        "formmethod",
        "formtarget",
        "frame",
        "frameborder",
        "headers",
        "height",
        "high",
        "hreflang",
        "hspace",
        "http-equiv",
        "icon",
        "id",
        "keytype",
        "kind",
        "label",
        "lang",
        "list",
        "longdesc",
        "low",
        "manifest",
        "marginheight",
        "marginwidth",
        "max",
        "maxlength",
        "media",
        "min",
        "minlength",
        "optimum",
        "pattern",
        "placeholder",
        "poster",
        "preload",
        "radiogroup",
        "rel",
        "rev",
        "rows",
        "rowspan",
        "rules",
        "sandbox",
        "scheme",
        "scope",
        "scrolling",
        "size",
        "sizes",
        "span",
        "spellcheck",
        "standby",
        "style",
        "srclang",
        "start",
        "step",
        "summary",
        "tabindex",
        "target",
        "title",
        "usemap",
        "valuetype",
        "vspace",
        "width",
        "wrap",
    ];

    /// 创建指定属性 Processor。
    /// 对应 Java 语义：`StandardRemovableAttributeTagProcessor` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(
        dialect_prefix: Option<Utf16String>,
        attr_name: Utf16String,
    ) -> Result<Self, crate::exceptions::TemplateProcessingException> {
        Ok(Self {
            processor: AbstractStandardAttributeModifierTagProcessor::new(
                TemplateMode::HTML,
                dialect_prefix,
                attr_name,
                Self::PRECEDENCE,
                true,
                false,
                "org.thymeleaf.standard.processor.StandardRemovableAttributeTagProcessor",
            )?,
        })
    }
}

delegate_standard_element_tag_processor!(StandardRemovableAttributeTagProcessor, processor);
