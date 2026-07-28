import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collection;
import java.util.LinkedHashMap;
import java.util.LinkedHashSet;
import java.util.Map;
import java.util.Set;

import org.thymeleaf.expression.Maps;
import org.thymeleaf.expression.Objects;

/**
 * 从固定 Thymeleaf Java 源码导出 Maps 与 Objects 表达式对象 Golden。
 */
public final class MapObjectExpressionGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private MapObjectExpressionGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);

        final Maps maps = new Maps();
        final Map<Object,Object> map = new LinkedHashMap<Object,Object>();
        map.put("one", "value");
        map.put("two", "other");
        map.put(null, null);
        final Object[] keys = new Object[] {"one", null};
        final Collection<Object> keyCollection = Arrays.<Object>asList("two", null);
        final Object[] values = new Object[] {"value", null};
        final Collection<Object> valueCollection = Arrays.<Object>asList("other", null);

        emitOutcome("maps.size", () -> maps.size(map));
        emit("maps.empty.value", Boolean.toString(maps.isEmpty(map)));
        emit("maps.empty.null", Boolean.toString(maps.isEmpty(null)));
        emitOutcome("maps.key", () -> maps.containsKey(map, null));
        emitOutcome("maps.keys.array", () -> maps.containsAllKeys(map, keys));
        emitOutcome("maps.keys.collection", () -> maps.containsAllKeys(map, keyCollection));
        emitOutcome("maps.value", () -> maps.containsValue(map, null));
        emitOutcome("maps.values.array", () -> maps.containsAllValues(map, values));
        emitOutcome("maps.values.collection",
                () -> maps.containsAllValues(map, valueCollection));
        emitOutcome("maps.validation",
                () -> maps.containsAllKeys(null, (Object[])null));

        final Objects objects = new Objects();
        final Object target = new Object();
        final Object defaultValue = new Object();
        emit("objects.scalar.target",
                Boolean.toString(objects.nullSafe(target, defaultValue) == target));
        emit("objects.scalar.default",
                Boolean.toString(objects.nullSafe(null, defaultValue) == defaultValue));
        emit("objects.scalar.null",
                Boolean.toString(objects.nullSafe(null, null) == null));

        final String[] sourceArray = new String[] {"one", null, "one"};
        final String[] resultArray = objects.arrayNullSafe(sourceArray, "default");
        emit("objects.array.values", Arrays.toString(resultArray));
        emit("objects.array.source", Arrays.toString(sourceArray));
        emit("objects.array.distinct", Boolean.toString(resultArray != sourceArray));
        emit("objects.array.class", resultArray.getClass().getName());
        resultArray[0] = "changed";
        emit("objects.array.mutable", Arrays.toString(resultArray));
        emitOutcome("objects.array.null_default",
                () -> Arrays.toString(objects.arrayNullSafe(
                        new String[] {null}, (String)null)));
        emitExceptionClass("objects.array.incompatible_with_null",
                () -> objects.arrayNullSafe(new String[] {null}, Integer.valueOf(1)));
        emitOutcome("objects.array.incompatible_without_null",
                () -> Arrays.toString(objects.arrayNullSafe(
                        new String[] {"one"}, Integer.valueOf(1))));
        emitOutcome("objects.array.null_target",
                () -> objects.arrayNullSafe((String[])null, "default"));

        final ArrayList<String> sourceList =
                new ArrayList<String>(Arrays.asList("one", null, "one"));
        final java.util.List<String> resultList =
                objects.listNullSafe(sourceList, "default");
        emit("objects.list.values", resultList.toString());
        emit("objects.list.source", sourceList.toString());
        emit("objects.list.distinct", Boolean.toString(resultList != sourceList));
        emit("objects.list.class", resultList.getClass().getName());
        resultList.add("tail");
        emit("objects.list.mutable", resultList.toString());
        emitOutcome("objects.list.null_target",
                () -> objects.listNullSafe(null, "default"));

        final Set<String> sourceSet =
                new LinkedHashSet<String>(Arrays.asList("default", null, "other"));
        final Set<String> resultSet = objects.setNullSafe(sourceSet, "default");
        emit("objects.set.values", resultSet.toString());
        emit("objects.set.source", sourceSet.toString());
        emit("objects.set.distinct", Boolean.toString(resultSet != sourceSet));
        emit("objects.set.class", resultSet.getClass().getName());
        resultSet.add("tail");
        emit("objects.set.mutable", resultSet.toString());
        emitOutcome("objects.set.null_target",
                () -> objects.setNullSafe(null, "default"));
    }

    private static void emitOutcome(final String key, final ThrowingSupplier action) {
        try {
            emit(key, String.valueOf(action.get()));
        } catch (final RuntimeException exception) {
            emit(key, exception.getClass().getName() + ":" +
                    String.valueOf(exception.getMessage()));
        }
    }

    private static void emitExceptionClass(final String key, final ThrowingSupplier action) {
        try {
            action.get();
            emit(key, "NO_EXCEPTION");
        } catch (final RuntimeException exception) {
            emit(key, exception.getClass().getName());
        }
    }

    private static void emit(final String key, final String value) {
        System.out.println(key + "=" + value);
    }

    private interface ThrowingSupplier {
        Object get();
    }
}
