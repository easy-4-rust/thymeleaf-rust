use std::io;

use super::{JavaCharSequence, TemplateWriter};

/// 可直接写入 Java writer 的字符序列合同。
///
/// 对应 Java: `org.thymeleaf.util.IWritableCharSequence`。
///
/// 该能力让引擎生成的大段动态文本无需先分配完整 `String` 即可进入响应输出；
/// 同时继承 `JavaCharSequence`，保留 UTF-16 长度和按代码单元访问语义。
pub trait IWritableCharSequence: JavaCharSequence {
    /// 将此字符序列的全部内容直接写入目标 writer。
    ///
    /// 对应 Java: `IWritableCharSequence#write(Writer)`。
    ///
    /// # 参数
    ///
    /// - `writer`：接收 UTF-16 代码单元的输出目标。
    ///
    /// # 错误
    ///
    /// 底层输出失败时返回 Java `IOException` 对应的 I/O 错误。
    fn write(&self, writer: &mut dyn TemplateWriter) -> io::Result<()>;
}
