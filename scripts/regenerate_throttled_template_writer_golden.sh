#!/usr/bin/env bash
set -euo pipefail
java_root="${1:?usage: regenerate_throttled_template_writer_golden.sh /absolute/path/to/thymeleaf [output]}"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
project_root="$(cd "${script_dir}/.." && pwd)"
output="${2:-${project_root}/tests/fixtures/throttled_template_writer_golden.txt}"
expected_sha="10f9dd2eb8cbd98515ce14b149d115e0287d0add"
actual_sha="$(git -C "${java_root}" rev-parse HEAD)"
[[ "${actual_sha}" == "${expected_sha}" ]] || { echo "expected Thymeleaf ${expected_sha}, got ${actual_sha}" >&2; exit 1; }
temporary_dir="$(mktemp -d)"
trap 'rm -rf "${temporary_dir}"' EXIT
mvn -q -f "${java_root}/pom.xml" -pl lib/thymeleaf -am -DskipTests compile
mvn -q -f "${java_root}/lib/thymeleaf/pom.xml" dependency:build-classpath "-Dmdep.outputFile=${temporary_dir}/classpath"
dependency_classpath="$(<"${temporary_dir}/classpath")"
java_classpath="${java_root}/lib/thymeleaf/target/classes:${dependency_classpath}"
javac -encoding UTF-8 -cp "${java_classpath}" -d "${temporary_dir}" \
  "${project_root}/tests/java/org/thymeleaf/engine/ThrottledTemplateWriterGolden.java"
mkdir -p "$(dirname "${output}")"
java -cp "${temporary_dir}:${java_classpath}" org.thymeleaf.engine.ThrottledTemplateWriterGolden > "${output}"
echo "generated ${output} from ${actual_sha}"
