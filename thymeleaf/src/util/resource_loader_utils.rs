use std::cell::RefCell;
use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::{Path, PathBuf};
use std::sync::{OnceLock, RwLock};

thread_local! {
    static THREAD_RESOURCE_ROOTS: RefCell<Vec<PathBuf>> = const { RefCell::new(Vec::new()) };
}

static REGISTERED_CLASSES: OnceLock<RwLock<HashSet<String>>> = OnceLock::new();

/// Rust 资源装载与运行时类型发现工具。
///
/// 对应 Java: `org.thymeleaf.util.ClassLoaderUtils`。
///
/// Rust 没有 JVM `ClassLoader`。该对象把 Java 的线程上下文、类自身与系统加载器
/// 优先级迁移为“线程资源根目录、crate 根目录、可执行文件目录、当前工作目录”的
/// 有序查找；Java 按类名发现则映射为显式注册的 Rust 运行时能力名称。
pub struct ResourceLoaderUtils;

impl ResourceLoaderUtils {
    /// 在当前线程的资源根目录作用域中执行操作。
    ///
    /// `resource_roots` 对应 Java 线程上下文类加载器，优先于应用默认根目录。
    /// 闭包结束后恢复先前配置，即使闭包发生 panic 也由析构守卫完成恢复。
    pub fn with_thread_resource_roots<T>(
        resource_roots: Vec<PathBuf>,
        operation: impl FnOnce() -> T,
    ) -> T {
        struct RestoreThreadRoots(Option<Vec<PathBuf>>);

        impl Drop for RestoreThreadRoots {
            fn drop(&mut self) {
                if let Some(previous) = self.0.take() {
                    THREAD_RESOURCE_ROOTS.with(|roots| {
                        roots.replace(previous);
                    });
                }
            }
        }

        let previous = THREAD_RESOURCE_ROOTS.with(|roots| roots.replace(resource_roots));
        let _restore = RestoreThreadRoots(Some(previous));
        operation()
    }

    /// 返回当前有效的有序资源根目录。
    ///
    /// 线程上下文根目录优先；随后依次包含 crate、可执行文件和当前工作目录，
    /// 重复目录只保留第一次出现的位置。
    /// 对应 Java 语义：`ClassLoaderUtils` 的 `get_resource_roots` 行为（Rust 侧辅助/私有路径）。
    #[must_use]
    pub fn get_resource_roots() -> Vec<PathBuf> {
        let mut roots = Vec::new();
        THREAD_RESOURCE_ROOTS.with(|thread_roots| {
            for root in thread_roots.borrow().iter() {
                push_unique(&mut roots, root);
            }
        });
        push_unique(&mut roots, Path::new(env!("CARGO_MANIFEST_DIR")));
        if let Ok(executable) = std::env::current_exe()
            && let Some(parent) = executable.parent()
        {
            push_unique(&mut roots, parent);
        }
        if let Ok(current_directory) = std::env::current_dir() {
            push_unique(&mut roots, &current_directory);
        }
        roots
    }

    /// 注册可按名称发现的 Rust 运行时能力。
    ///
    /// `class_name` 保留 Java `ClassLoaderUtils` 的参数语义；集成 crate 可用其 Java
    /// 兼容类名或 Rust 能力名进行注册。
    /// 对应 Java 语义：`ClassLoaderUtils` 的 `register_class` 行为（Rust 侧辅助/私有路径）。
    pub fn register_class(class_name: impl Into<String>) {
        let mut classes = registered_classes()
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        classes.insert(class_name.into());
    }

    /// 按名称加载已注册的 Rust 运行时能力。
    ///
    /// 对应 Java: `ClassLoaderUtils#loadClass(String)`。
    ///
    /// # 错误
    /// 未注册该名称时返回 `NotFound`，等价于 Java `ClassNotFoundError`。
    pub fn load_class(class_name: &str) -> io::Result<String> {
        Self::find_class(class_name).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("Could not locate runtime class '{class_name}'"),
            )
        })
    }

    /// 查找已注册的 Rust 运行时能力；未找到时返回 `None`。
    ///
    /// 对应 Java: `ClassLoaderUtils#findClass(String)`。
    #[must_use]
    pub fn find_class(class_name: &str) -> Option<String> {
        let classes = registered_classes()
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        classes.get(class_name).cloned()
    }

    /// 判断指定运行时能力是否已经注册。
    ///
    /// 对应 Java: `ClassLoaderUtils#isClassPresent(String)`。
    #[must_use]
    pub fn is_class_present(class_name: &str) -> bool {
        Self::find_class(class_name).is_some()
    }

    /// 按 Java classpath 风格资源名查找文件。
    ///
    /// 对应 Java: `ClassLoaderUtils#findResource(String)`。
    #[must_use]
    pub fn find_resource(resource_name: &str) -> Option<PathBuf> {
        let resource_name = resource_name.strip_prefix('/').unwrap_or(resource_name);
        Self::get_resource_roots()
            .into_iter()
            .map(|root| root.join(resource_name))
            .find(|candidate| candidate.is_file())
    }

    /// 判断资源是否存在于当前有序资源路径中。
    ///
    /// 对应 Java: `ClassLoaderUtils#isResourcePresent(String)`。
    #[must_use]
    pub fn is_resource_present(resource_name: &str) -> bool {
        Self::find_resource(resource_name).is_some()
    }

    /// 打开资源；找不到时返回与 Java `IOException` 等价的 `NotFound`。
    ///
    /// 对应 Java: `ClassLoaderUtils#loadResourceAsStream(String)`。
    pub fn load_resource_as_stream(resource_name: &str) -> io::Result<Box<dyn Read>> {
        Self::find_resource_as_stream(resource_name)?.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!(
                    "Could not locate resource '{resource_name}' in the application's resource path"
                ),
            )
        })
    }

    /// 尝试打开资源；找不到时返回 `Ok(None)`。
    ///
    /// 对应 Java: `ClassLoaderUtils#findResourceAsStream(String)`。
    pub fn find_resource_as_stream(resource_name: &str) -> io::Result<Option<Box<dyn Read>>> {
        let Some(resource_path) = Self::find_resource(resource_name) else {
            return Ok(None);
        };
        let reader = BufReader::new(File::open(resource_path)?);
        Ok(Some(Box::new(reader)))
    }
}

fn registered_classes() -> &'static RwLock<HashSet<String>> {
    REGISTERED_CLASSES.get_or_init(|| RwLock::new(HashSet::new()))
}

fn push_unique(roots: &mut Vec<PathBuf>, root: &Path) {
    let root = root.to_path_buf();
    if !roots.contains(&root) {
        roots.push(root);
    }
}
