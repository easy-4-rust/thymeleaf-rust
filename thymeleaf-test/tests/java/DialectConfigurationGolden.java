import org.thymeleaf.DialectConfiguration;
import org.thymeleaf.dialect.AbstractDialect;
import org.thymeleaf.dialect.IDialect;

/**
 * 从固定 Thymeleaf Java 源码导出方言基础对象与方言配置 Golden。
 */
public final class DialectConfigurationGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private DialectConfigurationGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);
        exportAbstractDialect();
        exportDialectConfiguration();
    }

    private static void exportAbstractDialect() {
        emit("abstract.name", new TestDialect("Test").getName());
        emit("abstract.empty", new TestDialect("").getName());
        emit("abstract.unicode", new TestDialect("标准方言").getName());
        emitFailure("abstract.null", () -> new TestDialect(null));

        final IDialect nullableNameDialect = () -> null;
        emit("interface.null_name", nullableNameDialect.getName());
    }

    private static void exportDialectConfiguration() {
        final IDialect dialect = new TestDialect("Test");

        final DialectConfiguration defaults = new DialectConfiguration(dialect);
        emit("default.specified", defaults.isPrefixSpecified());
        emit("default.prefix", defaults.getPrefix());
        emit("default.dialect_identity", defaults.getDialect() == dialect);
        emit("default.dialect_name", defaults.getDialect().getName());

        final DialectConfiguration explicitNull = new DialectConfiguration(null, dialect);
        emit("explicit_null.specified", explicitNull.isPrefixSpecified());
        emit("explicit_null.prefix", explicitNull.getPrefix());
        emit("explicit_null.dialect_identity", explicitNull.getDialect() == dialect);

        final DialectConfiguration empty = new DialectConfiguration("", dialect);
        emit("empty.specified", empty.isPrefixSpecified());
        emit("empty.prefix", empty.getPrefix());

        final DialectConfiguration custom = new DialectConfiguration("th", dialect);
        emit("custom.specified", custom.isPrefixSpecified());
        emit("custom.prefix", custom.getPrefix());

        final DialectConfiguration whitespace = new DialectConfiguration(" \t", dialect);
        emit("whitespace.prefix", whitespace.getPrefix());

        emitFailure("null.default", () -> new DialectConfiguration((IDialect) null));
        emitFailure("null.explicit", () -> new DialectConfiguration("th", null));
    }

    private static void emitFailure(final String key, final Operation operation) {
        try {
            operation.run();
            emit(key + ".class", "<none>");
            emit(key + ".message", "<none>");
        } catch (final RuntimeException exception) {
            emit(key + ".class", exception.getClass().getSimpleName());
            emit(key + ".message", exception.getMessage());
        }
    }

    private static void emit(final String key, final Object value) {
        System.out.println(key + "=" + escape(String.valueOf(value)));
    }

    private static String escape(final String value) {
        return value.replace("\\", "\\\\").replace("\t", "\\t");
    }

    private interface Operation {
        void run();
    }

    private static final class TestDialect extends AbstractDialect {

        private TestDialect(final String name) {
            super(name);
        }
    }
}
