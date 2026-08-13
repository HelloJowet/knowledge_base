#!/bin/sh

set -eu

# This script always regenerates every skill reference, so it has no options.
if [ "$#" -ne 0 ]; then
    echo "Usage: $0" >&2
    exit 2
fi

# Find the repository root so the script works from any current directory.
SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
REPOSITORY_ROOT=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)
CLI_REFERENCES=$REPOSITORY_ROOT/skills/knowledge-base-cli/references/docs
DATA_MODEL_REFERENCES=$REPOSITORY_ROOT/skills/knowledge-base-data-model/references/docs

# These directories contain generated copies only. Replacing them completely
# removes pages that were deleted or renamed in docs/.
rm -rf "$CLI_REFERENCES" "$DATA_MODEL_REFERENCES"
mkdir -p "$CLI_REFERENCES/cli" "$DATA_MODEL_REFERENCES"

# Bundle the CLI documentation and the pages it links to, allowing the CLI
# skill to be installed and used without the rest of this repository.
cp "$REPOSITORY_ROOT"/docs/cli/*.md "$CLI_REFERENCES/cli/"
cp "$REPOSITORY_ROOT/docs/ingestion.md" "$CLI_REFERENCES/"
cp "$REPOSITORY_ROOT/docs/data-model.md" "$CLI_REFERENCES/"

# The data-model skill only needs the canonical data-model documentation.
cp "$REPOSITORY_ROOT/docs/data-model.md" "$DATA_MODEL_REFERENCES/"

echo "Generated skill references from docs/."
