#!/usr/bin/env bash
# Checks a release tag against the versions in backend/Cargo.toml and frontend/package.json.
#
# Usage: check-release-version.sh <tag> [repo-root]
#
# A pre-release tag (vX.Y.Z-suffix) matches manifests that say either X.Y.Z-suffix or X.Y.Z.
# On success prints `version=<X.Y.Z[-pre]>` and `prerelease=<true|false>`, ready to append to
# $GITHUB_OUTPUT. On a mismatch prints the reason to stderr and exits non-zero.
set -euo pipefail

tag="$1"
root="${2:-.}"

# The release trigger's `v*.*.*` glob also matches tags such as `vnext.major.release`.
if [[ ! "$tag" =~ ^v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$ ]]; then
  echo "$tag is not a version tag (vX.Y.Z or vX.Y.Z-suffix)" >&2
  exit 1
fi

version="${tag#v}"
core="${version%%-*}"

# The `version` key of the [package] table, so a dependency's inline `version = ...` is ignored.
cargo_version="$(awk '
  /^\[/ { in_package = ($0 == "[package]") }
  in_package && /^version[[:space:]]*=/ {
    sub(/^version[[:space:]]*=[[:space:]]*"/, ""); sub(/".*$/, ""); print; exit
  }
' "$root/backend/Cargo.toml")"
npm_version="$(jq -r '.version // empty' "$root/frontend/package.json")"

# require_match <manifest> <its version>
require_match() {
  if [[ "$2" != "$version" && "$2" != "$core" ]]; then
    echo "tag $tag does not match $1 version '$2'" >&2
    exit 1
  fi
}
require_match backend/Cargo.toml "$cargo_version"
require_match frontend/package.json "$npm_version"

echo "version=$version"
if [[ "$version" == "$core" ]]; then
  echo "prerelease=false"
else
  echo "prerelease=true"
fi
