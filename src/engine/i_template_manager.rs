use crate::context::{IContext, ITemplateContext};
use crate::model::IModel;
use crate::util::{JavaString, JavaWriter};
use crate::{IThrottledTemplateProcessor, TemplateMode, TemplateSpec};

use super::TemplateData;

/// TemplateManager 对配置层公开的解析与处理合同。
///
/// 对应 Java: `org.thymeleaf.engine.TemplateManager` 的可调用方法集合。
pub trait ITemplateManager {
    /// 清理解析缓存和解耦逻辑缓存。
    fn clear_caches(&self);
    /// 清理指定模板的缓存项。
    fn clear_caches_for(&self, template: &JavaString);
    /// 独立解析模板。
    fn parse_standalone(
        &self,
        context: &dyn ITemplateContext,
        template: &JavaString,
        template_selectors: Option<&[JavaString]>,
        template_mode: Option<TemplateMode>,
        use_cache: bool,
        fail_if_not_exists: bool,
    ) -> Result<Option<Box<dyn IModel>>, crate::exceptions::TemplateInputException>;
    /// 在已知 owner 数据下解析字符串片段。
    fn parse_string(
        &self,
        owner_template_data: &TemplateData,
        template: &JavaString,
        line_offset: i32,
        col_offset: i32,
        template_mode: TemplateMode,
        use_cache: bool,
    ) -> Result<Box<dyn IModel>, crate::exceptions::TemplateInputException>;
    /// 处理已解析模型。
    fn process(
        &self,
        template: &dyn IModel,
        context: &dyn ITemplateContext,
        writer: &mut dyn JavaWriter,
    ) -> Result<(), crate::exceptions::TemplateProcessingException>;
    /// 解析并同步处理 TemplateSpec。
    fn parse_and_process(
        &self,
        template_spec: &TemplateSpec,
        context: &dyn IContext,
        writer: &mut dyn JavaWriter,
    ) -> Result<(), crate::exceptions::TemplateProcessingException>;
    /// 解析并创建节流处理器。
    fn parse_and_process_throttled(
        &self,
        template_spec: &TemplateSpec,
        context: &dyn IContext,
    ) -> Result<Box<dyn IThrottledTemplateProcessor>, crate::exceptions::TemplateProcessingException>;
}
