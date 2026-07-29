use std::fmt::{Display, Formatter};
use std::io;
use std::sync::{Arc, OnceLock};

use crate::model::{IModelVisitor, ITemplateEnd, ITemplateEvent};
use crate::util::{JavaString, JavaWriter};

use super::{AbstractTemplateEvent, IEngineTemplateEvent, ITemplateHandler};

static TEMPLATE_END_INSTANCE: OnceLock<Arc<TemplateEnd>> = OnceLock::new();

/// 模板处理结束的无内容单例事件。
///
/// 对应 Java: `org.thymeleaf.engine.TemplateEnd`。
pub struct TemplateEnd {
    template_event: AbstractTemplateEvent,
}

impl TemplateEnd {
    /// 返回全局模板结束事件单例。
    ///
    /// 对应 Java: `TemplateEnd.TEMPLATE_END_INSTANCE`。
    #[must_use]
    pub fn instance() -> Arc<Self> {
        Arc::clone(TEMPLATE_END_INSTANCE.get_or_init(|| {
            Arc::new(Self {
                template_event: AbstractTemplateEvent::new(),
            })
        }))
    }
}

impl ITemplateEnd for TemplateEnd {}

impl ITemplateEvent for TemplateEnd {
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
        visitor.visit_template_end(self);
    }

    fn be_handled(
        self: Arc<Self>,
        handler: &mut dyn ITemplateHandler,
    ) -> Result<(), Box<dyn crate::exceptions::TemplateEngineException>> {
        handler.handle_template_end(self)
    }

    fn is_template_end(&self) -> bool {
        true
    }

    fn write(&self, _writer: &mut dyn JavaWriter) -> io::Result<()> {
        Ok(())
    }
}

impl IEngineTemplateEvent for TemplateEnd {}

impl Display for TemplateEnd {
    fn fmt(&self, _formatter: &mut Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}
