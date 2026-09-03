//! 验证 `dtd-files/` 中 Thymeleaf 官方版本化 DTD 文件集的完整性与可解析性。
//!
//! 文件来源：Thymeleaf 2.1.6.RELEASE jar `org/thymeleaf/dtd/`（58 文件），
//! SHA-256 台账见 dtd-files/README.md；本测试逐字节比对防漂移。
#![cfg(feature = "dtd-validation")]

use std::collections::BTreeMap;
use std::path::PathBuf;

use oxixml_dtd::{DtdParser, FilesystemResolver};

/// 每个文件的预期 SHA-256（从 dtd-files/README.md 台账提取）。
#[rustfmt::skip]
fn expected_hashes() -> BTreeMap<&'static str, &'static str> {
    BTreeMap::from([
        ("standard/xhtml-attribs-1.mod", "ae985bddc77fc784faccca299fde617c7ccc26035a6e897c75559fd09bcfcf26"),
        ("standard/xhtml-base-1.mod", "8353c607164c447e33f1cfee0085f80bd5af235a1d4abe2efcedabca91cf3349"),
        ("standard/xhtml-bdo-1.mod", "9b5ee746173f11e54bf6127c256ebd58f4e09ba1eb430ee50dc1b863f041abc9"),
        ("standard/xhtml-blkphras-1.mod", "bf8c21ace0ca45e568c0a1ba7d72893e6fb1a44c37b96d3f8387786ae8273fc7"),
        ("standard/xhtml-blkpres-1.mod", "c683ca6a29f5de56918ab6080fbe2d6ff8dabeb3b9088fc1b604268bb8c1d6e3"),
        ("standard/xhtml-blkstruct-1.mod", "6a22e7ef8b9875942dda7c58dbbb341ed8fa860a152153bfad70957285ae6b4c"),
        ("standard/xhtml-charent-1.mod", "fbf04736016990f765a7d28befbdfbb45507acfa52d795bf8b75b2d88e58486b"),
        ("standard/xhtml-csismap-1.mod", "b1016b6e5dcde9976b823c77bb217bf48704d8f8d8b71dec7480b6416d824f18"),
        ("standard/xhtml-datatypes-1.mod", "2b9c46de47aa53ebb55f1543a9d04fd22c768b5c814cf901f70e60b1eb878365"),
        ("standard/xhtml-edit-1.mod", "2217bc4d203b7ae5dbc33dd968f868e1df34239c1e6331be64d7420dd1972af7"),
        ("standard/xhtml-events-1.mod", "ef567d3c2acaab2f40ce9f4a1bfcaf3453d323eb2af1db699803ea82313a948a"),
        ("standard/xhtml-form-1.mod", "2aeb684e305c5c72e58eb387d1742811414afa053ef0f9065545a88757bdb78b"),
        ("standard/xhtml-framework-1.mod", "3728bdb25e42fe353e8f0fef141e95cf922d88864d5dd2f25e30abdaa8a643ca"),
        ("standard/xhtml-hypertext-1.mod", "0526840209a10905e4aca911676c5a445d157a512b8f71be422a9df6bbdfb9a3"),
        ("standard/xhtml-image-1.mod", "3d445292c0ca7cc7f8d77775932b47517ef31b3587b00a59e75a2960f74c5289"),
        ("standard/xhtml-inlphras-1.mod", "62792fcf521ab3f8e01c7e48cce55ae90be0b45c1b94ac8bfbee809daf8dd203"),
        ("standard/xhtml-inlpres-1.mod", "b38ff25eab43cdf1c3dca2ffc13f534e35dd264193915af9b54b329d6618ec4a"),
        ("standard/xhtml-inlstruct-1.mod", "325f6af41508e2ccfaef9cc5e85bec76d5e700895070f4b5a07466c27fd47bb2"),
        ("standard/xhtml-inlstyle-1.mod", "f1fa93c85974b8c3014daf32c5c65938f91cf076c889dbbfbc36d6f4d0d0f008"),
        ("standard/xhtml-lat1.ent", "ae3177ba9489c978bcbb2124d8d7397ec8169819696489fa1159a87ac3ba3dd1"),
        ("standard/xhtml-link-1.mod", "605e7050fcd9bc3c1860f01de33da488ca91eeff7799cf11015f5e43caad9d3d"),
        ("standard/xhtml-list-1.mod", "82a93e22ed828b2c508663d966afe94f1aaa7d24c2f167b31a5f27ddd637e79e"),
        ("standard/xhtml-meta-1.mod", "97cfc157e86dd602273591f0f69fd93cf1c627dcac629bb4623a547c00717356"),
        ("standard/xhtml-object-1.mod", "4c52443ace57baac11553a3cb1d64df2b4c27fb80afbdebf2b778deeed29d239"),
        ("standard/xhtml-param-1.mod", "9cc3b136d6847c38f29d0cd8669d8cb0c7c4a90421a0c92b5fae230fec354600"),
        ("standard/xhtml-pres-1.mod", "91c020c4bd8e2490f76759e3acc1d7358efe0e2417929ba712ddafefb230df79"),
        ("standard/xhtml-qname-1.mod", "dda3ad99cac142f9e18947ff55d11a4210e1826d5276bb8e5247ae9968642a92"),
        ("standard/xhtml-ruby-1.mod", "a8777cf01e4022baf04e04136cf21775adb8ccd966bfd13bbb25a5befc401aff"),
        ("standard/xhtml-script-1.mod", "dad5857e9c35e468923495f4a16c3315de9069a589e6fc83873d72aff847d0bf"),
        ("standard/xhtml-special.ent", "5bc38075663d40b140eb2e7b4ef38103d4a190da8ed59117b4d39e8db3d4c7e2"),
        ("standard/xhtml-ssismap-1.mod", "9658fef17d0d202f0ce6db499591abe3d3fc7934cfc69652819812b9fc7a3a21"),
        ("standard/xhtml-struct-1.mod", "9a9c058fd15b828e1ff1e17faa8ff7b662147e22e10408117aaecc2a432da834"),
        ("standard/xhtml-style-1.mod", "37de64765ea5616497d8afcafbc5f835fd71b39f616b028ccce40aa6ab1263c6"),
        ("standard/xhtml-symbol.ent", "77bd677a1fe7f120796ff7238d0add5dde3a35c3676a3f9fb8473309164e115f"),
        ("standard/xhtml-table-1.mod", "64daa05ad40d4b81f39d36003e373434a9e1911ace042eb88fafdb0430c6321a"),
        ("standard/xhtml-text-1.mod", "9217c1d0ca86764a9691247e217949b50f581f92203fe0555091c04d1f9ec3b1"),
        ("standard/xhtml1-frameset.dtd", "bd2a72b0060a0e4600d2c3b3c2a7d39e63981d46f9ac777715051777017c9aec"),
        ("standard/xhtml1-strict.dtd", "56cd2441229d82e30fb936b98adae91dd5a72e96fa78e822aa7d7bc4f21da13c"),
        ("standard/xhtml1-transitional.dtd", "c5c60018edeb3e007d86958bf03cef67b634c55c7986b0b5fe5946e489cd4058"),
        ("standard/xhtml11-model-1.mod", "f4e25d8075aa46f7dc4107ead829d73b48378a6cb41295d9eaba0ba1b4976037"),
        ("standard/xhtml11.dtd", "c57e34caa66b32296a744d908a9b8b698ef1dd88102caa18cb0c4ba56969da71"),
        ("standard/xhtml5-legacy-wildcard.dtd", "ede827e15c1b58724fc05596da60695ccaa2ef2fce4542a0de6a48b3a13eff47"),
        ("thymeleaf/xhtml1-frameset-thymeleaf-1.dtd", "86c2628b3d064132f64998e5056b0d0c0220b79be56a66e35dab458124172870"),
        ("thymeleaf/xhtml1-frameset-thymeleaf-2.dtd", "1bd7823b254c9dd6c9cf8a623244c09b45b79f756926367b0f0b705a2565449d"),
        ("thymeleaf/xhtml1-frameset-thymeleaf-3.dtd", "f20e6c3c122855fece4c5561f014dc045774b7e353681fc1f07b99cb5e228b5c"),
        ("thymeleaf/xhtml1-frameset-thymeleaf-4.dtd", "79c0b7dc04b5f460b66d70d3c0ccb493c7272ab1ca5bf77928eef43638a52f2e"),
        ("thymeleaf/xhtml1-strict-thymeleaf-1.dtd", "e82146648b85c5f0bc4d3caf3ac2efa7b5e83f4a7ac8642edb0013ce12b4410f"),
        ("thymeleaf/xhtml1-strict-thymeleaf-2.dtd", "3423b60401f5273348453f84d75b9be48a146d8fd246e1f871748362c1460458"),
        ("thymeleaf/xhtml1-strict-thymeleaf-3.dtd", "9216461fd386fb942624118430618295562bd50272147e7f5b9df4e851de4a8d"),
        ("thymeleaf/xhtml1-strict-thymeleaf-4.dtd", "952c8ca3b7e4b50be22f8a635d0a6226957244ea8fb598a41c5cf6c84c80a9d8"),
        ("thymeleaf/xhtml1-transitional-thymeleaf-1.dtd", "d0be45a9004edfcdd2bf3a62a1bc4de342502494ef6b214c0a6644afc92db12d"),
        ("thymeleaf/xhtml1-transitional-thymeleaf-2.dtd", "3fad44e116ddaa8698c93e5c797823d75a9d6f0bf2add9f0a93f317096ad7bf0"),
        ("thymeleaf/xhtml1-transitional-thymeleaf-3.dtd", "6fd08eab35743162420af79dfceae09b5cce45e6e6b2629760ed51d71686367a"),
        ("thymeleaf/xhtml1-transitional-thymeleaf-4.dtd", "33678e4ef6768b8b46e20fdf226005d18c69ae8405311910defcfd0c5ea80d3d"),
        ("thymeleaf/xhtml11-thymeleaf-1.dtd", "97e2dd6ea5531342dc6fd6af6dd6df0442cedf7fbdd546404a06b1f1a77f596d"),
        ("thymeleaf/xhtml11-thymeleaf-2.dtd", "cdeba7f76f99913e537b6767c970c3441bc6ba498ea3f11a78fa03fced7da6c6"),
        ("thymeleaf/xhtml11-thymeleaf-3.dtd", "8e0522a41173b68ab77e9e4153f231fa0a8c3567c40b3ad4f174cbc8109afa84"),
        ("thymeleaf/xhtml11-thymeleaf-4.dtd", "111fa7e97278a45b17146182cdf3bf82af53da1c572ecb11289be056b9104208"),
    ])
}

#[test]
fn dtd_files_sha256_match_expected() {
    let expected = expected_hashes();
    let dtd_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../dtd-files/thymeleaf-2.1.6/org/thymeleaf/dtd");
    assert_eq!(
        expected.len(),
        std::fs::read_dir(&dtd_root)
            .and_then(|root| root.collect::<Result<Vec<_>, _>>())
            .map(|entries| {
                entries
                    .iter()
                    .filter(|entry| entry.path().is_dir())
                    .map(|dir| std::fs::read_dir(dir.path()).unwrap().count())
                    .sum::<usize>()
            })
            .unwrap_or(0),
        "dtd-files 文件数与台账条目数不一致——新增/删除文件必须同步更新台账",
    );
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

/// 计算文件的 SHA-256 哈希值。
fn sha256_hex(path: &PathBuf) -> String {
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
fn filesystem_resolver_parses_xhtml1_strict() {
    // FilesystemResolver 根目录 = standard/（各单体 DTD 无外部引用，
    // 模块文件亦可按裸文件名解析）。
    let dtd_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../dtd-files/thymeleaf-2.1.6/org/thymeleaf/dtd/standard");
    let main_dtd = std::fs::read_to_string(dtd_root.join("xhtml1-strict.dtd"))
        .expect("read xhtml1-strict.dtd");
    let resolver = FilesystemResolver::new(&dtd_root);
    let _dtd = DtdParser::new()
        .with_resolver(Box::new(resolver))
        .parse_external_subset(&main_dtd)
        .expect("parse xhtml1-strict DTD via filesystem resolver");
}

#[test]
fn embedded_resolver_parses_xhtml11() {
    // xhtml11 单体副本以绝对 URL 引用模块（FilesystemResolver 按安全设计
    // 拒绝 URL 键）；内嵌 MemoryResolver 注册了 URL→本地模块映射，
    // 这正是 DtdValidator 实际使用的解析路径，必须完全离线可解析。
    let declaration = "html SYSTEM \"http://www.w3.org/TR/xhtml11/DTD/xhtml11.dtd\"";
    let validator = thymeleaf::dtd::DtdValidator::new(declaration)
        .expect("xhtml11 DTD must parse via embedded resolver");
    let _ = validator;
}
