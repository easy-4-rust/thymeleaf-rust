use std::error::Error;
use std::sync::Arc;

use crate::util::JavaString;

use super::TemplateValue;

/// Rust 宿主扩展 OGNL 静态成员和构造器访问的运行时合同。
///
/// JVM 版 OGNL 通过 ClassLoader 与反射解析任意应用类型；Rust 没有对应的全局反射
/// 机制，因此应用可显式注册该能力。普通对象属性和实例方法仍由
/// `TemplateObject` 提供，本合同只承接 `@Type@member` 与 `new Type(...)`。
/// 对应 Java 语义：Rust 侧内部类型（Java 无直接对应对象）。
pub trait OgnlRuntime: Send + Sync {
    /// 读取宿主注册类型的静态字段。
    fn read_static_field(
        &self,
        _type_name: &JavaString,
        _member_name: &JavaString,
    ) -> Option<Result<Option<Arc<TemplateValue>>, OgnlRuntimeError>> {
        None
    }

    /// 调用宿主注册类型的静态方法；参数中的 `None` 表示 Java null。
    fn invoke_static_method(
        &self,
        _type_name: &JavaString,
        _method_name: &JavaString,
        _arguments: &[Option<Arc<TemplateValue>>],
    ) -> Option<Result<Option<Arc<TemplateValue>>, OgnlRuntimeError>> {
        None
    }

    /// 调用宿主注册类型的构造器。
    fn construct(
        &self,
        _type_name: &JavaString,
        _arguments: &[Option<Arc<TemplateValue>>],
    ) -> Option<Result<Option<Arc<TemplateValue>>, OgnlRuntimeError>> {
        None
    }

    /// 判断动态值是否为指定 Java 类型的实例。
    ///
    /// 返回 `None` 表示运行时未注册该类型关系，由核心处理内建 Java 值类型；
    /// 返回 `Some` 时保留宿主类型系统的继承/接口判断结果。对应 Java:
    /// `ognl.ASTInstanceof#getValueBody`。
    fn is_instance_of(
        &self,
        _value: &TemplateValue,
        _type_name: &JavaString,
    ) -> Option<Result<bool, OgnlRuntimeError>> {
        None
    }
}

/// OGNL 宿主反射等价访问产生的动态错误。
pub type OgnlRuntimeError = Box<dyn Error + Send + Sync>;

/// 不暴露额外静态类型或构造器的默认 OGNL 运行时。
/// 对应 Java 语义：Rust 侧内部类型（Java 无直接对应对象）。
pub struct NoOpOgnlRuntime;

impl OgnlRuntime for NoOpOgnlRuntime {}
