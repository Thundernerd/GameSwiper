#!/usr/bin/env bash
# Tests for check-release-version.sh. Run: bash .github/scripts/check-release-version.test.sh
set -uo pipefail

script="$(cd "$(dirname "$0")" && pwd)/check-release-version.sh"
failures=0

# Builds a throwaway repo root with the given backend and frontend versions.
make_repo() {
  local root
  root="$(mktemp -d)"
  mkdir -p "$root/backend" "$root/frontend"
  cat >"$root/backend/Cargo.toml" <<EOF
[package]
name = "hypecheck_api"
version = "$1"
edition = "2021"

[dependencies]
anyhow = "1"
serde = { version = "1", features = ["derive"] }
EOF
  cat >"$root/frontend/package.json" <<EOF
{
  "name": "frontend",
  "version": "$2",
  "private": true
}
EOF
  echo "$root"
}

# expect_pass NAME TAG CARGO_VERSION NPM_VERSION EXPECTED_STDOUT
expect_pass() {
  local root out
  root="$(make_repo "$3" "$4")"
  if ! out="$(bash "$script" "$2" "$root" 2>&1 </dev/null)"; then
    echo "FAIL $1: exited non-zero: $out"
    failures=$((failures + 1))
  elif [[ "$out" != "$5" ]]; then
    printf 'FAIL %s:\n  expected: %q\n  actual:   %q\n' "$1" "$5" "$out"
    failures=$((failures + 1))
  else
    echo "ok   $1"
  fi
  rm -rf "$root"
}

# expect_fail NAME TAG CARGO_VERSION NPM_VERSION EXPECTED_STDERR_SUBSTRING
expect_fail() {
  local root err
  root="$(make_repo "$3" "$4")"
  if err="$(bash "$script" "$2" "$root" 2>&1 >/dev/null </dev/null)"; then
    echo "FAIL $1: expected a non-zero exit"
    failures=$((failures + 1))
  elif [[ "$err" != *"$5"* ]]; then
    printf 'FAIL %s: stderr lacks %q:\n  %s\n' "$1" "$5" "$err"
    failures=$((failures + 1))
  else
    echo "ok   $1"
  fi
  rm -rf "$root"
}

# expect_latest NAME TAG TAG_LIST EXPECTED_LATEST
# Feeds TAG_LIST (the repo's tags, one per line) on stdin, against manifests that match TAG.
expect_latest() {
  local root version out
  version="${2#v}"
  root="$(make_repo "$version" "$version")"
  if ! out="$(bash "$script" "$2" "$root" 2>&1 <<<"$3")"; then
    echo "FAIL $1: exited non-zero: $out"
    failures=$((failures + 1))
  elif ! grep -qx "latest=$4" <<<"$out"; then
    printf 'FAIL %s: expected latest=%s in:\n  %s\n' "$1" "$4" "$out"
    failures=$((failures + 1))
  else
    echo "ok   $1"
  fi
  rm -rf "$root"
}

expect_pass "release tag matching both manifests" \
  v0.1.0 0.1.0 0.1.0 $'version=0.1.0\nprerelease=false\nlatest=true'
expect_fail "tag that doesn't match Cargo.toml" \
  v0.2.0 0.1.0 0.2.0 "backend/Cargo.toml"
expect_fail "tag that doesn't match package.json" \
  v0.2.0 0.2.0 0.1.0 "frontend/package.json"
expect_pass "pre-release tag of the manifests' version" \
  v0.1.0-rc.1 0.1.0 0.1.0 $'version=0.1.0-rc.1\nprerelease=true\nlatest=false'
expect_pass "pre-release tag matching pre-release manifests" \
  v1.0.0-rc.1 1.0.0-rc.1 1.0.0-rc.1 $'version=1.0.0-rc.1\nprerelease=true\nlatest=false'
expect_fail "release tag against pre-release manifests" \
  v1.0.0 1.0.0-rc.1 1.0.0-rc.1 "backend/Cargo.toml"
expect_fail "package.json without a version" \
  v0.1.0 0.1.0 "" "frontend/package.json"
expect_fail "tag that isn't vX.Y.Z" \
  vnext.major.release 0.1.0 0.1.0 "not a version tag"

expect_latest "newest stable release moves latest" \
  v1.2.1 $'v1.1.0\nv1.1.5\nv1.2.0\nv1.2.1' true
expect_latest "patch for an older line leaves latest alone" \
  v1.1.6 $'v1.1.0\nv1.1.5\nv1.2.0\nv1.1.6' false
expect_latest "pre-release never moves latest" \
  v2.0.0-rc.1 $'v1.2.0\nv2.0.0-rc.1' false
expect_latest "first ever release moves latest" \
  v0.1.0 'v0.1.0' true
expect_latest "newer pre-release tags don't hold latest back" \
  v1.2.1 $'v1.2.0\nv1.3.0-rc.1\nv1.2.1\nvnext.major.release' true

# The workflow pipes `git tag --list` in under pipefail, so the script must read all of stdin even
# when it doesn't need the tags, or the writer dies of SIGPIPE and fails the step.
root="$(make_repo 2.0.0-rc.1 2.0.0-rc.1)"
if ! (set -o pipefail; seq -f "v1.0.%g" 1 20000 | bash "$script" v2.0.0-rc.1 "$root" >/dev/null); then
  echo "FAIL pre-release reads a long tag list to the end: the pipe failed"
  failures=$((failures + 1))
else
  echo "ok   pre-release reads a long tag list to the end"
fi
rm -rf "$root"

if ((failures > 0)); then
  echo "$failures test(s) failed"
  exit 1
fi
