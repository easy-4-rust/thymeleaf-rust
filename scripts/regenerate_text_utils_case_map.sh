#!/usr/bin/env bash
set -euo pipefail

project_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
temporary_dir="$(mktemp -d)"
trap 'rm -rf "${temporary_dir}"' EXIT

java_major="$(java -XshowSettings:properties -version 2>&1 | awk -F= '/java.specification.version/ { gsub(/[[:space:]]/, "", $2); print $2; exit }')"
if [[ "${java_major}" != "21" ]]; then
    echo "TextUtils case-map generation requires JDK 21, got ${java_major:-unknown}" >&2
    exit 1
fi

javac \
    -encoding UTF-8 \
    -d "${temporary_dir}" \
    "${project_root}/thymeleaf-test/tests/java/TextUtilsCaseMapGenerator.java"

java \
    -cp "${temporary_dir}" \
    TextUtilsCaseMapGenerator \
    "${project_root}/thymeleaf/src/util/text_utils_case_map.bin"
