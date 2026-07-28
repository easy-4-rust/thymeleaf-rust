/// 多种模板资源实现共享的路径处理工具。
///
/// 对应 Java: `org.thymeleaf.templateresource.TemplateResourceUtils`。
///
/// 这些方法刻意保留 Thymeleaf 自己的词法路径规则，不调用文件系统规范化：
/// `..` 只抵消普通路径段，Windows 分隔符先转换为 `/`，相对位置仍可保留尚未
/// 清理的 `..`。该对象在 Java 中为包可见工具类，因此 Rust 也只在 crate 内可见。
pub(crate) struct TemplateResourceUtils;

impl TemplateResourceUtils {
    /// 清理模板资源使用的词法路径。
    ///
    /// 对应 Java: `TemplateResourceUtils#cleanPath(String)`。
    ///
    /// # 参数
    /// - `path`：待清理路径；`None` 对应 Java `null`。
    ///
    /// # 返回
    /// 清理后的路径；输入为 `None` 时仍返回 `None`。
    pub(crate) fn clean_path(path: Option<&str>) -> Option<String> {
        let path = path?;
        let unix_path = path.replace('\\', "/");

        // Java 实现对常见路径走快捷分支，因而单独的 "."、".." 和 "./" 保持原样。
        if unix_path.is_empty() || (!unix_path.contains("/.") && !unix_path.contains("//")) {
            return Some(unix_path);
        }

        let root_based = unix_path.starts_with('/');
        let traversal_path = if root_based {
            unix_path
        } else {
            format!("/{unix_path}")
        };

        // 与 Java 一样从末尾向前遍历；先收集逆序的有效 token，最后再恢复原顺序。
        let mut valid_tokens = Vec::new();
        let mut top_count = 0_usize;
        for token in traversal_path.split('/').rev() {
            if token.is_empty() || token == "." {
                continue;
            }
            if token == ".." {
                top_count += 1;
            } else if top_count > 0 {
                top_count -= 1;
            } else {
                valid_tokens.push(token);
            }
        }

        let mut cleaned = String::new();
        for token in valid_tokens {
            cleaned.insert_str(0, token);
            cleaned.insert(0, '/');
        }
        for _ in 0..top_count {
            cleaned.insert_str(0, "/..");
        }

        if !root_based {
            // 对能完全抵消成空路径的相对输入，Java 的 deleteCharAt(0) 会抛出
            // StringIndexOutOfBoundsException；remove(0) 保留这一未检查失败语义。
            cleaned.remove(0);
        }
        Some(cleaned)
    }

    /// 以当前资源所在目录为基准计算相对资源位置。
    ///
    /// 对应 Java: `TemplateResourceUtils#computeRelativeLocation(String,String)`。
    ///
    /// 调用方必须先保证 `relative_location` 非空；Java 实现也直接读取其首字符。
    ///
    /// # 参数
    /// - `location`：当前资源位置。
    /// - `relative_location`：Java 参数 `relativeLocation`，即新的相对位置。
    ///
    /// # 返回
    /// 尚未执行路径清理的组合位置。
    pub(crate) fn compute_relative_location(location: &str, relative_location: &str) -> String {
        if let Some(separator_pos) = location.rfind('/') {
            let mut relative = String::with_capacity(location.len() + relative_location.len());
            relative.push_str(&location[..separator_pos]);
            if !relative_location.starts_with('/') {
                relative.push('/');
            }
            relative.push_str(relative_location);
            return relative;
        }
        relative_location.to_owned()
    }

    /// 从模板资源路径计算不含扩展名的 base name。
    ///
    /// 对应 Java: `TemplateResourceUtils#computeBaseName(String)`。
    ///
    /// # 参数
    /// - `path`：模板资源路径；`None` 对应 Java `null`。
    ///
    /// # 返回
    /// 可计算时返回 base name；根路径、空路径和 `None` 返回 `None`。
    pub(crate) fn compute_base_name(path: Option<&str>) -> Option<String> {
        let path = path?;
        if path.is_empty() {
            return None;
        }

        let base_path = path.strip_suffix('/').unwrap_or(path);
        if let Some(slash_pos) = base_path.rfind('/') {
            if let Some(dot_pos) = base_path.rfind('.') {
                if dot_pos > slash_pos + 1 {
                    return Some(base_path[slash_pos + 1..dot_pos].to_owned());
                }
            }
            return Some(base_path[slash_pos + 1..].to_owned());
        }

        if let Some(dot_pos) = base_path.rfind('.') {
            return Some(base_path[..dot_pos].to_owned());
        }
        (!base_path.is_empty()).then(|| base_path.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::TemplateResourceUtils;

    #[test]
    fn computes_relative_locations_with_java_directory_rules() {
        let cases = [
            ("/", "/", "/"),
            ("/", "something", "/something"),
            ("/something", "/", "/"),
            ("something", "/", "/"),
            ("something/else", "/", "something/"),
            ("something/else/more", "/", "something/else/"),
            ("something/else/more", "less", "something/else/less"),
            (
                "something/else/more.html",
                "more.properties",
                "something/else/more.properties",
            ),
            (
                "something/else/more.html",
                "more_es.properties",
                "something/else/more_es.properties",
            ),
            (
                "something/else/more.html",
                "../more_es.properties",
                "something/else/../more_es.properties",
            ),
            (
                "something/else/more.html",
                "../../more_es.properties",
                "something/else/../../more_es.properties",
            ),
        ];

        for (location, relative_location, expected) in cases {
            assert_eq!(
                TemplateResourceUtils::compute_relative_location(location, relative_location),
                expected
            );
        }
    }

    #[test]
    fn cleans_paths_with_the_upstream_reverse_traversal_algorithm() {
        let cases = [
            ("/", "/"),
            ("something", "something"),
            ("/something", "/something"),
            ("something/else", "something/else"),
            ("//something//else", "/something/else"),
            ("//something//a//..//else", "/something/else"),
            ("something/else/more", "something/else/more"),
            ("something/else//more", "something/else/more"),
            ("something/else/../more", "something/more"),
            ("something/else/./more", "something/else/more"),
            ("../something/else/./more", "../something/else/more"),
            ("./something/else/./more", "something/else/more"),
            ("something/else/more.html", "something/else/more.html"),
            ("../something/else/more.html", "../something/else/more.html"),
            ("../something/else/more.html/..", "../something/else"),
            (
                "something/else/more.html/../../more_es.properties",
                "something/more_es.properties",
            ),
            ("windows\\folder\\file.html", "windows/folder/file.html"),
            ("", ""),
            (".", "."),
            ("..", ".."),
            ("./", "./"),
        ];

        assert_eq!(TemplateResourceUtils::clean_path(None), None);
        for (path, expected) in cases {
            assert_eq!(
                TemplateResourceUtils::clean_path(Some(path)),
                Some(expected.to_owned())
            );
        }
    }

    #[test]
    #[should_panic(expected = "cannot remove a char")]
    fn preserves_java_failure_when_a_relative_path_is_fully_cancelled() {
        let _ = TemplateResourceUtils::clean_path(Some("segment/.."));
    }

    #[test]
    fn computes_base_names_with_java_dot_and_slash_boundaries() {
        let cases = [
            (None, None),
            (Some(""), None),
            (Some("/"), None),
            (Some("something"), Some("something")),
            (Some("/something"), Some("something")),
            (Some("something/else"), Some("else")),
            (Some("//something//else"), Some("else")),
            (Some("//something//a//..//else"), Some("else")),
            (Some("something/else/more"), Some("more")),
            (Some("something/else//more"), Some("more")),
            (Some("something/else/../more"), Some("more")),
            (Some("something/else/./more"), Some("more")),
            (Some("../something/else/./more"), Some("more")),
            (Some("./something/else/./more"), Some("more")),
            (Some("something/else/more.html"), Some("more")),
            (Some("../something/else/more.html"), Some("more")),
            (Some("../something/else/more.html/.."), Some(".")),
            (
                Some("something/else/more.html/../../more_es.properties"),
                Some("more_es"),
            ),
            (Some("more.html"), Some("more")),
            (Some(".hidden"), Some("")),
            (Some("folder/.hidden"), Some(".hidden")),
            (Some("folder/"), Some("folder")),
        ];

        for (path, expected) in cases {
            assert_eq!(
                TemplateResourceUtils::compute_base_name(path),
                expected.map(str::to_owned)
            );
        }
    }
}
