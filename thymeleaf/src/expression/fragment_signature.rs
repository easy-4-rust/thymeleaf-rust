use std::sync::{Arc, RwLock, RwLockReadGuard};

use crate::util::{Utf16String, ValidateError};

/// `th:fragment` 的名称与可选参数名称序列。
///
/// 对应 Java: `org.thymeleaf.standard.expression.FragmentSignature`。
pub struct FragmentSignature {
    fragment_name: Utf16String,
    parameter_names: Option<Arc<RwLock<Vec<Option<Utf16String>>>>>,
}

impl FragmentSignature {
    /// 创建签名；参数列表保留原始共享身份，与 Java 直接保存 List 引用一致。
    /// 对应 Java 语义：`FragmentSignature` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(
        fragment_name: Option<Utf16String>,
        parameter_names: Option<Arc<RwLock<Vec<Option<Utf16String>>>>>,
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
    /// 对应 Java: `FragmentSignature#getFragmentName()`。
    pub fn get_fragment_name(&self) -> &Utf16String {
        &self.fragment_name
    }

    /// 判断当前共享参数列表非 null 且非空。
    /// 对应 Java: `FragmentSignature#hasParameters()`。
    pub fn has_parameters(&self) -> bool {
        self.parameter_names
            .as_ref()
            .is_some_and(|parameters| !read_recovering_poison(parameters).is_empty())
    }

    /// 返回原共享参数列表的实时只读视图。
    /// 对应 Java: `FragmentSignature#getParameterNames()`。
    pub fn get_parameter_names(&self) -> Option<RwLockReadGuard<'_, Vec<Option<Utf16String>>>> {
        self.parameter_names
            .as_ref()
            .map(|parameters| read_recovering_poison(parameters))
    }

    /// 返回与 Java `StringUtils.join` 一致的当前签名文本。
    /// 对应 Java: `FragmentSignature#getStringRepresentation()`。
    pub fn get_string_representation(&self) -> Utf16String {
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
        Utf16String::from_utf16(units)
    }
}

fn read_recovering_poison<T>(lock: &RwLock<T>) -> RwLockReadGuard<'_, T> {
    lock.read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
