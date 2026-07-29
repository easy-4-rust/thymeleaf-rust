import java.io.DataOutputStream;
import java.io.FileOutputStream;
import java.io.IOException;
import java.util.ArrayList;
import java.util.List;

/**
 * 生成 TextUtils 在 JDK 21 上调用 Character 大小写 API 所需的 BMP 单码元映射。
 *
 * <p>对应 Java 运行时依赖：
 * {@code Character#toUpperCase(char)} / {@code Character#toLowerCase(char)}。
 */
public final class TextUtilsCaseMapGenerator {

    private static final int MAX_CHAR = Character.MAX_VALUE;

    public static void main(final String[] args) throws IOException {
        if (args.length != 1) {
            throw new IllegalArgumentException("Expected output file path");
        }

        final List<Character> upperSources = new ArrayList<>();
        final List<Character> upperTargets = new ArrayList<>();
        final List<Character> lowerSources = new ArrayList<>();
        final List<Character> lowerTargets = new ArrayList<>();

        for (int value = Character.MIN_VALUE; value <= MAX_CHAR; value++) {
            final char source = (char) value;
            final char upper = Character.toUpperCase(source);
            final char lower = Character.toLowerCase(source);
            if (upper != source) {
                upperSources.add(source);
                upperTargets.add(upper);
            }
            if (lower != source) {
                lowerSources.add(source);
                lowerTargets.add(lower);
            }
        }

        try (DataOutputStream output = new DataOutputStream(new FileOutputStream(args[0]))) {
            output.writeInt(0x5455544D); // "TUTM"
            output.writeShort(1);
            writeMappings(output, upperSources, upperTargets);
            writeMappings(output, lowerSources, lowerTargets);
        }
    }

    private static void writeMappings(
            final DataOutputStream output,
            final List<Character> sources,
            final List<Character> targets) throws IOException {
        output.writeShort(sources.size());
        for (int index = 0; index < sources.size(); index++) {
            output.writeChar(sources.get(index));
            output.writeChar(targets.get(index));
        }
    }

    private TextUtilsCaseMapGenerator() {
    }
}
