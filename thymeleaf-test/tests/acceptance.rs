//! 全项目验收门禁：source-test parity manifest 完整性 + 资产字节一致性。
//!
//! 依据 rust-java-migration-testing 技能：SOURCE_PARITY 必须有 TEST_CASE 与
//! TEST_ASSET 两个部分，且每个资产按 SHA-256 校验字节一致。

use std::collections::HashSet;
use std::path::PathBuf;

use serde_json::Value;

const MANIFEST: &str = include_str!("../source-test-parity.json");
const BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

fn sha256(path: &std::path::Path) -> String {
    let bytes = std::fs::read(path).expect("asset must exist");
    let mut hasher = SimpleSha256::new();
    hasher.update(&bytes);
    hasher.finish()
}

// 轻量 SHA-256（不引入依赖）：仅用于清单校验，与生成脚本一致
struct SimpleSha256 {
    state: [u32; 8],
    buffer: Vec<u8>,
}

impl SimpleSha256 {
    fn new() -> Self {
        Self {
            state: [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
                0x5be0cd19,
            ],
            buffer: Vec::new(),
        }
    }
    fn update(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
    }
    fn finish(&mut self) -> String {
        let bit_len = (self.buffer.len() as u64).wrapping_mul(8);
        self.buffer.push(0x80);
        while self.buffer.len() % 64 != 56 {
            self.buffer.push(0);
        }
        self.buffer.extend_from_slice(&bit_len.to_be_bytes());
        let k: [u32; 64] = [
            0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
            0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
            0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
            0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
            0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
            0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
            0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
            0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
            0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
            0xc67178f2,
        ];
        let mut state = self.state;
        for chunk in self.buffer.chunks(64) {
            let mut w = [0u32; 64];
            for i in 0..16 {
                w[i] = u32::from_be_bytes([
                    chunk[i * 4],
                    chunk[i * 4 + 1],
                    chunk[i * 4 + 2],
                    chunk[i * 4 + 3],
                ]);
            }
            for i in 16..64 {
                let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
                let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
                w[i] = w[i - 16]
                    .wrapping_add(s0)
                    .wrapping_add(w[i - 7])
                    .wrapping_add(s1);
            }
            let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h) = (
                state[0], state[1], state[2], state[3], state[4], state[5], state[6], state[7],
            );
            for i in 0..64 {
                let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
                let ch = (e & f) ^ (!e & g);
                let temp1 = h
                    .wrapping_add(s1)
                    .wrapping_add(ch)
                    .wrapping_add(k[i])
                    .wrapping_add(w[i]);
                let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
                let maj = (a & b) ^ (a & c) ^ (b & c);
                let temp2 = s0.wrapping_add(maj);
                h = g;
                g = f;
                f = e;
                e = d.wrapping_add(temp1);
                d = c;
                c = b;
                b = a;
                a = temp1.wrapping_add(temp2);
            }
            state[0] = state[0].wrapping_add(a);
            state[1] = state[1].wrapping_add(b);
            state[2] = state[2].wrapping_add(c);
            state[3] = state[3].wrapping_add(d);
            state[4] = state[4].wrapping_add(e);
            state[5] = state[5].wrapping_add(f);
            state[6] = state[6].wrapping_add(g);
            state[7] = state[7].wrapping_add(h);
        }
        state.iter().map(|v| format!("{v:08x}")).collect::<String>()
    }
}

#[test]
fn manifest_pins_baseline_and_full_denominator() {
    let manifest: Value = serde_json::from_str(MANIFEST).expect("valid JSON");
    assert_eq!(manifest["schema_version"], 1);
    assert_eq!(manifest["upstream"]["baseline"], BASELINE);
    assert_eq!(manifest["upstream"]["version"], "3.1.5.RELEASE");

    let cases = manifest["test_case"]["entries"].as_array().expect("cases");
    // 上游 2,608 个可执行 .thtest
    assert_eq!(cases.len(), 2_608, "TEST_CASE denominator must be complete");

    let assets = manifest["test_asset"]["entries"]
        .as_array()
        .expect("assets");
    // 上游 .thtest 语料镜像：3,493 thtest + 77 golden（Java 源码不镜像，
    // 测试逻辑以 Rust 1:1 复刻于 thymeleaf-test/tests/*_java_parity.rs）
    assert_eq!(assets.len(), 3_570, "TEST_ASSET: 3,493 thtest + 77 golden");
}

#[test]
fn every_case_has_byte_identical_asset_copy() {
    let manifest: Value = serde_json::from_str(MANIFEST).expect("valid JSON");
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .to_path_buf();
    let cases = manifest["test_case"]["entries"].as_array().expect("cases");

    let mut seen = HashSet::new();
    for case in cases {
        let rel = case["source_relative_path"].as_str().expect("path");
        assert!(seen.insert(rel.to_owned()), "duplicate case path: {rel}");
        // 每个 case 对应的资产副本必须存在且字节一致
        let asset_rel = rel.strip_prefix("tests/").expect("tests/ prefix");
        let asset_path = root
            .join("thymeleaf-test/assets/thymeleaf-tests")
            .join(asset_rel);
        assert!(asset_path.exists(), "asset copy missing: {asset_rel}");
        let recorded = case["asset_sha256"].as_str().expect("sha");
        let actual = sha256(&asset_path);
        assert_eq!(actual, recorded, "asset hash mismatch: {asset_rel}");
    }
}

#[test]
fn every_asset_entry_matches_disk() {
    let manifest: Value = serde_json::from_str(MANIFEST).expect("valid JSON");
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .to_path_buf();
    let assets = manifest["test_asset"]["entries"]
        .as_array()
        .expect("assets");

    for asset in assets {
        let target = asset["target_path"].as_str().expect("target");
        let path = root.join(target);
        assert!(path.exists(), "asset missing on disk: {target}");
        let recorded = asset["sha256"].as_str().expect("sha");
        let actual = sha256(&path);
        assert_eq!(actual, recorded, "asset hash mismatch: {target}");
    }
}
