#!/usr/bin/env bash
# ---
# relationships:
#   implements: publishing
# ---
set -euo pipefail

version="$(python3 -c 'import tomllib; print(tomllib.load(open("Cargo.toml", "rb"))["workspace"]["package"]["version"])')"
case "${GITHUB_EVENT_NAME:?event is required}" in
  push)
    if [[ "${GITHUB_REF_TYPE:-}" != tag || ! "${GITHUB_REF_NAME:-}" =~ ^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$ ]]; then
      echo "Release push must name a plain Semantic Versioning tag." >&2
      exit 1
    fi
    version="$GITHUB_REF_NAME"
    scripts/release/extract-release-notes.sh \
      "$version" CHANGELOG.md "${RUNNER_TEMP:?runner temp is required}/release-notes-preflight.md"
    ;;
  workflow_dispatch) ;;
  *)
    echo "Unsupported release workflow event: $GITHUB_EVENT_NAME" >&2
    exit 1
    ;;
esac
python3 scripts/release/verify-versions.py --expected "$version"
printf 'version=%s\n' "$version" >> "${GITHUB_OUTPUT:?output file is required}"
