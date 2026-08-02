#!/usr/bin/env bash
set -euo pipefail

java_root="${1:?usage: regenerate_text_parser_golden.sh /absolute/path/to/thymeleaf [output]}"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
project_root="$(cd "${script_dir}/.." && pwd)"
output="${2:-${project_root}/thymeleaf/tests/fixtures/text_parser_golden.txt}"
expected_sha="10f9dd2eb8cbd98515ce14b149d115e0287d0add"
actual_sha="$(git -C "${java_root}" rev-parse HEAD)"

if [[ "${actual_sha}" != "${expected_sha}" ]]; then
    echo "expected Thymeleaf ${expected_sha}, got ${actual_sha}" >&2
    exit 1
fi

temporary_dir="$(mktemp -d)"
trap 'rm -rf "${temporary_dir}"' EXIT

javac -encoding UTF-8 -d "${temporary_dir}" \
    "${project_root}/thymeleaf-test/tests/java/stubs/org/slf4j/Logger.java" \
    "${project_root}/thymeleaf-test/tests/java/stubs/org/slf4j/LoggerFactory.java" \
    "${project_root}/thymeleaf-test/tests/java/stubs/org/thymeleaf/TemplateEngine.java" \
    "${java_root}/lib/thymeleaf/src/main/java/org/thymeleaf/templatemode/TemplateMode.java" \
    "${java_root}/lib/thymeleaf/src/main/java/org/thymeleaf/util/TextUtils.java" \
    "${java_root}/lib/thymeleaf/src/main/java/org/thymeleaf/templateparser/text/ITextHandler.java" \
    "${java_root}/lib/thymeleaf/src/main/java/org/thymeleaf/templateparser/text/TextParseException.java" \
    "${java_root}/lib/thymeleaf/src/main/java/org/thymeleaf/templateparser/text/TextParseStatus.java" \
    "${java_root}/lib/thymeleaf/src/main/java/org/thymeleaf/templateparser/text/AbstractTextHandler.java" \
    "${java_root}/lib/thymeleaf/src/main/java/org/thymeleaf/templateparser/text/AbstractChainedTextHandler.java" \
    "${java_root}/lib/thymeleaf/src/main/java/org/thymeleaf/templateparser/text/ParsingLocatorUtil.java" \
    "${java_root}/lib/thymeleaf/src/main/java/org/thymeleaf/templateparser/text/TextParsingCommentUtil.java" \
    "${java_root}/lib/thymeleaf/src/main/java/org/thymeleaf/templateparser/text/TextParsingLiteralUtil.java" \
    "${java_root}/lib/thymeleaf/src/main/java/org/thymeleaf/templateparser/text/TextParsingUtil.java" \
    "${java_root}/lib/thymeleaf/src/main/java/org/thymeleaf/templateparser/text/TextParsingAttributeSequenceUtil.java" \
    "${java_root}/lib/thymeleaf/src/main/java/org/thymeleaf/templateparser/text/TextParsingElementUtil.java" \
    "${java_root}/lib/thymeleaf/src/main/java/org/thymeleaf/templateparser/text/EventProcessorTextHandler.java" \
    "${java_root}/lib/thymeleaf/src/main/java/org/thymeleaf/templateparser/text/CommentProcessorTextHandler.java" \
    "${java_root}/lib/thymeleaf/src/main/java/org/thymeleaf/templateparser/text/TextParser.java" \
    "${project_root}/thymeleaf-test/tests/java/TextParserGolden.java"

mkdir -p "$(dirname "${output}")"
java -XX:-OmitStackTraceInFastThrow \
    -cp "${temporary_dir}" \
    org.thymeleaf.templateparser.text.TextParserGolden > "${output}"

echo "generated ${output} from ${actual_sha}"
