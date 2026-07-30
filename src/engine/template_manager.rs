use std::cell::RefCell;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use std::sync::{Arc, Mutex, Weak};

use crate::cache::{
    AlwaysValidCacheEntryValidity, ICache, NonCacheableCacheEntryValidity, TemplateCacheKey,
};
use crate::context::{IContext, IEngineContext, ITemplateContext};
use crate::exceptions::{
    TemplateEngineException, TemplateInputException, TemplateProcessingException,
};
use crate::markup::{HTMLTemplateParser, XMLTemplateParser};
use crate::model::{
    ICDATASection, ICloseElementTag, IComment, IDocType, IModel, IOpenElementTag,
    IProcessingInstruction, IStandaloneElementTag, ITemplateEnd, ITemplateStart, IText,
    IXMLDeclaration,
};
use crate::raw::RawTemplateParser;
use crate::templateparser::{ITemplateParser, TemplateParserError};
use crate::templateresolver::TemplateResolution;
use crate::text::{CSSTemplateParser, JavaScriptTemplateParser, TextTemplateParser};
use crate::util::{JavaString, JavaWriter};
use crate::{
    IEngineConfiguration, IThrottledTemplateProcessor, TemplateMode, TemplateResolutionAttributes,
    TemplateSelectorSet, TemplateSpec,
};

use super::engine_context_manager::EngineContextManager;
use super::{
    ITemplateHandler, ITemplateManager, ModelBuilderTemplateHandler, OutputTemplateHandler,
    ProcessorTemplateHandler, TemplateData, TemplateHandlerHandle, TemplateModel,
    ThrottledTemplateProcessor,
};

const DEFAULT_PARSER_POOL_SIZE: i32 = 40;
const DEFAULT_PARSER_BLOCK_SIZE: i32 = 2048;

/// 负责模板解析、缓存、处理器链组装与完整渲染生命周期。
///
/// 对应 Java: `org.thymeleaf.engine.TemplateManager`。
pub struct TemplateManager {
    configuration: Weak<dyn IEngineConfiguration>,
    html_parser: HTMLTemplateParser,
    xml_parser: XMLTemplateParser,
    text_parser: TextTemplateParser,
    javascript_parser: JavaScriptTemplateParser,
    css_parser: CSSTemplateParser,
    raw_parser: RawTemplateParser,
}

impl TemplateManager {
    /// 创建并绑定唯一引擎配置的模板管理器。
    ///
    /// 对应 Java: `TemplateManager#TemplateManager(IEngineConfiguration)`。
    #[must_use]
    pub fn new(configuration: Arc<dyn IEngineConfiguration>) -> Self {
        let standard_dialect_present = configuration.is_standard_dialect_present();
        Self {
            configuration: Arc::downgrade(&configuration),
            html_parser: HTMLTemplateParser::new(
                DEFAULT_PARSER_POOL_SIZE,
                DEFAULT_PARSER_BLOCK_SIZE,
            ),
            xml_parser: XMLTemplateParser::new(DEFAULT_PARSER_POOL_SIZE, DEFAULT_PARSER_BLOCK_SIZE),
            text_parser: TextTemplateParser::new(
                DEFAULT_PARSER_POOL_SIZE,
                DEFAULT_PARSER_BLOCK_SIZE,
                standard_dialect_present,
            ),
            javascript_parser: JavaScriptTemplateParser::new(
                DEFAULT_PARSER_POOL_SIZE,
                DEFAULT_PARSER_BLOCK_SIZE,
                standard_dialect_present,
            ),
            css_parser: CSSTemplateParser::new(
                DEFAULT_PARSER_POOL_SIZE,
                DEFAULT_PARSER_BLOCK_SIZE,
                standard_dialect_present,
            ),
            raw_parser: RawTemplateParser::new(
                DEFAULT_PARSER_POOL_SIZE as usize,
                DEFAULT_PARSER_BLOCK_SIZE as usize,
            ),
        }
    }

    fn configuration(&self) -> Arc<dyn IEngineConfiguration> {
        self.configuration
            .upgrade()
            .expect("TemplateManager cannot outlive its EngineConfiguration")
    }

    fn with_template_cache<R>(
        &self,
        operation: impl FnOnce(Option<&dyn ICache<TemplateCacheKey, TemplateModel>>) -> R,
    ) -> R {
        let configuration = self.configuration();
        operation(
            configuration
                .get_cache_manager()
                .and_then(|manager| manager.get_template_cache()),
        )
    }

    fn clean_selectors(template_selectors: Option<&[JavaString]>) -> Option<Vec<JavaString>> {
        let mut selectors = template_selectors
            .filter(|selectors| !selectors.is_empty())?
            .to_vec();
        selectors.sort_by(|left, right| left.as_utf16().cmp(right.as_utf16()));
        selectors.dedup();
        Some(selectors)
    }

    fn cache_selectors(selectors: Option<&[JavaString]>) -> Option<Arc<TemplateSelectorSet>> {
        selectors.map(|selectors| {
            Arc::new(
                selectors
                    .iter()
                    .map(|selector| Some(selector.to_string_lossy()))
                    .collect(),
            )
        })
    }

    fn resolve_template(
        &self,
        owner_template: Option<&JavaString>,
        template: &JavaString,
        template_resolution_attributes: Option<&TemplateResolutionAttributes>,
        fail_if_not_exists: bool,
    ) -> Result<Option<TemplateResolution>, TemplateInputException> {
        // Markup selectors intentionally never reach resolvers: selection belongs to parsers.
        let configuration = self.configuration();
        for resolver in configuration.get_template_resolvers() {
            if let Some(resolution) = resolver.resolve_template(
                configuration.as_ref(),
                owner_template,
                template,
                template_resolution_attributes,
            ) {
                return Ok(Some(resolution));
            }
        }
        if !fail_if_not_exists {
            return Ok(None);
        }
        Err(TemplateInputException::new(Some(format!(
            "Error resolving template [{}], template might not exist or might not be accessible by any of the configured Template Resolvers",
            template.to_string_lossy()
        ))))
    }

    fn build_template_data(
        resolution: &TemplateResolution,
        template: &JavaString,
        selectors: Option<&[JavaString]>,
        template_mode: Option<TemplateMode>,
        use_cache: bool,
    ) -> Arc<TemplateData> {
        let definitive_mode = template_mode.unwrap_or_else(|| resolution.get_template_mode());
        let validity = if use_cache {
            resolution.get_validity_arc()
        } else {
            Arc::new(NonCacheableCacheEntryValidity::new())
        };
        Arc::new(TemplateData::new(
            Some(template.clone()),
            selectors.map(<[JavaString]>::to_vec),
            Some(resolution.get_template_resource_arc()),
            Some(definitive_mode),
            Some(validity),
        ))
    }

    fn parser_for_mode(&self, template_mode: TemplateMode) -> &dyn ITemplateParser {
        match template_mode {
            TemplateMode::HTML => &self.html_parser,
            TemplateMode::XML => &self.xml_parser,
            TemplateMode::TEXT => &self.text_parser,
            TemplateMode::JAVASCRIPT => &self.javascript_parser,
            TemplateMode::CSS => &self.css_parser,
            TemplateMode::RAW => &self.raw_parser,
        }
    }

    fn parse_resource_model(
        &self,
        owner_template: Option<&JavaString>,
        template: &JavaString,
        selectors: Option<&[JavaString]>,
        resolution: &TemplateResolution,
        template_data: Arc<TemplateData>,
    ) -> Result<TemplateModel, TemplateInputException> {
        let mode = template_data
            .get_template_mode()
            .expect("TemplateManager always builds a definitive template mode");
        let configuration = self.configuration();
        let builder = ModelBuilderTemplateHandler::new(
            Arc::clone(&configuration),
            Arc::clone(&template_data),
        );
        self.parser_for_mode(mode)
            .parse_standalone(
                configuration,
                owner_template,
                template,
                selectors,
                resolution.get_template_resource_arc(),
                mode,
                resolution.get_use_decoupled_logic(),
                Box::new(builder.clone()),
            )
            .map_err(parser_input_error)?;
        builder.get_model().map_err(|error| {
            TemplateInputException::with_cause(
                Some("An error happened during template parsing".to_owned()),
                error,
            )
        })
    }

    fn apply_pre_processors_if_needed(
        &self,
        context: &dyn ITemplateContext,
        template_model: TemplateModel,
    ) -> Result<TemplateModel, TemplateInputException> {
        let template_data = template_model.get_template_data().clone();
        let mode = template_data
            .get_template_mode()
            .expect("TemplateModel requires a definitive template mode");
        let configuration = self.configuration();
        if configuration.get_pre_processors(mode).is_empty() {
            return Ok(template_model);
        }
        let engine_context = EngineContextManager::prepare_engine_context(
            Arc::clone(&configuration),
            template_data.clone(),
            context.get_template_resolution_attributes(),
            context,
        );
        let builder = ModelBuilderTemplateHandler::new(configuration, Arc::new(template_data));
        let mut chain = self.create_handler_chain(
            Arc::clone(&engine_context),
            true,
            false,
            Box::new(builder.clone()),
            None,
        );
        let result = template_model.process(chain.as_mut());
        EngineContextManager::dispose_engine_context(engine_context.as_ref());
        result.map_err(engine_input_error)?;
        builder.get_model().map_err(|error| {
            TemplateInputException::with_cause(
                Some("An error happened during template preprocessing".to_owned()),
                error,
            )
        })
    }

    fn create_handler_chain(
        &self,
        context: Arc<dyn IEngineContext>,
        set_pre_processors: bool,
        set_post_processors: bool,
        central_handler: Box<dyn ITemplateHandler>,
        writer: Option<Box<dyn JavaWriter>>,
    ) -> Box<dyn ITemplateHandler> {
        let mode = context.get_template_mode();
        let template_context: Arc<dyn ITemplateContext> = context;
        let configuration = self.configuration();
        let mut handlers: Vec<Box<dyn ITemplateHandler>> = Vec::new();
        if set_pre_processors {
            handlers.extend(
                configuration
                    .get_pre_processors(mode)
                    .into_iter()
                    .map(|processor| (processor.get_handler_factory())()),
            );
        }
        handlers.push(central_handler);
        if set_post_processors {
            handlers.extend(
                configuration
                    .get_post_processors(mode)
                    .into_iter()
                    .map(|processor| (processor.get_handler_factory())()),
            );
        }
        if let Some(writer) = writer {
            handlers.push(Box::new(OutputTemplateHandler::new(writer)));
        }

        let mut next: Option<TemplateHandlerHandle> = None;
        for mut handler in handlers.into_iter().rev() {
            handler.set_context(Arc::clone(&template_context));
            handler.set_next(next);
            next = Some(Rc::new(RefCell::new(handler)));
        }
        Box::new(SharedTemplateHandler {
            delegate: next.expect("the central handler always makes the chain non-empty"),
        })
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "参数逐项对齐 Java TemplateManager 内部处理方法"
    )]
    fn parse_and_process_model(
        &self,
        model: &dyn IModel,
        template_data: TemplateData,
        attributes: Option<&TemplateResolutionAttributes>,
        context: &dyn IContext,
        writer: Box<dyn JavaWriter>,
        pre_processors: bool,
        post_processors: bool,
    ) -> Result<(), TemplateProcessingException> {
        let engine_context = EngineContextManager::prepare_engine_context(
            self.configuration(),
            template_data,
            attributes,
            context,
        );
        let mut chain = self.create_handler_chain(
            Arc::clone(&engine_context),
            pre_processors,
            post_processors,
            Box::new(ProcessorTemplateHandler::new()),
            Some(writer),
        );
        let result = process_model_events(model, chain.as_mut());
        EngineContextManager::dispose_engine_context(engine_context.as_ref());
        result.map_err(engine_processing_error)
    }
}

impl ITemplateManager for TemplateManager {
    fn clear_caches(&self) {
        self.with_template_cache(|cache| {
            if let Some(cache) = cache {
                cache.clear();
            }
        });
    }

    fn clear_caches_for(&self, template: &JavaString) {
        self.with_template_cache(|cache| {
            let Some(cache) = cache else {
                return;
            };
            let template = template.to_string_lossy();
            let keys = cache
                .key_set()
                .into_iter()
                .filter(|key| {
                    key.get_owner_template()
                        .map_or_else(|| key.get_template() == template, |owner| owner == template)
                })
                .collect::<Vec<_>>();
            for key in keys {
                cache.clear_key(&key);
            }
        });
    }

    fn parse_standalone(
        &self,
        context: &dyn ITemplateContext,
        template: &JavaString,
        template_selectors: Option<&[JavaString]>,
        template_mode: Option<TemplateMode>,
        use_cache: bool,
        fail_if_not_exists: bool,
    ) -> Result<Option<Box<dyn IModel>>, TemplateInputException> {
        let owner_data = context.get_template_data();
        let owner_template = owner_data.get_template();
        let attributes = context.get_template_resolution_attributes();
        let selectors = Self::clean_selectors(template_selectors);
        let cache_key = use_cache.then(|| {
            TemplateCacheKey::new(
                owner_template.map(JavaString::to_string_lossy).as_deref(),
                Some(&template.to_string_lossy()),
                Self::cache_selectors(selectors.as_deref()),
                0,
                0,
                template_mode,
                attributes.cloned().map(Arc::new),
            )
            .expect("template is non-null")
        });
        if let Some(key) = cache_key.as_ref() {
            if let Some(cached) =
                self.with_template_cache(|cache| cache.and_then(|cache| cache.get(key)))
            {
                let model =
                    self.apply_pre_processors_if_needed(context, cached.as_ref().clone())?;
                return Ok(Some(Box::new(model)));
            }
        }

        let Some(resolution) =
            self.resolve_template(owner_template, template, attributes, fail_if_not_exists)?
        else {
            return Ok(None);
        };
        if !fail_if_not_exists
            && !resolution.is_template_resource_existence_verified()
            && !resolution.get_template_resource().exists()
        {
            return Ok(None);
        }
        let template_data = Self::build_template_data(
            &resolution,
            template,
            selectors.as_deref(),
            template_mode,
            use_cache,
        );
        let model = self.parse_resource_model(
            owner_template,
            template,
            selectors.as_deref(),
            &resolution,
            template_data,
        )?;
        if let Some(key) = cache_key {
            if resolution.get_validity().is_cacheable() {
                self.with_template_cache(|cache| {
                    if let Some(cache) = cache {
                        cache.put(key, Arc::new(model.clone()));
                    }
                });
            }
        }
        self.apply_pre_processors_if_needed(context, model)
            .map(|model| Some(Box::new(model) as Box<dyn IModel>))
    }

    fn parse_string(
        &self,
        owner_template_data: &TemplateData,
        template: &JavaString,
        line_offset: i32,
        col_offset: i32,
        template_mode: Option<TemplateMode>,
        use_cache: bool,
    ) -> Result<Box<dyn IModel>, TemplateInputException> {
        let owner_template = owner_template_data
            .get_template()
            .expect("owner TemplateData requires a template name");
        let definitive_mode = template_mode
            .or_else(|| owner_template_data.get_template_mode())
            .expect("owner TemplateData requires a template mode");
        let cache_key = use_cache.then(|| {
            TemplateCacheKey::new(
                Some(&owner_template.to_string_lossy()),
                Some(&template.to_string_lossy()),
                None,
                line_offset,
                col_offset,
                Some(definitive_mode),
                None,
            )
            .expect("template is non-null")
        });
        if let Some(key) = cache_key.as_ref() {
            if let Some(cached) =
                self.with_template_cache(|cache| cache.and_then(|cache| cache.get(key)))
            {
                return Ok(Box::new(cached.as_ref().clone()));
            }
        }
        let validity: Arc<dyn crate::cache::ICacheEntryValidity> = if use_cache
            && owner_template_data
                .get_validity()
                .is_some_and(crate::cache::ICacheEntryValidity::is_cacheable)
        {
            Arc::new(AlwaysValidCacheEntryValidity::new())
        } else {
            Arc::new(NonCacheableCacheEntryValidity::new())
        };
        let template_data = if template_mode.is_none() {
            Arc::new(owner_template_data.clone())
        } else {
            Arc::new(TemplateData::new(
                owner_template_data.get_template().cloned(),
                owner_template_data
                    .get_template_selectors()
                    .map(<[JavaString]>::to_vec),
                owner_template_data.get_template_resource_arc(),
                Some(definitive_mode),
                Some(Arc::clone(&validity)),
            ))
        };
        let configuration = self.configuration();
        let builder = ModelBuilderTemplateHandler::new(Arc::clone(&configuration), template_data);
        self.parser_for_mode(definitive_mode)
            .parse_string(
                configuration,
                owner_template,
                template,
                line_offset,
                col_offset,
                definitive_mode,
                Box::new(builder.clone()),
            )
            .map_err(parser_input_error)?;
        let model = builder.get_model().map_err(|error| {
            TemplateInputException::with_cause(
                Some("An error happened during template parsing".to_owned()),
                error,
            )
        })?;
        if let Some(key) = cache_key {
            if validity.is_cacheable() {
                self.with_template_cache(|cache| {
                    if let Some(cache) = cache {
                        cache.put(key, Arc::new(model.clone()));
                    }
                });
            }
        }
        Ok(Box::new(model))
    }

    fn process(
        &self,
        template: &dyn IModel,
        context: &dyn ITemplateContext,
        writer: Box<dyn JavaWriter>,
    ) -> Result<(), TemplateProcessingException> {
        let configuration = self.configuration();
        if !std::ptr::eq(configuration.as_ref(), template.get_configuration()) {
            return Err(TemplateProcessingException::new(Some(
                "Specified template was built by a different Template Engine instance".to_owned(),
            )));
        }
        let template_data = template.get_template_data().cloned().ok_or_else(|| {
            TemplateProcessingException::new(Some(
                "Specified template model has no template data".to_owned(),
            ))
        })?;
        self.parse_and_process_model(
            template,
            template_data,
            context.get_template_resolution_attributes(),
            context,
            writer,
            false,
            false,
        )
    }

    fn parse_and_process(
        &self,
        template_spec: &TemplateSpec,
        context: &dyn IContext,
        writer: Box<dyn JavaWriter>,
    ) -> Result<(), TemplateProcessingException> {
        let template = JavaString::from_rust_str(template_spec.get_template());
        let selectors = template_spec.get_template_selectors().map(|selectors| {
            selectors
                .iter()
                .map(|selector| JavaString::from_rust_str(selector))
                .collect::<Vec<_>>()
        });
        let attributes = template_spec.get_template_resolution_attributes();
        let cache_key = TemplateCacheKey::new(
            None,
            Some(template_spec.get_template()),
            template_spec
                .get_template_selectors()
                .map(|selectors| Arc::new(selectors.iter().cloned().map(Some).collect())),
            0,
            0,
            template_spec.get_template_mode(),
            attributes.cloned().map(Arc::new),
        )
        .expect("TemplateSpec template is non-null");
        if let Some(cached) =
            self.with_template_cache(|cache| cache.and_then(|cache| cache.get(&cache_key)))
        {
            return self.parse_and_process_model(
                cached.as_ref(),
                cached.get_template_data().clone(),
                attributes,
                context,
                writer,
                true,
                true,
            );
        }
        let resolution = self
            .resolve_template(None, &template, attributes, true)
            .map_err(|error| {
                TemplateProcessingException::with_cause(
                    Some("An error happened during template resolution".to_owned()),
                    error,
                )
            })?
            .expect("fail_if_not_exists guarantees a resolution or error");
        let template_data = Self::build_template_data(
            &resolution,
            &template,
            selectors.as_deref(),
            template_spec.get_template_mode(),
            true,
        );
        let model = self
            .parse_resource_model(
                None,
                &template,
                selectors.as_deref(),
                &resolution,
                Arc::clone(&template_data),
            )
            .map_err(|error| {
                TemplateProcessingException::with_cause(
                    Some("An error happened during template parsing".to_owned()),
                    error,
                )
            })?;
        if resolution.get_validity().is_cacheable() {
            self.with_template_cache(|cache| {
                if let Some(cache) = cache {
                    cache.put(cache_key, Arc::new(model.clone()));
                }
            });
        }
        self.parse_and_process_model(
            &model,
            template_data.as_ref().clone(),
            attributes,
            context,
            writer,
            true,
            true,
        )
    }

    fn parse_and_process_throttled(
        &self,
        template_spec: &TemplateSpec,
        context: &dyn IContext,
    ) -> Result<Box<dyn IThrottledTemplateProcessor>, TemplateProcessingException> {
        let template = JavaString::from_rust_str(template_spec.get_template());
        let selectors = template_spec.get_template_selectors().map(|selectors| {
            selectors
                .iter()
                .map(|selector| JavaString::from_rust_str(selector))
                .collect::<Vec<_>>()
        });
        let attributes = template_spec.get_template_resolution_attributes();
        let cache_key = TemplateCacheKey::new(
            None,
            Some(template_spec.get_template()),
            template_spec
                .get_template_selectors()
                .map(|selectors| Arc::new(selectors.iter().cloned().map(Some).collect())),
            0,
            0,
            template_spec.get_template_mode(),
            attributes.cloned().map(Arc::new),
        )
        .expect("TemplateSpec template is non-null");

        let template_model = if let Some(cached) =
            self.with_template_cache(|cache| cache.and_then(|cache| cache.get(&cache_key)))
        {
            cached
        } else {
            let resolution = self
                .resolve_template(None, &template, attributes, true)
                .map_err(|error| {
                    TemplateProcessingException::with_cause(
                        Some("An error happened during template resolution".to_owned()),
                        error,
                    )
                })?
                .expect("fail_if_not_exists guarantees a resolution or error");
            let template_data = Self::build_template_data(
                &resolution,
                &template,
                selectors.as_deref(),
                template_spec.get_template_mode(),
                true,
            );
            let model = Arc::new(
                self.parse_resource_model(
                    None,
                    &template,
                    selectors.as_deref(),
                    &resolution,
                    template_data,
                )
                .map_err(|error| {
                    TemplateProcessingException::with_cause(
                        Some("An error happened during template parsing".to_owned()),
                        error,
                    )
                })?,
            );
            if resolution.get_validity().is_cacheable() {
                self.with_template_cache(|cache| {
                    if let Some(cache) = cache {
                        cache.put(cache_key, Arc::clone(&model));
                    }
                });
            }
            model
        };

        let engine_context = EngineContextManager::prepare_engine_context(
            self.configuration(),
            template_model.get_template_data().clone(),
            attributes,
            context,
        );
        let flow_controller = Arc::new(Mutex::new(
            super::template_flow_controller::TemplateFlowController::new(),
        ));
        let throttled_writer = ThrottledTemplateProcessor::create_writer(
            template_spec.get_template().to_owned(),
            Arc::clone(&flow_controller),
            template_spec.is_output_sse(),
        );
        let processor_handler = ProcessorTemplateHandler::new();
        processor_handler.set_flow_controller(Some(Arc::clone(&flow_controller)));
        let chain = self.create_handler_chain(
            Arc::clone(&engine_context),
            true,
            true,
            Box::new(processor_handler.clone()),
            Some(ThrottledTemplateProcessor::writer_proxy(Arc::clone(
                &throttled_writer,
            ))),
        );
        Ok(Box::new(ThrottledTemplateProcessor::new(
            template_spec.clone(),
            engine_context,
            template_model,
            chain,
            processor_handler,
            flow_controller,
            throttled_writer,
        )))
    }
}

fn parser_input_error(error: TemplateParserError) -> TemplateInputException {
    match error {
        TemplateParserError::Input(error) => error,
        TemplateParserError::IllegalArgument { message } => {
            TemplateInputException::new(Some(message))
        }
    }
}

fn engine_input_error(error: Box<dyn TemplateEngineException>) -> TemplateInputException {
    TemplateInputException::with_cause(
        Some("An error happened during template preprocessing".to_owned()),
        EngineExceptionCause(error),
    )
}

fn engine_processing_error(error: Box<dyn TemplateEngineException>) -> TemplateProcessingException {
    TemplateProcessingException::with_cause(
        Some("An error happened during template rendering".to_owned()),
        EngineExceptionCause(error),
    )
}

struct EngineExceptionCause(Box<dyn TemplateEngineException>);

impl Display for EngineExceptionCause {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.0.as_ref(), formatter)
    }
}

impl std::fmt::Debug for EngineExceptionCause {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_tuple("EngineExceptionCause")
            .field(&self.0.to_string())
            .finish()
    }
}

impl std::error::Error for EngineExceptionCause {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.0.as_ref())
    }
}

fn process_model_events(
    model: &dyn IModel,
    handler: &mut dyn ITemplateHandler,
) -> Result<(), Box<dyn TemplateEngineException>> {
    for index in 0..model.size() {
        model.get(index).be_handled(handler)?;
    }
    Ok(())
}

/// 在 `Rc<RefCell<_>>` Handler 链与 parser 所有权接口之间保留同一处理器身份。
struct SharedTemplateHandler {
    delegate: TemplateHandlerHandle,
}

impl ITemplateHandler for SharedTemplateHandler {
    fn set_next(&mut self, next: Option<TemplateHandlerHandle>) {
        self.delegate.borrow_mut().set_next(next);
    }

    fn set_context(&mut self, context: Arc<dyn ITemplateContext>) {
        self.delegate.borrow_mut().set_context(context);
    }

    fn handle_template_start(
        &mut self,
        event: Arc<dyn ITemplateStart>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.delegate.borrow_mut().handle_template_start(event)
    }

    fn handle_template_end(
        &mut self,
        event: Arc<dyn ITemplateEnd>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.delegate.borrow_mut().handle_template_end(event)
    }

    fn handle_xml_declaration(
        &mut self,
        event: Arc<dyn IXMLDeclaration>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.delegate.borrow_mut().handle_xml_declaration(event)
    }

    fn handle_doc_type(
        &mut self,
        event: Arc<dyn IDocType>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.delegate.borrow_mut().handle_doc_type(event)
    }

    fn handle_cdata_section(
        &mut self,
        event: Arc<dyn ICDATASection>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.delegate.borrow_mut().handle_cdata_section(event)
    }

    fn handle_comment(
        &mut self,
        event: Arc<dyn IComment>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.delegate.borrow_mut().handle_comment(event)
    }

    fn handle_text(
        &mut self,
        event: Arc<dyn IText>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.delegate.borrow_mut().handle_text(event)
    }

    fn handle_standalone_element(
        &mut self,
        event: Arc<dyn IStandaloneElementTag>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.delegate.borrow_mut().handle_standalone_element(event)
    }

    fn handle_open_element(
        &mut self,
        event: Arc<dyn IOpenElementTag>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.delegate.borrow_mut().handle_open_element(event)
    }

    fn handle_close_element(
        &mut self,
        event: Arc<dyn ICloseElementTag>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.delegate.borrow_mut().handle_close_element(event)
    }

    fn handle_processing_instruction(
        &mut self,
        event: Arc<dyn IProcessingInstruction>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.delegate
            .borrow_mut()
            .handle_processing_instruction(event)
    }
}
