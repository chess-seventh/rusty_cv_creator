#!/usr/bin/env bash
set -euo pipefail

# 1. Get the most recent tag (fall back to v0.0.0 if none exists)
LAST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "")

echo "Last tag: ${LAST_TAG}"

# 2. Gather all commits since LAST_TAG
COMMITS=$(git log "${LAST_TAG}"..HEAD --pretty=format:"%s%n%b" | tr '\n' ' ')

echo "Commits since ${LAST_TAG}:"
echo "${COMMITS}"
echo

# 3. Initialize bump levels
BUMP_MAJOR=0
BUMP_MINOR=0
BUMP_PATCH=0

# 4. Check for breakage keywords or feat/fix in the commits
#    We'll mark major/minor/patch = 1 if we see them
#    Priority: major > minor > patch
if [[ "${COMMITS}" =~ (BREAKING[[:space:]]CHANGE|feat!) ]]; then
  BUMP_MAJOR=1
elif [[ "${COMMITS}" =~ feat ]]; then
  BUMP_MINOR=1
elif [[ "${COMMITS}" =~ fix ]]; then
  BUMP_PATCH=1
fi

# 5. Decide final bump
#    If there's a major trigger, we skip checking the others, etc.
if [ "${BUMP_MAJOR}" -eq 1 ]; then
  echo "major"
  exit 0
elif [ "${BUMP_MINOR}" -eq 1 ]; then
  echo "minor"
  exit 0
elif [ "${BUMP_PATCH}" -eq 1 ]; then
  echo "patch"
  exit 0
else
  # No recognized keywords => default to patch
  echo "patch"
  exit 0
fi
