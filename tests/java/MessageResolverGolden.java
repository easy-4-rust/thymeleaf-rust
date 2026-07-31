import java.io.IOException;
import java.io.Reader;
import java.io.StringReader;
import java.lang.reflect.Proxy;
import java.math.BigDecimal;
import java.math.BigInteger;
import java.util.ArrayList;
import java.util.Date;
import java.util.HashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.Properties;
import java.util.TimeZone;

import org.thymeleaf.context.ITemplateContext;
import org.thymeleaf.messageresolver.StandardMessageResolver;
import org.thymeleaf.templateresource.ITemplateResource;

/**
 * 从固定 Thymeleaf 3.1.5.RELEASE 导出消息解析器合同。
 */
public final class MessageResolverGolden {

    private static final String JAVA_BASELINE =
            "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private MessageResolverGolden() {
    }

    public static void main(final String[] args) {
        TimeZone.setDefault(TimeZone.getTimeZone("UTC"));
        emit("java_baseline", JAVA_BASELINE);
        final ProbeResolver resolver = new ProbeResolver();
        emit("abstract.name.default", resolver.getName());
        emit("abstract.order.default", resolver.getOrder());
        resolver.setName(null);
        resolver.setOrder(-7);
        emit("abstract.name.null", resolver.getName());
        emit("abstract.order.negative", resolver.getOrder());

        final Properties first = new Properties();
        first.setProperty("first", "one");
        first.setProperty("same", "old");
        resolver.setDefaultMessages(first);
        final Properties second = new Properties();
        second.setProperty("second", "two");
        second.setProperty("same", "new");
        resolver.setDefaultMessages(second);
        resolver.setDefaultMessages(null);
        emit("defaults.size", resolver.getDefaultMessages().size());
        emit("defaults.first", resolver.getDefaultMessages().getProperty("first"));
        emit("defaults.second", resolver.getDefaultMessages().getProperty("second"));
        emit("defaults.same", resolver.getDefaultMessages().getProperty("same"));
        emit("defaults.identity", resolver.getDefaultMessages() == resolver.getDefaultMessages());
        exportFailure("defaults.key_null", () -> resolver.addDefaultMessage(null, "v"));
        exportFailure("defaults.value_null", () -> resolver.addDefaultMessage("k", null));

        resolver.addDefaultMessage("plain", "unchanged");
        resolver.addDefaultMessage("indexed", "Hello {0}");
        final ITemplateContext en = context(Locale.US);
        emit("resolve.default.plain", resolver.resolveMessage(en, null, "plain", null));
        emit("resolve.default.indexed", resolver.resolveMessage(
                en, null, "indexed", new Object[] {"Rust"}));
        emit("resolve.default.missing", resolver.resolveMessage(en, null, "missing", null));
        emit("absent.en_US", resolver.createAbsentMessageRepresentation(
                en, null, "missing", null));
        exportFailure("resolve.context_null",
                () -> resolver.resolveMessage(null, null, "plain", null));
        exportFailure("resolve.key_null",
                () -> resolver.resolveMessage(en, null, null, null));
        exportFailure("absent.key_null",
                () -> resolver.createAbsentMessageRepresentation(en, null, null, null));
        exportFailure("absent.context_null",
                () -> resolver.createAbsentMessageRepresentation(null, null, "missing", null));

        emit("format.fast_identity",
                resolver.exposeFormat(Locale.US, "unchanged", null) == "unchanged");
        emit("format.open_brace_fast_path", resolver.exposeFormat(
                Locale.US, "left { open", null));
        emit("format.indexed", resolver.exposeFormat(
                Locale.US, "{0} / {1}", new Object[] {"A", 12}));
        emit("format.null_array", resolver.exposeFormat(
                Locale.US, "{0}", null));
        emit("format.explicit_null", resolver.exposeFormat(
                Locale.US, "{0}", new Object[] {null}));
        emit("format.default_date", resolver.exposeFormat(
                Locale.US, "{0}", new Object[] {new Date(0)}));
        emit("format.quote_literal", resolver.exposeFormat(
                Locale.US, "'{0}' {0}", new Object[] {"A"}));
        emit("format.quote_double", resolver.exposeFormat(
                Locale.US, "L''amour {0}", new Object[] {"A"}));
        emit("format.quote_unclosed", resolver.exposeFormat(
                Locale.US, "before '{0} after", new Object[] {"A"}));
        emit("format.quote_nested", resolver.exposeFormat(
                Locale.US, "a''b '{' {0} '}'", new Object[] {"A"}));
        emit("format.missing_parameter", resolver.exposeFormat(
                Locale.US, "{0}-{2}", new Object[] {"A"}));
        emit("format.parameter_surrogate_hex", utf16Hex(resolver.exposeFormat(
                Locale.US, "{0}", new Object[] {
                        new String(new char[] {'x', '\ud800', 'y'})
                })));
        emit("format.pattern_surrogate_hex", utf16Hex(resolver.exposeFormat(
                Locale.US, new String(new char[] {'x', '\ud800', '\'', '\'', 'y'}), null)));
        emit("format.number.us", resolver.exposeFormat(
                Locale.US, "{0,number}", new Object[] {12345.5}));
        emit("format.number.de", resolver.exposeFormat(
                Locale.GERMANY, "{0,number}", new Object[] {12345.5}));
        emit("format.integer", resolver.exposeFormat(
                Locale.US, "{0,number,integer}", new Object[] {12345.6}));
        emit("format.percent", resolver.exposeFormat(
                Locale.US, "{0,number,percent}", new Object[] {0.125}));
        emit("format.currency", resolver.exposeFormat(
                Locale.US, "{0,number,currency}", new Object[] {12345.5}));
        emit("format.number.custom", resolver.exposeFormat(
                Locale.US, "{0,number,#,##0.00}", new Object[] {12345.5}));
        emit("format.number.negative_default", resolver.exposeFormat(
                Locale.US, "{0,number,#,##0.00}", new Object[] {-12345.5}));
        emit("format.number.negative_subpattern", resolver.exposeFormat(
                Locale.US, "{0,number,#,##0.00;(#,##0.00)}", new Object[] {-12345.5}));
        emit("format.number.optional_fraction", resolver.exposeFormat(
                Locale.US, "{0,number,0000.##}", new Object[] {12.5}));
        emit("format.number.percent_pattern", resolver.exposeFormat(
                Locale.US, "{0,number,0.0%}", new Object[] {0.125}));
        emit("format.number.permille_pattern", resolver.exposeFormat(
                Locale.US, "{0,number,0.0\u2030}", new Object[] {0.125}));
        emit("format.number.quoted_affix", resolver.exposeFormat(
                Locale.US, "{0,number,'USD' #,##0.00 'net'}", new Object[] {12345.5}));
        emit("format.number.currency_pattern", resolver.exposeFormat(
                Locale.US, "{0,number,\u00a4 #,##0.00}", new Object[] {12345.5}));
        emit("format.number.scientific", resolver.exposeFormat(
                Locale.US, "{0,number,0.###E0}", new Object[] {12345.5}));
        emit("format.number.nan", resolver.exposeFormat(
                Locale.US, "{0,number}", new Object[] {Double.NaN}));
        emit("format.number.positive_infinity", resolver.exposeFormat(
                Locale.US, "{0,number}", new Object[] {Double.POSITIVE_INFINITY}));
        emit("format.number.negative_infinity", resolver.exposeFormat(
                Locale.US, "{0,number}", new Object[] {Double.NEGATIVE_INFINITY}));
        emit("format.number.long_max", resolver.exposeFormat(
                Locale.US, "{0,number,integer}", new Object[] {Long.MAX_VALUE}));
        emit("format.number.big_integer", resolver.exposeFormat(
                Locale.US, "{0,number,integer}",
                new Object[] {new BigInteger("123456789012345678901234567890")}));
        emit("format.number.big_decimal", resolver.exposeFormat(
                Locale.US, "{0,number,#,##0.0000}",
                new Object[] {new BigDecimal("12345678901234567890.1250")}));
        emit("format.number.round_half_even", resolver.exposeFormat(
                Locale.US, "{0,number,0}", new Object[] {2.5}));
        emit("format.number.round_half_even_odd", resolver.exposeFormat(
                Locale.US, "{0,number,0}", new Object[] {3.5}));
        emit("format.choice", resolver.exposeFormat(
                Locale.US,
                "{0,choice,0#none|1#one|1<{0,number,integer} items}",
                new Object[] {3}));
        emit("format.choice.below_first", resolver.exposeFormat(
                Locale.US, "{0,choice,1#one|2#two}", new Object[] {0}));
        emit("format.choice.inclusive", resolver.exposeFormat(
                Locale.US, "{0,choice,0#zero|1#one|1<more}", new Object[] {1}));
        emit("format.choice.exclusive", resolver.exposeFormat(
                Locale.US, "{0,choice,0#zero|1#one|1<more}", new Object[] {1.0001}));
        emit("format.choice.infinity", resolver.exposeFormat(
                Locale.US, "{0,choice,0#finite|\u221e#infinite}",
                new Object[] {Double.POSITIVE_INFINITY}));
        emit("format.choice.quoted_pipe", resolver.exposeFormat(
                Locale.US, "{0,choice,0#'a|b'|1#one}", new Object[] {0}));
        emit("format.date.short", resolver.exposeFormat(
                Locale.US, "{0,date,short}", new Object[] {new Date(0)}));
        emit("format.date.medium", resolver.exposeFormat(
                Locale.US, "{0,date,medium}", new Object[] {new Date(0)}));
        emit("format.date.long", resolver.exposeFormat(
                Locale.US, "{0,date,long}", new Object[] {new Date(0)}));
        emit("format.date.full", resolver.exposeFormat(
                Locale.US, "{0,date,full}", new Object[] {new Date(0)}));
        emit("format.time.short", resolver.exposeFormat(
                Locale.US, "{0,time,short}", new Object[] {new Date(0)}));
        emit("format.time.medium", resolver.exposeFormat(
                Locale.US, "{0,time,medium}", new Object[] {new Date(0)}));
        emit("format.date.custom", resolver.exposeFormat(
                Locale.US, "{0,date,yyyy-MM-dd HH:mm:ss}", new Object[] {new Date(0)}));
        emit("format.date.de_full", resolver.exposeFormat(
                Locale.GERMANY, "{0,date,full}", new Object[] {new Date(0)}));
        emit("format.date.fr_long", resolver.exposeFormat(
                Locale.FRANCE, "{0,date,long}", new Object[] {new Date(0)}));
        emit("format.date.ja_short", resolver.exposeFormat(
                Locale.JAPAN, "{0,date,short}", new Object[] {new Date(0)}));
        emit("format.time.us_long", resolver.exposeFormat(
                Locale.US, "{0,time,long}", new Object[] {new Date(0)}));
        emit("format.time.de_full", resolver.exposeFormat(
                Locale.GERMANY, "{0,time,full}", new Object[] {new Date(0)}));
        emit("format.date.quoted_custom", resolver.exposeFormat(
                Locale.US, "{0,date,yyyy-MM-dd'T'HH:mm:ss XXX}", new Object[] {new Date(0)}));
        exportFailure("format.bad_index",
                () -> resolver.exposeFormat(Locale.US, "{x}", new Object[] {"A"}));
        exportFailure("format.bad_type",
                () -> resolver.exposeFormat(Locale.US, "{0,unknown}", new Object[] {"A"}));
        emit("format.unmatched_close",
                resolver.exposeFormat(Locale.US, "bad }", null));
        emit("format.unmatched_open",
                resolver.exposeFormat(Locale.US, "bad {0", new Object[] {"A"}));
        exportFailure("format.number.non_number",
                () -> resolver.exposeFormat(Locale.US, "{0,number}", new Object[] {"A"}));
        exportFailure("format.date.non_date",
                () -> resolver.exposeFormat(Locale.US, "{0,date}", new Object[] {"A"}));
        exportFailure("format.choice.bad",
                () -> resolver.exposeFormat(Locale.US, "{0,choice,bad}", new Object[] {1}));

        exportTemplateResources(resolver);
        exportExtensionPoints();
        resolver.clearDefaultMessages();
        emit("defaults.cleared", resolver.getDefaultMessages().size());
    }

    private static void exportTemplateResources(final ProbeResolver resolver) {
        final Map<String, String> resources = new HashMap<String, String>();
        resources.put("home.properties", "base=base\nsame=base\nunicode=你好 😀\n");
        resources.put("home_en.properties", "language=en\nsame=language\n");
        resources.put("home_en_US.properties", "country=US\nsame=country\n");
        resources.put("home_en_US-posix.properties", "variant=posix\nsame=variant\n");
        final ProbeResource resource = new ProbeResource("home", resources, new ArrayList<>());
        final Map<String, String> messages = resolver.exposeTemplate(
                "template", resource, new Locale("en", "US", "posix"));
        emit("resource.requested", String.join(",", resource.requested));
        emit("resource.size", messages.size());
        emit("resource.base", messages.get("base"));
        emit("resource.language", messages.get("language"));
        emit("resource.country", messages.get("country"));
        emit("resource.variant", messages.get("variant"));
        emit("resource.same", messages.get("same"));
        emit("resource.unicode", messages.get("unicode"));

        final ProbeResource noBase = new ProbeResource(null, resources, new ArrayList<>());
        emit("resource.null_base_size", resolver.exposeTemplate(
                "template", noBase, new Locale("", "US")).size());
        final ProbeResource emptyBase = new ProbeResource("", resources, new ArrayList<>());
        emit("resource.empty_base_size", resolver.exposeTemplate(
                "template", emptyBase, new Locale("", "US")).size());
        exportFailure("resource.locale_without_language",
                () -> resolver.exposeTemplate("template", resource, new Locale("", "US")));
        final ProbeResource variantWithoutCountry =
                new ProbeResource("variant", new HashMap<>(), new ArrayList<>());
        resolver.exposeTemplate(
                "template", variantWithoutCountry, new Locale("en", "", "posix"));
        emit("resource.variant_without_country_requested",
                String.join(",", variantWithoutCountry.requested));

        final Map<String, String> syntax = new HashMap<String, String>();
        syntax.put("syntax.properties",
                "# comment\n"
                + "! comment\n"
                + "space key : spaced value  \n"
                + "escaped\\ key\\:\\==escaped\\ value\\:\\=\n"
                + "continued=first\\\n    second\\\n\tthird\n"
                + "controls=tab\\tline\\nreturn\\rform\\fslash\\\\\n"
                + "unicodeEscape=\\u4f60\\u597d\n"
                + "duplicate=first\n"
                + "duplicate=second\n"
                + "emptyValue\n"
                + "=emptyKey\n");
        final Map<String, String> syntaxMessages = resolver.exposeTemplate(
                "template",
                new ProbeResource("syntax", syntax, new ArrayList<>()),
                Locale.US);
        emit("resource.syntax.size", syntaxMessages.size());
        emit("resource.syntax.space", syntaxMessages.get("space"));
        emit("resource.syntax.escaped_key", syntaxMessages.get("escaped key:="));
        emit("resource.syntax.continued", syntaxMessages.get("continued"));
        emit("resource.syntax.controls_hex", utf16Hex(syntaxMessages.get("controls")));
        emit("resource.syntax.unicode_escape", syntaxMessages.get("unicodeEscape"));
        emit("resource.syntax.duplicate", syntaxMessages.get("duplicate"));
        emit("resource.syntax.empty_value", syntaxMessages.get("emptyValue"));
        emit("resource.syntax.empty_key", syntaxMessages.get(""));

        final Map<String, String> malformed = new HashMap<String, String>();
        malformed.put("bad.properties", "bad=\\u12G4\n");
        exportFailure("resource.malformed_unicode",
                () -> resolver.exposeTemplate(
                        "template",
                        new ProbeResource("bad", malformed, new ArrayList<>()),
                        Locale.US));
    }

    private static void exportExtensionPoints() {
        final HookResolver resolver = new HookResolver();
        final ITemplateContext en = context(Locale.US);
        emit("hook.origin.value", resolver.resolveMessage(
                en, MessageResolverGolden.class, "origin", new Object[] {"p"}));
        emit("hook.origin.calls", resolver.originCalls);
        emit("hook.format.calls", resolver.formatCalls);
        emit("hook.absent.value", resolver.createAbsentMessageRepresentation(
                en, MessageResolverGolden.class, "missing", null));
        emit("hook.absent.calls", resolver.absentCalls);
    }

    private static ITemplateContext context(final Locale locale) {
        return (ITemplateContext) Proxy.newProxyInstance(
                MessageResolverGolden.class.getClassLoader(),
                new Class<?>[] {ITemplateContext.class},
                (proxy, method, arguments) -> {
                    if ("getLocale".equals(method.getName())) {
                        return locale;
                    }
                    if ("getTemplateStack".equals(method.getName())) {
                        return java.util.Collections.emptyList();
                    }
                    if ("toString".equals(method.getName())) {
                        return "MessageResolverGoldenContext";
                    }
                    throw new UnsupportedOperationException(method.getName());
                });
    }

    private static void exportFailure(final String key, final ThrowingRunnable action) {
        try {
            action.run();
            emit(key, "NO_ERROR");
        } catch (final Throwable error) {
            emit(key, error.getClass().getName() + "|" + error.getMessage());
        }
    }

    private static void emit(final String key, final Object value) {
        System.out.println(key + "=" + String.valueOf(value));
    }

    private static String utf16Hex(final String value) {
        if (value == null) {
            return "null";
        }
        final StringBuilder result = new StringBuilder(value.length() * 5);
        for (int i = 0; i < value.length(); i++) {
            if (i > 0) {
                result.append(',');
            }
            result.append(String.format("%04X", (int) value.charAt(i)));
        }
        return result.toString();
    }

    private interface ThrowingRunnable {
        void run() throws Exception;
    }

    private static final class ProbeResolver extends StandardMessageResolver {
        String exposeFormat(
                final Locale locale, final String message, final Object[] parameters) {
            return formatMessage(locale, message, parameters);
        }

        Map<String, String> exposeTemplate(
                final String template,
                final ITemplateResource resource,
                final Locale locale) {
            return resolveMessagesForTemplate(template, resource, locale);
        }
    }

    private static final class HookResolver extends StandardMessageResolver {
        int originCalls;
        int formatCalls;
        int absentCalls;

        @Override
        protected Map<String, String> resolveMessagesForOrigin(
                final Class<?> origin, final Locale locale) {
            this.originCalls++;
            return java.util.Collections.singletonMap("origin", "origin-{0}");
        }

        @Override
        protected String formatMessage(
                final Locale locale, final String message, final Object[] parameters) {
            this.formatCalls++;
            return "[" + super.formatMessage(locale, message, parameters) + "]";
        }

        @Override
        public String createAbsentMessageRepresentation(
                final ITemplateContext context,
                final Class<?> origin,
                final String key,
                final Object[] parameters) {
            this.absentCalls++;
            return "ABSENT:" + key;
        }
    }

    private static final class ProbeResource implements ITemplateResource {
        private final String baseName;
        private final Map<String, String> resources;
        private final List<String> requested;
        private final String selected;

        ProbeResource(
                final String baseName,
                final Map<String, String> resources,
                final List<String> requested) {
            this(baseName, resources, requested, null);
        }

        private ProbeResource(
                final String baseName,
                final Map<String, String> resources,
                final List<String> requested,
                final String selected) {
            this.baseName = baseName;
            this.resources = resources;
            this.requested = requested;
            this.selected = selected;
        }

        public String getDescription() {
            return selected == null ? baseName : selected;
        }

        public String getBaseName() {
            return baseName;
        }

        public boolean exists() {
            return selected == null || resources.containsKey(selected);
        }

        public Reader reader() throws IOException {
            final String contents = resources.get(selected);
            if (contents == null) {
                throw new IOException("missing " + selected);
            }
            return new StringReader(contents);
        }

        public ITemplateResource relative(final String relativeLocation) {
            requested.add(relativeLocation);
            return new ProbeResource(baseName, resources, requested, relativeLocation);
        }
    }
}
