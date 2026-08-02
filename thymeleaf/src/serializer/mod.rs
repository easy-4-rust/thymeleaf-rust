//! Standard Dialect 的 CSS 与 JavaScript 序列化合同。

mod i_standard_css_serializer;
mod i_standard_java_script_serializer;
mod standard_css_serializer;
mod standard_java_script_serializer;
mod standard_serializers;

pub use i_standard_css_serializer::IStandardCSSSerializer;
pub use i_standard_java_script_serializer::IStandardJavaScriptSerializer;
pub use standard_css_serializer::StandardCSSSerializer;
pub use standard_java_script_serializer::StandardJavaScriptSerializer;
pub use standard_serializers::StandardSerializers;
