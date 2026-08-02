#!/usr/bin/env bash
set -euo pipefail

java_root="${1:?usage: regenerate_fast_string_writer_golden.sh /absolute/path/to/thymeleaf [output]}"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
project_root="$(cd "${script_dir}/.." && pwd)"
output="${2:-${project_root}/thymeleaf/tests/fixtures/fast_string_writer_golden.txt}"
expected_sha="10f9dd2eb8cbd98515ce14b149d115e0287d0add"
actual_sha="$(git -C "${java_root}" rev-parse HEAD)"

if [[ "${actual_sha}" != "${expected_sha}" ]]; then
    echo "expected Thymeleaf ${expected_sha}, got ${actual_sha}" >&2
    exit 1
fi

temporary_dir="$(mktemp -d)"
trap 'rm -rf "${temporary_dir}"' EXIT

javac -encoding UTF-8 -d "${temporary_dir}" \
    "${java_root}/lib/thymeleaf/src/main/java/org/thymeleaf/util/FastStringWriter.java" \
    "${project_root}/thymeleaf-test/tests/java/FastStringWriterGolden.java"

mkdir -p "$(dirname "${output}")"
java -cp "${temporary_dir}" FastStringWriterGolden > "${output}"
echo "generated ${output} from ${actual_sha}"
