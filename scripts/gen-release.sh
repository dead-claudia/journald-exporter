#!/usr/bin/env bash
set -euo pipefail

version="${1:-}"

# Verify a version is given.
if [[ -z "$version" ]]; then
    echo 'A version is required'
    exit 1
fi

# Verify the version is correct.
if ! [[ "$version" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "Invalid version $version"
    exit 1
fi

# Fetch remote data to check remaining conditions
git fetch origin main 'refs/tags/*:refs/tags/*'

# Verify the tag doesn't already exist.
if git show-ref --quiet "refs/tags/$version"; then
    echo "Tag $version already exists. Please select a new version number."
    exit 1
fi

# Verify the tree is clean.
if [[ -n "$(git status --porcelain 2>/dev/null || true)" ]]; then
    echo 'Tree is dirty. Please commit all changes before creating any new versions.'
    exit 1
fi

# Verify `main` is in sync with `HEAD`
if ! git diff-tree --quiet main HEAD; then
    echo "'HEAD' is not in sync with 'main'. Only create releases from 'main'."
    exit 1
fi

# Verify the tree is up to date.
remote_diff="$(git rev-list main..origin/main)"
if [[ -n "$remote_diff" ]]; then
    echo 'Make sure the current tree is in sync with the remote branch.'
    exit 1
fi

# Compile the release binary and run the unit and E2E tests as a quick smoke check. The full test
# battery is run in CI, so it doesn't need to be done *locally*.
cargo +stable build --release
# Retry up to 3 times like in CI
cargo +stable test || cargo +stable test || cargo +stable test
sudo node ./scripts/e2e.js

# Now, generate the tag and ensure it's signed.
git tag --sign --annotate --message="$version" "$version"

# And finally, push.
git push origin main "refs/tags/$version:refs/tags/$version"
