## 变更类型
- [ ] Java parity（对齐上游语义）
- [ ] 缺陷修复（fix）
- [ ] 新能力（feat）
- [ ] 文档 / 迁移台账
- [ ] CI / 工程
- [ ] 其他

## 概要

## Java parity 证据（parity/fix 必填）
- 上游锚点（类/方法 + 基线 10f9dd2）：
- 测试：corpus / golden / parity 文件名与结果：

## 自查
- [ ] `cargo fmt --all -- --check` 通过
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` 零告警
- [ ] 新增语义带测试证据（V2 mirrored / V3 golden）
- [ ] 迁移台账（对象级/语义/方法级）已同步
- [ ] CHANGELOG 已更新（如影响用户可见行为）
