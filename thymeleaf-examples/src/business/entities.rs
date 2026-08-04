//! GTVG 业务实体 —— 对应 Java `business/entities/*.java`（6 个 POJO）。
//!
//! 实体实现 [`TemplateObject`]，通过 `java_get_property` 暴露 JavaBean 属性，
//! 供模板表达式（`${o.customer.name}`、`*{purchasePrice}` 等）按原语义读取。

use std::sync::Arc;

use thymeleaf::expression::{TemplateObject, TemplateValue};
use thymeleaf::util::{DateUtils, JavaBigDecimal, JavaDate, JavaNumber, Utf16String};

/// 构造 Java `Integer` 模板值。
fn num(value: i32) -> Arc<TemplateValue> {
    Arc::new(TemplateValue::Number(JavaNumber::Integer(value)))
}

/// 构造 Java `BigDecimal` 模板值（保持十进制字面量与 scale）。
fn decimal(value: &str) -> Arc<TemplateValue> {
    Arc::new(TemplateValue::Number(JavaNumber::BigDecimal(
        JavaBigDecimal::parse(value).expect("valid decimal"),
    )))
}

/// 构造 Java `String` 模板值。
fn text(value: &str) -> Arc<TemplateValue> {
    Arc::new(TemplateValue::string(Utf16String::from_rust_str(value)))
}

/// 构造 Java `Boolean` 模板值。
fn boolean(value: bool) -> Arc<TemplateValue> {
    Arc::new(TemplateValue::Boolean(value))
}

/// 构造 Java `Calendar` 模板值（保留时区语义的 Calendar 对象）。
fn calendar(value: JavaDate) -> Arc<TemplateValue> {
    DateUtils::into_template_value(value)
}

/// 把实体包装为 Java 对象模板值。
fn object<T: TemplateObject>(value: T) -> Arc<TemplateValue> {
    Arc::new(TemplateValue::Object(Arc::new(value)))
}

/// 构造 Java `List` 模板值。
fn list(values: Vec<Arc<TemplateValue>>) -> Arc<TemplateValue> {
    Arc::new(TemplateValue::List(Arc::new(values)))
}

/// 订单行 —— 对应 Java `OrderLine.java`。
#[derive(Clone, Debug)]
pub struct OrderLine {
    pub product: Product,
    pub amount: i32,
    pub purchase_price: String,
}

impl TemplateObject for OrderLine {
    fn java_class_name(&self) -> &str {
        "org.thymeleaf.examples.core.gtvg.jakarta.business.entities.OrderLine"
    }
    fn to_utf16_string(&self) -> Utf16String {
        Utf16String::from_rust_str(&self.product.name)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn java_get_property(
        &self,
        property_name: &Utf16String,
    ) -> Option<
        Result<Option<Arc<TemplateValue>>, thymeleaf::expression::TemplateObjectPropertyError>,
    > {
        Some(Ok(match property_name.to_string_lossy().as_str() {
            "product" => Some(object(self.product.clone())),
            "amount" => Some(num(self.amount)),
            "purchasePrice" => Some(decimal(&self.purchase_price)),
            _ => None,
        }))
    }
}

/// 产品 —— 对应 Java `Product.java`。
#[derive(Clone, Debug)]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub price: String,
    pub in_stock: bool,
    pub comments: Vec<Comment>,
}

impl TemplateObject for Product {
    fn java_class_name(&self) -> &str {
        "org.thymeleaf.examples.core.gtvg.jakarta.business.entities.Product"
    }
    fn to_utf16_string(&self) -> Utf16String {
        Utf16String::from_rust_str(&self.name)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn java_get_property(
        &self,
        property_name: &Utf16String,
    ) -> Option<
        Result<Option<Arc<TemplateValue>>, thymeleaf::expression::TemplateObjectPropertyError>,
    > {
        Some(Ok(match property_name.to_string_lossy().as_str() {
            "id" => Some(num(self.id)),
            "name" => Some(text(&self.name)),
            "price" => Some(decimal(&self.price)),
            "inStock" => Some(boolean(self.in_stock)),
            "comments" => Some(list(self.comments.iter().cloned().map(object).collect())),
            _ => None,
        }))
    }
}

/// 顾客 —— 对应 Java `Customer.java`。
#[derive(Clone, Debug)]
pub struct Customer {
    pub id: i32,
    pub name: String,
    pub customer_since: JavaDate,
}

impl TemplateObject for Customer {
    fn java_class_name(&self) -> &str {
        "org.thymeleaf.examples.core.gtvg.jakarta.business.entities.Customer"
    }
    fn to_utf16_string(&self) -> Utf16String {
        Utf16String::from_rust_str(&self.name)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn java_get_property(
        &self,
        property_name: &Utf16String,
    ) -> Option<
        Result<Option<Arc<TemplateValue>>, thymeleaf::expression::TemplateObjectPropertyError>,
    > {
        Some(Ok(match property_name.to_string_lossy().as_str() {
            "id" => Some(num(self.id)),
            "name" => Some(text(&self.name)),
            "customerSince" => Some(calendar(self.customer_since.clone())),
            _ => None,
        }))
    }
}

/// 订单 —— 对应 Java `Order.java`（orderLines 保持 Java `LinkedHashSet` 的插入序）。
#[derive(Clone, Debug)]
pub struct Order {
    pub id: i32,
    pub date: JavaDate,
    pub customer: Customer,
    pub order_lines: Vec<OrderLine>,
}

impl TemplateObject for Order {
    fn java_class_name(&self) -> &str {
        "org.thymeleaf.examples.core.gtvg.jakarta.business.entities.Order"
    }
    fn to_utf16_string(&self) -> Utf16String {
        Utf16String::from_rust_str(&self.customer.name)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn java_get_property(
        &self,
        property_name: &Utf16String,
    ) -> Option<
        Result<Option<Arc<TemplateValue>>, thymeleaf::expression::TemplateObjectPropertyError>,
    > {
        Some(Ok(match property_name.to_string_lossy().as_str() {
            "id" => Some(num(self.id)),
            "date" => Some(calendar(self.date.clone())),
            "customer" => Some(object(self.customer.clone())),
            "orderLines" => Some(list(self.order_lines.iter().cloned().map(object).collect())),
            _ => None,
        }))
    }
}

/// 用户 —— 对应 Java `User.java`（`age` 可为空，`name` 为 firstName + lastName）。
#[derive(Clone, Debug)]
pub struct User {
    pub first_name: String,
    pub last_name: String,
    pub nationality: String,
    pub age: Option<i32>,
}

impl TemplateObject for User {
    fn java_class_name(&self) -> &str {
        "org.thymeleaf.examples.core.gtvg.jakarta.business.entities.User"
    }
    fn to_utf16_string(&self) -> Utf16String {
        Utf16String::from_rust_str(&self.name())
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn java_get_property(
        &self,
        property_name: &Utf16String,
    ) -> Option<
        Result<Option<Arc<TemplateValue>>, thymeleaf::expression::TemplateObjectPropertyError>,
    > {
        Some(Ok(match property_name.to_string_lossy().as_str() {
            "firstName" => Some(text(&self.first_name)),
            "lastName" => Some(text(&self.last_name)),
            "name" => Some(text(&self.name())),
            "nationality" => Some(text(&self.nationality)),
            "age" => self.age.map(num),
            _ => None,
        }))
    }
}

impl User {
    /// Java `User#getName()`：firstName + 空格 + lastName。
    pub fn name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }
}

/// 评论 —— 对应 Java `Comment.java`。
#[derive(Clone, Debug)]
pub struct Comment {
    pub id: i32,
    pub text: String,
}

impl TemplateObject for Comment {
    fn java_class_name(&self) -> &str {
        "org.thymeleaf.examples.core.gtvg.jakarta.business.entities.Comment"
    }
    fn to_utf16_string(&self) -> Utf16String {
        Utf16String::from_rust_str(&self.text)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn java_get_property(
        &self,
        property_name: &Utf16String,
    ) -> Option<
        Result<Option<Arc<TemplateValue>>, thymeleaf::expression::TemplateObjectPropertyError>,
    > {
        Some(Ok(match property_name.to_string_lossy().as_str() {
            "id" => Some(num(self.id)),
            "text" => Some(text(&self.text)),
            _ => None,
        }))
    }
}
