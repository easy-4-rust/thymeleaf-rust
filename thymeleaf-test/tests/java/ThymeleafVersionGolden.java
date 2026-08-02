import org.thymeleaf.Thymeleaf;

/**
 * 从固定 Thymeleaf Java 源码和正式制品元数据生成版本 Golden。
 */
public final class ThymeleafVersionGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private ThymeleafVersionGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);
        emit("version", Thymeleaf.getVersion());
        emit("build_timestamp", Thymeleaf.getBuildTimestamp());
        emit("major", Thymeleaf.getVersionMajor());
        emit("minor", Thymeleaf.getVersionMinor());
        emit("patch", Thymeleaf.getVersionPatch());
        emit("qualifier", Thymeleaf.getVersionQualifier());
        emit("stable", Thymeleaf.isVersionStableRelease());
    }

    private static void emit(final String key, final Object value) {
        System.out.println(key + "=" + String.valueOf(value));
    }
}
