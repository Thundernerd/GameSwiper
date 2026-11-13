#!/usr/bin/env bash
# Tests that the Docker image build reuses compiled Rust dependencies from an exported layer cache,
# as the release workflow does with its gha cache.
# Run: bash .github/scripts/docker-build-cache.test.sh
#
# It's slow: it compiles the backend in Docker more than once. CI doesn't run it.
#
# Each build runs on a new docker-container builder, like each release runs on a new runner. It
# imports the layer cache the previous build exported, and nothing else: cache mounts don't carry
# over. The test edits a copy of the working tree, never the checkout.
set -uo pipefail

root="$(cd "$(dirname "$0")/../.." && pwd)"
work="$(mktemp -d)"
context="$work/context"
cache="$work/cache"
builder="hypecheck-build-cache-test-$$"
failures=0

cleanup() {
  docker buildx rm "$builder" >/dev/null 2>&1
  rm -rf "$work"
}
trap cleanup EXIT

# Tracked and untracked files, leaving out what .gitignore ignores.
mkdir "$context"
git -C "$root" ls-files -z --cached --others --exclude-standard \
  | tar -C "$root" --null -T - -cf - \
  | tar -C "$context" -xf -

# build NAME [BUILD_ARGS...]: builds the copy on a new builder, importing and replacing the layer
# cache, and leaves the plain build log in $work/NAME.log.
build() {
  local name="$1" status
  shift
  local from=()
  if [[ -d "$cache" ]]; then from=(--cache-from "type=local,src=$cache"); fi

  docker buildx create --name "$builder" --driver docker-container >/dev/null || return 1
  docker buildx build --builder "$builder" --progress=plain \
    ${from[@]+"${from[@]}"} --cache-to "type=local,dest=$cache.new,mode=max" \
    "$@" "$context" >"$work/$name.log" 2>&1
  status=$?
  docker buildx rm "$builder" >/dev/null 2>&1

  if ((status != 0)); then
    echo "FAIL $name: docker build exited $status. Last lines of the log:"
    tail -n 40 "$work/$name.log"
    return 1
  fi
  rm -rf "$cache"
  mv "$cache.new" "$cache"
}

# compiled NAME: the crates cargo compiled in that build, one per line, sorted.
compiled() {
  sed -nE 's/.* Compiling ([^ ]+) v.*/\1/p' "$work/$1.log" | sort -u
}

# expect_only_backend_compiled NAME
expect_only_backend_compiled() {
  local crates
  crates="$(compiled "$1")"
  if [[ "$crates" != hypecheck_api ]]; then
    printf 'FAIL %s: expected only hypecheck_api to compile, compiled:\n%s\n' "$1" "${crates:-(nothing)}"
    failures=$((failures + 1))
  else
    echo "ok   $1"
  fi
}

echo "Building from scratch to fill the cache. This compiles every dependency."
build initial || exit 1

echo '// changed by docker-build-cache.test.sh' >>"$context/backend/src/main.rs"
if build source-change; then
  expect_only_backend_compiled source-change
else
  failures=$((failures + 1))
fi

# A release bumps the version in Cargo.toml, and with it the backend's own entry in Cargo.lock,
# while its dependencies stay the same.
sed -i.bak -E '1,/^version = /s/^version = .*/version = "99.0.0"/' "$context/backend/Cargo.toml"
sed -i.bak -E '/^name = "hypecheck_api"$/{n;s/^version = .*/version = "99.0.0"/;}' "$context/backend/Cargo.lock"
rm "$context"/backend/*.bak
if build version-bump; then
  expect_only_backend_compiled version-bump
else
  failures=$((failures + 1))
fi

if ((failures > 0)); then
  echo "$failures test(s) failed"
  exit 1
fi
