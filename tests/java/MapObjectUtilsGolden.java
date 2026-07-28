import java.util.Arrays;
import java.util.Collection;
import java.util.Collections;
import java.util.LinkedHashMap;
import java.util.Map;

import org.thymeleaf.util.MapUtils;
import org.thymeleaf.util.ObjectUtils;

/**
 * 从固定 Thymeleaf Java 源码导出 MapUtils 与 ObjectUtils Golden。
 */
public final class MapObjectUtilsGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private MapObjectUtilsGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);

        final Map<Object,Object> map = new LinkedHashMap<Object,Object>();
        map.put("one", "value");
        map.put("two", "other");
        map.put(null, null);
        final Map<Object,Object> empty = Collections.emptyMap();

        emitOutcome("map.size.value", () -> MapUtils.size(map));
        emitOutcome("map.size.empty", () -> MapUtils.size(empty));
        emitOutcome("map.size.null", () -> MapUtils.size(null));
        emit("map.empty.value", Boolean.toString(MapUtils.isEmpty(map)));
        emit("map.empty.empty", Boolean.toString(MapUtils.isEmpty(empty)));
        emit("map.empty.null", Boolean.toString(MapUtils.isEmpty(null)));

        emitOutcome("map.key.present", () -> MapUtils.containsKey(map, "one"));
        emitOutcome("map.key.missing", () -> MapUtils.containsKey(map, "missing"));
        emitOutcome("map.key.null", () -> MapUtils.containsKey(map, null));
        emitOutcome("map.key.null_target", () -> MapUtils.containsKey(null, "one"));

        final Object[] presentKeys = new Object[] {"one", null};
        final Object[] missingKeys = new Object[] {"one", "missing"};
        final Object[] duplicateKeys = new Object[] {"one", "one"};
        emitOutcome("map.keys_array.present",
                () -> MapUtils.containsAllKeys(map, presentKeys));
        emitOutcome("map.keys_array.missing",
                () -> MapUtils.containsAllKeys(map, missingKeys));
        emitOutcome("map.keys_array.empty",
                () -> MapUtils.containsAllKeys(map, new Object[0]));
        emitOutcome("map.keys_array.duplicate",
                () -> MapUtils.containsAllKeys(map, duplicateKeys));
        emitOutcome("map.keys_array.null_target",
                () -> MapUtils.containsAllKeys(null, (Object[])null));
        emitOutcome("map.keys_array.null_keys",
                () -> MapUtils.containsAllKeys(map, (Object[])null));

        final Collection<Object> presentKeyCollection = Arrays.<Object>asList("one", null);
        final Collection<Object> missingKeyCollection = Arrays.<Object>asList("one", "missing");
        emitOutcome("map.keys_collection.present",
                () -> MapUtils.containsAllKeys(map, presentKeyCollection));
        emitOutcome("map.keys_collection.missing",
                () -> MapUtils.containsAllKeys(map, missingKeyCollection));
        emitOutcome("map.keys_collection.empty",
                () -> MapUtils.containsAllKeys(map, Collections.emptyList()));
        emitOutcome("map.keys_collection.null_target",
                () -> MapUtils.containsAllKeys(null, (Collection<Object>)null));
        emitOutcome("map.keys_collection.null_keys",
                () -> MapUtils.containsAllKeys(map, (Collection<Object>)null));

        emitOutcome("map.value.present", () -> MapUtils.containsValue(map, "value"));
        emitOutcome("map.value.missing", () -> MapUtils.containsValue(map, "missing"));
        emitOutcome("map.value.null", () -> MapUtils.containsValue(map, null));
        emitOutcome("map.value.null_target", () -> MapUtils.containsValue(null, "value"));

        final Object[] presentValues = new Object[] {"value", null};
        final Object[] missingValues = new Object[] {"value", "missing"};
        final Object[] duplicateValues = new Object[] {"value", "value"};
        emitOutcome("map.values_array.present",
                () -> MapUtils.containsAllValues(map, presentValues));
        emitOutcome("map.values_array.missing",
                () -> MapUtils.containsAllValues(map, missingValues));
        emitOutcome("map.values_array.empty",
                () -> MapUtils.containsAllValues(map, new Object[0]));
        emitOutcome("map.values_array.duplicate",
                () -> MapUtils.containsAllValues(map, duplicateValues));
        emitOutcome("map.values_array.null_target",
                () -> MapUtils.containsAllValues(null, (Object[])null));
        emitOutcome("map.values_array.null_values",
                () -> MapUtils.containsAllValues(map, (Object[])null));

        final Collection<Object> presentValueCollection = Arrays.<Object>asList("value", null);
        final Collection<Object> missingValueCollection = Arrays.<Object>asList("value", "missing");
        emitOutcome("map.values_collection.present",
                () -> MapUtils.containsAllValues(map, presentValueCollection));
        emitOutcome("map.values_collection.missing",
                () -> MapUtils.containsAllValues(map, missingValueCollection));
        emitOutcome("map.values_collection.empty",
                () -> MapUtils.containsAllValues(map, Collections.emptyList()));
        emitOutcome("map.values_collection.null_target",
                () -> MapUtils.containsAllValues(null, (Collection<Object>)null));
        emitOutcome("map.values_collection.null_values",
                () -> MapUtils.containsAllValues(map, (Collection<Object>)null));

        final Object target = new Object();
        final Object defaultValue = new Object();
        emit("object.target",
                Boolean.toString(ObjectUtils.nullSafe(target, defaultValue) == target));
        emit("object.default",
                Boolean.toString(ObjectUtils.nullSafe(null, defaultValue) == defaultValue));
        emit("object.both_null",
                Boolean.toString(ObjectUtils.nullSafe(null, null) == null));
    }

    private static void emitOutcome(final String key, final ThrowingSupplier action) {
        try {
            emit(key, String.valueOf(action.get()));
        } catch (final RuntimeException exception) {
            emit(key, exception.getClass().getName() + ":" + String.valueOf(exception.getMessage()));
        }
    }

    private static void emit(final String key, final String value) {
        System.out.println(key + "=" + value);
    }

    private interface ThrowingSupplier {
        Object get();
    }
}
