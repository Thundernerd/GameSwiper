#!/usr/bin/env bash
# Checks a release tag against the versions in backend/Cargo.toml and frontend/package.json.
#
# Usage: git tag --list | check-release-version.sh <tag> [repo-root]
#
# A pre-release tag (vX.Y.Z-suffix) matches manifests that say either X.Y.Z-suffix or X.Y.Z.
# Stdin is the repo's tags, one per line; they decide whether the release becomes `latest`.
# On success prints `version=<X.Y.Z[-pre]>`, `prerelease=<true|false>` and `latest=<true|false>`,
# ready to append to $GITHUB_OUTPUT. On a mismatch prints the reason to stderr and exits non-zero.
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
if [[ "$version" != "$core" ]]; then
  echo "prerelease=true"
  echo "latest=false"
  exit 0
fi
echo "prerelease=false"

# Latest only if no stable vX.Y.Z tag is newer, so a patch for an older line leaves it alone.
newest="$( { grep -E '^v[0-9]+\.[0-9]+\.[0-9]+$' || true; echo "$tag"; } | sort -V | tail -n 1)"
if [[ "$newest" == "$tag" ]]; then
  echo "latest=true"
else
  echo "latest=false"
fi
