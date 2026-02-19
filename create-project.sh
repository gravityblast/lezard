#!/usr/bin/env bash
set -euo pipefail

if [ $# -ne 1 ]; then
    echo "Usage: $0 <project-name>"
    echo "Creates a new Lezard project as a sibling of the lezard directory."
    exit 1
fi

NAME="$1"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DEST="$SCRIPT_DIR/../$NAME"

if [ -e "$DEST" ]; then
    echo "Error: $DEST already exists"
    exit 1
fi

cp -r "$SCRIPT_DIR/template" "$DEST"

# Remove build artifacts that may exist from in-place development
rm -rf "$DEST/target" "$DEST/programs/.deps" "$DEST/Cargo.lock" "$DEST/programs/Cargo.lock"

# Fix Cargo.toml: lezard path goes from ".." (in-place) to "../lezard" (sibling)
sed -i '' 's|path = "\.\."|path = "../lezard"|' "$DEST/Cargo.toml"

# Fix Makefile: lssa path goes from "../../lssa" (in-place) to "../lssa" (sibling)
sed -i '' 's|../../lssa|../lssa|' "$DEST/Makefile"

# Set package name
sed -i '' "s|name = \"my-project\"|name = \"$NAME\"|" "$DEST/Cargo.toml"

echo "Created $DEST"
echo "  cd $DEST"
echo "  make build   # compile guest programs"
echo "  make test    # run tests"
