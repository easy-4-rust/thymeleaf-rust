#!/usr/bin/env bash
set -euo pipefail
java_root="${1:?usage: regenerate_element_tag_structure_handler_golden.sh /absolute/path/to/thymeleaf [output]}"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
project_root="$(cd "${script_dir}/.." && pwd)"
output="${2:-${project_root}/tests/fixtures/element_tag_structure_handler_golden.txt}"
expected_sha="10f9dd2eb8cbd98515ce14b149d115e0287d0add"
actual_sha="$(git -C "${java_root}" rev-parse HEAD)"
[[ "${actual_sha}" == "${expected_sha}" ]] || { echo "expected Thymeleaf ${expected_sha}, got ${actual_sha}" >&2; exit 1; }
temporary_dir="$(mktemp -d)"
trap 'rm -rf "${temporary_dir}"' EXIT
mvn -q -f "${java_root}/pom.xml" -pl lib/thymeleaf -am -DskipTests test-compile
mvn -q -f "${java_root}/lib/thymeleaf/pom.xml" dependency:build-classpath "-Dmdep.outputFile=${temporary_dir}/classpath"
dependency_classpath="$(<"${temporary_dir}/classpath")"
# Golden 使用上游的测试配置构造器建立真实引擎配置，因此同时需要测试类路径。
java_classpath="${java_root}/lib/thymeleaf/target/classes:${java_root}/lib/thymeleaf/target/test-classes:${dependency_classpath}"
# 上游的配置构造器在 tests/thymeleaf-tests-core 测试源码中，不随 lib 模块产物发布；
# 与 Golden 一同临时编译，避免依赖 Maven reactor 的历史 test-classes 状态。
javac -encoding UTF-8 -cp "${java_classpath}" -d "${temporary_dir}" \
  "${java_root}/tests/thymeleaf-tests-core/src/test/java/org/thymeleaf/context/TestTemplateEngineConfigurationBuilder.java" \
  "${project_root}/tests/java/org/thymeleaf/engine/ElementTagStructureHandlerGolden.java"
mkdir -p "$(dirname "${output}")"
java -cp "${temporary_dir}:${java_classpath}" org.thymeleaf.engine.ElementTagStructureHandlerGolden > "${output}"
echo "generated ${output} from ${actual_sha}"
