package org.thymeleaf.engine;

/** 固定 IterationStatusVar 的状态机、溢出和 JavaBean 可见语义。 */
public final class IterationStatusVarGolden {
    private IterationStatusVarGolden() { }

    public static void main(final String[] args) {
        final IterationStatusVar unknown = new IterationStatusVar();
        System.out.println("unknown=" + values(unknown) + ",last=" + last(unknown)
                + ",text=" + unknown);

        final IterationStatusVar known = new IterationStatusVar();
        known.size = 3;
        known.current = new StringBuilder("value");
        System.out.println("known0=" + values(known) + ",last=" + known.isLast()
                + ",text=" + known);
        known.index++;
        System.out.println("known1=" + values(known) + ",last=" + known.isLast()
                + ",text=" + known);
        known.index++;
        known.current = null;
        System.out.println("known2=" + values(known) + ",last=" + known.isLast()
                + ",text=" + known);

        final IterationStatusVar overflow = new IterationStatusVar();
        overflow.index = Integer.MAX_VALUE;
        overflow.size = Integer.MIN_VALUE;
        System.out.println("overflow=" + values(overflow) + ",last=" + overflow.isLast()
                + ",text=" + overflow);
    }

    private static String values(final IterationStatusVar status) {
        return status.getIndex() + "," + status.getCount() + "," + status.hasSize() + ","
                + status.getSize() + "," + status.getCurrent() + "," + status.isEven() + ","
                + status.isOdd() + "," + status.isFirst();
    }

    private static String last(final IterationStatusVar status) {
        try {
            return Boolean.toString(status.isLast());
        } catch (final RuntimeException exception) {
            return exception.getClass().getName() + ":" + exception.getMessage();
        }
    }
}
