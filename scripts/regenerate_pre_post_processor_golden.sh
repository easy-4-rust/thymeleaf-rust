#!/usr/bin/env bash
set -euo pipefail

java_root="${1:?usage: regenerate_pre_post_processor_golden.sh /absolute/path/to/thymeleaf [output]}"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
project_root="$(cd "${script_dir}/.." && pwd)"
output="${2:-${project_root}/thymeleaf/tests/fixtures/pre_post_processor_golden.txt}"
expected_sha="10f9dd2eb8cbd98515ce14b149d115e0287d0add"
actual_sha="$(git -C "${java_root}" rev-parse HEAD)"

if [[ "${actual_sha}" != "${expected_sha}" ]]; then
    echo "expected Thymeleaf ${expected_sha}, got ${actual_sha}" >&2
    exit 1
fi

temporary_dir="$(mktemp -d)"
trap 'rm -rf "${temporary_dir}"' EXIT

mvn -q -f "${java_root}/pom.xml" -pl lib/thymeleaf dependency:build-classpath \
    -Dmdep.outputFile="${temporary_dir}/dependencies.txt"
dependencies="$(<"${temporary_dir}/dependencies.txt")"
classes="${java_root}/lib/thymeleaf/target/classes"
classpath="${classes}:${dependencies}"

javac -encoding UTF-8 -cp "${classpath}" -d "${temporary_dir}" \
    "${project_root}/thymeleaf-test/tests/java/PrePostProcessorGolden.java"

mkdir -p "$(dirname "${output}")"
java -cp "${temporary_dir}:${classpath}" PrePostProcessorGolden > "${output}"
echo "generated ${output} from ${actual_sha}"
