//! `TemplateResolution` 的 Thymeleaf 3.1.5 Java/Rust Golden 差分测试。

use std::io::{Cursor, Read};
use std::rc::Rc;
use std::sync::Arc;

use thymeleaf::cache::{
    AlwaysValidCacheEntryValidity, ICacheEntryValidity, NonCacheableCacheEntryValidity,
};
use thymeleaf::{
    ITemplateResource, TemplateMode, TemplateResolution, TemplateResolutionError,
    TemplateResourceError,
};

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const JAVA_GOLDEN: &str = include_str!("fixtures/template_resolution_golden.txt");

struct TestResource {
    description: String,
    exists: bool,
}

impl TestResource {
    fn new(description: impl Into<String>, exists: bool) -> Self {
        Self {
            description: description.into(),
            exists,
        }
    }
}

impl ITemplateResource for TestResource {
    fn get_description(&self) -> String {
        self.description.clone()
    }

    fn get_base_name(&self) -> Option<String> {
        None
    }

    fn exists(&self) -> bool {
        self.exists
    }

    fn reader(&self) -> std::io::Result<Box<dyn Read>> {
        Ok(Box::new(Cursor::new(
            self.description.as_bytes().to_owned(),
        )))
    }

    fn relative(
        &self,
        _relative_location: Option<&str>,
    ) -> Result<Box<dyn ITemplateResource>, TemplateResourceError> {
        Ok(Box::new(Self::new(&self.description, self.exists)))
    }
}

#[test]
fn template_resolution_matches_java_golden() {
    let mut output = String::new();
    emit(&mut output, "baseline", JAVA_BASELINE);
    export_validation(&mut output);
    export_defaults_and_identity(&mut output);
    export_full_flags_and_modes(&mut output);
    assert_eq!(output, JAVA_GOLDEN);
}

fn export_validation(output: &mut String) {
    let resource: Rc<dyn ITemplateResource> = Rc::new(TestResource::new("resource", false));
    let validity: Arc<dyn ICacheEntryValidity> = Arc::new(AlwaysValidCacheEntryValidity::new());

    emit_failure(
        output,
        "null.resource",
        TemplateResolution::new(None, Some(TemplateMode::HTML), Some(Arc::clone(&validity))),
    );
    emit_failure(
        output,
        "null.mode",
        TemplateResolution::new(
            Some(Rc::clone(&resource)),
            None,
            Some(Arc::clone(&validity)),
        ),
    );
    emit_failure(
        output,
        "null.validity",
        TemplateResolution::new(Some(Rc::clone(&resource)), Some(TemplateMode::HTML), None),
    );
    emit_failure(
        output,
        "null.validation_order",
        TemplateResolution::with_options(None, true, None, true, None),
    );
}

fn export_defaults_and_identity(output: &mut String) {
    let resource: Rc<dyn ITemplateResource> = Rc::new(TestResource::new("missing", false));
    let validity: Arc<dyn ICacheEntryValidity> = Arc::new(AlwaysValidCacheEntryValidity::new());
    let resolution = TemplateResolution::new(
        Some(Rc::clone(&resource)),
        Some(TemplateMode::HTML),
        Some(Arc::clone(&validity)),
    )
    .expect("default resolution");

    emit(
        output,
        "default.resource_identity",
        &std::ptr::eq(resolution.get_template_resource(), resource.as_ref()).to_string(),
    );
    emit(
        output,
        "default.resource_description",
        &resolution.get_template_resource().get_description(),
    );
    emit(
        output,
        "default.resource_exists",
        &resolution.get_template_resource().exists().to_string(),
    );
    emit(
        output,
        "default.existence_verified",
        &resolution
            .is_template_resource_existence_verified()
            .to_string(),
    );
    emit(
        output,
        "default.mode",
        &resolution.get_template_mode().to_string(),
    );
    emit(
        output,
        "default.use_decoupled_logic",
        &resolution.get_use_decoupled_logic().to_string(),
    );
    emit(
        output,
        "default.validity_identity",
        &std::ptr::eq(resolution.get_validity(), validity.as_ref()).to_string(),
    );
    emit(
        output,
        "default.validity_cacheable",
        &resolution.get_validity().is_cacheable().to_string(),
    );
    emit(
        output,
        "default.validity_still_valid",
        &resolution.get_validity().is_cache_still_valid().to_string(),
    );
}

fn export_full_flags_and_modes(output: &mut String) {
    for mode in [
        TemplateMode::HTML,
        TemplateMode::XML,
        TemplateMode::TEXT,
        TemplateMode::JAVASCRIPT,
        TemplateMode::CSS,
        TemplateMode::RAW,
    ] {
        let existence_verified = mode.is_markup();
        let use_decoupled_logic = mode.is_text();
        let resource: Rc<dyn ITemplateResource> = Rc::new(TestResource::new(
            format!("mode-{mode}"),
            !existence_verified,
        ));
        let validity: Arc<dyn ICacheEntryValidity> =
            Arc::new(NonCacheableCacheEntryValidity::new());
        let resolution = TemplateResolution::with_options(
            Some(Rc::clone(&resource)),
            existence_verified,
            Some(mode),
            use_decoupled_logic,
            Some(Arc::clone(&validity)),
        )
        .expect("full resolution");
        let prefix = format!("full.{mode}");

        emit(
            output,
            &format!("{prefix}.resource_identity"),
            &std::ptr::eq(resolution.get_template_resource(), resource.as_ref()).to_string(),
        );
        emit(
            output,
            &format!("{prefix}.resource_exists"),
            &resolution.get_template_resource().exists().to_string(),
        );
        emit(
            output,
            &format!("{prefix}.existence_verified"),
            &resolution
                .is_template_resource_existence_verified()
                .to_string(),
        );
        emit(
            output,
            &format!("{prefix}.mode"),
            &resolution.get_template_mode().to_string(),
        );
        emit(
            output,
            &format!("{prefix}.use_decoupled_logic"),
            &resolution.get_use_decoupled_logic().to_string(),
        );
        emit(
            output,
            &format!("{prefix}.validity_identity"),
            &std::ptr::eq(resolution.get_validity(), validity.as_ref()).to_string(),
        );
        emit(
            output,
            &format!("{prefix}.validity_cacheable"),
            &resolution.get_validity().is_cacheable().to_string(),
        );
        emit(
            output,
            &format!("{prefix}.validity_still_valid"),
            &resolution.get_validity().is_cache_still_valid().to_string(),
        );
    }
}

fn emit_failure(
    output: &mut String,
    key: &str,
    result: Result<TemplateResolution, TemplateResolutionError>,
) {
    match result {
        Ok(_) => {
            emit(output, &format!("{key}.class"), "<none>");
            emit(output, &format!("{key}.message"), "<none>");
        }
        Err(error) => {
            emit(output, &format!("{key}.class"), "IllegalArgumentException");
            emit(output, &format!("{key}.message"), &error.to_string());
        }
    }
}

fn emit(output: &mut String, key: &str, value: &str) {
    output.push_str(key);
    output.push('=');
    output.push_str(value);
    output.push('\n');
}
