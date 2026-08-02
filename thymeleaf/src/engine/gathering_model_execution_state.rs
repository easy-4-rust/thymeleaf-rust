use std::cell::RefCell;
use std::rc::Weak;

use super::model::Model;
use super::processor_execution_vars::ProcessorExecutionVars;
use super::{SkipBody, TemplateModelController};

/// Gathering Model 重放首事件时交给 Processor Handler 的执行快照。
///
/// Java 直接把 `IGatheringModelProcessable` 自身保存到
/// `ProcessorTemplateHandler.currentGatheringModel`。Rust 复制不可变事件身份和
/// Processor 游标，同时保留 ModelController 弱引用，避免对同一 `RefCell`
/// 产生嵌套可变借用。
/// 对应 Java 语义：Rust 侧内部类型（Java 无直接对应对象）。
pub struct GatheringModelExecutionState {
    inner_model: Model,
    processor_execution_vars: ProcessorExecutionVars,
    model_controller: Weak<RefCell<TemplateModelController>>,
    build_time_skip_body: SkipBody,
    build_time_skip_close_tag: bool,
}

impl GatheringModelExecutionState {
    /// 从收集对象当前状态创建执行快照。
    /// 对应 Java 语义：Rust 侧辅助函数（Java 无直接对应）。
    pub(crate) fn new(
        inner_model: Model,
        processor_execution_vars: ProcessorExecutionVars,
        model_controller: Weak<RefCell<TemplateModelController>>,
        build_time_skip_body: SkipBody,
        build_time_skip_close_tag: bool,
    ) -> Self {
        Self {
            inner_model,
            processor_execution_vars,
            model_controller,
            build_time_skip_body,
            build_time_skip_close_tag,
        }
    }

    /// 返回保持原事件身份与顺序的内部 Model。
    pub(crate) const fn inner_model(&self) -> &Model {
        &self.inner_model
    }

    /// 恢复暂停前的 Processor 游标和结构动作状态。
    /// 对应 Java 语义：Java 接口/超类方法 `initializeProcessorExecutionVars()` 的 Rust 移植（`None` 继承路径）。
    pub(crate) fn initialize_processor_execution_vars(&self) -> ProcessorExecutionVars {
        self.processor_execution_vars.clone_vars()
    }

    /// 恢复收集开始时的 body 与 close-tag skip 标志。
    /// 对应 Java 语义：Java 接口/超类方法 `resetGatheredSkipFlags()` 的 Rust 移植（`None` 继承路径）。
    pub(crate) fn reset_gathered_skip_flags(&self) {
        if let Some(controller) = self.model_controller.upgrade() {
            controller
                .borrow_mut()
                .skip(self.build_time_skip_body, self.build_time_skip_close_tag);
        }
    }
}
