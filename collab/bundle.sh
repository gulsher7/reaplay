#!/usr/bin/env bash
# Build a paste-ready context bundle for the browser-only agent.
#
#   ./collab/bundle.sh [owner]      # owner defaults to "chatgpt"
#
# Writes collab/outbox/context-<ts>.md and copies it to the clipboard.
set -euo pipefail

OWNER="${1:-chatgpt}"
ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

CONF="collab/areas.conf"
[ -f "$CONF" ] || { echo "missing $CONF" >&2; exit 1; }

globs_for() {
  grep -v '^[[:space:]]*#' "$CONF" | grep -v '^[[:space:]]*$' \
    | awk -F: -v o="$1" '$1==o {sub(/^[^:]*:/,""); print}'
}

# Read into arrays the bash-3.2 way; macOS has no `mapfile`.
OWN_GLOBS=()
while IFS= read -r line; do OWN_GLOBS[${#OWN_GLOBS[@]}]="$line"; done < <(globs_for "$OWNER")
SHARED_GLOBS=()
while IFS= read -r line; do SHARED_GLOBS[${#SHARED_GLOBS[@]}]="$line"; done < <(globs_for shared)
[ "${#OWN_GLOBS[@]}" -gt 0 ] || { echo "no areas defined for '$OWNER' in $CONF" >&2; exit 1; }

STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
OUT="collab/outbox/context-${STAMP}.md"
mkdir -p collab/outbox

MARK=".git/collab-last-bundle-${OWNER}"
SINCE=""
[ -f "$MARK" ] && SINCE="$(cat "$MARK")"

{
  echo "# Repo context for: ${OWNER}"
  echo
  echo "- repo: \`$(basename "$ROOT")\`"
  echo "- branch: \`$(git rev-parse --abbrev-ref HEAD)\`"
  echo "- commit: \`$(git rev-parse --short HEAD)\`"
  echo "- generated: ${STAMP}"
  echo
  echo "## Your assignment"
  echo
  echo "You own these paths and may edit **only** these:"
  echo
  for g in "${OWN_GLOBS[@]}"; do echo "- \`$g\`"; done
  echo
  echo "Everything else is read-only to you. Another agent (Claude Code) owns the"
  echo "rest of the repo and is working in parallel. Editing outside your paths"
  echo "creates merge conflicts, so don't."
  echo

  if [ -n "$SINCE" ] && git rev-parse --verify -q "$SINCE" >/dev/null; then
    echo "## Changed since your last bundle (${SINCE})"
    echo
    echo '```'
    git log --oneline "${SINCE}..HEAD" || true
    echo '```'
    echo
    echo '```'
    git diff --stat "${SINCE}..HEAD" || true
    echo '```'
    echo
  fi

  echo "## Files you own"
  echo
  git ls-files -- "${OWN_GLOBS[@]}" | while IFS= read -r f; do
    [ -f "$f" ] || continue
    echo "### \`$f\`"
    echo
    echo '```'"${f##*.}"
    cat "$f"
    echo '```'
    echo
  done

  if [ "${#SHARED_GLOBS[@]}" -gt 0 ]; then
    echo "## Shared files (read-only context)"
    echo
    git ls-files -- "${SHARED_GLOBS[@]}" 2>/dev/null | while IFS= read -r f; do
      [ -f "$f" ] || continue
      echo "### \`$f\`"
      echo
      echo '```'"${f##*.}"
      cat "$f"
      echo '```'
      echo
    done
  fi

  echo "## Rest of the repo (names only — do not edit)"
  echo
  echo '```'
  git ls-files | grep -vxF -f <(git ls-files -- "${OWN_GLOBS[@]}") || git ls-files
  echo '```'
  echo

  cat <<'PROTOCOL'
## Required output format

When you write code, emit each complete file between these exact markers —
no unified diffs, no partial snippets, no "...rest unchanged":

=== FILE: path/relative/to/repo/root.ts ===
<the entire new contents of the file>
=== END FILE ===

To delete a file:

=== DELETE: path/to/file.ts ===

Use one block per file, as many blocks as you need. Put any prose, reasoning,
or questions outside the markers — a parser reads the blocks and ignores the
rest. Paths must fall inside the paths you own, listed above.

End your reply with a short HANDOFF note: what you changed, what you assumed,
and anything you need from the other agent.
PROTOCOL
} > "$OUT"

git rev-parse HEAD > "$MARK"

BYTES=$(wc -c < "$OUT" | tr -d ' ')
echo "wrote $OUT (${BYTES} bytes)"
if [ "$BYTES" -gt 300000 ]; then
  echo "WARNING: bundle is large; a chat window may truncate it." >&2
  echo "Narrow the owner's globs in $CONF and regenerate." >&2
fi
if command -v pbcopy >/dev/null 2>&1; then
  pbcopy < "$OUT" && echo "copied to clipboard — paste into ChatGPT"
fi
