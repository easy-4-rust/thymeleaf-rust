use std::io;
use std::sync::{Arc, RwLock, RwLockReadGuard};

use indexmap::IndexMap;

use crate::engine::TemplateModel;
use crate::expression::TemplateValue;
use crate::model::IModel;
use crate::util::{FastStringWriter, JavaString, ValidateError};

type FragmentParameterMap =
    IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>>;

/// Fragment Expression 的执行结果。
///
/// 对应 Java: `org.thymeleaf.standard.expression.Fragment`。
pub struct Fragment {
    template_model: Option<Arc<TemplateModel>>,
    parameters: Option<Arc<RwLock<FragmentParameterMap>>>,
    synthetic_parameters: bool,
}

impl Fragment {
    /// 不包含模型或参数的空 Fragment。
    pub const EMPTY_FRAGMENT: Self = Self {
        template_model: None,
        parameters: None,
        synthetic_parameters: false,
    };

    /// 创建 Fragment；参数 Map 使用只读包装但保留原 backing map 身份。
    pub fn new(
        template_model: Option<Arc<TemplateModel>>,
        parameters: Option<Arc<RwLock<FragmentParameterMap>>>,
        synthetic_parameters: bool,
    ) -> Result<Self, ValidateError> {
        let template_model = template_model.ok_or_else(|| ValidateError::IllegalArgument {
            message: Some("Template model cannot be null".to_owned()),
        })?;
        let synthetic_parameters = parameters.as_ref().is_some_and(|values| {
            !read_recovering_poison(values).is_empty() && synthetic_parameters
        });
        Ok(Self {
            template_model: Some(template_model),
            parameters,
            synthetic_parameters,
        })
    }

    /// 返回全局 EMPTY_FRAGMENT 单例。
    pub fn empty_fragment() -> &'static Self {
        &Self::EMPTY_FRAGMENT
    }

    /// 返回可空模板模型。
    pub fn get_template_model(&self) -> Option<&TemplateModel> {
        self.template_model.as_deref()
    }

    /// 返回原参数 Map 的实时只读视图。
    pub fn get_parameters(&self) -> Option<RwLockReadGuard<'_, FragmentParameterMap>> {
        self.parameters
            .as_ref()
            .map(|parameters| read_recovering_poison(parameters))
    }

    /// 判断构造瞬间非空参数是否为合成位置参数。
    pub fn has_synthetic_parameters(&self) -> bool {
        self.synthetic_parameters
    }

    /// 将 Fragment 模型写入 Java Writer；EMPTY_FRAGMENT 不写任何内容。
    pub fn write(&self, writer: &mut dyn crate::util::JavaWriter) -> io::Result<()> {
        if let Some(template_model) = &self.template_model {
            template_model.write(writer)?;
        }
        Ok(())
    }

    /// 返回模型序列化文本。
    pub fn to_java_string(&self) -> io::Result<JavaString> {
        let mut writer = FastStringWriter::new();
        self.write(&mut writer)?;
        Ok(writer.to_string())
    }
}

fn read_recovering_poison<T>(lock: &RwLock<T>) -> RwLockReadGuard<'_, T> {
    lock.read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
