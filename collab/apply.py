#!/usr/bin/env python3
"""Apply a browser-agent reply to the repo.

Paste ChatGPT's whole reply into a file (or pipe it in) and run:

    ./collab/apply.py collab/inbox/reply.md            # dry run, shows diffs
    ./collab/apply.py collab/inbox/reply.md --write     # actually write

Parses blocks in the handoff format and ignores all surrounding prose:

    === FILE: path/to/file.rs ===
    <full contents>
    === END FILE ===

    === DELETE: path/to/file.rs ===

Every path is checked against the owner's pathspecs in collab/areas.conf, so a
reply that strays outside its area is rejected rather than silently applied.
"""
import argparse
import difflib
import re
import subprocess
import sys
from pathlib import Path

FILE_RE = re.compile(r"^===[ \t]*FILE:[ \t]*(.+?)[ \t]*===[ \t]*$")
END_RE = re.compile(r"^===[ \t]*END FILE[ \t]*===[ \t]*$")
DEL_RE = re.compile(r"^===[ \t]*DELETE:[ \t]*(.+?)[ \t]*===[ \t]*$")


def repo_root() -> Path:
    out = subprocess.run(
        ["git", "rev-parse", "--show-toplevel"],
        capture_output=True, text=True, check=True,
    )
    return Path(out.stdout.strip())


def load_globs(conf: Path, owner: str) -> list[str]:
    globs = []
    for line in conf.read_text().splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        who, _, spec = line.partition(":")
        if who.strip() == owner:
            globs.append(spec.strip())
    return globs


def matches(path: str, globs: list[str]) -> bool:
    from fnmatch import fnmatch
    for g in globs:
        if g.endswith("/**"):
            if path == g[:-3] or path.startswith(g[:-2]):
                return True
        elif fnmatch(path, g):
            return True
    return False


def parse(text: str):
    """Return (writes, deletes, errors). writes maps path -> contents."""
    writes, deletes, errors = {}, [], []
    lines = text.splitlines()
    i = 0
    while i < len(lines):
        m = FILE_RE.match(lines[i])
        if m:
            path = m.group(1)
            body, i = [], i + 1
            closed = False
            while i < len(lines):
                if END_RE.match(lines[i]):
                    closed = True
                    i += 1
                    break
                if FILE_RE.match(lines[i]):
                    errors.append(f"{path}: new FILE block opened before END FILE")
                    break
                body.append(lines[i])
                i += 1
            if not closed and not any(e.startswith(f"{path}:") for e in errors):
                errors.append(f"{path}: block never closed with === END FILE ===")
            if closed:
                # Strip a single fenced code block if the model wrapped the body.
                if body and body[0].lstrip().startswith("```"):
                    j = len(body) - 1
                    while j > 0 and not body[j].strip():
                        j -= 1
                    if j > 0 and body[j].strip().startswith("```"):
                        body = body[1:j]
                writes[path] = "\n".join(body).rstrip("\n") + "\n"
            continue
        m = DEL_RE.match(lines[i])
        if m:
            deletes.append(m.group(1))
        i += 1
    return writes, deletes, errors


def unsafe(path: str) -> str | None:
    p = Path(path)
    if p.is_absolute():
        return "absolute path"
    if ".." in p.parts:
        return "contains '..'"
    if path.startswith(".git/") or path == ".git":
        return "inside .git"
    return None


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("reply", nargs="?", help="file with the agent's reply; omit to read stdin")
    ap.add_argument("--owner", default="chatgpt", help="whose area to enforce (default: chatgpt)")
    ap.add_argument("--write", action="store_true", help="apply changes (default is a dry run)")
    args = ap.parse_args()

    root = repo_root()
    text = Path(args.reply).read_text() if args.reply else sys.stdin.read()

    globs = load_globs(root / "collab" / "areas.conf", args.owner)
    if not globs:
        print(f"no areas defined for '{args.owner}' in collab/areas.conf", file=sys.stderr)
        return 1

    writes, deletes, errors = parse(text)
    if not writes and not deletes:
        print("no FILE or DELETE blocks found.", file=sys.stderr)
        print("Did the reply use the handoff format? See the protocol section", file=sys.stderr)
        print("at the end of any collab/outbox/context-*.md bundle.", file=sys.stderr)
        return 1

    rejected = []
    for path in list(writes) + deletes:
        why = unsafe(path)
        if why:
            rejected.append((path, why))
        elif not matches(path, globs):
            rejected.append((path, f"outside {args.owner}'s area"))

    for path, why in rejected:
        writes.pop(path, None)
        if path in deletes:
            deletes.remove(path)

    print(f"=== {'APPLYING' if args.write else 'DRY RUN'} (owner: {args.owner}) ===\n")

    for path, body in sorted(writes.items()):
        target = root / path
        old = target.read_text().splitlines(keepends=True) if target.is_file() else []
        new = body.splitlines(keepends=True)
        verb = "modify" if old else "create"
        diff = list(difflib.unified_diff(old, new, f"a/{path}", f"b/{path}", n=2))
        if old and not diff:
            print(f"  unchanged  {path}")
            continue
        print(f"  {verb:9} {path}  (+{sum(1 for d in diff if d.startswith('+') and not d.startswith('+++'))}"
              f"/-{sum(1 for d in diff if d.startswith('-') and not d.startswith('---'))})")
        if old:
            for d in diff[:40]:
                print("      " + d.rstrip("\n"))
            if len(diff) > 40:
                print(f"      ... {len(diff) - 40} more diff lines")
        if args.write:
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(body)

    for path in sorted(deletes):
        target = root / path
        if target.is_file():
            print(f"  delete    {path}")
            if args.write:
                target.unlink()
        else:
            print(f"  skip      {path} (does not exist)")

    if errors:
        print("\n=== PARSE PROBLEMS ===")
        for e in errors:
            print(f"  ! {e}")

    if rejected:
        print("\n=== REJECTED ===")
        for path, why in rejected:
            print(f"  ! {path}: {why}")
        print("\nThese were NOT applied. Either the agent strayed outside its")
        print("area, or collab/areas.conf needs updating by agreement.")

    if not args.write:
        print("\nDry run only. Re-run with --write to apply.")
    else:
        print(f"\nApplied. Review with: git -C {root} diff")

    return 2 if (errors or rejected) else 0


if __name__ == "__main__":
    sys.exit(main())
