//! GTVG 服务层 —— 对应 Java `business/services/*.java`。

use super::entities::{Customer, Order, Product};
use super::repositories::{CustomerRepository, OrderRepository, ProductRepository};

/// 顾客服务 —— 对应 Java `CustomerService.java`。
pub struct CustomerService;

impl CustomerService {
    /// Java `findAll()`。
    pub fn find_all() -> Vec<Customer> {
        CustomerRepository::find_all()
    }

    /// Java `findById(Integer)`。
    pub fn find_by_id(id: i32) -> Option<Customer> {
        CustomerRepository::find_by_id(id)
    }
}

/// 订单服务 —— 对应 Java `OrderService.java`。
pub struct OrderService;

impl OrderService {
    /// Java `findAll()`。
    pub fn find_all() -> Vec<Order> {
        OrderRepository::find_all()
    }

    /// Java `findById(Integer)`。
    pub fn find_by_id(id: i32) -> Option<Order> {
        OrderRepository::find_by_id(id)
    }
}

/// 产品服务 —— 对应 Java `ProductService.java`。
pub struct ProductService;

impl ProductService {
    /// Java `findAll()`。
    pub fn find_all() -> Vec<Product> {
        ProductRepository::find_all()
    }

    /// Java `findById(Integer)`。
    pub fn find_by_id(id: i32) -> Option<Product> {
        ProductRepository::find_by_id(id)
    }
}
