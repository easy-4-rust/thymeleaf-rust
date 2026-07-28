/// Thymeleaf 的通用对象工具。
///
/// 对应 Java: `org.thymeleaf.util.ObjectUtils`。
///
/// 本对象无状态，只保留 Java `nullSafe` 的选择和对象身份语义。Rust 使用 `Option`
/// 精确区分 Java null，并通过移动值而非克隆值保证返回的仍是被选中的原对象。
pub struct ObjectUtils;

impl ObjectUtils {
    /// 目标对象非 null 时返回目标，否则返回默认值。
    ///
    /// 对应 Java: `ObjectUtils#nullSafe(Object, Object)`。
    ///
    /// # 参数
    /// - `target`：首选对象；`None` 对应 Java null；
    /// - `default_value`：target 为 null 时返回的对象，也允许为 `None`。
    ///
    /// # 返回
    /// target 非空时原样返回 target；否则原样返回 default_value。
    #[must_use]
    pub fn null_safe<T>(target: Option<T>, default_value: Option<T>) -> Option<T> {
        target.or(default_value)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::ObjectUtils;

    #[test]
    fn selects_target_or_nullable_default_without_cloning() {
        let target = Rc::new("target".to_owned());
        let default_value = Rc::new("default".to_owned());
        let selected =
            ObjectUtils::null_safe(Some(Rc::clone(&target)), Some(Rc::clone(&default_value)))
                .expect("target");
        assert!(Rc::ptr_eq(&selected, &target));
        assert!(!Rc::ptr_eq(&selected, &default_value));

        let selected_default =
            ObjectUtils::null_safe(None, Some(Rc::clone(&default_value))).expect("default");
        assert!(Rc::ptr_eq(&selected_default, &default_value));
        assert_eq!(ObjectUtils::null_safe::<String>(None, None), None);
    }
}
