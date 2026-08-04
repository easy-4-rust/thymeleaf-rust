use std::sync::{Arc, RwLock, RwLockReadGuard};

use crate::IEngineConfiguration;
use crate::exceptions::TemplateProcessingException;
use crate::util::{Utf16String, ValidateError};

use super::{
    ExpressionCache, FragmentParameterMap, FragmentSignature, StandardExpressionResult,
    expression_parsing_util::ExpressionParsingUtil,
};

/// Fragment 签名的解析与参数匹配入口。
///
/// 对应 Java: `org.thymeleaf.standard.expression.FragmentSignatureUtils`。
pub struct FragmentSignatureUtils;

impl FragmentSignatureUtils {
    /// 解析并缓存 Fragment 名称及形参名称。
    /// 对应 Java: `FragmentSignatureUtils#parseFragmentSignature()`。
    pub fn parse_fragment_signature(
        configuration: Option<&dyn IEngineConfiguration>,
        input: Option<&Utf16String>,
    ) -> StandardExpressionResult<Arc<FragmentSignature>> {
        let input = input.ok_or_else(|| {
            Box::new(ValidateError::IllegalArgument {
                message: Some("Input cannot be null".to_owned()),
            }) as super::StandardExpressionError
        })?;
        if let Some(cached) = configuration.and_then(|configuration| {
            ExpressionCache::get_fragment_signature_from_cache(configuration, input)
        }) {
            return Ok(cached);
        }
        let parsed = Self::internal_parse_fragment_signature(&trim(input)).ok_or_else(|| {
            Box::new(TemplateProcessingException::new(Some(format!(
                "Could not parse as fragment signature: \"{}\"",
                input.to_string_lossy()
            )))) as super::StandardExpressionError
        })?;
        let parsed = Arc::new(parsed);
        if let Some(configuration) = configuration {
            ExpressionCache::put_fragment_signature_into_cache(
                configuration,
                input,
                Arc::clone(&parsed),
            );
        }
        Ok(parsed)
    }

    /// 执行不访问缓存的 Fragment 签名解析。
    ///
    /// 对应 Java: `FragmentSignatureUtils#internalParseFragmentSignature(String)`。
    pub(crate) fn internal_parse_fragment_signature(
        input: &Utf16String,
    ) -> Option<FragmentSignature> {
        ExpressionParsingUtil::parse_fragment_signature(input)
    }

    /// 返回位置参数的上游合成名称。
    ///
    /// 对应 Java: `FragmentSignatureUtils#getSyntheticParameterNameForIndex(int)`。
    #[must_use]
    pub(crate) fn get_synthetic_parameter_name_for_index(index: usize) -> Utf16String {
        Utf16String::from_rust_str(&format!("_arg{index}"))
    }

    /// 按 Fragment 签名匹配命名或合成位置参数。
    ///
    /// 对应 Java: `FragmentSignatureUtils#processParameters`。命名参数可包含签名之外
    /// 的额外项；位置参数必须与签名参数数量完全一致。
    pub fn process_parameters(
        fragment_signature: Option<&FragmentSignature>,
        specified_parameters: Option<Arc<RwLock<FragmentParameterMap>>>,
        parameters_are_synthetic: bool,
    ) -> StandardExpressionResult<Option<Arc<RwLock<FragmentParameterMap>>>> {
        let fragment_signature = fragment_signature.ok_or_else(|| {
            Box::new(ValidateError::IllegalArgument {
                message: Some("Fragment signature cannot be null".to_owned()),
            }) as super::StandardExpressionError
        })?;
        let specified_empty = specified_parameters
            .as_ref()
            .is_none_or(|values| read_recovering_poison(values).is_empty());
        if specified_empty {
            if fragment_signature.has_parameters() {
                return Err(process_error(format!(
                    "Cannot resolve fragment. Signature \"{}\" declares parameters, but fragment selection did not specify any parameters.",
                    fragment_signature
                        .get_string_representation()
                        .to_string_lossy()
                )));
            }
            return Ok(None);
        }
        if parameters_are_synthetic && !fragment_signature.has_parameters() {
            return Err(process_error(format!(
                "Cannot resolve fragment. Signature \"{}\" declares no parameters, but fragment selection did specify parameters in a synthetic manner (without names), which is not correct due to the fact parameters cannot be assigned names unless signature specifies these names.",
                fragment_signature
                    .get_string_representation()
                    .to_string_lossy()
            )));
        }
        let specified_parameters = specified_parameters.expect("non-empty parameters are present");
        if parameters_are_synthetic {
            let parameter_names = fragment_signature
                .get_parameter_names()
                .expect("signature reports parameters");
            let specified = read_recovering_poison(&specified_parameters);
            if parameter_names.len() != specified.len() {
                return Err(process_error(format!(
                    "Cannot resolve fragment. Signature \"{}\" declares {} parameters, but fragment selection specifies {} parameters. Fragment selection does not correctly match.",
                    fragment_signature
                        .get_string_representation()
                        .to_string_lossy(),
                    parameter_names.len(),
                    specified.len()
                )));
            }
            let mut processed = FragmentParameterMap::with_capacity(parameter_names.len() + 1);
            for (index, parameter_name) in parameter_names.iter().enumerate() {
                let synthetic_name = Some(Self::get_synthetic_parameter_name_for_index(index));
                let value = specified.get(&synthetic_name).cloned().unwrap_or(None);
                processed.insert(parameter_name.clone(), value);
            }
            return Ok(Some(Arc::new(RwLock::new(processed))));
        }
        if !fragment_signature.has_parameters() {
            return Ok(Some(specified_parameters));
        }
        {
            let specified = read_recovering_poison(&specified_parameters);
            let parameter_names = fragment_signature
                .get_parameter_names()
                .expect("signature reports parameters");
            for parameter_name in parameter_names.iter() {
                if !specified.contains_key(parameter_name) {
                    let display_name = parameter_name
                        .as_ref()
                        .map_or_else(|| "null".to_owned(), Utf16String::to_string_lossy);
                    return Err(process_error(format!(
                        "Cannot resolve fragment. Signature \"{}\" declares parameter \"{}\", which is not specified at the fragment selection.",
                        fragment_signature
                            .get_string_representation()
                            .to_string_lossy(),
                        display_name
                    )));
                }
            }
        }
        Ok(Some(specified_parameters))
    }
}

fn process_error(message: String) -> super::StandardExpressionError {
    Box::new(TemplateProcessingException::new(Some(message)))
}

fn read_recovering_poison<T>(lock: &RwLock<T>) -> RwLockReadGuard<'_, T> {
    lock.read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn trim(input: &Utf16String) -> Utf16String {
    let units = input.as_utf16();
    let start = units
        .iter()
        .position(|unit| *unit > 0x20)
        .unwrap_or(units.len());
    let end = units
        .iter()
        .rposition(|unit| *unit > 0x20)
        .map_or(start, |position| position + 1);
    Utf16String::from_utf16(units[start..end].to_vec())
}
