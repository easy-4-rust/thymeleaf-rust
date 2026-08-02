#!/usr/bin/env bash
set -euo pipefail

java_root="${1:?usage: regenerate_standard_conversion_service_golden.sh /absolute/path/to/thymeleaf [output]}"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
project_root="$(cd "${script_dir}/.." && pwd)"
output="${2:-${project_root}/thymeleaf/tests/fixtures/standard_conversion_service_golden.txt}"
expected_sha="10f9dd2eb8cbd98515ce14b149d115e0287d0add"
actual_sha="$(git -C "${java_root}" rev-parse HEAD)"

if [[ "${actual_sha}" != "${expected_sha}" ]]; then
    echo "expected Thymeleaf ${expected_sha}, got ${actual_sha}" >&2
    exit 1
fi

temporary_dir="$(mktemp -d)"
trap 'rm -rf "${temporary_dir}"' EXIT

expression_sources="${java_root}/lib/thymeleaf/src/main/java/org/thymeleaf/standard/expression"

javac -encoding UTF-8 -Xlint:unchecked -d "${temporary_dir}" \
    "${project_root}/thymeleaf-test/tests/java/stubs/org/thymeleaf/context/IExpressionContext.java" \
    "${project_root}/thymeleaf-test/tests/java/stubs/org/thymeleaf/util/StringUtils.java" \
    "${java_root}/lib/thymeleaf/src/main/java/org/thymeleaf/util/Validate.java" \
    "${expression_sources}/IStandardConversionService.java" \
    "${expression_sources}/AbstractStandardConversionService.java" \
    "${expression_sources}/StandardConversionService.java" \
    "${expression_sources}/NoOpToken.java" \
    "${project_root}/thymeleaf-test/tests/java/StandardConversionServiceGolden.java"

mkdir -p "$(dirname "${output}")"
java -cp "${temporary_dir}" StandardConversionServiceGolden > "${output}"
echo "generated ${output} from ${actual_sha}"
