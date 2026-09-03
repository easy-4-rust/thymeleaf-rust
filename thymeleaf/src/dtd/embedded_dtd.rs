//! 内嵌 W3C XHTML DTD 文件集，构建 MemoryResolver。
//!
//! 将 dtd-files/ 下的 W3C DTD 文件以 `include_str!` 内嵌到二进制，
//! 按 SYSTEM 标识符（DTD 内部子集引用的相对文件名）注册到 `MemoryResolver`。
//! 零网络访问，零文件系统依赖。

#[cfg(feature = "dtd-validation")]
use oxixml_dtd::MemoryResolver;

/// DTD 文件条目：(SYSTEM 标识符, 内嵌文本)。
#[cfg(feature = "dtd-validation")]
macro_rules! dtd_entry {
    ($path:expr) => {
        include_str!(concat!("../../../dtd-files/", $path))
    };
}

/// 构建包含所有内嵌 W3C XHTML DTD 的 MemoryResolver。
///
/// 注册表覆盖 4 组 DTD 主文件 + 3 个共享实体文件（.ent）：
/// - xhtml1-strict.dtd / xhtml1-transitional.dtd / xhtml1-frameset.dtd
/// - xhtml11.dtd
/// - xhtml-lat1.ent / xhtml-symbol.ent / xhtml-special.ent
///
/// SYSTEM 标识符精确匹配 DTD 内部子集的引用路径。
/// .mod 文件（W3C 不提供直接下载）不在本 resolver 中，
/// 完整 DTD 解析需在 parse_xml 集成时用 `FilesystemResolver` 补充（Stage 2）。
///
/// # 返回
/// 包含全部内嵌 .ent/.dtd 文件的 MemoryResolver（SYSTEM key 精确匹配）。
///
/// 对应 Java 语义：Rust 侧扩展（Java 经 Xerces EntityResolver/类路径
/// 解析 DTD；Rust 侧以 `include_str!` 内嵌等价内容，零运行时 IO）。
#[cfg(feature = "dtd-validation")]
pub fn build_xhtml_resolver() -> MemoryResolver {
    let mut resolver = MemoryResolver::new();
    // .ent 文件（字符实体，W3C MarkUp/DTD 下载，真实 SGML 内容）
    resolver.insert("xhtml-lat1.ent", dtd_entry!("xhtml1/xhtml-lat1.ent"));
    resolver.insert("xhtml-symbol.ent", dtd_entry!("xhtml1/xhtml-symbol.ent"));
    resolver.insert("xhtml-special.ent", dtd_entry!("xhtml1/xhtml-special.ent"));
    // xhtml11 .ent 文件（与 xhtml1 共享文件名，内容一致，W3C MarkUp/DTD）
    resolver.insert("xhtml11-lat1.ent", dtd_entry!("xhtml11/xhtml11-lat1.ent"));
    resolver.insert(
        "xhtml11-symbol.ent",
        dtd_entry!("xhtml11/xhtml11-symbol.ent"),
    );
    resolver.insert(
        "xhtml11-special.ent",
        dtd_entry!("xhtml11/xhtml11-special.ent"),
    );
    // .mod 文件（W3C 提供的模块文件，真实 SGML 内容）
    resolver.insert(
        "xhtml1-strict-model-1.mod",
        dtd_entry!("xhtml1/xhtml-strict-model-1.mod"),
    );
    resolver.insert(
        "xhtml1-framework-1.mod",
        dtd_entry!("xhtml1/xhtml-framework-1.mod"),
    );
    resolver.insert(
        "xhtml11-model-1.mod",
        dtd_entry!("xhtml11/xhtml11-model-1.mod"),
    );
    resolver.insert(
        "xhtml11-framework-1.mod",
        dtd_entry!("xhtml11/xhtml11-framework-1.mod"),
    );
    // 主 DTD 文件（按完整 SYSTEM 标识符注册）
    resolver.insert(
        "http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd",
        dtd_entry!("xhtml1/xhtml1-strict.dtd"),
    );
    resolver.insert(
        "http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd",
        dtd_entry!("xhtml1/xhtml1-transitional.dtd"),
    );
    resolver.insert(
        "http://www.w3.org/TR/xhtml1/DTD/xhtml1-frameset.dtd",
        dtd_entry!("xhtml1/xhtml1-frameset.dtd"),
    );
    resolver.insert(
        "http://www.w3.org/TR/xhtml11/DTD/xhtml11.dtd",
        dtd_entry!("xhtml11/xhtml11.dtd"),
    );
    resolver
}
