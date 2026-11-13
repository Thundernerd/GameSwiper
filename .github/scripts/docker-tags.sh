#!/usr/bin/env bash
# Lists the tags a release pushes for the container image, one per line.
#
# Usage: docker-tags.sh <image> <version> <prerelease> <latest>
#
# <version>, <prerelease> and <latest> are the outputs of check-release-version.sh. A stable
# release X.Y.Z gets X.Y.Z and X.Y, plus latest when <latest> is true, so a patch for an older
# line doesn't move latest back. A pre-release gets only its own version, so it never moves the
# tags people pull for a stable release.
set -euo pipefail

image="$1"
version="$2"
prerelease="$3"
latest="$4"

echo "$image:$version"
if [[ "$prerelease" == true ]]; then
  exit 0
fi

IFS=. read -r major minor _ <<<"$version"
echo "$image:$major.$minor"
if [[ "$latest" == true ]]; then
  echo "$image:latest"
fi
