use std::sync::Arc;

use crate::util::JavaString;

use super::TemplateFragmentMarkupReferenceResolver;

/// AttoParser markup selector 的项目内等价执行器。
///
/// 该类型是第三方组件替换设施，不计入 Thymeleaf Java 对象分母。它保留路径层级、
/// 属性布尔表达式、HTML id/class 简写、fragment reference 和 sibling index 语义。
pub(crate) struct MarkupSelectorEngine {
    html: bool,
    selectors: Vec<CompiledSelector>,
}

impl MarkupSelectorEngine {
    pub(crate) fn new(
        html: bool,
        selectors: &[JavaString],
        reference_resolver: Option<Arc<TemplateFragmentMarkupReferenceResolver>>,
    ) -> Result<Self, String> {
        let selectors = selectors
            .iter()
            .map(|selector| {
                CompiledSelector::parse(html, selector.clone(), reference_resolver.as_deref())
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { html, selectors })
    }

    pub(crate) fn matching_element_selectors(&self, path: &[SelectorNode]) -> Vec<JavaString> {
        self.selectors
            .iter()
            .filter(|selector| selector.matches_path(self.html, path))
            .map(|selector| selector.original.clone())
            .collect()
    }

    pub(crate) fn matching_event_selectors(
        &self,
        ancestor_path: &[SelectorNode],
        node_type: SelectorNodeType,
        preceding_siblings: Arc<Vec<SelectorNodeSummary>>,
    ) -> Vec<JavaString> {
        let mut path = ancestor_path.to_vec();
        path.push(SelectorNode::event(node_type, preceding_siblings));
        self.selectors
            .iter()
            .filter(|selector| selector.matches_path(self.html, &path))
            .map(|selector| selector.original.clone())
            .collect()
    }

    pub(crate) fn selects_content_of(&self, path: &[SelectorNode]) -> bool {
        let mut content_path = path.to_vec();
        content_path.push(SelectorNode::event(
            SelectorNodeType::Content,
            Arc::new(Vec::new()),
        ));
        self.selectors
            .iter()
            .any(|selector| selector.matches_path(self.html, &content_path))
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum SelectorNodeType {
    Element,
    Content,
    Text,
    Comment,
    Cdata,
    DocType,
    XmlDeclaration,
    ProcessingInstruction,
}

#[derive(Clone)]
pub(crate) struct SelectorNode {
    node_type: SelectorNodeType,
    name: String,
    attributes: Vec<(String, Option<String>)>,
    preceding_siblings: Arc<Vec<SelectorNodeSummary>>,
}

impl SelectorNode {
    #[expect(
        dead_code,
        reason = "保留 AttoParser AUTO_OPEN_CLOSE 选择器节点语义；Thymeleaf HTML 解析固定使用 AUTO_CLOSE"
    )]
    pub(crate) fn synthetic_element(
        html: bool,
        name: &str,
        preceding_siblings: Arc<Vec<SelectorNodeSummary>>,
        injected_attributes: &[(JavaString, Option<JavaString>)],
    ) -> Self {
        Self {
            node_type: SelectorNodeType::Element,
            name: normalize_name(html, name),
            attributes: injected_attributes
                .iter()
                .map(|(name, value)| {
                    (
                        normalize_name(html, &name.to_string_lossy()),
                        value.as_ref().map(JavaString::to_string_lossy),
                    )
                })
                .collect(),
            preceding_siblings,
        }
    }

    pub(crate) fn from_tag(
        html: bool,
        source: &str,
        name_start: usize,
        name_end: usize,
        content_end: usize,
        preceding_siblings: Arc<Vec<SelectorNodeSummary>>,
        injected_attributes: &[(JavaString, Option<JavaString>)],
    ) -> Self {
        let name = normalize_name(html, &source[name_start..name_end]);
        let mut attributes = parse_attributes(html, source, name_end, content_end);
        attributes.extend(injected_attributes.iter().map(|(name, value)| {
            (
                normalize_name(html, &name.to_string_lossy()),
                value.as_ref().map(JavaString::to_string_lossy),
            )
        }));
        Self {
            node_type: SelectorNodeType::Element,
            name,
            attributes,
            preceding_siblings,
        }
    }

    pub(crate) fn event(
        node_type: SelectorNodeType,
        preceding_siblings: Arc<Vec<SelectorNodeSummary>>,
    ) -> Self {
        Self {
            node_type,
            name: String::new(),
            attributes: Vec::new(),
            preceding_siblings,
        }
    }

    pub(crate) fn summary(&self) -> SelectorNodeSummary {
        SelectorNodeSummary {
            node_type: self.node_type,
            name: self.name.clone(),
            attributes: self.attributes.clone(),
        }
    }
}

#[derive(Clone)]
pub(crate) struct SelectorNodeSummary {
    node_type: SelectorNodeType,
    name: String,
    attributes: Vec<(String, Option<String>)>,
}

struct CompiledSelector {
    original: JavaString,
    levels: Vec<SelectorLevel>,
}

impl CompiledSelector {
    fn parse(
        html: bool,
        original: JavaString,
        reference_resolver: Option<&TemplateFragmentMarkupReferenceResolver>,
    ) -> Result<Self, String> {
        let text = original.to_string_lossy();
        if text.trim().is_empty() {
            return Err("Selector cannot be null".to_owned());
        }
        let normalized = if text.trim_start().starts_with('/') {
            text.trim().to_owned()
        } else {
            format!("//{}", text.trim())
        };
        let raw_levels = split_levels(&normalized)?;
        let mut levels = Vec::with_capacity(raw_levels.len());
        for (any_level, level) in raw_levels {
            levels.push(SelectorLevel::parse(
                html,
                any_level,
                level,
                reference_resolver,
            )?);
        }
        Ok(Self { original, levels })
    }

    fn matches_path(&self, html: bool, path: &[SelectorNode]) -> bool {
        if self.levels.is_empty() || path.is_empty() {
            return false;
        }
        self.matches_level(html, self.levels.len() - 1, path.len() - 1, path)
    }

    fn matches_level(
        &self,
        html: bool,
        level_index: usize,
        node_index: usize,
        path: &[SelectorNode],
    ) -> bool {
        let level = &self.levels[level_index];
        if !level.matches_node(html, &path[node_index]) {
            return false;
        }
        if level_index == 0 {
            return level.any_level || node_index == 0;
        }
        if level.any_level {
            (0..node_index)
                .rev()
                .any(|candidate| self.matches_level(html, level_index - 1, candidate, path))
        } else if node_index > 0 {
            self.matches_level(html, level_index - 1, node_index - 1, path)
        } else {
            false
        }
    }
}

struct SelectorLevel {
    any_level: bool,
    node_type: SelectorNodeType,
    element_name: Option<String>,
    attribute_condition: Option<AttributeCondition>,
    index_condition: Option<IndexCondition>,
    reference_condition: Option<ReferenceCondition>,
    plain_name_can_be_reference: bool,
}

impl SelectorLevel {
    fn parse(
        html: bool,
        any_level: bool,
        text: &str,
        reference_resolver: Option<&TemplateFragmentMarkupReferenceResolver>,
    ) -> Result<Self, String> {
        let (mut path, modifiers) = split_path_modifiers(text)?;
        let mut attribute_condition = None;
        let mut index_condition = None;

        let id_position = if html {
            find_unescaped(&path, '#')
        } else {
            None
        };
        let class_position = if html {
            find_unescaped(&path, '.')
        } else {
            None
        };
        let reference_position = find_unescaped(&path, '%');
        let modifier_count = usize::from(id_position.is_some())
            + usize::from(class_position.is_some())
            + usize::from(reference_position.is_some());
        if modifier_count > 1 {
            return Err(format!(
                "More than one modifier (id, class, reference) has been specified in selector level \"{text}\""
            ));
        }

        let mut reference = None;
        if let Some(position) = id_position {
            let value = &path[position + 1..];
            if value.trim().is_empty() {
                return Err("Empty id modifier in selector expression".to_owned());
            }
            attribute_condition = Some(AttributeCondition::Compare {
                name: "id".to_owned(),
                operator: AttributeOperator::Equals,
                value: value.to_owned(),
            });
            path.truncate(position);
        } else if let Some(position) = class_position {
            let value = &path[position + 1..];
            if value.trim().is_empty() {
                return Err("Empty class modifier in selector expression".to_owned());
            }
            attribute_condition = Some(AttributeCondition::Compare {
                name: "class".to_owned(),
                operator: AttributeOperator::Equals,
                value: value.to_owned(),
            });
            path.truncate(position);
        } else if let Some(position) = reference_position {
            let value = &path[position + 1..];
            if value.trim().is_empty() {
                return Err("Empty reference modifier in selector expression".to_owned());
            }
            reference = Some(value.to_owned());
            path.truncate(position);
        }

        for modifier in modifiers {
            if let Some(index) = IndexCondition::parse(modifier.trim()) {
                if index_condition.is_some() {
                    return Err("Cannot combine two index modifiers".to_owned());
                }
                index_condition = Some(index);
            } else {
                let condition = AttributeExpressionParser::new(html, modifier).parse()?;
                attribute_condition = Some(match attribute_condition {
                    Some(existing) => AttributeCondition::Relation {
                        relation: AttributeRelation::And,
                        left: Box::new(existing),
                        right: Box::new(condition),
                    },
                    None => condition,
                });
            }
        }

        let path = path.trim();
        let node_type = match path {
            "content()" => SelectorNodeType::Content,
            "text()" => SelectorNodeType::Text,
            "comment()" => SelectorNodeType::Comment,
            "cdata()" => SelectorNodeType::Cdata,
            "doctype()" => SelectorNodeType::DocType,
            "xmldecl()" => SelectorNodeType::XmlDeclaration,
            "procinstr()" => SelectorNodeType::ProcessingInstruction,
            _ => SelectorNodeType::Element,
        };
        let element_name = (node_type == SelectorNodeType::Element && !path.is_empty())
            .then(|| normalize_name(html, path));
        let plain_name_can_be_reference =
            reference.is_none() && element_name.is_some() && reference_resolver.is_some();
        let reference_condition = reference
            .or_else(|| plain_name_can_be_reference.then(|| path.to_owned()))
            .map(|value| ReferenceCondition {
                value,
                attribute_names: reference_resolver
                    .map(TemplateFragmentMarkupReferenceResolver::reference_attribute_names)
                    .unwrap_or_default(),
            });
        Ok(Self {
            any_level,
            node_type,
            element_name,
            attribute_condition,
            index_condition,
            reference_condition,
            plain_name_can_be_reference,
        })
    }

    fn matches_node(&self, html: bool, node: &SelectorNode) -> bool {
        if self.node_type != node.node_type {
            return false;
        }
        let name_matches = self
            .element_name
            .as_ref()
            .is_none_or(|name| name == &node.name);
        let reference_matches = self
            .reference_condition
            .as_ref()
            .is_some_and(|reference| reference.matches(html, &node.attributes));
        let path_matches = if self.plain_name_can_be_reference {
            name_matches || reference_matches
        } else {
            name_matches
                && self
                    .reference_condition
                    .as_ref()
                    .is_none_or(|_| reference_matches)
        };
        if !path_matches
            || !self
                .attribute_condition
                .as_ref()
                .is_none_or(|condition| condition.matches(html, &node.attributes))
        {
            return false;
        }
        let Some(index_condition) = self.index_condition else {
            return true;
        };
        let sibling_index = node
            .preceding_siblings
            .iter()
            .filter(|sibling| self.matches_summary_without_index(html, sibling))
            .count();
        index_condition.matches(sibling_index)
    }

    fn matches_summary_without_index(&self, html: bool, node: &SelectorNodeSummary) -> bool {
        if self.node_type != node.node_type {
            return false;
        }
        let name_matches = self
            .element_name
            .as_ref()
            .is_none_or(|name| name == &node.name);
        let reference_matches = self
            .reference_condition
            .as_ref()
            .is_some_and(|reference| reference.matches(html, &node.attributes));
        let path_matches = if self.plain_name_can_be_reference {
            name_matches || reference_matches
        } else {
            name_matches
                && self
                    .reference_condition
                    .as_ref()
                    .is_none_or(|_| reference_matches)
        };
        path_matches
            && self
                .attribute_condition
                .as_ref()
                .is_none_or(|condition| condition.matches(html, &node.attributes))
    }
}

struct ReferenceCondition {
    value: String,
    attribute_names: Vec<String>,
}

impl ReferenceCondition {
    fn matches(&self, _html: bool, attributes: &[(String, Option<String>)]) -> bool {
        let value = &self.value;
        attributes.iter().any(|(name, attribute_value)| {
            let recognized = self
                .attribute_names
                .iter()
                .any(|attribute_name| attribute_name == name);
            recognized
                && attribute_value.as_ref().is_some_and(|candidate| {
                    candidate == value
                        || candidate.starts_with(&format!("{value}("))
                        || candidate.starts_with(&format!("{value} ("))
                })
        })
    }
}

enum AttributeCondition {
    Exists {
        name: String,
        negated: bool,
    },
    Compare {
        name: String,
        operator: AttributeOperator,
        value: String,
    },
    Relation {
        relation: AttributeRelation,
        left: Box<AttributeCondition>,
        right: Box<AttributeCondition>,
    },
}

impl AttributeCondition {
    fn matches(&self, html: bool, attributes: &[(String, Option<String>)]) -> bool {
        match self {
            Self::Exists { name, negated } => {
                let exists = find_attribute(html, attributes, name).is_some();
                if *negated { !exists } else { exists }
            }
            Self::Compare {
                name,
                operator,
                value,
            } => attributes
                .iter()
                .filter(|(candidate_name, _)| names_equal(html, candidate_name, name))
                .any(|(_, candidate)| {
                    let candidate = candidate.as_deref().unwrap_or("");
                    if html && name.eq_ignore_ascii_case("class") {
                        if candidate.is_empty() {
                            value.trim().is_empty()
                        } else {
                            candidate
                                .split(char::is_whitespace)
                                .filter(|token| !token.is_empty())
                                .any(|token| operator.matches(token, value))
                        }
                    } else {
                        operator.matches(candidate, value)
                    }
                }),
            Self::Relation {
                relation,
                left,
                right,
            } => match relation {
                AttributeRelation::And => {
                    left.matches(html, attributes) && right.matches(html, attributes)
                }
                AttributeRelation::Or => {
                    left.matches(html, attributes) || right.matches(html, attributes)
                }
            },
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum AttributeOperator {
    Equals,
    NotEquals,
    StartsWith,
    EndsWith,
    Contains,
}

impl AttributeOperator {
    fn matches(self, candidate: &str, expected: &str) -> bool {
        match self {
            Self::Equals => candidate == expected,
            Self::NotEquals => candidate != expected,
            Self::StartsWith => candidate.starts_with(expected),
            Self::EndsWith => candidate.ends_with(expected),
            Self::Contains => candidate.contains(expected),
        }
    }
}

enum AttributeRelation {
    And,
    Or,
}

#[derive(Clone, Copy)]
enum IndexCondition {
    Exact(i32),
    Even,
    Odd,
    Greater(i32),
    Lesser(i32),
}

impl IndexCondition {
    fn parse(value: &str) -> Option<Self> {
        let value = value.trim();
        if value.eq_ignore_ascii_case("even()") {
            Some(Self::Even)
        } else if value.eq_ignore_ascii_case("odd()") {
            Some(Self::Odd)
        } else if let Some(number) = value.strip_prefix('>') {
            number.trim().parse().ok().map(Self::Greater)
        } else if let Some(number) = value.strip_prefix('<') {
            number.trim().parse().ok().map(Self::Lesser)
        } else {
            value.parse().ok().map(Self::Exact)
        }
    }

    fn matches(self, index: usize) -> bool {
        let index = i32::try_from(index).unwrap_or(i32::MAX);
        match self {
            Self::Exact(expected) => index == expected,
            Self::Even => index % 2 == 0,
            Self::Odd => index % 2 == 1,
            Self::Greater(expected) => index > expected,
            Self::Lesser(expected) => index < expected,
        }
    }
}

struct AttributeExpressionParser<'a> {
    html: bool,
    input: &'a str,
    position: usize,
}

impl<'a> AttributeExpressionParser<'a> {
    const fn new(html: bool, input: &'a str) -> Self {
        Self {
            html,
            input,
            position: 0,
        }
    }

    fn parse(mut self) -> Result<AttributeCondition, String> {
        let result = self.parse_relation()?;
        self.skip_space();
        if self.position != self.input.len() {
            return Err(format!(
                "Invalid attribute selector expression near \"{}\"",
                &self.input[self.position..]
            ));
        }
        Ok(result)
    }

    fn parse_relation(&mut self) -> Result<AttributeCondition, String> {
        let left = self.parse_primary()?;
        self.skip_space();
        let relation = if self.consume_keyword("and") {
            Some(AttributeRelation::And)
        } else if self.consume_keyword("or") {
            Some(AttributeRelation::Or)
        } else {
            None
        };
        if let Some(relation) = relation {
            let right = self.parse_relation()?;
            Ok(AttributeCondition::Relation {
                relation,
                left: Box::new(left),
                right: Box::new(right),
            })
        } else {
            Ok(left)
        }
    }

    fn parse_primary(&mut self) -> Result<AttributeCondition, String> {
        self.skip_space();
        if self.consume_char('(') {
            let value = self.parse_relation()?;
            self.skip_space();
            if !self.consume_char(')') {
                return Err("Unclosed parenthesis in attribute selector".to_owned());
            }
            return Ok(value);
        }
        let negated = self.consume_char('!');
        let _ = self.consume_char('@');
        let html = self.html;
        let name = self.parse_name();
        if name.is_empty() {
            return Err("Attribute name cannot be empty".to_owned());
        }
        let name = normalize_name(html, name);
        self.skip_space();
        let Some(operator) = self.parse_operator() else {
            return Ok(AttributeCondition::Exists { name, negated });
        };
        if negated {
            return Err("Negation cannot be combined with an attribute operator".to_owned());
        }
        self.skip_space();
        let value = self.parse_value()?;
        Ok(AttributeCondition::Compare {
            name,
            operator,
            value,
        })
    }

    fn parse_name(&mut self) -> &str {
        let start = self.position;
        while self.position < self.input.len() {
            let character = self.input[self.position..]
                .chars()
                .next()
                .expect("position is a character boundary");
            if character.is_whitespace()
                || matches!(character, '=' | '!' | '^' | '$' | '*' | '(' | ')')
            {
                break;
            }
            self.position += character.len_utf8();
        }
        &self.input[start..self.position]
    }

    fn parse_operator(&mut self) -> Option<AttributeOperator> {
        for (text, operator) in [
            ("!=", AttributeOperator::NotEquals),
            ("^=", AttributeOperator::StartsWith),
            ("$=", AttributeOperator::EndsWith),
            ("*=", AttributeOperator::Contains),
            ("=", AttributeOperator::Equals),
        ] {
            if self.input[self.position..].starts_with(text) {
                self.position += text.len();
                return Some(operator);
            }
        }
        None
    }

    fn parse_value(&mut self) -> Result<String, String> {
        let Some(first) = self.input[self.position..].chars().next() else {
            return Err("Attribute selector value must be quoted".to_owned());
        };
        if first == '\'' || first == '"' {
            self.position += first.len_utf8();
            let start = self.position;
            while self.position < self.input.len() {
                let character = self.input[self.position..]
                    .chars()
                    .next()
                    .expect("position is a character boundary");
                if character == first {
                    let value = self.input[start..self.position].to_owned();
                    self.position += character.len_utf8();
                    return Ok(value);
                }
                self.position += character.len_utf8();
            }
            Err("Unclosed quoted attribute selector value".to_owned())
        } else {
            Err("Attribute selector value must be quoted".to_owned())
        }
    }

    fn skip_space(&mut self) {
        while self.position < self.input.len() {
            let character = self.input[self.position..]
                .chars()
                .next()
                .expect("position is a character boundary");
            if !character.is_whitespace() {
                break;
            }
            self.position += character.len_utf8();
        }
    }

    fn consume_keyword(&mut self, keyword: &str) -> bool {
        let remaining = &self.input[self.position..];
        if !remaining
            .get(..keyword.len())
            .is_some_and(|candidate| candidate.eq_ignore_ascii_case(keyword))
        {
            return false;
        }
        let after = self.position + keyword.len();
        if after < self.input.len()
            && self.input[after..]
                .chars()
                .next()
                .is_some_and(|character| !character.is_whitespace())
        {
            return false;
        }
        self.position = after;
        true
    }

    fn consume_char(&mut self, expected: char) -> bool {
        if self.input[self.position..].starts_with(expected) {
            self.position += expected.len_utf8();
            true
        } else {
            false
        }
    }
}

fn split_levels(selector: &str) -> Result<Vec<(bool, &str)>, String> {
    let mut levels = Vec::new();
    let mut position = 0;
    while position < selector.len() {
        let any_level = if selector[position..].starts_with("//") {
            position += 2;
            true
        } else if selector[position..].starts_with('/') {
            position += 1;
            false
        } else {
            return Err("Selector level must start with '/' or '//'".to_owned());
        };
        let start = position;
        let mut bracket_depth = 0_i32;
        let mut quote = None;
        while position < selector.len() {
            let character = selector[position..]
                .chars()
                .next()
                .expect("position is a character boundary");
            if let Some(expected) = quote {
                if character == expected {
                    quote = None;
                }
            } else {
                match character {
                    '\'' | '"' => quote = Some(character),
                    '[' => bracket_depth += 1,
                    ']' => bracket_depth -= 1,
                    '/' if bracket_depth == 0 => break,
                    _ => {}
                }
            }
            position += character.len_utf8();
        }
        if position == start {
            return Err("Empty selector level".to_owned());
        }
        levels.push((any_level, &selector[start..position]));
    }
    Ok(levels)
}

fn split_path_modifiers(value: &str) -> Result<(String, Vec<&str>), String> {
    let first = value.find('[').unwrap_or(value.len());
    let path = value[..first].to_owned();
    let mut modifiers = Vec::new();
    let mut position = first;
    while position < value.len() {
        if !value[position..].starts_with('[') {
            return Err("Invalid selector modifier syntax".to_owned());
        }
        position += 1;
        let start = position;
        let mut depth = 1_i32;
        let mut quote = None;
        while position < value.len() {
            let character = value[position..]
                .chars()
                .next()
                .expect("position is a character boundary");
            if let Some(expected) = quote {
                if character == expected {
                    quote = None;
                }
            } else {
                match character {
                    '\'' | '"' => quote = Some(character),
                    '[' | '(' => depth += 1,
                    ']' | ')' => {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    _ => {}
                }
            }
            position += character.len_utf8();
        }
        if position >= value.len() {
            return Err("Unclosed selector modifier".to_owned());
        }
        modifiers.push(&value[start..position]);
        position += 1;
    }
    Ok((path, modifiers))
}

fn parse_attributes(
    html: bool,
    source: &str,
    mut position: usize,
    content_end: usize,
) -> Vec<(String, Option<String>)> {
    let mut result = Vec::new();
    while position < content_end {
        while position < content_end && source.as_bytes()[position].is_ascii_whitespace() {
            position += 1;
        }
        if position >= content_end {
            break;
        }
        let start = position;
        while position < content_end {
            let byte = source.as_bytes()[position];
            if byte.is_ascii_whitespace() || matches!(byte, b'=' | b'/' | b'>') {
                break;
            }
            position += source[position..].chars().next().map_or(1, char::len_utf8);
        }
        let name = normalize_name(html, &source[start..position]);
        while position < content_end && source.as_bytes()[position].is_ascii_whitespace() {
            position += 1;
        }
        let mut value = None;
        if position < content_end && source.as_bytes()[position] == b'=' {
            position += 1;
            while position < content_end && source.as_bytes()[position].is_ascii_whitespace() {
                position += 1;
            }
            if position < content_end {
                let quote = source.as_bytes()[position];
                if quote == b'\'' || quote == b'"' {
                    position += 1;
                    let value_start = position;
                    while position < content_end && source.as_bytes()[position] != quote {
                        position += source[position..].chars().next().map_or(1, char::len_utf8);
                    }
                    value = Some(source[value_start..position].to_owned());
                    if position < content_end {
                        position += 1;
                    }
                } else {
                    let value_start = position;
                    while position < content_end
                        && !source.as_bytes()[position].is_ascii_whitespace()
                    {
                        position += source[position..].chars().next().map_or(1, char::len_utf8);
                    }
                    value = Some(source[value_start..position].to_owned());
                }
            }
        }
        result.push((name, value));
    }
    result
}

fn find_attribute<'a>(
    html: bool,
    attributes: &'a [(String, Option<String>)],
    name: &str,
) -> Option<Option<&'a str>> {
    attributes
        .iter()
        .find(|(candidate, _)| {
            if html {
                candidate.eq_ignore_ascii_case(name)
            } else {
                candidate == name
            }
        })
        .map(|(_, value)| value.as_deref())
}

fn names_equal(html: bool, left: &str, right: &str) -> bool {
    if html {
        left.eq_ignore_ascii_case(right)
    } else {
        left == right
    }
}

fn normalize_name(html: bool, name: &str) -> String {
    if html {
        name.to_ascii_lowercase()
    } else {
        name.to_owned()
    }
}

fn find_unescaped(value: &str, expected: char) -> Option<usize> {
    value
        .char_indices()
        .find_map(|(index, character)| (character == expected).then_some(index))
}
