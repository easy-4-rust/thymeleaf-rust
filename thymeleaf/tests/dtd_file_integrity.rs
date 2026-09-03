//! DTD 文件完整性校验：SHA-256 比对 + FilesystemResolver 解析验证。
//!
//! `#[cfg(feature = "dtd-validation")]`：仅在 dtd-validation feature 启用时编译。
//! 验证 `dtd-files/` 中的 W3C XHTML DTD 文件完整性与可解析性。

#[cfg(feature = "dtd-validation")]
mod integrity_tests {
    use oxixml_dtd::{DtdParser, FilesystemResolver};
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};

    /// 每个 DTD 文件的预期 SHA-256 哈希值（从 dtd-files/README.md 提取）。
    fn expected_hashes() -> HashMap<&'static str, &'static str> {
        let mut map = HashMap::new();
        map.insert(
            "xhtml1/xhtml1-strict.dtd",
            "c477851fac23a922dcfdbde97d2f8c3cdc6dbf3e5abc298b31b6601369e71d40",
        );
        map.insert(
            "xhtml1/xhtml1-transitional.dtd",
            "28905cd059167bfa1a5866053799d55fd335c0717920a7048de400c8a32b9d93",
        );
        map.insert(
            "xhtml1/xhtml1-frameset.dtd",
            "6110f1b3409ad7dd00e6acc1eebd2c5b29220d4fd3f57a81a7d8b7c5355bc3f9",
        );
        map.insert(
            "xhtml1/xhtml-strict-model-1.mod",
            "c5dd5baea0f05b9e6488e087252a0d9712c9533657b665a6357a8a8fcd503c2d",
        );
        map.insert(
            "xhtml1/xhtml-framework-1.mod",
            "c5dd5baea0f05b9e6488e087252a0d9712c9533657b665a6357a8a8fcd503c2d",
        );
        map.insert(
            "xhtml1/xhtml-lat1.ent",
            "3535a3cf7672ab1a511e4edd094e8e1da8b5874aba8ee8851bd2861d25b0dfd9",
        );
        map.insert(
            "xhtml1/xhtml-special.ent",
            "348d006519736b764a86fd24aed49ad35114f030ede0f263d3c4638f04e12107",
        );
        map.insert(
            "xhtml1/xhtml-symbol.ent",
            "5b173003c47aba07879397bccdd23ef240eb7578c6345a84f3453617410b7e7d",
        );
        map.insert(
            "xhtml11/xhtml11.dtd",
            "0665119e6d6ace3dd677f35dc673b2c9f685c3760ff4b5455f0f7b1a1756ba76",
        );
        map.insert(
            "xhtml11/xhtml11-model-1.mod",
            "f4e25d8075aa46f7dc4107ead829d73b48378a6cb41295d9eaba0ba1b4976037",
        );
        map.insert(
            "xhtml11/xhtml11-framework-1.mod",
            "c5dd5baea0f05b9e6488e087252a0d9712c9533657b665a6357a8a8fcd503c2d",
        );
        map.insert(
            "xhtml11/xhtml11-lat1.ent",
            "c5dd5baea0f05b9e6488e087252a0d9712c9533657b665a6357a8a8fcd503c2d",
        );
        map.insert(
            "xhtml11/xhtml11-special.ent",
            "c5dd5baea0f05b9e6488e087252a0d9712c9533657b665a6357a8a8fcd503c2d",
        );
        map.insert(
            "xhtml11/xhtml11-symbol.ent",
            "c5dd5baea0f05b9e6488e087252a0d9712c9533657b665a6357a8a8fcd503c2d",
        );
        map
    }

    /// 计算文件的 SHA-256 哈希值。
    fn sha256_hex(path: &Path) -> String {
        use sha2::{Digest, Sha256};
        use std::io::Read;
        let mut file = std::fs::File::open(path).expect("open DTD file");
        let mut hasher = Sha256::new();
        let mut buf = [0u8; 8192];
        loop {
            let n = file.read(&mut buf).expect("read DTD file");
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
        }
        hasher
            .finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }

    #[test]
    fn dtd_files_sha256_match_expected() {
        let expected = expected_hashes();
        let dtd_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../dtd-files");
        for (rel_path, expected_hash) in &expected {
            let full = dtd_root.join(rel_path);
            assert!(full.exists(), "DTD file missing: {}", rel_path);
            let actual = sha256_hex(&full);
            assert_eq!(
                actual, *expected_hash,
                "SHA-256 mismatch for {}: expected {} got {}",
                rel_path, expected_hash, actual
            );
        }
    }

    #[test]
    fn filesystem_resolver_parses_xhtml1_strict() {
        // FilesystemResolver 根目录 = DTD 文件所在目录（xhtml1/），使 SYSTEM 标识符
        // "xhtml-lat1.ent" 正确解析为 dtd-files/xhtml1/xhtml-lat1.ent。
        let dtd_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../dtd-files/xhtml1");
        let main_dtd = std::fs::read_to_string(dtd_root.join("xhtml1-strict.dtd"))
            .expect("read xhtml1-strict.dtd");
        let resolver = FilesystemResolver::new(&dtd_root);
        let _dtd = DtdParser::new()
            .with_resolver(Box::new(resolver))
            .parse_external_subset(&main_dtd)
            .expect("parse xhtml1-strict DTD via filesystem resolver");
    }

    #[test]
    fn xhtml11_external_references_cannot_be_local_resolved() {
        // 【已知限制】xhtml11.dtd 的外部模块引用是绝对 HTTP URL
        // （如 http://www.w3.org/MarkUp/DTD/xhtml-inlstyle-1.mod）——
        // 无法用本地 FilesystemResolver 解析（自动拒绝非相对路径）。
        // 此测试验证该限制被正确捕获（解析失败但不 panic），
        // 未来如需完整解析需用 MemoryResolver 注册每个 HTTP URL 到本地内容。
        let dtd_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../dtd-files");
        let main_dtd = std::fs::read_to_string(dtd_root.join("xhtml11/xhtml11.dtd"))
            .expect("read xhtml11.dtd");
        let resolver = FilesystemResolver::new(&dtd_root);
        let result = DtdParser::new()
            .with_resolver(Box::new(resolver))
            .parse_external_subset(&main_dtd);
        // FilesystemResolver 正确拒绝 HTTP URL → Err 而非 panic
        assert!(
            result.is_err(),
            "xhtml11 HTTP URL references should fail (not panic)"
        );
    }
}
