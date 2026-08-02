use std::io;

/// Java `java.io.Writer` 的 UTF-16 输出适配合同。
///
/// Thymeleaf 内部以 Java `char` 代码单元工作，不能直接退化为 Rust UTF-8
/// `std::io::Write`，否则孤立代理项及字符计数会改变。宿主适配器负责决定最终编码
/// 或保存 UTF-16，并通过 `io::Result` 映射 Java `IOException`。
pub trait JavaWriter: Send {
    /// 写出完整 UTF-16 代码单元切片。
    ///
    /// # 参数
    ///
    /// - `characters`：按 Java `Writer#write(char[])` 语义写出的代码单元。
    ///
    /// # 错误
    ///
    /// 底层输出失败时返回 I/O 错误。
    fn write_utf16(&mut self, characters: &[u16]) -> io::Result<()>;

    /// 刷新底层输出。
    ///
    /// 对应 Java: `Writer#flush()`。不需要缓冲的实现可沿用默认空操作。
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }

    /// 关闭底层输出。
    ///
    /// 对应 Java: `Writer#close()`。默认实现先刷新，拥有外部资源的适配器可以覆盖。
    fn close(&mut self) -> io::Result<()> {
        self.flush()
    }
}
