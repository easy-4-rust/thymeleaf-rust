//! 内嵌 Thymeleaf 官方版本化 DTD 文件集，构建 MemoryResolver。
//!
//! 文件来源：Thymeleaf 2.1.6.RELEASE jar 内 `org/thymeleaf/dtd/`（Java 实现
//! 唯一官方捆绑的版本化 DTD 约束集），以 `include_str!` 逐字节内嵌。
//! 覆盖三组注册：
//! 1. Thymeleaf 命名空间 16 个 SYSTEM ID（4 族 × 4 个历史版本，
//!    与 `StandardTranslationDocTypeProcessor` 翻译表一一对应）——
//!    自包含单体 DTD（方言标签内联，无外部引用）
//! 2. transitional 族历史拼写别名（文件头自引用 `transitonal`，
//!    Thymeleaf 2 时代模板可能照抄进 DOCTYPE）
//! 3. W3C 命名空间 4 个标准 URL + 全部裸文件名（.mod/.ent）——
//!    `standard/` 下亦为自包含单体副本
//!
//! 零网络访问，零文件系统依赖。

#[cfg(feature = "dtd-validation")]
use oxixml_dtd::MemoryResolver;

/// DTD 文件内嵌：`dtd-files/thymeleaf-2.1.6/org/thymeleaf/dtd/` 相对路径。
#[cfg(feature = "dtd-validation")]
macro_rules! dtd_entry {
    ($path:literal) => {
        include_str!(concat!(
            "../../../dtd-files/thymeleaf-2.1.6/org/thymeleaf/dtd/",
            $path
        ))
    };
}

/// 构建包含全部内嵌 DTD 的 MemoryResolver（SYSTEM key 精确匹配）。
///
/// # 返回
/// 覆盖 Thymeleaf 16 + 拼写别名 4 + W3C 4 + 裸模块文件名 42 的 resolver。
///
/// 对应 Java 语义：Thymeleaf 2.1.6 以 classpath 资源
/// `org/thymeleaf/dtd/{thymeleaf,standard}/` 提供同一文件集离线解析；
/// Rust 侧以 `include_str!` 内嵌等价内容，零运行时 IO。
#[cfg(feature = "dtd-validation")]
pub fn build_xhtml_resolver() -> MemoryResolver {
    let mut resolver = MemoryResolver::new();
    // Thymeleaf 命名空间：4 族 × 4 版本（与 StandardTranslationDocTypeProcessor 对应）
    resolver.insert(
        "http://www.thymeleaf.org/dtd/xhtml1-strict-thymeleaf-1.dtd",
        dtd_entry!("thymeleaf/xhtml1-strict-thymeleaf-1.dtd"),
    );
    resolver.insert(
        "http://www.thymeleaf.org/dtd/xhtml1-strict-thymeleaf-2.dtd",
        dtd_entry!("thymeleaf/xhtml1-strict-thymeleaf-2.dtd"),
    );
    resolver.insert(
        "http://www.thymeleaf.org/dtd/xhtml1-strict-thymeleaf-3.dtd",
        dtd_entry!("thymeleaf/xhtml1-strict-thymeleaf-3.dtd"),
    );
    resolver.insert(
        "http://www.thymeleaf.org/dtd/xhtml1-strict-thymeleaf-4.dtd",
        dtd_entry!("thymeleaf/xhtml1-strict-thymeleaf-4.dtd"),
    );
    resolver.insert(
        "http://www.thymeleaf.org/dtd/xhtml1-transitional-thymeleaf-1.dtd",
        dtd_entry!("thymeleaf/xhtml1-transitional-thymeleaf-1.dtd"),
    );
    resolver.insert(
        "http://www.thymeleaf.org/dtd/xhtml1-transitional-thymeleaf-2.dtd",
        dtd_entry!("thymeleaf/xhtml1-transitional-thymeleaf-2.dtd"),
    );
    resolver.insert(
        "http://www.thymeleaf.org/dtd/xhtml1-transitional-thymeleaf-3.dtd",
        dtd_entry!("thymeleaf/xhtml1-transitional-thymeleaf-3.dtd"),
    );
    resolver.insert(
        "http://www.thymeleaf.org/dtd/xhtml1-transitional-thymeleaf-4.dtd",
        dtd_entry!("thymeleaf/xhtml1-transitional-thymeleaf-4.dtd"),
    );
    resolver.insert(
        "http://www.thymeleaf.org/dtd/xhtml1-frameset-thymeleaf-1.dtd",
        dtd_entry!("thymeleaf/xhtml1-frameset-thymeleaf-1.dtd"),
    );
    resolver.insert(
        "http://www.thymeleaf.org/dtd/xhtml1-frameset-thymeleaf-2.dtd",
        dtd_entry!("thymeleaf/xhtml1-frameset-thymeleaf-2.dtd"),
    );
    resolver.insert(
        "http://www.thymeleaf.org/dtd/xhtml1-frameset-thymeleaf-3.dtd",
        dtd_entry!("thymeleaf/xhtml1-frameset-thymeleaf-3.dtd"),
    );
    resolver.insert(
        "http://www.thymeleaf.org/dtd/xhtml1-frameset-thymeleaf-4.dtd",
        dtd_entry!("thymeleaf/xhtml1-frameset-thymeleaf-4.dtd"),
    );
    resolver.insert(
        "http://www.thymeleaf.org/dtd/xhtml11-thymeleaf-1.dtd",
        dtd_entry!("thymeleaf/xhtml11-thymeleaf-1.dtd"),
    );
    resolver.insert(
        "http://www.thymeleaf.org/dtd/xhtml11-thymeleaf-2.dtd",
        dtd_entry!("thymeleaf/xhtml11-thymeleaf-2.dtd"),
    );
    resolver.insert(
        "http://www.thymeleaf.org/dtd/xhtml11-thymeleaf-3.dtd",
        dtd_entry!("thymeleaf/xhtml11-thymeleaf-3.dtd"),
    );
    resolver.insert(
        "http://www.thymeleaf.org/dtd/xhtml11-thymeleaf-4.dtd",
        dtd_entry!("thymeleaf/xhtml11-thymeleaf-4.dtd"),
    );
    // transitional 族历史拼写别名（`transitonal`，文件头自引用拼写）
    resolver.insert(
        "http://www.thymeleaf.org/dtd/xhtml1-transitonal-thymeleaf-1.dtd",
        dtd_entry!("thymeleaf/xhtml1-transitional-thymeleaf-1.dtd"),
    );
    resolver.insert(
        "http://www.thymeleaf.org/dtd/xhtml1-transitonal-thymeleaf-2.dtd",
        dtd_entry!("thymeleaf/xhtml1-transitional-thymeleaf-2.dtd"),
    );
    resolver.insert(
        "http://www.thymeleaf.org/dtd/xhtml1-transitonal-thymeleaf-3.dtd",
        dtd_entry!("thymeleaf/xhtml1-transitional-thymeleaf-3.dtd"),
    );
    resolver.insert(
        "http://www.thymeleaf.org/dtd/xhtml1-transitonal-thymeleaf-4.dtd",
        dtd_entry!("thymeleaf/xhtml1-transitional-thymeleaf-4.dtd"),
    );
    // W3C 命名空间：标准 SYSTEM URL → standard/ 自包含单体副本
    resolver.insert(
        "http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd",
        dtd_entry!("standard/xhtml1-strict.dtd"),
    );
    resolver.insert(
        "http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd",
        dtd_entry!("standard/xhtml1-transitional.dtd"),
    );
    resolver.insert(
        "http://www.w3.org/TR/xhtml1/DTD/xhtml1-frameset.dtd",
        dtd_entry!("standard/xhtml1-frameset.dtd"),
    );
    resolver.insert(
        "http://www.w3.org/TR/xhtml11/DTD/xhtml11.dtd",
        dtd_entry!("standard/xhtml11.dtd"),
    );
    // xhtml11 命名空间：模型模块 URL（xhtml11-thymeleaf-1/2/3 引用）
    resolver.insert(
        "http://www.w3.org/TR/xhtml11/DTD/xhtml11-model-1.mod",
        dtd_entry!("standard/xhtml11-model-1.mod"),
    );
    // 字符实体 URL → 本地 .ent：thymeleaf 族单体以 W3C URL 引用字符实体集
    resolver.insert(
        "http://www.w3.org/TR/xhtml1/DTD/xhtml-lat1.ent",
        dtd_entry!("standard/xhtml-lat1.ent"),
    );
    resolver.insert(
        "http://www.w3.org/TR/xhtml1/DTD/xhtml-symbol.ent",
        dtd_entry!("standard/xhtml-symbol.ent"),
    );
    resolver.insert(
        "http://www.w3.org/TR/xhtml1/DTD/xhtml-special.ent",
        dtd_entry!("standard/xhtml-special.ent"),
    );
    // 模块 URL → 本地模块：xhtml11.dtd 等模块化 DTD 以绝对 URL 引用模块，
    // jar 已捆绑同名本地副本，注册映射实现完全离线解析。
    resolver.insert(
        "http://www.w3.org/TR/xhtml-modularization/DTD/xhtml-attribs-1.mod",
        dtd_entry!("standard/xhtml-attribs-1.mod"),
    );
    resolver.insert(
        "http://www.w3.org/TR/xhtml-modularization/DTD/xhtml-base-1.mod",
        dtd_entry!("standard/xhtml-base-1.mod"),
    );
    resolver.insert(
        "http://www.w3.org/TR/xhtml-modularization/DTD/xhtml-bdo-1.mod",
        dtd_entry!("standard/xhtml-bdo-1.mod"),
    );
    resolver.insert(
        "http://www.w3.org/TR/xhtml-modularization/DTD/xhtml-blkphras-1.mod",
        dtd_entry!("standard/xhtml-blkphras-1.mod"),
    );
    resolver.insert(
        "http://www.w3.org/TR/xhtml-modularization/DTD/xhtml-blkpres-1.mod",
        dtd_entry!("standard/xhtml-blkpres-1.mod"),
    );
    resolver.insert(
        "http://www.w3.org/TR/xhtml-modularization/DTD/xhtml-blkstruct-1.mod",
        dtd_entry!("standard/xhtml-blkstruct-1.mod"),
    );
    resolver.insert(
        "http://www.w3.org/TR/xhtml-modularization/DTD/xhtml-charent-1.mod",
        dtd_entry!("standard/xhtml-charent-1.mod"),
    );
    resolver.insert(
        "http://www.w3.org/TR/xhtml-modularization/DTD/xhtml-csismap-1.mod",
        dtd_entry!("standard/xhtml-csismap-1.mod"),
    );
    resolver.insert(
        "http://www.w3.org/TR/xhtml-modularization/DTD/xhtml-datatypes-1.mod",
        dtd_entry!("standard/xhtml-datatypes-1.mod"),
    );
    resolver.insert(
        "http://www.w3.org/TR/xhtml-modularization/DTD/xhtml-edit-1.mod",
        dtd_entry!("standard/xhtml-edit-1.mod"),
    );
    resolver.insert(
        "http://www.w3.org/TR/xhtml-modularization/DTD/xhtml-events-1.mod",
        dtd_entry!("standard/xhtml-events-1.mod"),
    );
    resolver.insert(
        "http://www.w3.org/TR/xhtml-modularization/DTD/xhtml-form-1.mod",
        dtd_entry!("standard/xhtml-form-1.mod"),
    );
    resolver.insert(
        "http://www.w3.org/TR/xhtml-modularization/DTD/xhtml-framework-1.mod",
        dtd_entry!("standard/xhtml-framework-1.mod"),
    );
    resolver.insert(
        "http://www.w3.org/TR/xhtml-modularization/DTD/xhtml-hypertext-1.mod",
        dtd_entry!("standard/xhtml-hypertext-1.mod"),
    );
    resolver.insert(
        "http://www.w3.org/TR/xhtml-modularization/DTD/xhtml-image-1.mod",
        dtd_entry!("standard/xhtml-image-1.mod"),
    );
    resolver.insert(
        "http://www.w3.org/TR/xhtml-modularization/DTD/xhtml-inlphras-1.mod",
        dtd_entry!("standard/xhtml-inlphras-1.mod"),
    );
    resolver.insert(
        "http://www.w3.org/TR/xhtml-modularization/DTD/xhtml-inlpres-1.mod",
        dtd_entry!("standard/xhtml-inlpres-1.mod"),
    );
    resolver.insert(
        "http://www.w3.org/TR/xhtml-modularization/DTD/xhtml-inlstruct-1.mod",
        dtd_entry!("standard/xhtml-inlstruct-1.mod"),
    );
    resolver.insert(
        "http://www.w3.org/TR/xhtml-modularization/DTD/xhtml-inlstyle-1.mod",
        dtd_entry!("standard/xhtml-inlstyle-1.mod"),
    );
    resolver.insert(
        "http://www.w3.org/TR/xhtml-modularization/DTD/xhtml-link-1.mod",
        dtd_entry!("standard/xhtml-link-1.mod"),
    );
    resolver.insert(
        "http://www.w3.org/TR/xhtml-modularization/DTD/xhtml-list-1.mod",
        dtd_entry!("standard/xhtml-list-1.mod"),
    );
    resolver.insert(
        "http://www.w3.org/TR/xhtml-modularization/DTD/xhtml-meta-1.mod",
        dtd_entry!("standard/xhtml-meta-1.mod"),
    );
    resolver.insert(
        "http://www.w3.org/TR/xhtml-modularization/DTD/xhtml-object-1.mod",
        dtd_entry!("standard/xhtml-object-1.mod"),
    );
    resolver.insert(
        "http://www.w3.org/TR/xhtml-modularization/DTD/xhtml-param-1.mod",
        dtd_entry!("standard/xhtml-param-1.mod"),
    );
    resolver.insert(
        "http://www.w3.org/TR/xhtml-modularization/DTD/xhtml-pres-1.mod",
        dtd_entry!("standard/xhtml-pres-1.mod"),
    );
    resolver.insert(
        "http://www.w3.org/TR/xhtml-modularization/DTD/xhtml-qname-1.mod",
        dtd_entry!("standard/xhtml-qname-1.mod"),
    );
    resolver.insert(
        "http://www.w3.org/TR/xhtml-modularization/DTD/xhtml-ruby-1.mod",
        dtd_entry!("standard/xhtml-ruby-1.mod"),
    );
    resolver.insert(
        "http://www.w3.org/TR/xhtml-modularization/DTD/xhtml-script-1.mod",
        dtd_entry!("standard/xhtml-script-1.mod"),
    );
    resolver.insert(
        "http://www.w3.org/TR/xhtml-modularization/DTD/xhtml-ssismap-1.mod",
        dtd_entry!("standard/xhtml-ssismap-1.mod"),
    );
    resolver.insert(
        "http://www.w3.org/TR/xhtml-modularization/DTD/xhtml-struct-1.mod",
        dtd_entry!("standard/xhtml-struct-1.mod"),
    );
    resolver.insert(
        "http://www.w3.org/TR/xhtml-modularization/DTD/xhtml-style-1.mod",
        dtd_entry!("standard/xhtml-style-1.mod"),
    );
    resolver.insert(
        "http://www.w3.org/TR/xhtml-modularization/DTD/xhtml-table-1.mod",
        dtd_entry!("standard/xhtml-table-1.mod"),
    );
    resolver.insert(
        "http://www.w3.org/TR/xhtml-modularization/DTD/xhtml-text-1.mod",
        dtd_entry!("standard/xhtml-text-1.mod"),
    );
    resolver.insert(
        "http://www.w3.org/TR/xhtml-modularization/DTD/xhtml11-model-1.mod",
        dtd_entry!("standard/xhtml11-model-1.mod"),
    );
    resolver.insert(
        "http://www.w3.org/TR/ruby/xhtml-ruby-1.mod",
        dtd_entry!("standard/xhtml-ruby-1.mod"),
    );
    // 裸文件名（模块化 DTD 的相对引用）：standard/ 全集
    resolver.insert(
        "xhtml-attribs-1.mod",
        dtd_entry!("standard/xhtml-attribs-1.mod"),
    );
    resolver.insert("xhtml-base-1.mod", dtd_entry!("standard/xhtml-base-1.mod"));
    resolver.insert("xhtml-bdo-1.mod", dtd_entry!("standard/xhtml-bdo-1.mod"));
    resolver.insert(
        "xhtml-blkphras-1.mod",
        dtd_entry!("standard/xhtml-blkphras-1.mod"),
    );
    resolver.insert(
        "xhtml-blkpres-1.mod",
        dtd_entry!("standard/xhtml-blkpres-1.mod"),
    );
    resolver.insert(
        "xhtml-blkstruct-1.mod",
        dtd_entry!("standard/xhtml-blkstruct-1.mod"),
    );
    resolver.insert(
        "xhtml-charent-1.mod",
        dtd_entry!("standard/xhtml-charent-1.mod"),
    );
    resolver.insert(
        "xhtml-csismap-1.mod",
        dtd_entry!("standard/xhtml-csismap-1.mod"),
    );
    resolver.insert(
        "xhtml-datatypes-1.mod",
        dtd_entry!("standard/xhtml-datatypes-1.mod"),
    );
    resolver.insert("xhtml-edit-1.mod", dtd_entry!("standard/xhtml-edit-1.mod"));
    resolver.insert(
        "xhtml-events-1.mod",
        dtd_entry!("standard/xhtml-events-1.mod"),
    );
    resolver.insert("xhtml-form-1.mod", dtd_entry!("standard/xhtml-form-1.mod"));
    resolver.insert(
        "xhtml-framework-1.mod",
        dtd_entry!("standard/xhtml-framework-1.mod"),
    );
    resolver.insert(
        "xhtml-hypertext-1.mod",
        dtd_entry!("standard/xhtml-hypertext-1.mod"),
    );
    resolver.insert(
        "xhtml-image-1.mod",
        dtd_entry!("standard/xhtml-image-1.mod"),
    );
    resolver.insert(
        "xhtml-inlphras-1.mod",
        dtd_entry!("standard/xhtml-inlphras-1.mod"),
    );
    resolver.insert(
        "xhtml-inlpres-1.mod",
        dtd_entry!("standard/xhtml-inlpres-1.mod"),
    );
    resolver.insert(
        "xhtml-inlstruct-1.mod",
        dtd_entry!("standard/xhtml-inlstruct-1.mod"),
    );
    resolver.insert(
        "xhtml-inlstyle-1.mod",
        dtd_entry!("standard/xhtml-inlstyle-1.mod"),
    );
    resolver.insert("xhtml-lat1.ent", dtd_entry!("standard/xhtml-lat1.ent"));
    resolver.insert("xhtml-link-1.mod", dtd_entry!("standard/xhtml-link-1.mod"));
    resolver.insert("xhtml-list-1.mod", dtd_entry!("standard/xhtml-list-1.mod"));
    resolver.insert("xhtml-meta-1.mod", dtd_entry!("standard/xhtml-meta-1.mod"));
    resolver.insert(
        "xhtml-object-1.mod",
        dtd_entry!("standard/xhtml-object-1.mod"),
    );
    resolver.insert(
        "xhtml-param-1.mod",
        dtd_entry!("standard/xhtml-param-1.mod"),
    );
    resolver.insert("xhtml-pres-1.mod", dtd_entry!("standard/xhtml-pres-1.mod"));
    resolver.insert(
        "xhtml-qname-1.mod",
        dtd_entry!("standard/xhtml-qname-1.mod"),
    );
    resolver.insert("xhtml-ruby-1.mod", dtd_entry!("standard/xhtml-ruby-1.mod"));
    resolver.insert(
        "xhtml-script-1.mod",
        dtd_entry!("standard/xhtml-script-1.mod"),
    );
    resolver.insert(
        "xhtml-special.ent",
        dtd_entry!("standard/xhtml-special.ent"),
    );
    resolver.insert(
        "xhtml-ssismap-1.mod",
        dtd_entry!("standard/xhtml-ssismap-1.mod"),
    );
    resolver.insert(
        "xhtml-struct-1.mod",
        dtd_entry!("standard/xhtml-struct-1.mod"),
    );
    resolver.insert(
        "xhtml-style-1.mod",
        dtd_entry!("standard/xhtml-style-1.mod"),
    );
    resolver.insert("xhtml-symbol.ent", dtd_entry!("standard/xhtml-symbol.ent"));
    resolver.insert(
        "xhtml-table-1.mod",
        dtd_entry!("standard/xhtml-table-1.mod"),
    );
    resolver.insert("xhtml-text-1.mod", dtd_entry!("standard/xhtml-text-1.mod"));
    resolver.insert(
        "xhtml1-frameset.dtd",
        dtd_entry!("standard/xhtml1-frameset.dtd"),
    );
    resolver.insert(
        "xhtml1-strict.dtd",
        dtd_entry!("standard/xhtml1-strict.dtd"),
    );
    resolver.insert(
        "xhtml1-transitional.dtd",
        dtd_entry!("standard/xhtml1-transitional.dtd"),
    );
    resolver.insert(
        "xhtml11-model-1.mod",
        dtd_entry!("standard/xhtml11-model-1.mod"),
    );
    resolver.insert("xhtml11.dtd", dtd_entry!("standard/xhtml11.dtd"));
    resolver.insert(
        "xhtml5-legacy-wildcard.dtd",
        dtd_entry!("standard/xhtml5-legacy-wildcard.dtd"),
    );
    resolver
}
