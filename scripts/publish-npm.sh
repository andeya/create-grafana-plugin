#!/usr/bin/env bash
# Build Rust binaries for all supported targets, copy them into npm platform packages,
# then publish scoped optional packages followed by the main `create-grafana-plugin` package.
#
# Usage (from repository root):
#   ./scripts/publish-npm.sh
#
# Environment:
#   SKIP_BUILD=1       — skip `cargo`/`cross` builds; use binaries already in npm/*/bin (must exist).
#   USE_CROSS=1        — use `cross build` instead of `cargo build` (requires https://github.com/cross-rs/cross).
#   BUILD_TARGETS=...  — space-separated Rust target triples to build (default: all five).
#   DRY_RUN=1          — pass `--dry-run` to every `npm publish`.
#   NPM_TAG=next       — optional `npm publish --tag` (omit or `latest` = default tag).
#   CONFIRM=0          — skip the interactive confirmation prompt (e.g. CI pipelines).
#
# Prerequisites:
#   - Rust toolchain with targets installed, e.g. `rustup target add x86_64-unknown-linux-gnu`
#   - For cross-compilation from a single host, install `cross` and set USE_CROSS=1, or use CI matrix builders.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

NPM_MAIN="${ROOT_DIR}/npm/create-grafana-plugin"
CRATE_BIN_NAME="create-grafana-plugin"

DEFAULT_BUILD_TARGETS=(
  "aarch64-apple-darwin"
  "x86_64-apple-darwin"
  "x86_64-unknown-linux-gnu"
  "aarch64-unknown-linux-gnu"
  "x86_64-pc-windows-msvc"
)

# Maps a Rust target triple to the npm platform package suffix (bash 3.2–compatible; no `declare -A`).
npm_suffix_for_target() {
  case "$1" in
    aarch64-apple-darwin) printf '%s' 'darwin-arm64' ;;
    x86_64-apple-darwin) printf '%s' 'darwin-x64' ;;
    x86_64-unknown-linux-gnu) printf '%s' 'linux-x64' ;;
    aarch64-unknown-linux-gnu) printf '%s' 'linux-arm64' ;;
    x86_64-pc-windows-msvc) printf '%s' 'win32-x64' ;;
    *) return 1 ;;
  esac
}

die() {
  printf '%s\n' "publish-npm.sh: $*" >&2
  exit 1
}

read_package_version() {
  local path="$1"
  node -e "process.stdout.write(require(process.argv[1]).version)" "${path}/package.json"
}

assert_same_version_everywhere() {
  local main_ver
  main_ver="$(read_package_version "${NPM_MAIN}")"
  local suffix
  for suffix in darwin-arm64 darwin-x64 linux-x64 linux-arm64 win32-x64; do
    local dir="${ROOT_DIR}/npm/${suffix}"
    local v
    v="$(read_package_version "${dir}")"
    if [[ "${v}" != "${main_ver}" ]]; then
      die "version mismatch: ${NPM_MAIN}/package.json is ${main_ver} but ${dir}/package.json is ${v}"
    fi
  done
  if [[ -f "${ROOT_DIR}/package.json" ]]; then
    local root_ver
    root_ver="$(read_package_version "${ROOT_DIR}")"
    if [[ "${root_ver}" != "${main_ver}" ]]; then
      die "version mismatch: root package.json (${root_ver}) vs npm/create-grafana-plugin (${main_ver})"
    fi
  fi
  printf '%s\n' "Version check OK: ${main_ver}"
}

run_build_command() {
  local target="$1"
  if [[ -n "${USE_CROSS:-}" ]]; then
    if ! command -v cross >/dev/null 2>&1; then
      die "USE_CROSS=1 but \`cross\` is not on PATH (install: https://github.com/cross-rs/cross)"
    fi
    cross build --release -p "${CRATE_BIN_NAME}" --target "${target}"
  else
    cargo build --release -p "${CRATE_BIN_NAME}" --target "${target}"
  fi
}

artifact_path_for_target() {
  local target="$1"
  local base="${ROOT_DIR}/target/${target}/release/${CRATE_BIN_NAME}"
  if [[ "${target}" == *"-pc-windows-msvc" ]]; then
    printf '%s' "${base}.exe"
  else
    printf '%s' "${base}"
  fi
}

copy_artifact_to_npm() {
  local target="$1"
  local suffix
  if ! suffix="$(npm_suffix_for_target "${target}")"; then
    die "unknown target mapping for ${target}"
  fi
  local dest_dir="${ROOT_DIR}/npm/${suffix}/bin"
  mkdir -p "${dest_dir}"

  local src
  src="$(artifact_path_for_target "${target}")"
  if [[ ! -f "${src}" ]]; then
    die "missing build artifact: ${src}"
  fi

  if [[ "${target}" == *"-pc-windows-msvc" ]]; then
    rm -f "${dest_dir}/create-grafana-plugin.cmd"
    cp -f "${src}" "${dest_dir}/create-grafana-plugin.exe"
    local pkg_json="${ROOT_DIR}/npm/${suffix}/package.json"
    node -e '
      const fs = require("fs");
      const pkgPath = process.argv[1];
      const j = JSON.parse(fs.readFileSync(pkgPath, "utf8"));
      if (!j.bin || typeof j.bin !== "object") j.bin = {};
      j.bin["create-grafana-plugin"] = "bin/create-grafana-plugin.exe";
      fs.writeFileSync(pkgPath, JSON.stringify(j, null, 2) + "\n");
    ' "${pkg_json}"
  else
    cp -f "${src}" "${dest_dir}/${CRATE_BIN_NAME}"
    chmod +x "${dest_dir}/${CRATE_BIN_NAME}"
  fi
}

build_all_and_copy() {
  local targets=()
  if [[ -n "${BUILD_TARGETS:-}" ]]; then
    read -r -a targets <<< "${BUILD_TARGETS}"
  else
    targets=("${DEFAULT_BUILD_TARGETS[@]}")
  fi

  local t
  for t in "${targets[@]}"; do
    printf '\n%s\n' "==> Building ${t}"
    run_build_command "${t}"
    copy_artifact_to_npm "${t}"
  done
}

npm_publish_dir() {
  local dir="$1"
  local args=(publish --access public)
  if [[ -n "${NPM_TAG:-}" ]]; then
    args+=(--tag "${NPM_TAG}")
  fi
  if [[ -n "${DRY_RUN:-}" ]]; then
    args+=(--dry-run)
  fi
  (cd "${dir}" && npm "${args[@]}")
}

confirm_publish() {
  local ver="$1"
  local reply
  read -r -p "Publish version ${ver} to the npm registry? [y/N] " reply
  case "${reply}" in
    y|Y|yes|YES) return 0 ;;
    *) return 1 ;;
  esac
}

main() {
  assert_same_version_everywhere
  local ver
  ver="$(read_package_version "${NPM_MAIN}")"

  if [[ -z "${SKIP_BUILD:-}" ]]; then
    build_all_and_copy
  else
    printf '%s\n' "SKIP_BUILD=1: skipping cargo builds and binary copy."
  fi

  if [[ "${CONFIRM:-1}" != "0" ]]; then
    if ! confirm_publish "${ver}"; then
      die "aborted by user."
    fi
  fi

  local suffix
  for suffix in darwin-arm64 darwin-x64 linux-x64 linux-arm64 win32-x64; do
    printf '\n%s\n' "==> Publishing @create-grafana-plugin/${suffix}"
    npm_publish_dir "${ROOT_DIR}/npm/${suffix}"
  done

  printf '\n%s\n' "==> Publishing create-grafana-plugin (main package)"
  npm_publish_dir "${NPM_MAIN}"
  printf '%s\n' "Done."
}

main "$@"
