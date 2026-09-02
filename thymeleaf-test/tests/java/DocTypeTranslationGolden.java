import org.thymeleaf.TemplateEngine;
import org.thymeleaf.context.Context;
import org.thymeleaf.templateresolver.StringTemplateResolver;

/**
 * 导出 StandardTranslationDocTypeProcessor（Thymeleaf 专有 XHTML DTD
 * SystemID → W3C 标准 DOCTYPE 翻译）的端到端 Golden。
 *
 * 矩阵：16 个专有 SystemID 全枚举（strict/transitional/frameset/xhtml11
 * × thymeleaf-1..4）+ 类型/大小写/未知 ID/internalSubset/XML 模式边界。
 * 每条记录完整渲染输出（DOCTYPE 序列化字节）或异常类名。
 */
public final class DocTypeTranslationGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private DocTypeTranslationGolden() {
    }

    public static void main(final String[] args) {
        final TemplateEngine engine = new TemplateEngine();
        final StringTemplateResolver resolver = new StringTemplateResolver();
        resolver.setTemplateMode("HTML");
        engine.setTemplateResolver(resolver);

        emit(engine, "baseline_case", "<p>10f9dd2eb8cbd98515ce14b149d115e0287d0add</p>");

        // ---- XHTML 1.0 Strict（thymeleaf-1..4 全枚举）----
        emitDoctype(engine, "strict_1", "http://www.thymeleaf.org/dtd/xhtml1-strict-thymeleaf-1.dtd");
        emitDoctype(engine, "strict_2", "http://www.thymeleaf.org/dtd/xhtml1-strict-thymeleaf-2.dtd");
        emitDoctype(engine, "strict_3", "http://www.thymeleaf.org/dtd/xhtml1-strict-thymeleaf-3.dtd");
        emitDoctype(engine, "strict_4", "http://www.thymeleaf.org/dtd/xhtml1-strict-thymeleaf-4.dtd");

        // ---- XHTML 1.0 Transitional ----
        emitDoctype(engine, "transitional_1", "http://www.thymeleaf.org/dtd/xhtml1-transitional-thymeleaf-1.dtd");
        emitDoctype(engine, "transitional_2", "http://www.thymeleaf.org/dtd/xhtml1-transitional-thymeleaf-2.dtd");
        emitDoctype(engine, "transitional_3", "http://www.thymeleaf.org/dtd/xhtml1-transitional-thymeleaf-3.dtd");
        emitDoctype(engine, "transitional_4", "http://www.thymeleaf.org/dtd/xhtml1-transitional-thymeleaf-4.dtd");

        // ---- XHTML 1.0 Frameset ----
        emitDoctype(engine, "frameset_1", "http://www.thymeleaf.org/dtd/xhtml1-frameset-thymeleaf-1.dtd");
        emitDoctype(engine, "frameset_2", "http://www.thymeleaf.org/dtd/xhtml1-frameset-thymeleaf-2.dtd");
        emitDoctype(engine, "frameset_3", "http://www.thymeleaf.org/dtd/xhtml1-frameset-thymeleaf-3.dtd");
        emitDoctype(engine, "frameset_4", "http://www.thymeleaf.org/dtd/xhtml1-frameset-thymeleaf-4.dtd");

        // ---- XHTML 1.1 ----
        emitDoctype(engine, "xhtml11_1", "http://www.thymeleaf.org/dtd/xhtml11-thymeleaf-1.dtd");
        emitDoctype(engine, "xhtml11_2", "http://www.thymeleaf.org/dtd/xhtml11-thymeleaf-2.dtd");
        emitDoctype(engine, "xhtml11_3", "http://www.thymeleaf.org/dtd/xhtml11-thymeleaf-3.dtd");
        emitDoctype(engine, "xhtml11_4", "http://www.thymeleaf.org/dtd/xhtml11-thymeleaf-4.dtd");

        // ---- 边界：大小写（Java HashMap 大小写敏感 → 不翻译）----
        emitDoctype(engine, "case_sensitive_upper_host",
                "HTTP://WWW.THYMELEAF.ORG/dtd/xhtml1-strict-thymeleaf-1.dtd");
        emitDoctype(engine, "case_sensitive_upper_path",
                "http://www.thymeleaf.org/dtd/XHTML1-STRICT-THYMELEAF-1.DTD");

        // ---- 边界：PUBLIC 类型 doctype（处理器只处理 SYSTEM）----
        emitRaw(engine, "public_type_not_translated",
                "<!DOCTYPE html PUBLIC \"-//W3C//DTD XHTML 1.0 Transitional//EN\" "
                        + "\"http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd\">\n"
                        + "<html><body><p>x</p></body></html>");

        // ---- 边界：thymeleaf.org 前缀正确但未知版本/未知名 → 不翻译 ----
        emitDoctype(engine, "unknown_version_5",
                "http://www.thymeleaf.org/dtd/xhtml1-strict-thymeleaf-5.dtd");
        emitDoctype(engine, "unknown_name",
                "http://www.thymeleaf.org/dtd/some-other.dtd");

        // ---- 边界：完全无关 systemId → 不翻译 ----
        emitDoctype(engine, "unrelated_system_id",
                "http://example.org/dtd/unrelated.dtd");

        // ---- 边界：internalSubset 保留 ----
        emit(engine, "internal_subset_preserved",
                "<!DOCTYPE html SYSTEM \"http://www.thymeleaf.org/dtd/xhtml1-strict-thymeleaf-1.dtd\" [\n"
                        + "<!ELEMENT html ANY>\n]>\n<html><body><p>x</p></body></html>");

        // ---- 边界：XML 模式（处理器仅注册于 HTML 模式 → 不翻译）----
        final TemplateEngine xmlEngine = new TemplateEngine();
        final StringTemplateResolver xmlResolver = new StringTemplateResolver();
        xmlResolver.setTemplateMode("XML");
        xmlEngine.setTemplateResolver(xmlResolver);
        emit(xmlEngine, "xml_mode_not_translated",
                "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n"
                        + "<!DOCTYPE html SYSTEM \"http://www.thymeleaf.org/dtd/xhtml1-strict-thymeleaf-1.dtd\">\n"
                        + "<html><body><p>x</p></body></html>");
    }

    /** SYSTEM DOCTYPE 模板。 */
    private static void emitDoctype(final TemplateEngine engine, final String id,
            final String systemId) {
        emit(engine, id,
                "<!DOCTYPE html SYSTEM \"" + systemId + "\">\n"
                        + "<html><body><p>x</p></body></html>");
    }

    /** 原样模板（PUBLIC 类型等）。 */
    private static void emitRaw(final TemplateEngine engine, final String id,
            final String template) {
        emit(engine, id, template);
    }

    private static void emit(final TemplateEngine engine, final String id,
            final String template) {
        String outcome;
        try {
            outcome = engine.process(template, new Context());
        } catch (final Throwable error) {
            outcome = "EXCEPTION:" + error.getClass().getSimpleName();
        }
        outcome = outcome.replace("\n", "\\n").replace("\r", "\\r").replace("\t", "\\t");
        System.out.println(id + "\t" + outcome);
    }
}
