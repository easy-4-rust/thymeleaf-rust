# RAII 守卫与进度不变量审查规范

本规范源自一次"3 项已知限制修正"过程中暴露的三个同源 bug（`${${||}}` 无限递归、
`<L/ꟓ>` 14GB 内存膨胀、`then_some(Self)` 计数下溢 panic）。三者根因收敛到同一条
移植债：**把 Java 的迭代算法搬成 Rust 的递归 + RAII + 手写推进循环，却没有为"病态
输入下不前进"的失败模式做结构性兜底。**

本规范的目标是把"进度不变量"从注释/约定变成**编译期拒绝或 PR 审查拒绝**。

## 规则 1：带 Drop 的守卫必须惰性构造

### 禁令

**禁止** `bool::then_some(带 Drop 的值)`。

`then_some` 的参数是**急切求值**的——即使 bool 为 false，参数也会先构造，再被丢弃。
对带 Drop 的守卫，这意味着：守卫值会被构造并 drop 一次，但对应的"配对操作"
（计数自增、栈 push、锁获取）**没有发生**，导致 drop 时计数下溢 / 栈下溢 / 重复释放。

最小复现（曾导致 `attempt to subtract with overflow` panic）：

```rust
// ❌ 错误：entered=false 时 Guard 被构造+丢弃，但计数没自增
fn try_enter() -> Option<Guard> {
    let entered = COUNTER.with(|c| { if c.get() < MAX { c.set(c.get()+1); true } else { false } });
    entered.then_some(Guard)   // Guard 无条件构造
}

// ✅ 正确：惰性构造，只有 entered=true 才创建
fn try_enter() -> Option<Guard> {
    let entered = COUNTER.with(|c| { if c.get() < MAX { c.set(c.get()+1); true } else { false } });
    entered.then(|| Guard)     // 或直接 if entered { Some(Guard) } else { None }
}
```

### 适用范围

任何 `impl Drop` 的守卫类型（计数守卫、栈帧守卫、锁守卫、临时状态切换守卫），
若其构造与"配对副作用"分离（先判定副作用，再返回守卫），**必须**用惰性构造。

### 已知安全构造（不受本禁令影响）

```rust
let _guard = OgnlLocalsGuard;   // 字面量构造：构造 ⟺ 副作用已完成（副作用在上一行）
```

这种模式副作用在外层先执行，守卫只负责回滚——构造本身就是值，急切/惰性无差别。

### 审查方式

PR 中出现 `\.then_some\(` 时，审查者必须确认参数类型是否 `impl Drop`。
若是 → 拒绝，改 `then(|| ...)` 或 `if` 表达式。

## 规则 2：手写推进循环必须保证每轮 position 必增

### 问题模式

```rust
// ❌ 零前进无限循环（曾导致 14GB 内存膨胀）
while position < content_end {
    let name_start = position;
    while position < content_end {
        if matches!(source.as_bytes()[position], b'=' | b'/' | b'>') { break; }
        position += next_char_boundary(source, position);
    }
    // 内层 break 在首字符就触发 → name 为空 → position 没动
    result.push((name, value));   // 无限 push 空项
}
```

`<L/ꟓ>`、`<L=x>` 类输入：自闭合斜杠 / `=` 落在属性名位置，内层循环立即 break，
外层 while 的 `position` 一动不动，无限 `Vec::push` → 内存膨胀 + 100% CPU 挂起。

### 强制规则

任何 `while pos < bound { ... pos += ... }` 形式的循环，**必须满足以下其一**：

1. **编译期可证每轮 pos 必增**：循环体内所有路径都执行 `pos += <正数常量>` 或
   `pos = consume_*(...)`（consume_* 本身保证前进，见下）。
2. **运行期校验**：循环顶部 `let prev = pos;`，循环底部 `assert!(pos > prev)`
   或 `if pos == prev { break/return Err; }`。

### consume_* helper 必须自带零前进保护

```rust
// ✅ consume_whitespace：消费至少直到非空白或 end，不可能原地不动（除非已到 end）
fn consume_whitespace(source: &str, mut position: usize, end: usize) -> usize {
    while position < end && is_markup_whitespace(source.as_bytes()[position]) {
        position += 1;   // 常量推进，安全
    }
    position
}
```

但若 consume_* 内部用 `position += source[position..].chars().next().len_utf8()`，
则首字符为非法字节时可能 panic（非 char boundary）或——更隐蔽——外层调用者假设
"consume 必前进"而省略运行期校验。**所有 `position += source[position..]` 必须包在
保证前进的 helper 里，且 helper 对空匹配返回 `Err` 而非原位。**

### 审查方式

PR 中出现 `while.*<.*(content_end|end|len|maxi).*\{` 且循环体含
`position += source\[position` 时，审查者必须确认：
- 内层 break 后是否有 `if position == start { return Err / continue-with-advance }`。
- 没有 → 拒绝。

## 规则 3：递归下降必须有深度上限 + 进度不变量

### 问题模式

```rust
// ❌ 无限递归：substituted == selector 时以相同输入递归（曾导致 >60s 超时）
fn substitute(input) {
    if let Some(nested) = find_nested(input) {
        let substituted = substitute(nested);   // 内容递归：strictly shorter，OK
        if substituted != input {
            return substitute(substituted);     // ← 相同输入！死循环
        }
        // 落到此处本应继续，但原代码在 != 分支无条件递归
    }
}
```

### 强制规则

递归下降解析器/变换器必须同时具备：

1. **深度上限**：线程局部计数器 + RAII 守卫（注意规则 1 的惰性构造），超限返回失败。
2. **进度不变量**：每一层递归的输入必须严格小于上一层（更短 / 更小 / 更深），
   或递归结果必须与输入不同（否则落到非递归路径）。

二者缺一：
- 只有深度上限 → 病态输入在到上限前已消耗大量时间/内存（虽然必终止）。
- 只有进度不变量 → 不变量被破坏时（如本次 `substituted == selector`）无限递归 → 栈溢出。

### 迭代→递归的移植决策必须显式标注

Java 的解析算法多为迭代（状态机 + 索引），天然终止。移植为 Rust 递归时：

```rust
// ✅ 必须在 fn 注释里标注这是 Rust 侧新增的递归 + 配套防护
/// 嵌套 selector 递归处理的最大深度。
///
/// Java 上游是零递归的单遍状态机；本递归是为 `@{|/orders/${id}|}` 这类
/// "字面量替换嵌在另一个 simple expression 内"的场景增加的 Rust 侧辅助。
/// 深度上限保证病态嵌套输入绝对终止，不改变合法输入的处理结果。
const MAX_SUBSTITUTION_DEPTH: usize = 16;
```

注释必须说明：(a) Java 上游是否递归；(b) 为什么 Rust 需要递归；(c) 防护是什么。

## 历史 bug 案例索引

| 日期 | Bug | 规则 | Commit |
|---|---|---|---|
| 2026-08-09 | `then_some(Self)` 深度计数下溢 panic | 规则 1 | `20aa161` |
| 2026-08-09 | `markup_selector::parse_attributes` 零前进 14GB 膨胀 | 规则 2 | `b7d2716` |
| 2026-08-09 | `decoupled_template_logic_builder` 同源零前进 | 规则 2 | `a6de619` |
| 2026-08-09 | `literal_substitution_util` 相同输入无限递归 | 规则 3 | `017a3d1` |

## 审查清单（PR 模板用）

- [ ] 本 PR 是否新增 `impl Drop` 的守卫？若是，构造方式是否惰性（规则 1）？
- [ ] 本 PR 是否新增 `while pos < bound { ... pos += ... }` 循环？若是，每轮是否
      保证前进（规则 2）？
- [ ] 本 PR 是否新增递归下降？若是，是否有深度上限 + 进度不变量（规则 3）？
- [ ] 本 PR 是否把 Java 迭代算法改成 Rust 递归？若是，fn 注释是否说明原因与防护？
