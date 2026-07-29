/// 可由 Thymeleaf 引擎增量推进的内部对象契约。
///
/// `process()` 每次调用执行下一段可用工作，并以布尔值报告本次推进结果。接口不
/// 规定幂等性、线程安全性或固定返回值，因此 Rust 使用 `&mut self` 保留实现按
/// 调用次序更新内部状态的能力。
///
/// 对应 Java: `org.thymeleaf.engine.IEngineProcessable`。
pub trait IEngineProcessable {
    /// 执行一次增量处理。
    ///
    /// 对应 Java: `IEngineProcessable#process()`。
    ///
    /// # 返回
    /// 由具体引擎对象定义的本次处理结果。
    fn process(&mut self) -> bool;
}

#[cfg(test)]
mod tests {
    use super::IEngineProcessable;

    struct AlternatingProcessable {
        calls: usize,
    }

    impl IEngineProcessable for AlternatingProcessable {
        fn process(&mut self) -> bool {
            self.calls += 1;
            self.calls % 2 == 0
        }
    }

    #[test]
    fn supports_stateful_dynamic_dispatch_without_thread_safety_constraints() {
        let mut processable = AlternatingProcessable { calls: 0 };
        let dynamic: &mut dyn IEngineProcessable = &mut processable;
        assert!(!dynamic.process());
        assert!(dynamic.process());
        assert!(!dynamic.process());
        assert!(dynamic.process());
        assert_eq!(processable.calls, 4);
    }
}
