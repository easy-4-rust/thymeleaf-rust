use std::fmt::{Display, Formatter};
use std::io;
use std::sync::OnceLock;

use crate::model::{IModelVisitor, ITemplateEvent, ITemplateStart};
use crate::util::{JavaString, JavaWriter};

use super::{AbstractTemplateEvent, IEngineTemplateEvent, ITemplateHandler};

static TEMPLATE_START_INSTANCE: OnceLock<TemplateStart> = OnceLock::new();

/// 模板处理开始的无内容单例事件。
///
/// 对应 Java: `org.thymeleaf.engine.TemplateStart`。
pub struct TemplateStart {
    template_event: AbstractTemplateEvent,
}

impl TemplateStart {
    /// 返回全局模板开始事件单例。
    ///
    /// 对应 Java: `TemplateStart.TEMPLATE_START_INSTANCE`。
    #[must_use]
    pub fn instance() -> &'static Self {
        TEMPLATE_START_INSTANCE.get_or_init(|| Self {
            template_event: AbstractTemplateEvent::new(),
        })
    }
}

impl ITemplateStart for TemplateStart {}

impl ITemplateEvent for TemplateStart {
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
        visitor.visit_template_start(self);
    }

    fn write(&self, _writer: &mut dyn JavaWriter) -> io::Result<()> {
        Ok(())
    }
}

impl IEngineTemplateEvent for TemplateStart {
    fn be_handled(&self, handler: &mut dyn ITemplateHandler) {
        handler.handle_template_start(self);
    }
}

impl Display for TemplateStart {
    fn fmt(&self, _formatter: &mut Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}
