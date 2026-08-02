#!/usr/bin/env bash
set -euo pipefail

java_root="${1:?usage: regenerate_cache_entry_validity_golden.sh /absolute/path/to/thymeleaf [output]}"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
project_root="$(cd "${script_dir}/.." && pwd)"
output="${2:-${project_root}/thymeleaf/tests/fixtures/cache_entry_validity_golden.txt}"
expected_sha="10f9dd2eb8cbd98515ce14b149d115e0287d0add"
actual_sha="$(git -C "${java_root}" rev-parse HEAD)"

if [[ "${actual_sha}" != "${expected_sha}" ]]; then
    echo "expected Thymeleaf ${expected_sha}, got ${actual_sha}" >&2
    exit 1
fi

temporary_dir="$(mktemp -d)"
trap 'rm -rf "${temporary_dir}"' EXIT

cache_sources="${java_root}/lib/thymeleaf/src/main/java/org/thymeleaf/cache"

javac -encoding UTF-8 -d "${temporary_dir}" \
    "${cache_sources}/ICacheEntryValidity.java" \
    "${cache_sources}/AlwaysValidCacheEntryValidity.java" \
    "${cache_sources}/NonCacheableCacheEntryValidity.java" \
    "${cache_sources}/TTLCacheEntryValidity.java" \
    "${project_root}/thymeleaf-test/tests/java/CacheEntryValidityGolden.java"

mkdir -p "$(dirname "${output}")"
java -cp "${temporary_dir}" CacheEntryValidityGolden > "${output}"
echo "generated ${output} from ${actual_sha}"
