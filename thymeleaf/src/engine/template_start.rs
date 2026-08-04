use std::fmt::{Display, Formatter};
use std::io;
use std::sync::{Arc, OnceLock};

use crate::model::{IModelVisitor, ITemplateEvent, ITemplateStart};
use crate::util::{TemplateWriter, Utf16String};

use super::{AbstractTemplateEvent, IEngineTemplateEvent, ITemplateHandler};

static TEMPLATE_START_INSTANCE: OnceLock<Arc<TemplateStart>> = OnceLock::new();

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
    pub fn instance() -> Arc<Self> {
        Arc::clone(TEMPLATE_START_INSTANCE.get_or_init(|| {
            Arc::new(Self {
                template_event: AbstractTemplateEvent::new(),
            })
        }))
    }
}

impl ITemplateStart for TemplateStart {}

impl ITemplateEvent for TemplateStart {
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
        visitor.visit_template_start(self);
    }

    fn be_handled(
        self: Arc<Self>,
        handler: &mut dyn ITemplateHandler,
    ) -> Result<(), Box<dyn crate::exceptions::TemplateEngineException>> {
        handler.handle_template_start(self)
    }

    fn is_template_start(&self) -> bool {
        true
    }

    fn write(&self, _writer: &mut dyn TemplateWriter) -> io::Result<()> {
        Ok(())
    }
}

impl IEngineTemplateEvent for TemplateStart {}

impl Display for TemplateStart {
    fn fmt(&self, _formatter: &mut Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}
