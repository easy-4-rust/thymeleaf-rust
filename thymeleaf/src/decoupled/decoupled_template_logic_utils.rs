use std::io::Read;
use std::sync::Arc;

use crate::exceptions::TemplateInputException;
use crate::templateparser::TemplateParserError;
use crate::templateresource::ITemplateResource;
use crate::util::JavaString;
use crate::{IEngineConfiguration, TemplateMode};

use super::{DecoupledTemplateLogic, DecoupledTemplateLogicBuilderMarkupHandler};

/// 定位并解析模板对应的解耦逻辑资源。
///
/// 对应 Java:
/// `org.thymeleaf.templateparser.markup.decoupled.DecoupledTemplateLogicUtils`。
pub struct DecoupledTemplateLogicUtils {
    _private: (),
}

impl DecoupledTemplateLogicUtils {
    /// 解析主模板对应的解耦逻辑；资源不存在时返回 `Ok(None)`。
    ///
    /// Resolver、relative resource、reader、UTF-8 解码和标记解析错误均按发生顺序
    /// 传播，不能把失败降级为“模板没有解耦逻辑”。
    ///
    /// 对应 Java:
    /// `DecoupledTemplateLogicUtils#computeDecoupledTemplateLogic`。
    #[allow(clippy::too_many_arguments)]
    pub fn compute_decoupled_template_logic(
        configuration: &dyn IEngineConfiguration,
        owner_template: Option<&JavaString>,
        template: &JavaString,
        template_selectors: Option<&[JavaString]>,
        resource: &dyn ITemplateResource,
        template_mode: TemplateMode,
    ) -> Result<Option<Arc<DecoupledTemplateLogic>>, TemplateParserError> {
        let Some(decoupled_resource) = configuration
            .get_decoupled_template_logic_resolver()
            .resolve_decoupled_template_logic(
                configuration,
                owner_template,
                template,
                template_selectors,
                resource,
                template_mode,
            )
            .map_err(|error| {
                TemplateInputException::with_template_and_cause(
                    Some("An error happened during template parsing".to_owned()),
                    Some(template.to_string_lossy()),
                    error,
                )
            })?
        else {
            return Ok(None);
        };

        if !decoupled_resource.exists() {
            return Ok(None);
        }
        let mut reader = decoupled_resource.reader().map_err(|error| {
            TemplateInputException::with_template_and_cause(
                Some("An error happened during template parsing".to_owned()),
                Some(decoupled_resource.get_description()),
                error,
            )
        })?;
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).map_err(|error| {
            TemplateInputException::with_template_and_cause(
                Some("An error happened during template parsing".to_owned()),
                Some(decoupled_resource.get_description()),
                error,
            )
        })?;
        let source = String::from_utf8(bytes).map_err(|error| {
            TemplateInputException::with_template_and_cause(
                Some("An error happened during template parsing".to_owned()),
                Some(decoupled_resource.get_description()),
                error,
            )
        })?;
        let mut handler =
            DecoupledTemplateLogicBuilderMarkupHandler::new(template.clone(), template_mode)?;
        handler.parse(&source)?;
        Ok(Some(Arc::new(handler.into_decoupled_template_logic())))
    }
}
