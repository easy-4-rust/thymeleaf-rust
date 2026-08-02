import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collection;
import java.util.Comparator;
import java.util.Iterator;
import java.util.LinkedList;
import java.util.List;

import org.thymeleaf.expression.Lists;
import org.thymeleaf.util.ListUtils;

/**
 * 从固定 Thymeleaf Java 源码导出 ListUtils 与 Lists Golden。
 */
public final class ListUtilsGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private ListUtilsGolden() {
    }

    @SuppressWarnings({"rawtypes", "unchecked"})
    public static void main(final String[] args) {
        emit("baseline", BASELINE);

        final List<Object> source =
                new LinkedList<Object>(Arrays.<Object>asList("two", "one", "two", null));
        final List<Object> empty = new ArrayList<Object>();

        emitOutcome("convert.list.value", () -> render(ListUtils.toList(source)));
        emitOutcome("convert.list.identity", () -> ListUtils.toList(source) == source);
        emitOutcome("convert.list.type", () -> ListUtils.toList(source).getClass().getName());
        emitOutcome("convert.array.value",
                () -> render(ListUtils.toList(new Object[] {"two", "one", "two", null})));
        emitOutcome("convert.array.type",
                () -> ListUtils.toList(new Object[] {"one"}).getClass().getName());
        emitOutcome("convert.array.empty", () -> render(ListUtils.toList(new Object[0])));
        emitOutcome("convert.iterable.value",
                () -> render(ListUtils.toList(new IterableOnly<Object>("b", "a", "b", null))));
        emitOutcome("convert.iterable.type",
                () -> ListUtils.toList(new IterableOnly<Object>()).getClass().getName());
        emitOutcome("convert.iterable.empty",
                () -> render(ListUtils.toList(new IterableOnly<Object>())));
        emitOutcome("convert.null", () -> render(ListUtils.toList(null)));
        emitOutcome("convert.unsupported", () -> render(ListUtils.toList(Integer.valueOf(1))));
        final Iterator<String> iterator = Arrays.asList("a", "b").iterator();
        emitOutcome("convert.iterator_not_iterable", () -> render(ListUtils.toList(iterator)));
        emitOutcome("convert.primitive_array", () -> render(ListUtils.toList(new int[] {1, 2})));

        emitOutcome("size.value", () -> ListUtils.size(source));
        emitOutcome("size.empty", () -> ListUtils.size(empty));
        emitOutcome("size.null", () -> ListUtils.size(null));
        emit("empty.value", Boolean.toString(ListUtils.isEmpty(source)));
        emit("empty.empty", Boolean.toString(ListUtils.isEmpty(empty)));
        emit("empty.null", Boolean.toString(ListUtils.isEmpty(null)));
        emitOutcome("contains.present", () -> ListUtils.contains(source, "two"));
        emitOutcome("contains.missing", () -> ListUtils.contains(source, "missing"));
        emitOutcome("contains.null", () -> ListUtils.contains(source, null));
        emitOutcome("contains.null_target", () -> ListUtils.contains(null, "two"));

        final Object[] present = new Object[] {"two", null};
        final Object[] missing = new Object[] {"two", "missing"};
        final Object[] duplicate = new Object[] {"two", "two"};
        emitOutcome("all.array.present", () -> ListUtils.containsAll(source, present));
        emitOutcome("all.array.missing", () -> ListUtils.containsAll(source, missing));
        emitOutcome("all.array.empty", () -> ListUtils.containsAll(source, new Object[0]));
        emitOutcome("all.array.duplicate", () -> ListUtils.containsAll(source, duplicate));
        emitOutcome("all.array.null_target",
                () -> ListUtils.containsAll(null, (Object[]) null));
        emitOutcome("all.array.null_elements",
                () -> ListUtils.containsAll(source, (Object[]) null));

        final Collection<Object> presentCollection = Arrays.<Object>asList("two", null);
        final Collection<Object> missingCollection = Arrays.<Object>asList("two", "missing");
        emitOutcome("all.collection.present",
                () -> ListUtils.containsAll(source, presentCollection));
        emitOutcome("all.collection.missing",
                () -> ListUtils.containsAll(source, missingCollection));
        emitOutcome("all.collection.empty",
                () -> ListUtils.containsAll(source, new ArrayList<Object>()));
        emitOutcome("all.collection.duplicate",
                () -> ListUtils.containsAll(source, Arrays.<Object>asList("two", "two")));
        emitOutcome("all.collection.null_target",
                () -> ListUtils.containsAll(null, (Collection<Object>) null));
        emitOutcome("all.collection.null_elements",
                () -> ListUtils.containsAll(source, (Collection<Object>) null));

        final List<String> linked = new LinkedList<String>(Arrays.asList("c", "a", "b"));
        emitOutcome("sort.linked.value", () -> render(ListUtils.sort(linked)));
        emitOutcome("sort.linked.type", () -> ListUtils.sort(linked).getClass().getName());
        emitOutcome("sort.linked.original", () -> render(linked));
        emitOutcome("sort.linked.identity", () -> ListUtils.sort(linked) == linked);
        final List<String> fixed = Arrays.asList("c", "a", "b");
        emitOutcome("sort.fixed.value", () -> render(ListUtils.sort(fixed)));
        emitOutcome("sort.fixed.type", () -> ListUtils.sort(fixed).getClass().getName());
        final PublicList<String> publicList = PublicList.of("c", "a", "b");
        emitOutcome("sort.public.value", () -> render(ListUtils.sort(publicList)));
        emitOutcome("sort.public.type", () -> ListUtils.sort(publicList).getClass().getName());
        final PrivateList<String> privateList = PrivateList.of("c", "a", "b");
        emitOutcome("sort.private.value", () -> render(ListUtils.sort(privateList)));
        emitOutcome("sort.private.type", () -> ListUtils.sort(privateList).getClass().getName());
        final AddFailingList<String> addFailingList = AddFailingList.of("c", "a", "b");
        emitOutcome("sort.add_failure", () -> render(ListUtils.sort(addFailingList)));
        emitOutcome("sort.null_list", () -> render(ListUtils.sort((List<String>) null)));
        emitOutcome("sort.null_element",
                () -> render(ListUtils.sort(Arrays.asList("a", null))));
        emitOutcome("sort.heterogeneous",
                () -> render(ListUtils.sort((List) Arrays.<Object>asList("a", Integer.valueOf(1)))));
        emitOutcome("sort.utf16",
                () -> render(ListUtils.sort(Arrays.asList("\uE000", "\uD83D\uDE00"))));
        emitOutcome("sort.double",
                () -> render(ListUtils.sort(Arrays.asList(
                        Double.NaN, 0.0d, -0.0d, Double.POSITIVE_INFINITY, -1.0d))));

        final Comparator<String> descending = (left, right) -> right.compareTo(left);
        emitOutcome("sort.comparator.descending",
                () -> render(ListUtils.sort(linked, descending)));
        emitOutcome("sort.comparator.null",
                () -> render(ListUtils.sort(linked, null)));
        final List<String> stable = Arrays.asList("b1", "a", "b2");
        emitOutcome("sort.comparator.stable",
                () -> render(ListUtils.sort(stable,
                        Comparator.comparingInt(String::length))));
        emitOutcome("sort.comparator.failure",
                () -> render(ListUtils.sort(linked, (left, right) -> {
                    throw new IllegalStateException("compare failed");
                })));

        final Lists lists = new Lists();
        emitOutcome("facade.convert.value",
                () -> render(lists.toList(new Object[] {"two", "one", "two", null})));
        emitOutcome("facade.convert.identity", () -> lists.toList(source) == source);
        emitOutcome("facade.convert.null", () -> render(lists.toList(null)));
        emitOutcome("facade.size", () -> lists.size(source));
        emit("facade.empty.null", Boolean.toString(lists.isEmpty(null)));
        emitOutcome("facade.contains", () -> lists.contains(source, null));
        emitOutcome("facade.all.array",
                () -> lists.containsAll(source, new Object[] {"two", null}));
        emitOutcome("facade.all.collection",
                () -> lists.containsAll(source, Arrays.<Object>asList("two", null)));
        emitOutcome("facade.sort", () -> render(lists.sort(linked)));
        emitOutcome("facade.sort.comparator",
                () -> render(lists.sort(linked, descending)));
    }

    private static String render(final List<?> values) {
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
            if (exception instanceof ClassCastException
                    || exception instanceof NullPointerException) {
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

    private static final class IterableOnly<E> implements Iterable<E> {
        private final List<E> elements;

        @SafeVarargs
        private IterableOnly(final E... elements) {
            this.elements = Arrays.asList(elements);
        }

        @Override
        public Iterator<E> iterator() {
            return this.elements.iterator();
        }
    }

    public static final class PublicList<E> extends ArrayList<E> {
        public PublicList() {
            super();
        }

        @SafeVarargs
        public static <E> PublicList<E> of(final E... elements) {
            final PublicList<E> result = new PublicList<E>();
            result.addAll(Arrays.asList(elements));
            return result;
        }
    }

    private static final class PrivateList<E> extends ArrayList<E> {
        private PrivateList() {
            super();
        }

        @SafeVarargs
        private static <E> PrivateList<E> of(final E... elements) {
            final PrivateList<E> result = new PrivateList<E>();
            result.addAll(Arrays.asList(elements));
            return result;
        }
    }

    public static final class AddFailingList<E> extends ArrayList<E> {
        private final boolean failAdds;

        public AddFailingList() {
            this(true);
        }

        private AddFailingList(final boolean failAdds) {
            super();
            this.failAdds = failAdds;
        }

        @SafeVarargs
        public static <E> AddFailingList<E> of(final E... elements) {
            final AddFailingList<E> result = new AddFailingList<E>(false);
            result.addAll(Arrays.asList(elements));
            return result;
        }

        @Override
        public boolean add(final E element) {
            if (this.failAdds) {
                throw new UnsupportedOperationException("add failed");
            }
            return super.add(element);
        }
    }
}
