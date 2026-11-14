//! Gathering/iteration state-transition regressions against Java Thymeleaf 3.1.5.
//!
//! The scenarios deliberately nest the three engine paths:
//! IteratedGatheringModelProcessable (outer th:each),
//! GatheringModelProcessable (fragment th:replace), and another
//! IteratedGatheringModelProcessable (inner th:each).
//!
//! Java reference transitions:
//! - TemplateModelController#startGatheringIteratedModel(IOpenElementTag,...)
//! - TemplateModelController#startGatheringDelayedModel(IOpenElementTag,...)
//! - AbstractGatheringModelProcessable#gatherOpenElement/gatherCloseElement
//! - ProcessorTemplateHandler#handleCloseElement
//! - IteratedGatheringModelProcessable#process/processIterationModel
//! - AbstractGatheringModelProcessable#resetGatheredSkipFlagsAfterNoIterations

use std::io;
use std::sync::{Arc, Mutex};

use thymeleaf::context::Context;
use thymeleaf::expression::TemplateValue;
use thymeleaf::templateresolver::StringTemplateResolver;
use thymeleaf::util::{TemplateWriter, Utf16String};
use thymeleaf::{ITemplateResolver, TemplateEngine, TemplateMode};

fn js(value: &str) -> Utf16String {
    Utf16String::from_rust_str(value)
}

fn engine() -> TemplateEngine {
    let mut resolver = StringTemplateResolver::new();
    resolver.set_template_mode(TemplateMode::HTML);
    let engine = TemplateEngine::new();
    engine
        .set_template_resolver(Arc::new(resolver) as Arc<dyn ITemplateResolver>)
        .expect("set HTML resolver");
    engine
}

fn nested_groups_context() -> Context {
    let groups = vec![
        Vec::<Arc<TemplateValue>>::new(),
        vec![Arc::new(TemplateValue::string(js("one")))],
        vec![
            Arc::new(TemplateValue::string(js("two"))),
            Arc::new(TemplateValue::string(js("three"))),
        ],
    ]
    .into_iter()
    .map(|values| Arc::new(TemplateValue::List(Arc::new(values))))
    .collect::<Vec<_>>();

    let context = Context::new();
    context.set_variable(
        Some(js("groups")),
        Some(Arc::new(TemplateValue::List(Arc::new(groups)))),
    );
    // These outer values must survive the nested iteration scopes.
    context.set_variable(
        Some(js("entries")),
        Some(Arc::new(TemplateValue::string(js("outer-entries")))),
    );
    context.set_variable(
        Some(js("value")),
        Some(Arc::new(TemplateValue::string(js("outer-value")))),
    );
    context
}

const NESTED_GATHERING_TEMPLATE: &str = concat!(
    "<div th:fragment='group(values)' th:remove='tag'>",
    "<span th:each='value : ${values}' th:text='${value}'>value</span>",
    "<b>after</b>",
    "</div>",
    "<ul>",
    "<li th:each='entries : ${groups}'>",
    "<section th:replace='~{::group(${entries})}'>fallback</section>",
    "</li>",
    "</ul>",
    "<i th:text='${entries}'>entries</i>",
    "<i th:text='${value}'>value</i>"
);

const EXPECTED_NESTED_GATHERING: &str = concat!(
    "<b>after</b>",
    "<ul>",
    "<li><b>after</b></li>",
    "<li><span>one</span><b>after</b></li>",
    "<li><span>two</span><span>three</span><b>after</b></li>",
    "</ul>",
    "<i>outer-entries</i>",
    "<i>outer-value</i>"
);

#[derive(Clone)]
struct CapturedWriter {
    buffer: Arc<Mutex<Vec<u16>>>,
}

impl TemplateWriter for CapturedWriter {
    fn write_utf16(&mut self, characters: &[u16]) -> io::Result<()> {
        self.buffer
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .extend_from_slice(characters);
        Ok(())
    }
}

fn writer_text(buffer: &Arc<Mutex<Vec<u16>>>) -> String {
    String::from_utf16_lossy(&buffer.lock().unwrap_or_else(|error| error.into_inner()))
}

#[test]
fn nested_gathering_handles_zero_single_and_multiple_iterations() {
    let output = engine()
        .process_template(NESTED_GATHERING_TEMPLATE, &nested_groups_context())
        .expect("nested gathering process")
        .to_string_lossy();

    assert_eq!(
        EXPECTED_NESTED_GATHERING, output,
        "zero inner iterations must reset gathered skip flags; single/multiple iterations \
         must preserve the nested fragment model and restore both outer variables"
    );
}

#[test]
fn nested_gathering_resumes_across_throttled_writer_boundaries() {
    let engine = engine();
    let context = nested_groups_context();

    let expected = engine
        .process_template(NESTED_GATHERING_TEMPLATE, &context)
        .expect("plain nested gathering process")
        .to_string_lossy();

    let mut processor = engine
        .process_throttled_template(NESTED_GATHERING_TEMPLATE, &context)
        .expect("throttled nested gathering processor");
    let buffer = Arc::new(Mutex::new(Vec::new()));
    let writer = CapturedWriter {
        buffer: buffer.clone(),
    };

    let mut steps = 0usize;
    let max_steps = expected.len().saturating_mul(4).saturating_add(256);
    while !processor.is_finished() {
        let written = processor
            .process_writer(1, Box::new(writer.clone()))
            .expect("resume nested gathering");
        assert!(
            written <= 1,
            "each writer step must emit at most one UTF-16 unit"
        );
        steps += 1;
        assert!(
            steps < max_steps,
            "throttled nested gathering did not converge after {steps} steps"
        );
    }

    assert!(
        steps > 1,
        "the test must cross multiple throttle boundaries"
    );
    assert_eq!(expected, writer_text(&buffer));
    assert_eq!(EXPECTED_NESTED_GATHERING, writer_text(&buffer));
    assert!(processor.is_finished());
}
