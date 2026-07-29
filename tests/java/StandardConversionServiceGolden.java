import org.thymeleaf.context.IExpressionContext;
import org.thymeleaf.standard.expression.AbstractStandardConversionService;
import org.thymeleaf.standard.expression.IStandardConversionService;
import org.thymeleaf.standard.expression.NoOpToken;
import org.thymeleaf.standard.expression.StandardConversionService;

/**
 * 从固定 Thymeleaf Java 源码导出标准转换服务与 NO-OP 值 Golden。
 */
public final class StandardConversionServiceGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private StandardConversionServiceGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);
        noOpCases();
        defaultServiceCases();
        customServiceCases();
    }

    private static void noOpCases() {
        emit("noop.same", NoOpToken.VALUE == NoOpToken.VALUE);
        emit("noop.text", NoOpToken.VALUE.toString());
        emit("noop.identityEquals", NoOpToken.VALUE.equals(NoOpToken.VALUE));
        emit("noop.otherEquals", NoOpToken.VALUE.equals(new Object()));
    }

    private static void defaultServiceCases() {
        final IStandardConversionService service = new StandardConversionService();
        final String source = new String("source");

        emitOutcome("default.targetNull", () -> service.convert(null, source, null));
        emit("default.stringNull", service.convert(null, null, String.class));
        emit("default.stringIdentity", service.convert(null, source, String.class) == source);
        emit(
                "default.objectString",
                service.convert(null, new ToStringProbe("object", false), String.class));
        final String shared = new String("shared");
        emit(
                "default.objectStringIdentity",
                service.convert(null, new ToStringProbe(shared, false), String.class) == shared);
        emit(
                "default.objectNull",
                service.convert(null, new ToStringProbe(null, false), String.class));
        emitOutcome(
                "default.objectError",
                () -> service.convert(null, new ToStringProbe("unused", true), String.class));
        emitOutcome(
                "default.otherNull",
                () -> service.convert(null, null, Integer.class));
        emitOutcome(
                "default.otherObject",
                () -> service.convert(null, new ToStringProbe("object", false), Integer.class));
        emitOutcome(
                "default.arrayTarget",
                () -> service.convert(null, source, int[].class));
    }

    private static void customServiceCases() {
        final IStandardConversionService service = new CustomService();
        final IExpressionContext context = new IExpressionContext() {
        };
        final String source = new String("source");

        emit("custom.stringNull", service.convert(context, null, String.class));
        emit("custom.stringIdentity", service.convert(context, source, String.class) == source);
        emit(
                "custom.objectContext",
                service.convert(context, new ToStringProbe("object", false), String.class));
        emit(
                "custom.objectNullContext",
                service.convert(null, new ToStringProbe("object", false), String.class));
        emit("custom.otherNull", service.convert(context, null, Integer.class));
        emit(
                "custom.otherObject",
                service.convert(context, new ToStringProbe("object", false), Integer.class));
    }

    private static final class ToStringProbe {
        private final String value;
        private final boolean throwsException;

        private ToStringProbe(final String value, final boolean throwsException) {
            this.value = value;
            this.throwsException = throwsException;
        }

        @Override
        public String toString() {
            if (this.throwsException) {
                throw new IllegalStateException("boom");
            }
            return this.value;
        }
    }

    private static final class CustomService extends AbstractStandardConversionService {
        @Override
        protected String convertToString(
                final IExpressionContext context,
                final Object object) {
            return (context == null ? "null:" : "context:") + object.toString();
        }

        @Override
        @SuppressWarnings("unchecked")
        protected <T> T convertOther(
                final IExpressionContext context,
                final Object object,
                final Class<T> targetClass) {
            if (targetClass.equals(Integer.class)) {
                return (T) Integer.valueOf(object == null ? 7 : 8);
            }
            return super.convertOther(context, object, targetClass);
        }
    }

    private static void emitOutcome(final String key, final Operation operation) {
        try {
            emit(key, "OK:" + String.valueOf(operation.execute()));
        } catch (final Throwable throwable) {
            emit(
                    key,
                    "ERR:"
                            + throwable.getClass().getName()
                            + ":"
                            + String.valueOf(throwable.getMessage()));
        }
    }

    private static void emit(final String key, final Object value) {
        System.out.println(key + "=" + String.valueOf(value));
    }

    private interface Operation {
        Object execute();
    }
}
