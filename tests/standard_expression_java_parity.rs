//! Standard Expression 端到端 Java 1:1 差分测试。
//!
//! 对应上游 `thymeleaf-tests-core` 的
//! `org.thymeleaf.standard.expression.ExpressionTest`（约 200 个案例）。
//!
//! 与上游完全一致：模板名即表达式，`ExpressionTemplateResolver` 把
//! `{%%}` 占位符替换为表达式文本；`TestMessageResolver` 承载
//! Properties 消息表（含 `{0,date,dd/MM/yyyy}` MessageFormat 日期子格式）；
//! 上下文注入 User/Department 动态 Bean、logins 列表、数组与标量。
//!
//! 覆盖表达式 AST：Addition/Subtraction/Multiplication/Division/
//! Remainder/And/Or/Equals/NotEquals/GreaterLesser/Conditional/
//! Default/Negation/BooleanToken/NumberToken/TextLiteral/
//! GenericToken/NoOpToken/Variable/SelectionVariable/Message/Link/
//! Fragment/Assignation 与预处理 `__...__`。

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

use chrono::{Datelike, TimeZone, Timelike};

use thymeleaf::context::{Context, IContext};
use thymeleaf::expression::{
    TemplateObject, TemplateObjectMethodError, TemplateObjectPropertyError, TemplateValue,
};
use thymeleaf::messageresolver::{IMessageResolver, MessageResolutionResult};
use thymeleaf::templateresolver::{ITemplateResolver, TemplateResolution, TemplateResolverError};
use thymeleaf::templateresource::{ITemplateResource, StringTemplateResource};
use thymeleaf::util::{JavaDate, JavaLocale, JavaNumber, JavaString};
use thymeleaf::{TemplateEngine, TemplateMode};

const TEMPLATE: &str =
    "<!DOCTYPE html><html><body><span th:text=\"{%%}\">PLACEHOLDER</span></body></html>";
const RESULT_PREFIX: &str = "<!DOCTYPE html><html><body><span>";
const RESULT_SUFFIX: &str = "</span></body></html>";

fn js(value: &str) -> JavaString {
    JavaString::from_rust_str(value)
}

fn string_value(value: &str) -> Arc<TemplateValue> {
    Arc::new(TemplateValue::String(Arc::new(js(value))))
}

// ===========================================================================
// 1. 模板名→{%%} 替换 Resolver（对应 Java TestTemplateResolver）
// ===========================================================================

struct ExpressionTemplateResolver {
    template: &'static str,
    name: JavaString,
}

impl ExpressionTemplateResolver {
    fn new(template: &'static str) -> Self {
        Self {
            template,
            name: js("TEST EXPRESSION TEMPLATE RESOLVER"),
        }
    }
}

impl ITemplateResolver for ExpressionTemplateResolver {
    fn get_name(&self) -> Option<&JavaString> {
        Some(&self.name)
    }

    fn get_order(&self) -> Option<i32> {
        Some(1)
    }

    fn resolve_template(
        &self,
        _configuration: &dyn thymeleaf::IEngineConfiguration,
        _owner_template: Option<&JavaString>,
        template: &JavaString,
        _template_resolution_attributes: Option<&thymeleaf::TemplateResolutionAttributes>,
    ) -> Result<Option<TemplateResolution>, TemplateResolverError> {
        let placeholder = "{%%}";
        let position = self.template.find(placeholder).expect("placeholder");
        let resource_text = format!(
            "{}{}{}",
            &self.template[..position],
            template.to_string_lossy(),
            &self.template[position + placeholder.len()..]
        );
        let resource: Arc<dyn ITemplateResource> =
            Arc::new(StringTemplateResource::new(Some(&resource_text)).expect("string resource"));
        let validity: Arc<dyn thymeleaf::cache::ICacheEntryValidity> =
            Arc::new(thymeleaf::cache::NonCacheableCacheEntryValidity::new());
        Ok(Some(
            TemplateResolution::with_options(
                Some(resource),
                true,
                Some(TemplateMode::HTML),
                false,
                Some(validity),
            )
            .expect("resolution"),
        ))
    }
}

// ===========================================================================
// 2. 消息 Resolver（对应 Java TestMessageResolver + Properties）
// ===========================================================================

struct ExpressionMessageResolver {
    messages: HashMap<JavaString, JavaString>,
}

impl ExpressionMessageResolver {
    fn new() -> Self {
        let mut messages = HashMap::new();
        for (key, value) in [
            ("application.name", "Thymeleaf test"),
            ("hello.message", "Hello, {0}!"),
            ("today", "Today is {0,date,dd/MM/yyyy}, so hello {1}!"),
            ("title.dept.Marketing", "The almighty Marketing department"),
            (
                "title.user.meurope",
                "User {0} works for the {1} department",
            ),
            ("priority.basic", "3"),
            ("company.yearfounded", "1976"),
            ("dateForPath", "10/10/1976"),
            ("sum", "1+1=2"),
        ] {
            messages.insert(js(key), js(value));
        }
        Self { messages }
    }
}

impl IMessageResolver for ExpressionMessageResolver {
    fn get_name(&self) -> Option<&JavaString> {
        None
    }

    fn get_order(&self) -> Option<i32> {
        None
    }

    fn resolve_message_nullable(
        &self,
        context: Option<&dyn thymeleaf::context::ITemplateContext>,
        _origin: Option<std::any::TypeId>,
        key: Option<&JavaString>,
        message_parameters: Option<&[Option<Arc<TemplateValue>>]>,
    ) -> MessageResolutionResult<Option<JavaString>> {
        let (Some(_context), Some(key)) = (context, key) else {
            return Ok(None);
        };
        Ok(self
            .messages
            .get(key)
            .map(|message| format_message_like_java(message, message_parameters.unwrap_or(&[]))))
    }

    fn create_absent_message_representation_nullable(
        &self,
        context: Option<&dyn thymeleaf::context::ITemplateContext>,
        _origin: Option<std::any::TypeId>,
        key: Option<&JavaString>,
        _message_parameters: Option<&[Option<Arc<TemplateValue>>]>,
    ) -> MessageResolutionResult<Option<JavaString>> {
        let (Some(context), Some(key)) = (context, key) else {
            return Ok(None);
        };
        Ok(Some(JavaString::from_rust_str(&format!(
            "??{}_{}??",
            key.to_string_lossy(),
            context.get_locale()
        ))))
    }
}

/// 实现 Java MessageFormat 索引占位符与 `{i,date,pattern}` 子格式。
fn format_message_like_java(
    message: &JavaString,
    parameters: &[Option<Arc<TemplateValue>>],
) -> JavaString {
    let text = message.to_string_lossy();
    let characters = text.chars().collect::<Vec<_>>();
    let mut result = String::with_capacity(text.len());
    let mut position = 0_usize;
    let mut quoted = false;

    while position < characters.len() {
        let character = characters[position];
        if character == '\'' {
            if characters.get(position + 1) == Some(&'\'') {
                result.push('\'');
                position += 2;
                continue;
            }
            quoted = !quoted;
            position += 1;
            continue;
        }
        if character == '{'
            && !quoted
            && let Some(end) = characters[position + 1..]
                .iter()
                .position(|candidate| *candidate == '}')
                .map(|offset| position + 1 + offset)
        {
            let element = characters[position + 1..end].iter().collect::<String>();
            let parts = element.split(',').collect::<Vec<_>>();
            if let Some(parameter_index) = parts
                .first()
                .map(|value| value.trim())
                .and_then(|value| value.parse::<usize>().ok())
            {
                let Some(parameter) = parameters.get(parameter_index) else {
                    result.push('{');
                    result.push_str(&element);
                    result.push('}');
                    position = end + 1;
                    continue;
                };
                let rendered = if parts.len() > 2
                    && parts[1].trim() == "date"
                    && let Some(parameter) = parameter.as_deref()
                {
                    // Java MessageFormat {0,date,pattern}：Long 毫秒 → 日期格式化
                    let pattern = parts[2..].join(",");
                    let millis = match parameter {
                        TemplateValue::Number(JavaNumber::Long(value)) => Some(*value),
                        TemplateValue::Number(JavaNumber::Integer(value)) => {
                            Some(i64::from(*value))
                        }
                        _ => None,
                    };
                    match millis {
                        Some(millis) => {
                            let naive = chrono::Local
                                .timestamp_millis_opt(millis)
                                .single()
                                .expect("millis in range")
                                .naive_local();
                            let date = thymeleaf::util::DateUtils::create(
                                Some(naive.year()),
                                Some(naive.month() as i32),
                                Some(naive.day() as i32),
                                Some(naive.hour() as i32),
                                Some(naive.minute() as i32),
                                Some(naive.second() as i32),
                                Some(naive.and_utc().timestamp_subsec_millis() as i32),
                                None,
                                None,
                            )
                            .expect("date from millis");
                            thymeleaf::util::DateUtils::format(
                                Some(&date),
                                Some(&js(pattern.trim())),
                                Some(&JavaLocale::new(js("en"), js("US"))),
                            )
                            .expect("date format")
                            .expect("formatted date")
                            .to_string_lossy()
                        }
                        None => parameter
                            .to_java_string()
                            .map_or_else(|| "null".to_owned(), |value| value.to_string_lossy()),
                    }
                } else {
                    parameter
                        .as_deref()
                        .and_then(TemplateValue::to_java_string)
                        .map_or_else(|| "null".to_owned(), |value| value.to_string_lossy())
                };
                result.push_str(&rendered);
                position = end + 1;
                continue;
            }
        }
        result.push(character);
        position += 1;
    }
    JavaString::from_rust_str(&result)
}

// ===========================================================================
// 3. User/Department 动态 Bean（对应 Java 嵌套类）
// ===========================================================================

struct TestDepartment {
    id: i32,
    name: &'static str,
}

impl TemplateObject for TestDepartment {
    fn java_class_name(&self) -> &str {
        "org.thymeleaf.standard.expression.ExpressionTest$Department"
    }

    fn to_java_string(&self) -> JavaString {
        js(&format!("ExpressionTest$Department@{}", self.id))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn java_get_property(
        &self,
        property_name: &JavaString,
    ) -> Option<Result<Option<Arc<TemplateValue>>, TemplateObjectPropertyError>> {
        match property_name.to_string_lossy().as_str() {
            "id" => Some(Ok(Some(Arc::new(TemplateValue::Number(
                JavaNumber::Integer(self.id),
            ))))),
            "name" => Some(Ok(Some(string_value(self.name)))),
            _ => None,
        }
    }
}

struct TestUser {
    login: &'static str,
    name: &'static str,
    department: Arc<TestDepartment>,
    priority: i32,
    creation_date: Arc<JavaDate>,
    coefficient: f64,
    admin: bool,
    permissions: Option<Vec<&'static str>>,
}

impl TemplateObject for TestUser {
    fn java_class_name(&self) -> &str {
        "org.thymeleaf.standard.expression.ExpressionTest$User"
    }

    fn to_java_string(&self) -> JavaString {
        js(&format!("ExpressionTest$User@{}", self.login))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn java_get_property(
        &self,
        property_name: &JavaString,
    ) -> Option<Result<Option<Arc<TemplateValue>>, TemplateObjectPropertyError>> {
        match property_name.to_string_lossy().as_str() {
            "login" => Some(Ok(Some(string_value(self.login)))),
            "name" => Some(Ok(Some(string_value(self.name)))),
            "department" => Some(Ok(Some(Arc::new(TemplateValue::Object(
                self.department.clone(),
            ))))),
            "priority" => Some(Ok(Some(Arc::new(TemplateValue::Number(
                JavaNumber::Integer(self.priority),
            ))))),
            "creationDate" => Some(Ok(Some(Arc::new(TemplateValue::Object(
                self.creation_date.clone(),
            ))))),
            "coefficient" => Some(Ok(Some(Arc::new(TemplateValue::Number(
                JavaNumber::Double(self.coefficient),
            ))))),
            "admin" => Some(Ok(Some(Arc::new(TemplateValue::Boolean(self.admin))))),
            "permissions" => Some(Ok(Some(match &self.permissions {
                Some(values) => Arc::new(TemplateValue::List(Arc::new(
                    values.iter().map(|value| string_value(value)).collect(),
                ))),
                None => Arc::new(TemplateValue::Null),
            }))),
            _ => None,
        }
    }

    fn java_invoke_method(
        &self,
        method_name: &JavaString,
        arguments: &[Option<Arc<TemplateValue>>],
    ) -> Option<Result<Option<Arc<TemplateValue>>, TemplateObjectMethodError>> {
        (method_name.to_string_lossy() == "isAdmin" && arguments.is_empty())
            .then(|| Ok(Some(Arc::new(TemplateValue::Boolean(self.admin)))))
    }
}

// ===========================================================================
// 4. 引擎与上下文
// ===========================================================================

fn engine() -> TemplateEngine {
    let e = TemplateEngine::new();
    e.set_template_resolver(Arc::new(ExpressionTemplateResolver::new(TEMPLATE)))
        .expect("resolver");
    e.set_message_resolver(Arc::new(ExpressionMessageResolver::new()))
        .expect("message resolver");
    e
}

fn to_date(year: i32, month: i32, day: i32) -> Arc<JavaDate> {
    Arc::new(
        thymeleaf::util::DateUtils::create(
            Some(year),
            Some(month),
            Some(day),
            Some(0),
            Some(0),
            None,
            None,
            None,
            None,
        )
        .expect("date"),
    )
}

fn build_context(locale: JavaLocale) -> Context {
    let ctx = Context::new();
    ctx.set_locale(Some(locale)).expect("locale");

    let dept_accounting = Arc::new(TestDepartment {
        id: 1,
        name: "Accounting and Finance",
    });
    let dept_engineering = Arc::new(TestDepartment {
        id: 2,
        name: "Engineering and Consulting",
    });
    let dept_sales = Arc::new(TestDepartment {
        id: 3,
        name: "Sales",
    });
    let dept_marketing = Arc::new(TestDepartment {
        id: 4,
        name: "Marketing",
    });

    let login_values = Arc::new(
        ["loceania", "meurope", "jafrica", "pamerica"]
            .iter()
            .map(|login| string_value(login))
            .collect::<Vec<_>>(),
    );

    let users = vec![
        (
            "loceania",
            TestUser {
                login: "loceania",
                name: "Leslie Oceania",
                department: dept_marketing,
                priority: 3,
                creation_date: to_date(2004, 11, 23),
                coefficient: 5.3,
                admin: false,
                permissions: Some(vec![
                    "Event Organizer",
                    "Marketing Worldwide Head",
                    "Office Master",
                ]),
            },
        ),
        (
            "meurope",
            TestUser {
                login: "meurope",
                name: "Markus Europe",
                department: dept_engineering,
                priority: 5,
                creation_date: to_date(2008, 1, 3),
                coefficient: 8.0,
                admin: true,
                permissions: None,
            },
        ),
        (
            "jafrica",
            TestUser {
                login: "jafrica",
                name: "Jacques Africa",
                department: dept_sales,
                priority: 3,
                creation_date: to_date(2010, 9, 23),
                coefficient: 4.3,
                admin: false,
                permissions: Some(vec!["Sales Manager", "Department Director"]),
            },
        ),
        (
            "pamerica",
            TestUser {
                login: "pamerica",
                name: "Petronila America",
                department: dept_accounting,
                priority: 1,
                creation_date: to_date(2002, 4, 19),
                coefficient: 9.2,
                admin: false,
                permissions: None,
            },
        ),
    ];

    for (login, user) in users {
        ctx.set_variable(
            Some(js(login)),
            Some(Arc::new(TemplateValue::Object(Arc::new(user)))),
        );
    }

    ctx.set_variable(
        Some(js("currentYear")),
        Some(Arc::new(TemplateValue::Number(JavaNumber::Integer(2011)))),
    );
    ctx.set_variable(
        Some(js("logins")),
        Some(Arc::new(TemplateValue::List(login_values.clone()))),
    );
    ctx.set_variable(
        Some(js("loginsArray")),
        Some(Arc::new(TemplateValue::List(login_values))),
    );
    ctx.set_variable(Some(js("size")), Some(string_value("Size is 5")));
    ctx
}

fn context_en() -> Context {
    build_context(JavaLocale::new(js("en"), js("US")))
}

fn context_es() -> Context {
    build_context(JavaLocale::new(js("es"), js("ES")))
}

/// 与 Java `test(expression, result)` 完全一致：处理模板名即表达式，
/// 再裁剪固定前缀/后缀。
fn test(expression: &str, expected: &str) {
    test_with_context(expression, expected, &context_en());
}

fn test_with_context(expression: &str, expected: &str, context: &dyn IContext) {
    let output = engine()
        .process_template(expression, context)
        .expect("template must process")
        .to_string_lossy();
    let output = output
        .strip_prefix(RESULT_PREFIX)
        .and_then(|rest| rest.strip_suffix(RESULT_SUFFIX))
        .unwrap_or_else(|| panic!("unexpected output shape: {output}"));
    assert_eq!(output, expected, "expression: {expression}");
}

// ===========================================================================
// 5. ExpressionTest#testExpression —— 按语义段拆分
// ===========================================================================

#[test]
fn expression_arithmetic_and_ternary() {
    test("23 + 43 + 1", "67");
    test("${true}? 'x' : 'y'", "x");
    test("${false}? 'x' : 'y'", "y");
    test("${loceania.admin}", "false");
    test("!${loceania.admin}", "true");
    test("${!loceania.admin}", "true");
    test("${loceania.department.name}", "Marketing");
    test(
        "${loceania.department.name} + ' Department'",
        "Marketing Department",
    );
    test("${loceania}? 'x' : 34", "x");
    test("!${loceania}? 'x' : 34", "34");
    test("${false}? 'x'", "");
    test("${loceania.name}?: 'nobody'", "Leslie Oceania");
    test("${loceania.name} ?: 'nobody'", "Leslie Oceania");
    test("${loceania.name}  ?:'nobody'", "Leslie Oceania");
    test("${null}  ?:'nobody'", "nobody");
    test(
        "${loceania.name != jafrica.name}? (23 - 3) / 10 : 'Number ' + 3 + 2 + '.'",
        "2",
    );
    test(
        "!${loceania.name != jafrica.name}? (23 - 3) / 10 : 'Number ' + (3 + 2) + '.'",
        "Number 5.",
    );
    test("'.' + 3 + 2", ".32");
    test("3 + '.' + 2", "3.2");
    test("3 + 2 + '.'", "5.");
    test("'Number ' + 3 + 2 + '.'", "Number 32.");
    test("'true'? 'x' : 'y'", "x");
    test("'false'? 'x' : 'y'", "y");
    test("2 + 1", "3");
    test("'2' + '1'", "21");
    test("'2' + 1", "21");
    test("2 + '1'", "21");
    test("-3 -4 - -5 + -10 - 1", "-13");
    test("-3 -4 - (-5 + -10)", "8");
    test("${size}", "Size is 5");
    test("${'x' + size}", "xSize is 5");
    test("${size + 'y'}", "Size is 5y");
    test("${'x' + size + 'y'}", "xSize is 5y");
    test("'The value is ' + 34 + (- 1 + 88.2)", "The value is 3487.2");
    test("(${currentYear} - #{company.yearfounded})", "35");
}

#[test]
fn expression_comparisons_and_logical() {
    test("${loceania.priority} &gt;=  ${meurope.priority}", "false");
    test("${loceania.priority} &gt;=  ${jafrica.priority}", "true");
    test("${loceania.priority} &lt;  ${meurope.priority}", "true");
    test("${loceania.priority} &lt;=  ${jafrica.priority}", "true");
    test("${meurope.priority} &gt;  ${loceania.priority}", "true");
    test("${loceania.name} &lt; 'Mare Nostrum'", "true");
    test("${true} and ${true}", "true");
    test("${true} AND ${true}", "true");
    test("'x' == 'x'", "true");
    test("'4' == 4", "true");
    test("'x' EQ 'x'", "true");
    test("'4' eq 4", "true");
    test("'x' NEQ 'x'", "false");
    test("'4' neq 4", "false");
    test("5 gt 4", "true");
    test("5 GE 4", "true");
    test("5 GT 5", "false");
    test("5 ge 5", "true");
    test("4 lt 5", "true");
    test("4 LE 5", "true");
    test("5 LT 5", "false");
    test("5 le 5", "true");
    test("#{priority.basic} &gt;= 3", "true");
    test("#{priority.basic} &gt;= 4", "false");
    test("#{priority.basic} + 10", "13");
}

#[test]
fn expression_indexing_and_preprocessing() {
    test("${loceania.permissions[0]}", "Event Organizer");
    test("${loceania.permissions[3 - 2]}", "Marketing Worldwide Head");
    test("${loceania.permissions[__3.3 - 1.3__]}", "Office Master");
    test("${pamerica.permissions} == ${meurope.permissions}", "true");
    test(
        "${pamerica.permissions} == ${loceania.permissions}",
        "false",
    );
    test("${loceania.permissions} == ${meurope.permissions}", "false");
}

#[test]
fn expression_calendars_and_messages() {
    test_with_context(
        "${#calendars.monthName(loceania.creationDate)}",
        "noviembre",
        &context_es(),
    );
    test("${#calendars.monthName(loceania.creationDate)}", "November");
    test_with_context(
        "${#calendars.monthName(loceania.creationDate)} == 'noviembre'",
        "true",
        &context_es(),
    );
    test_with_context(
        "${#calendars.monthName(loceania.creationDate)} == 'Noviembre'",
        "false",
        &context_es(),
    );
    test_with_context(
        "${#calendars.monthName(loceania.creationDate)} != 'Noviembre'",
        "true",
        &context_es(),
    );
    test("#{application.name}", "Thymeleaf test");
    test(
        "#{hello.message(${pamerica.name})}",
        "Hello, Petronila America!",
    );
    test(
        "#{today(${jafrica.creationDate.time},${jafrica.name})}",
        "Today is 23/09/2010, so hello Jacques Africa!",
    );
    test(
        "#{'title.dept.' + ${loceania.department.name}}",
        "The almighty Marketing department",
    );
    test(
        "#{'title.user.' + ${meurope.login}(${meurope.name}, ${meurope.department.name})}",
        "User Markus Europe works for the Engineering and Consulting department",
    );
}

#[test]
fn expression_link_parameters() {
    test("@{http://a.b.com}", "http://a.b.com");
    test("@{http://a.b.com/xx}", "http://a.b.com/xx");
    test(
        "@{http://a.b.com/xx/yy(p1='zz')}",
        "http://a.b.com/xx/yy?p1=zz",
    );
    test(
        "@{http://a.b.com/xx/yy(p1='zz', p2=${pamerica.name})}",
        "http://a.b.com/xx/yy?p1=zz&amp;p2=Petronila%20America",
    );
    test(
        "@{http://a.b.com/xx/yy#frag(p1='zz', p2=${pamerica.name})}",
        "http://a.b.com/xx/yy?p1=zz&amp;p2=Petronila%20America#frag",
    );
    test(
        "@{http://a.b.com/xx/yy#frag(p1='zz', p2=((!${pamerica.isAdmin()})? ${pamerica.login} : 'Admin'))}",
        "http://a.b.com/xx/yy?p1=zz&amp;p2=pamerica#frag",
    );
    test(
        "@{'http://a.b.com/xx/yy' + '#frag'(p1='zz', p2=((!${pamerica.isAdmin()})? ${pamerica.login} : 'Admin'))}",
        "http://a.b.com/xx/yy?p1=zz&amp;p2=pamerica#frag",
    );
    test(
        "@{('http://a.b.com/xx/yy' + '#frag')(p1='zz', p2=((!${pamerica.isAdmin()})? ${pamerica.login} : 'Admin'))}",
        "http://a.b.com/xx/yy?p1=zz&amp;p2=pamerica#frag",
    );
    test(
        "@{http://a.b.com/xx/yy(p1=(${pamerica.priority} == 1))}",
        "http://a.b.com/xx/yy?p1=true",
    );
    test(
        "@{http://a.b.com/xx/yy(p1=(${pamerica.login} == 'pamerica'))}",
        "http://a.b.com/xx/yy?p1=true",
    );
    test("@{http://a.b.com/xx/yy(p1)}", "http://a.b.com/xx/yy?p1");
    test(
        "@{http://a.b.com/xx/yy(p1, p2=${pamerica.name})}",
        "http://a.b.com/xx/yy?p1&amp;p2=Petronila%20America",
    );
    test(
        "@{http://a.b.com/xx/yy(p1='zz', p2)}",
        "http://a.b.com/xx/yy?p1=zz&amp;p2",
    );
    test(
        "@{http://a.b.com/xx/yy(p1, p2)}",
        "http://a.b.com/xx/yy?p1&amp;p2",
    );
}

#[test]
fn expression_link_context_relative() {
    test("@{~/xx/yy}", "/xx/yy");
    test("@{~/xx/yy(p1)}", "/xx/yy?p1");
    test(
        "@{~/xx/yy(p1, p2=${pamerica.name})}",
        "/xx/yy?p1&amp;p2=Petronila%20America",
    );
    test(
        "@{~/xx/yy(a[0]=${pamerica.name},a[1]=${pamerica.name})}",
        "/xx/yy?a%5B0%5D=Petronila%20America&amp;a%5B1%5D=Petronila%20America",
    );
    test(
        "@{~/xx/yy(login=${logins})}",
        "/xx/yy?login=loceania&amp;login=meurope&amp;login=jafrica&amp;login=pamerica",
    );
    test(
        "@{~/xx/yy(login=${loginsArray})}",
        "/xx/yy?login=loceania&amp;login=meurope&amp;login=jafrica&amp;login=pamerica",
    );
    test(
        "@{~/xx/yy(a[0]=${pamerica.name},a[0]=${pamerica.name})}",
        "/xx/yy?a%5B0%5D=Petronila%20America&amp;a%5B0%5D=Petronila%20America",
    );
    test(
        "@{~/xx/yy/{name}(name=${pamerica.name})}",
        "/xx/yy/Petronila%20America",
    );
    test(
        "@{~/xx/{name}/yy(name=${pamerica.name})}",
        "/xx/Petronila%20America/yy",
    );
    test("@{~/xx/{name}(name=#{sum})}", "/xx/1+1=2");
    test("@{~/xx(name=#{sum})}", "/xx?name=1%2B1%3D2");
    test(
        "@{~/xx/{name}/yy?(name=${pamerica.name})}",
        "/xx/Petronila%20America/yy?",
    );
    test(
        "@{~/xx/{/name}/yy(name=${pamerica.name})}",
        "/xx/Petronila%20America/yy",
    );
    test("@{~/xx/{date}(date=#{dateForPath})}", "/xx/10/10/1976");
    test("@{~/xx/{/date}(date=#{dateForPath})}", "/xx/10%2F10%2F1976");
    test(
        "@{~/xx/{/date1}/yy/{date2}(date1=#{dateForPath},date2=#{dateForPath})}",
        "/xx/10%2F10%2F1976/yy/10/10/1976",
    );
    test(
        "@{~/xx/{/date}(date=#{dateForPath},date2=#{dateForPath})}",
        "/xx/10%2F10%2F1976?date2=10/10/1976",
    );
}

#[test]
fn expression_string_number_coercion() {
    test("${13 + '13'} == (13 + '13')", "true");
    test("${13 + '13.0'} == (13 + '13.0')", "true");
    test("${'13' + '13'} == ('13' + '13')", "true");
    test("${'13' + 13.0} == ('13' + 13.0)", "true");
    test("${13 + 13.0} == (13 + 13.0)", "true");
    test("${13 == '13.0'}", "true");
    test("${13 == '13'} == (13 == '13')", "true");
    test("${13 != '13'} == (13 != '13')", "true");
    test("${13 == 13.0} == (13 == 13.0)", "true");
    test("${13 != 13.0} == (13 != 13.0)", "true");
    test("${13 &gt;= '13'} == (13 &gt;= '13')", "true");
    test("${13 &gt; '13'} == (13 &gt; '13')", "true");
    test("${13 &gt;= 13.0} == (13 &gt;= 13.0)", "true");
    test("${13 &gt; 13.0} == (13 &gt; 13.0)", "true");
    test("${13 &lt;= '13'} == (13 &lt;= '13')", "true");
    test("${13 &lt; '13'} == (13 &lt; '13')", "true");
    test("${13 &lt;= 13.0} == (13 &lt;= 13.0)", "true");
    test("${13 &lt; 13.0} == (13 &lt; 13.0)", "true");
    test("${13 == '13.0'} == (13 == '13.0')", "true");
    test("${13 != '13.0'} == (13 != '13.0')", "true");
    test("${13 &gt;= '13.0'} == (13 &gt;= '13.0')", "true");
    test("${13 &gt; '13.0'} == (13 &gt; '13.0')", "true");
    test("${13 &lt;= '13.0'} == (13 &lt;= '13.0')", "true");
    test("${13 &lt; '13.0'} == (13 &lt; '13.0')", "true");
}

#[test]
fn expression_number_comparison_14() {
    test("${14 == '13'} == (14 == '13')", "true");
    test("${14 != '13'} == (14 != '13')", "true");
    test("${14 == 13.0} == (14 == 13.0)", "true");
    test("${14 != 13.0} == (14 != 13.0)", "true");
    test("${14 &gt;= '13'} == (14 &gt;= '13')", "true");
    test("${14 &gt; '13'} == (14 &gt; '13')", "true");
    test("${14 &gt;= 13.0} == (14 &gt;= 13.0)", "true");
    test("${14 &gt; 13.0} == (14 &gt; 13.0)", "true");
    test("${14 &lt;= '13'} == (14 &lt;= '13')", "true");
    test("${14 &lt; '13'} == (14 &lt; '13')", "true");
    test("${14 &lt;= 13.0} == (14 &lt;= 13.0)", "true");
    test("${14 &lt; 13.0} == (14 &lt; 13.0)", "true");
    test("${14 == '13.0'} == (14 == '13.0')", "true");
    test("${14 != '13.0'} == (14 != '13.0')", "true");
    test("${14 &gt;= '13.0'} == (14 &gt;= '13.0')", "true");
    test("${14 &gt; '13.0'} == (14 &gt; '13.0')", "true");
    test("${14 &lt;= '13.0'} == (14 &lt;= '13.0')", "true");
    test("${14 &lt; '13.0'} == (14 &lt; '13.0')", "true");
    test("${13 == '14'} == (13 == '14')", "true");
    test("${13 != '14'} == (13 != '14')", "true");
    test("${13 == 14.0} == (13 == 14.0)", "true");
    test("${13 != 14.0} == (13 != 14.0)", "true");
    test("${13 &gt;= '14'} == (13 &gt;= '14')", "true");
    test("${13 &gt; '14'} == (13 &gt; '14')", "true");
    test("${13 &gt;= 14.0} == (13 &gt;= 14.0)", "true");
    test("${13 &gt; 14.0} == (13 &gt; 14.0)", "true");
    test("${13 &lt;= '14'} == (13 &lt;= '14')", "true");
    test("${13 &lt; '14'} == (13 &lt; '14')", "true");
    test("${13 &lt;= 14.0} == (13 &lt;= 14.0)", "true");
    test("${13 &lt; 14.0} == (13 &lt; 14.0)", "true");
    test("${13 == '14.0'} == (13 == '14.0')", "true");
    test("${13 != '14.0'} == (13 != '14.0')", "true");
    test("${13 &gt;= '14.0'} == (13 &gt;= '14.0')", "true");
    test("${13 &gt; '14.0'} == (13 &gt; '14.0')", "true");
    test("${13 &lt;= '14.0'} == (13 &lt;= '14.0')", "true");
    test("${13 &lt; '14.0'} == (13 &lt; '14.0')", "true");
}

#[test]
fn expression_fragments_and_noop() {
    test(
        "~{::body}",
        "&lt;body&gt;&lt;span th:text=&quot;~{::body}&quot;&gt;PLACEHOLDER&lt;/span&gt;&lt;/body&gt;",
    );
    test("~{::span/text()}", "PLACEHOLDER");
    test(
        "~{whatever}",
        "&lt;!DOCTYPE html&gt;&lt;html&gt;&lt;body&gt;&lt;span th:text=&quot;whatever&quot;&gt;PLACEHOLDER&lt;/span&gt;&lt;/body&gt;&lt;/html&gt;",
    );
    test(
        "~{whatever::body}",
        "&lt;body&gt;&lt;span th:text=&quot;whatever&quot;&gt;PLACEHOLDER&lt;/span&gt;&lt;/body&gt;",
    );
    test(
        "~{whatever::body(92)}",
        "&lt;body&gt;&lt;span th:text=&quot;whatever&quot;&gt;PLACEHOLDER&lt;/span&gt;&lt;/body&gt;",
    );
    test(
        "~{::body(92)}",
        "&lt;body&gt;&lt;span th:text=&quot;~{::body(92)}&quot;&gt;PLACEHOLDER&lt;/span&gt;&lt;/body&gt;",
    );
    test("~{::doctype()}", "&lt;!DOCTYPE html&gt;");
    test("_", "PLACEHOLDER");
    test("${true} ? _", "PLACEHOLDER");
    test("${false} ? _", "");
    test("${'this'} ?: _", "this");
    test("${null} ?: _", "PLACEHOLDER");
    test("${true} ? ${'this'} : _", "this");
    test("${false} ? ${'this'} : _", "PLACEHOLDER");
    test("pepito_", "pepito_");
    test("pep_ito_", "pep_ito_");
    test("_pep_ito_", "_pep_ito_");
}
