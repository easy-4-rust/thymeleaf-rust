import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collection;
import java.util.Collections;
import java.util.Iterator;
import java.util.LinkedHashSet;
import java.util.Set;
import java.util.TreeSet;

import org.thymeleaf.expression.Sets;
import org.thymeleaf.util.SetUtils;

/**
 * 从固定 Thymeleaf Java 源码导出 SetUtils 与 Sets Golden。
 */
public final class SetUtilsGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private SetUtilsGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);

        final Set<Object> source = new LinkedHashSet<Object>();
        source.add("two");
        source.add("one");
        source.add(null);
        final Set<Object> empty = Collections.emptySet();
        final Set<String> sorted = new TreeSet<String>(Arrays.asList("two", "one"));

        emitOutcome("convert.set.value", () -> render(SetUtils.toSet(source)));
        emitOutcome("convert.set.identity", () -> SetUtils.toSet(source) == source);
        emitOutcome("convert.set.sorted", () -> render(SetUtils.toSet(sorted)));
        emitOutcome("convert.array.value",
                () -> render(SetUtils.toSet(new Object[] {"two", "one", "two", null})));
        emitOutcome("convert.array.empty",
                () -> render(SetUtils.toSet(new Object[0])));
        emitOutcome("convert.iterable.value",
                () -> render(SetUtils.toSet(
                        new ArrayList<Object>(Arrays.<Object>asList("b", "a", "b", null)))));
        emitOutcome("convert.iterable.empty",
                () -> render(SetUtils.toSet(Collections.emptyList())));
        emitOutcome("convert.null", () -> render(SetUtils.toSet(null)));
        emitOutcome("convert.unsupported", () -> render(SetUtils.toSet(Integer.valueOf(1))));
        final Iterator<String> iterator = Arrays.asList("a", "b").iterator();
        emitOutcome("convert.iterator_not_iterable", () -> render(SetUtils.toSet(iterator)));
        emitOutcome("convert.primitive_array", () -> render(SetUtils.toSet(new int[] {1, 2})));

        emitOutcome("size.value", () -> SetUtils.size(source));
        emitOutcome("size.empty", () -> SetUtils.size(empty));
        emitOutcome("size.null", () -> SetUtils.size(null));
        emit("empty.value", Boolean.toString(SetUtils.isEmpty(source)));
        emit("empty.empty", Boolean.toString(SetUtils.isEmpty(empty)));
        emit("empty.null", Boolean.toString(SetUtils.isEmpty(null)));

        emitOutcome("contains.present", () -> SetUtils.contains(source, "two"));
        emitOutcome("contains.missing", () -> SetUtils.contains(source, "missing"));
        emitOutcome("contains.null", () -> SetUtils.contains(source, null));
        emitOutcome("contains.null_target", () -> SetUtils.contains(null, "two"));

        final Object[] present = new Object[] {"two", null};
        final Object[] missing = new Object[] {"two", "missing"};
        final Object[] duplicate = new Object[] {"two", "two"};
        emitOutcome("all.array.present", () -> SetUtils.containsAll(source, present));
        emitOutcome("all.array.missing", () -> SetUtils.containsAll(source, missing));
        emitOutcome("all.array.empty", () -> SetUtils.containsAll(source, new Object[0]));
        emitOutcome("all.array.duplicate", () -> SetUtils.containsAll(source, duplicate));
        emitOutcome("all.array.null_target",
                () -> SetUtils.containsAll(null, (Object[])null));
        emitOutcome("all.array.null_elements",
                () -> SetUtils.containsAll(source, (Object[])null));

        final Collection<Object> presentCollection = Arrays.<Object>asList("two", null);
        final Collection<Object> missingCollection = Arrays.<Object>asList("two", "missing");
        emitOutcome("all.collection.present",
                () -> SetUtils.containsAll(source, presentCollection));
        emitOutcome("all.collection.missing",
                () -> SetUtils.containsAll(source, missingCollection));
        emitOutcome("all.collection.empty",
                () -> SetUtils.containsAll(source, Collections.emptyList()));
        emitOutcome("all.collection.duplicate",
                () -> SetUtils.containsAll(source, Arrays.<Object>asList("two", "two")));
        emitOutcome("all.collection.null_target",
                () -> SetUtils.containsAll(null, (Collection<Object>)null));
        emitOutcome("all.collection.null_elements",
                () -> SetUtils.containsAll(source, (Collection<Object>)null));

        final Set<Object> singleton = SetUtils.singletonSet("one");
        final Set<Object> nullSingleton = SetUtils.singletonSet(null);
        emit("singleton.value", render(singleton));
        emit("singleton.null", render(nullSingleton));
        emitOutcome("singleton.unmodifiable", () -> singleton.add("two"));

        final Sets sets = new Sets();
        emitOutcome("facade.convert.value",
                () -> render(sets.toSet(new Object[] {"two", "one", "two", null})));
        emitOutcome("facade.convert.identity", () -> sets.toSet(source) == source);
        emitOutcome("facade.convert.null", () -> render(sets.toSet(null)));
        emitOutcome("facade.size", () -> sets.size(source));
        emit("facade.empty.null", Boolean.toString(sets.isEmpty(null)));
        emitOutcome("facade.contains", () -> sets.contains(source, null));
        emitOutcome("facade.all.array",
                () -> sets.containsAll(source, new Object[] {"two", null}));
        emitOutcome("facade.all.collection",
                () -> sets.containsAll(source, Arrays.<Object>asList("two", null)));
        emitOutcome("facade.all.array.null_target",
                () -> sets.containsAll(null, (Object[])null));
        emitOutcome("facade.all.collection.null_target",
                () -> sets.containsAll(null, (Collection<Object>)null));
    }

    private static String render(final Set<?> values) {
        final StringBuilder builder = new StringBuilder();
        builder.append('[');
        boolean first = true;
        for (final Object value : values) {
            if (!first) {
                builder.append(',');
            }
            builder.append(value == null ? "<null>" : String.valueOf(value));
            first = false;
        }
        return builder.append(']').toString();
    }

    private static void emitOutcome(final String key, final ThrowingSupplier action) {
        try {
            emit(key, String.valueOf(action.get()));
        } catch (final RuntimeException exception) {
            if (exception instanceof ClassCastException) {
                emit(key, exception.getClass().getName());
            } else {
                emit(key,
                        exception.getClass().getName() + ":" + String.valueOf(exception.getMessage()));
            }
        }
    }

    private static void emit(final String key, final Object value) {
        System.out.println(key + "=" + String.valueOf(value));
    }

    private interface ThrowingSupplier {
        Object get();
    }
}
