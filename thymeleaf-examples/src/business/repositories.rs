//! GTVG 内存仓库 —— 对应 Java `business/entities/repositories/*.java`。
//!
//! 种子数据与 Java 单例逐条一致（客户 6、产品 30 与 21 条评论、订单 3 与
//! 订单行 6），`findAll` 保持 Java `LinkedHashMap` 的插入顺序。

use std::sync::OnceLock;

use super::calendar_util::calendar_for;
use super::entities::{Comment, Customer, Order, OrderLine, Product};

/// 顾客仓库 —— 对应 Java `CustomerRepository.java`。
pub struct CustomerRepository;

impl CustomerRepository {
    /// 返回单例数据。
    pub fn get_instance() -> &'static [Customer] {
        static CUSTOMERS: OnceLock<Vec<Customer>> = OnceLock::new();
        CUSTOMERS.get_or_init(|| {
            vec![
                Customer {
                    id: 1,
                    name: "James Cucumber".to_owned(),
                    customer_since: calendar_for(2006, 4, 2, 13, 20),
                },
                Customer {
                    id: 2,
                    name: "Anna Lettuce".to_owned(),
                    customer_since: calendar_for(2005, 1, 30, 17, 14),
                },
                Customer {
                    id: 3,
                    name: "Boris Tomato".to_owned(),
                    customer_since: calendar_for(2008, 12, 2, 9, 53),
                },
                Customer {
                    id: 4,
                    name: "Shannon Parsley".to_owned(),
                    customer_since: calendar_for(2009, 3, 24, 10, 45),
                },
                Customer {
                    id: 5,
                    name: "Susan Cheddar".to_owned(),
                    customer_since: calendar_for(2007, 10, 1, 15, 2),
                },
                Customer {
                    id: 6,
                    name: "George Garlic".to_owned(),
                    customer_since: calendar_for(2010, 5, 18, 20, 30),
                },
            ]
        })
    }

    /// Java `findAll()`：按插入顺序返回全部顾客。
    pub fn find_all() -> Vec<Customer> {
        Self::get_instance().to_vec()
    }

    /// Java `findById(Integer)`：不存在返回 Java null 等价 `None`。
    pub fn find_by_id(id: i32) -> Option<Customer> {
        Self::get_instance().iter().find(|c| c.id == id).cloned()
    }
}

/// 产品仓库 —— 对应 Java `ProductRepository.java`。
pub struct ProductRepository;

impl ProductRepository {
    /// 返回单例数据。
    pub fn get_instance() -> &'static [Product] {
        static PRODUCTS: OnceLock<Vec<Product>> = OnceLock::new();
        PRODUCTS.get_or_init(|| {
            let products = vec![
                Product::new(1, "Fresh Sweet Basil", true, "4.99"),
                Product::new(2, "Italian Tomato", false, "1.25"),
                Product::new(3, "Yellow Bell Pepper", true, "2.50"),
                Product::new(4, "Old Cheddar", true, "18.75"),
                Product::new(5, "Extra Virgin Coconut Oil", true, "6.34"),
                Product::new(6, "Organic Tomato Ketchup", true, "1.99"),
                Product::new(7, "Whole Grain Oatmeal Cereal", true, "3.07"),
                Product::new(8, "Traditional Tomato & Basil Sauce", true, "2.58"),
                Product::new(9, "Quinoa Flour", true, "3.02"),
                Product::new(10, "Grapefruit Juice", true, "2.58"),
                Product::new(11, "100% Pure Maple Syrup", true, "5.98"),
                Product::new(12, "Marinara Pasta Sauce", false, "2.08"),
                Product::new(13, "Vanilla Puff Cereal", false, "1.75"),
                Product::new(14, "Extra Virgin Oil", false, "5.01"),
                Product::new(15, "Roasted Garlic Pasta Sauce", true, "2.40"),
                Product::new(16, "Canned Minestrone Soup", true, "2.19"),
                Product::new(17, "Almond Milk 1L", true, "3.24"),
                Product::new(18, "Organic Chicken & Wild Rice Soup", true, "3.17"),
                Product::new(
                    19,
                    "Purple Carrot, Blackberry, Quinoa & Greek Yogurt",
                    true,
                    "8.88",
                ),
                Product::new(20, "Pumpkin, Carrot and Apple Juice", false, "3.90"),
                Product::new(21, "Organic Canola Oil", true, "10.13"),
                Product::new(22, "Potato Corn Tortilla Chips", true, "2.44"),
                Product::new(23, "Canned Corn Chowder Soup", true, "2.30"),
                Product::new(24, "Organic Lemonade Juice", true, "2.48"),
                Product::new(25, "Spicy Basil Dressing", true, "4.72"),
                Product::new(26, "Sweet Agave Nectar", true, "6.46"),
                Product::new(27, "Dark Roasted Peanut Butter", false, "3.48"),
                Product::new(28, "Unsweetened Lemon Green Tea", true, "18.34"),
                Product::new(29, "Whole Grain Flakes Cereal", true, "3.52"),
                Product::new(30, "Berry Chewy Granola Bars", true, "4.00"),
            ];
            let mut products = products;
            // Java 评论种子：product 2（2 条）、13（8 条）、9（2 条）、14（4 条）、
            // 16（1 条）、24（1 条）、30（3 条），id 1..=21 全局递增。
            let comments: &[(i32, i32, &str)] = &[
                (2, 1, "I'm so sad this product is no longer available!"),
                (2, 2, "When do you expect to have it back?"),
                (13, 3, "Very tasty! I'd definitely buy it again!"),
                (13, 4, "My kids love it!"),
                (
                    13,
                    5,
                    "Good, my basic breakfast cereal. Though maybe a bit in the sweet side...",
                ),
                (
                    13,
                    6,
                    "Not that I find it bad, but I think the vanilla flavouring is too intrusive",
                ),
                (
                    13,
                    7,
                    "I agree with the excessive flavouring, but still one of my favourites!",
                ),
                (13, 8, "Cheaper than at the local store!"),
                (
                    13,
                    9,
                    "I'm sorry to disagree, but IMO these are far too sweet",
                ),
                (13, 10, "Good. Pricey though."),
                (9, 11, "Made bread with this and it was great!"),
                (9, 12, "Note: this comes actually mixed with wheat flour"),
                (14, 13, "Awesome Spanish oil. Buy it now."),
                (
                    14,
                    14,
                    "Would definitely buy it again. Best one I've tasted",
                ),
                (
                    14,
                    15,
                    "A bit acid for my taste, but still a very nice one.",
                ),
                (14, 16, "Definitely not the average olive oil. Really good."),
                (16, 17, "Great value!"),
                (24, 18, "My favourite :)"),
                (30, 19, "Too hard! I would not buy again"),
                (
                    30,
                    20,
                    "Taste is OK, but I agree with previous comment that bars are too hard to eat",
                ),
                (30, 21, "Would definitely NOT buy again. Simply unedible!"),
            ];
            for (product_id, comment_id, comment_text) in comments {
                let product = products
                    .iter_mut()
                    .find(|p| p.id == *product_id)
                    .expect("comment product exists");
                product.comments.push(Comment {
                    id: *comment_id,
                    text: (*comment_text).to_owned(),
                });
            }
            products
        })
    }

    /// Java `findAll()`：按插入顺序返回全部产品。
    pub fn find_all() -> Vec<Product> {
        Self::get_instance().to_vec()
    }

    /// Java `findById(Integer)`：不存在返回 Java null 等价 `None`。
    pub fn find_by_id(id: i32) -> Option<Product> {
        Self::get_instance().iter().find(|p| p.id == id).cloned()
    }
}

impl Product {
    /// Java `Product(int, String, boolean, BigDecimal)` 构造器。
    #[must_use]
    pub fn new(id: i32, name: &str, in_stock: bool, price: &str) -> Self {
        Self {
            id,
            name: name.to_owned(),
            price: price.to_owned(),
            in_stock,
            comments: Vec::new(),
        }
    }
}

/// 订单仓库 —— 对应 Java `OrderRepository.java`。
pub struct OrderRepository;

impl OrderRepository {
    /// 返回单例数据。
    pub fn get_instance() -> &'static [Order] {
        static ORDERS: OnceLock<Vec<Order>> = OnceLock::new();
        ORDERS.get_or_init(|| {
            let prod1 = ProductRepository::find_by_id(1).expect("prod1");
            let prod2 = ProductRepository::find_by_id(2).expect("prod2");
            let prod3 = ProductRepository::find_by_id(3).expect("prod3");
            let prod4 = ProductRepository::find_by_id(4).expect("prod4");
            let cust1 = CustomerRepository::find_by_id(1).expect("cust1");
            let cust4 = CustomerRepository::find_by_id(4).expect("cust4");
            let cust6 = CustomerRepository::find_by_id(6).expect("cust6");

            vec![
                Order {
                    id: 1,
                    customer: cust4,
                    date: calendar_for(2009, 1, 12, 10, 23),
                    order_lines: vec![
                        OrderLine {
                            product: prod2,
                            amount: 2,
                            purchase_price: "0.99".to_owned(),
                        },
                        OrderLine {
                            product: prod3,
                            amount: 4,
                            purchase_price: "2.50".to_owned(),
                        },
                        // Java OrderLine 共享 Product 实例；Rust 克隆保持独立所有权。
                        OrderLine {
                            product: prod4.clone(),
                            amount: 1,
                            purchase_price: "15.50".to_owned(),
                        },
                    ],
                },
                Order {
                    id: 2,
                    customer: cust6,
                    date: calendar_for(2010, 6, 9, 21, 1),
                    order_lines: vec![
                        OrderLine {
                            // prod1 在 order3 最后一次使用，此处克隆。
                            product: prod1.clone(),
                            amount: 5,
                            purchase_price: "3.75".to_owned(),
                        },
                        OrderLine {
                            product: prod4,
                            amount: 2,
                            purchase_price: "17.99".to_owned(),
                        },
                    ],
                },
                Order {
                    id: 3,
                    customer: cust1,
                    date: calendar_for(2010, 7, 18, 22, 32),
                    // Java 原样保留：order3 挂到 cust4 的列表下（ordersByCustomerId），
                    // 但 customer 字段是 cust1 —— 移植保持该数据不变。
                    order_lines: vec![OrderLine {
                        product: prod1,
                        amount: 8,
                        purchase_price: "5.99".to_owned(),
                    }],
                },
            ]
        })
    }

    /// Java `findAll()`：按插入顺序返回全部订单。
    pub fn find_all() -> Vec<Order> {
        Self::get_instance().to_vec()
    }

    /// Java `findById(Integer)`：不存在返回 Java null 等价 `None`。
    pub fn find_by_id(id: i32) -> Option<Order> {
        Self::get_instance().iter().find(|o| o.id == id).cloned()
    }
}
