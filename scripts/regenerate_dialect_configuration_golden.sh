#!/usr/bin/env bash
set -euo pipefail

java_root="${1:?usage: regenerate_dialect_configuration_golden.sh /absolute/path/to/thymeleaf [output]}"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
project_root="$(cd "${script_dir}/.." && pwd)"
output="${2:-${project_root}/tests/fixtures/dialect_configuration_golden.txt}"
expected_sha="10f9dd2eb8cbd98515ce14b149d115e0287d0add"
actual_sha="$(git -C "${java_root}" rev-parse HEAD)"

if [[ "${actual_sha}" != "${expected_sha}" ]]; then
    echo "expected Thymeleaf ${expected_sha}, got ${actual_sha}" >&2
    exit 1
fi

temporary_dir="$(mktemp -d)"
trap 'rm -rf "${temporary_dir}"' EXIT

thymeleaf_sources="${java_root}/lib/thymeleaf/src/main/java/org/thymeleaf"

javac -encoding UTF-8 -d "${temporary_dir}" \
    "${project_root}/tests/java/stubs/org/thymeleaf/util/StringUtils.java" \
    "${thymeleaf_sources}/util/Validate.java" \
    "${thymeleaf_sources}/dialect/IDialect.java" \
    "${project_root}/tests/java/stubs/org/thymeleaf/dialect/IProcessorDialect.java" \
    "${thymeleaf_sources}/dialect/AbstractDialect.java" \
    "${thymeleaf_sources}/DialectConfiguration.java" \
    "${project_root}/tests/java/DialectConfigurationGolden.java"

mkdir -p "$(dirname "${output}")"
java -cp "${temporary_dir}" DialectConfigurationGolden > "${output}"
echo "generated ${output} from ${actual_sha}"
