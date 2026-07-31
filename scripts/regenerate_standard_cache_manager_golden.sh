#!/usr/bin/env bash
set -euo pipefail

java_root="${1:?usage: regenerate_standard_cache_manager_golden.sh /absolute/path/to/thymeleaf [output]}"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
project_root="$(cd "${script_dir}/.." && pwd)"
output="${2:-${project_root}/tests/fixtures/standard_cache_manager_golden.txt}"
expected_sha="10f9dd2eb8cbd98515ce14b149d115e0287d0add"
actual_sha="$(git -C "${java_root}" rev-parse HEAD)"

if [[ "${actual_sha}" != "${expected_sha}" ]]; then
    echo "expected Thymeleaf ${expected_sha}, got ${actual_sha}" >&2
    exit 1
fi

temporary_dir="$(mktemp -d)"
trap 'rm -rf "${temporary_dir}"' EXIT

classes="${java_root}/lib/thymeleaf/target/classes"
user_home="$(cd && pwd)"
slf4j_api="$(
    find "${user_home}/.m2/repository/org/slf4j/slf4j-api" \
        -name 'slf4j-api-*.jar' ! -name '*sources*' -print |
        sort |
        tail -n 1
)"

if [[ ! -f "${classes}/org/thymeleaf/cache/StandardCacheManager.class" ]]; then
    echo "missing compiled upstream classes under ${classes}" >&2
    exit 1
fi
if [[ -z "${slf4j_api}" || ! -f "${slf4j_api}" ]]; then
    echo "missing SLF4J API jar under the local Maven repository" >&2
    exit 1
fi

javac -encoding UTF-8 -cp "${classes}:${slf4j_api}" -d "${temporary_dir}" \
    "${project_root}/tests/java/StandardCacheManagerGolden.java"

mkdir -p "$(dirname "${output}")"
java -cp "${temporary_dir}:${classes}:${slf4j_api}" StandardCacheManagerGolden > "${output}"
echo "generated ${output} from ${actual_sha}"
