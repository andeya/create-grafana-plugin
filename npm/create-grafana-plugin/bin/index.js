#!/usr/bin/env node
/**
 * Resolves the platform-specific optional dependency and runs the native binary.
 */
"use strict";

const { spawnSync } = require("node:child_process");
const fs = require("node:fs");
const path = require("node:path");

/**
 * @returns {string | null}
 */
function platformPackageName() {
  const platform = process.platform;
  const arch = process.arch;
  if (platform === "darwin" && arch === "arm64") {
    return "@create-grafana-plugin/darwin-arm64";
  }
  if (platform === "darwin" && arch === "x64") {
    return "@create-grafana-plugin/darwin-x64";
  }
  if (platform === "linux" && arch === "x64") {
    return "@create-grafana-plugin/linux-x64";
  }
  if (platform === "linux" && arch === "arm64") {
    return "@create-grafana-plugin/linux-arm64";
  }
  if (platform === "win32" && arch === "x64") {
    return "@create-grafana-plugin/win32-x64";
  }
  return null;
}

function main() {
  const pkgName = platformPackageName();
  if (pkgName === null) {
    console.error(
      `create-grafana-plugin: unsupported platform ${process.platform} ${process.arch}`,
    );
    process.exit(1);
  }

  let pkgRoot;
  try {
    pkgRoot = path.dirname(require.resolve(`${pkgName}/package.json`));
  } catch {
    console.error(
      `create-grafana-plugin: missing platform package ${pkgName}. ` +
        "Reinstall create-grafana-plugin so optional dependencies can be fetched.",
    );
    process.exit(1);
  }

  const binaryName =
    process.platform === "win32" ? "create-grafana-plugin.exe" : "create-grafana-plugin";
  const binaryPath = path.join(pkgRoot, "bin", binaryName);

  if (!fs.existsSync(binaryPath)) {
    console.error(`create-grafana-plugin: binary not found at ${binaryPath}`);
    process.exit(1);
  }

  const result = spawnSync(binaryPath, process.argv.slice(2), {
    stdio: "inherit",
    windowsHide: true,
  });

  if (result.error) {
    throw result.error;
  }

  process.exit(result.status === null ? 1 : result.status);
}

main();
