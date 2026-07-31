#!/usr/bin/env bash
set -euo pipefail

java_root="${1:?usage: regenerate_standard_parsed_template_entry_validator_golden.sh /absolute/path/to/thymeleaf [output]}"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
project_root="$(cd "${script_dir}/.." && pwd)"
output="${2:-${project_root}/tests/fixtures/standard_parsed_template_entry_validator_golden.txt}"
expected_sha="10f9dd2eb8cbd98515ce14b149d115e0287d0add"
actual_sha="$(git -C "${java_root}" rev-parse HEAD)"

if [[ "${actual_sha}" != "${expected_sha}" ]]; then
    echo "expected Thymeleaf ${expected_sha}, got ${actual_sha}" >&2
    exit 1
fi

temporary_dir="$(mktemp -d)"
trap 'rm -rf "${temporary_dir}"' EXIT

classes="${java_root}/lib/thymeleaf/target/classes"
source="${project_root}/tests/java/org/thymeleaf/engine/StandardParsedTemplateEntryValidatorGolden.java"

if [[ ! -f "${classes}/org/thymeleaf/cache/StandardParsedTemplateEntryValidator.class" ]]; then
    echo "missing compiled upstream classes under ${classes}" >&2
    exit 1
fi

javac -encoding UTF-8 -cp "${classes}" -d "${temporary_dir}" "${source}"
mkdir -p "$(dirname "${output}")"
java -cp "${temporary_dir}:${classes}" \
    org.thymeleaf.engine.StandardParsedTemplateEntryValidatorGolden > "${output}"
echo "generated ${output} from ${actual_sha}"
