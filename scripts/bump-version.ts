#!/usr/bin/env bun
/**
 * Sync or bump version across the entire project.
 *
 * Source of truth: [workspace.package] version in root Cargo.toml
 *
 * Usage:
 *   bun scripts/bump-version.ts              — sync Cargo.toml version to all package.json files
 *   bun scripts/bump-version.ts patch        — bump patch, then sync everywhere
 *   bun scripts/bump-version.ts minor        — bump minor, then sync everywhere
 *   bun scripts/bump-version.ts major        — bump major, then sync everywhere
 *   bun scripts/bump-version.ts 1.2.3        — set explicit version everywhere
 */

import { readFileSync, writeFileSync, existsSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(HERE, '..');
const CARGO_TOML = path.join(ROOT, 'Cargo.toml');

const NPM_PACKAGES = [
  'packaging/npm/create-grafana-plugin',
  'packaging/npm/darwin-arm64',
  'packaging/npm/darwin-x64',
  'packaging/npm/linux-x64',
  'packaging/npm/linux-arm64',
  'packaging/npm/win32-x64',
];

const PLATFORM_SCOPES = [
  '@andeya/create-grafana-plugin-darwin-arm64',
  '@andeya/create-grafana-plugin-darwin-x64',
  '@andeya/create-grafana-plugin-linux-x64',
  '@andeya/create-grafana-plugin-linux-arm64',
  '@andeya/create-grafana-plugin-win32-x64',
];

const INCREMENTS = ['major', 'minor', 'patch'] as const;

function readCargoVersion(): string {
  const content = readFileSync(CARGO_TOML, 'utf8');
  const match = content.match(/\[workspace\.package\][\s\S]*?version\s*=\s*"([^"]+)"/);
  if (!match) {
    throw new Error('Could not find [workspace.package] version in Cargo.toml');
  }
  return match[1];
}

function writeCargoVersion(version: string): void {
  let content = readFileSync(CARGO_TOML, 'utf8');
  content = content.replace(
    /(\[workspace\.package\][\s\S]*?version\s*=\s*)"[^"]*"/,
    `$1"${version}"`,
  );
  writeFileSync(CARGO_TOML, content);
  console.log(`  ✓ Cargo.toml (workspace) → ${version}`);
}

function incrementVersion(current: string, level: (typeof INCREMENTS)[number]): string {
  const parts = current.split('.').map(Number);
  const idx = INCREMENTS.indexOf(level);
  parts[idx] += 1;
  for (let i = idx + 1; i < 3; i++) parts[i] = 0;
  return parts.join('.');
}

function updatePackageJson(dir: string, version: string): void {
  const pkgPath = path.join(ROOT, dir, 'package.json');
  if (!existsSync(pkgPath)) {
    console.warn(`  ⚠ ${pkgPath} not found, skipping`);
    return;
  }
  const pkg = JSON.parse(readFileSync(pkgPath, 'utf8')) as {
    version: string;
    optionalDependencies?: Record<string, string>;
  };
  pkg.version = version;

  if (pkg.optionalDependencies) {
    for (const scope of PLATFORM_SCOPES) {
      if (scope in pkg.optionalDependencies) {
        pkg.optionalDependencies[scope] = version;
      }
    }
  }

  writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + '\n');
  console.log(`  ✓ ${dir}/package.json → ${version}`);
}

function updateRootPackageJson(version: string): void {
  const pkgPath = path.join(ROOT, 'package.json');
  const pkg = JSON.parse(readFileSync(pkgPath, 'utf8')) as { version: string };
  pkg.version = version;
  writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + '\n');
  console.log(`  ✓ package.json (root) → ${version}`);
}

function main(): void {
  const arg = process.argv[2];
  let version: string;

  if (!arg) {
    version = readCargoVersion();
    console.log(`\nSyncing version from Cargo.toml: ${version}\n`);
  } else if ((INCREMENTS as readonly string[]).includes(arg)) {
    const current = readCargoVersion();
    version = incrementVersion(current, arg as (typeof INCREMENTS)[number]);
    console.log(`\nBumping ${arg}: ${current} → ${version}\n`);
    writeCargoVersion(version);
  } else if (/^\d+\.\d+\.\d+/.test(arg)) {
    version = arg;
    console.log(`\nSetting version: ${version}\n`);
    writeCargoVersion(version);
  } else {
    console.error(
      `Invalid argument: "${arg}". Use: patch, minor, major, or a semver string (x.y.z).`,
    );
    process.exit(1);
  }

  updateRootPackageJson(version);
  for (const dir of NPM_PACKAGES) {
    updatePackageJson(dir, version);
  }

  console.log(`\nDone. All files updated to ${version}.`);
  console.log('Remember to update Cargo.lock: cargo check\n');
}

main();
