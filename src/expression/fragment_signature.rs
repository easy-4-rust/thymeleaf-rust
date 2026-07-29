use std::sync::{Arc, RwLock, RwLockReadGuard};

use crate::util::{JavaString, ValidateError};

/// `th:fragment` 的名称与可选参数名称序列。
///
/// 对应 Java: `org.thymeleaf.standard.expression.FragmentSignature`。
pub struct FragmentSignature {
    fragment_name: JavaString,
    parameter_names: Option<Arc<RwLock<Vec<Option<JavaString>>>>>,
}

impl FragmentSignature {
    /// 创建签名；参数列表保留原始共享身份，与 Java 直接保存 List 引用一致。
    pub fn new(
        fragment_name: Option<JavaString>,
        parameter_names: Option<Arc<RwLock<Vec<Option<JavaString>>>>>,
    ) -> Result<Self, ValidateError> {
        let fragment_name = fragment_name
            .filter(|value| !value.is_empty())
            .ok_or_else(|| ValidateError::IllegalArgument {
                message: Some("Fragment name cannot be null or empty".to_owned()),
            })?;
        Ok(Self {
            fragment_name,
            parameter_names,
        })
    }

    /// 返回 Fragment 名称。
    pub fn get_fragment_name(&self) -> &JavaString {
        &self.fragment_name
    }

    /// 判断当前共享参数列表非 null 且非空。
    pub fn has_parameters(&self) -> bool {
        self.parameter_names
            .as_ref()
            .is_some_and(|parameters| !read_recovering_poison(parameters).is_empty())
    }

    /// 返回原共享参数列表的实时只读视图。
    pub fn get_parameter_names(&self) -> Option<RwLockReadGuard<'_, Vec<Option<JavaString>>>> {
        self.parameter_names
            .as_ref()
            .map(|parameters| read_recovering_poison(parameters))
    }

    /// 返回与 Java `StringUtils.join` 一致的当前签名文本。
    pub fn get_string_representation(&self) -> JavaString {
        let Some(parameter_names) = self.parameter_names.as_ref() else {
            return self.fragment_name.clone();
        };
        let parameters = read_recovering_poison(parameter_names);
        if parameters.is_empty() {
            return self.fragment_name.clone();
        }
        let mut units = self.fragment_name.as_utf16().to_vec();
        units.extend_from_slice(&[b' ' as u16, b'(' as u16]);
        for (index, parameter) in parameters.iter().enumerate() {
            if index != 0 {
                units.push(b',' as u16);
            }
            match parameter {
                Some(parameter) => units.extend_from_slice(parameter.as_utf16()),
                None => units.extend("null".encode_utf16()),
            }
        }
        units.push(b')' as u16);
        JavaString::from_utf16(units)
    }
}

fn read_recovering_poison<T>(lock: &RwLock<T>) -> RwLockReadGuard<'_, T> {
    lock.read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
