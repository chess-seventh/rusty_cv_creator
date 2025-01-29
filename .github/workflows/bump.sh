#!/usr/bin/env bash
set -euo pipefail

# Attempt to get the most recent tag. If none found, LAST_TAG will be an empty string.
LAST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "")

if [ -z "$LAST_TAG" ]; then
  # echo "No existing tags found. We'll treat all commits as new since the start of the repo."
  # Gather all commits in the repo
  COMMITS=$(git log --pretty=format:"%s%n%b" | tr '\n' ' ')
else
  # echo "Last tag: ${LAST_TAG}"
  # Gather all commits since LAST_TAG
  COMMITS=$(git log "${LAST_TAG}"..HEAD --pretty=format:"%s%n%b" | tr '\n' ' ')
fi

# echo "Commits analyzed:"
# echo "${COMMITS}"
# echo

# Initialize bump levels
BUMP_MAJOR=0
BUMP_MINOR=0
BUMP_PATCH=0

# Check for breakage keywords or feat/fix in the commits
# Priority: major > minor > patch
if [[ "${COMMITS}" =~ (BREAKING[[:space:]]CHANGE|feat!) ]]; then
  BUMP_MAJOR=1
elif [[ "${COMMITS}" =~ feat ]]; then
  BUMP_MINOR=1
elif [[ "${COMMITS}" =~ fix ]]; then
  BUMP_PATCH=1
fi

# Decide final bump
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
  # If no recognized keywords, default to patch (or 'none' if you want to skip)
  echo "patch"
  exit 0
fi
