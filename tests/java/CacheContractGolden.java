import java.util.HashMap;
import java.util.HashSet;
import java.util.Map;
import java.util.Set;
import java.util.TreeSet;

import org.thymeleaf.cache.ICache;
import org.thymeleaf.cache.ICacheEntryValidityChecker;

public final class CacheContractGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    public static void main(final String[] args) {
        emit("baseline", BASELINE);

        final ContractCache cache = new ContractCache();
        final String key = new String("key");
        final String value = new String("value");

        emit("cache.miss", cache.get(key) == null);
        cache.put(key, value);
        emit("cache.hit.identity", cache.get(key) == value);
        emit("cache.keys", new TreeSet<String>(cache.keySet()).toString());

        final RecordingChecker valid = new RecordingChecker(true);
        emit("cache.checked.identity", cache.get(key, valid) == value);
        emit("checker.key", valid.key);
        emit("checker.value", valid.value);
        emit("checker.timestamp", Long.toString(valid.timestamp));

        final RecordingChecker invalid = new RecordingChecker(false);
        emit("cache.invalid.miss", cache.get(key, invalid) == null);
        emit("cache.invalid.removed", cache.get(key) == null);
        emit("cache.invalid.keys", new TreeSet<String>(cache.keySet()).toString());
        emit("cache.missing.checked", cache.get("missing", valid) == null);

        cache.clearKey("missing");
        cache.put("first", new String("one"));
        cache.put("second", new String("two"));
        cache.clearKey("first");
        emit("cache.clear_key.remaining", new TreeSet<String>(cache.keySet()).toString());
        cache.clear();
        emit("cache.clear.empty", cache.keySet().isEmpty());
    }

    private static void emit(final String key, final boolean value) {
        emit(key, Boolean.toString(value));
    }

    private static void emit(final String key, final String value) {
        System.out.println(key + "=" + value);
    }

    private static final class Entry {
        private final String value;
        private final long creationTimestamp;

        private Entry(final String value, final long creationTimestamp) {
            this.value = value;
            this.creationTimestamp = creationTimestamp;
        }
    }

    private static final class ContractCache implements ICache<String,String> {
        private final Map<String,Entry> entries = new HashMap<String,Entry>();

        public void put(final String key, final String value) {
            this.entries.put(key, new Entry(value, 7L));
        }

        public String get(final String key) {
            final Entry entry = this.entries.get(key);
            return entry == null ? null : entry.value;
        }

        public String get(
                final String key,
                final ICacheEntryValidityChecker<? super String,? super String> validityChecker) {
            final Entry entry = this.entries.get(key);
            if (entry == null) {
                return null;
            }
            if (!validityChecker.checkIsValueStillValid(key, entry.value, entry.creationTimestamp)) {
                this.entries.remove(key);
                return null;
            }
            return entry.value;
        }

        public void clear() {
            this.entries.clear();
        }

        public void clearKey(final String key) {
            this.entries.remove(key);
        }

        public Set<String> keySet() {
            return new HashSet<String>(this.entries.keySet());
        }
    }

    private static final class RecordingChecker
            implements ICacheEntryValidityChecker<String,String> {
        private static final long serialVersionUID = 1L;

        private final boolean valid;
        private String key;
        private String value;
        private long timestamp;

        private RecordingChecker(final boolean valid) {
            this.valid = valid;
        }

        public boolean checkIsValueStillValid(
                final String key, final String value, final long entryCreationTimestamp) {
            this.key = key;
            this.value = value;
            this.timestamp = entryCreationTimestamp;
            return this.valid;
        }
    }
}
