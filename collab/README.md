# Two-agent workflow

Claude Code and browser-based ChatGPT both work on this repo, in parallel, on
separate areas. Git is the shared workspace; ChatGPT cannot touch files, so you
carry its output in — but the scripts here reduce that to one paste each way.

## The split

Ownership lives in `collab/areas.conf` and follows the boundary already in
`docs/ARCHITECTURE.md`:

| Area | Owner | Paths |
|---|---|---|
| Rust application layer | ChatGPT | `src-tauri/**` |
| React UI | Claude Code | `src/**`, `index.html` |
| Command surface + docs | neither, by agreement | `docs/IPC_CONTRACT.md`, docs, configs |

The two sides meet only at the Tauri command surface. That surface is written
down in `docs/IPC_CONTRACT.md` **before** either side builds against it. Both
agents code to the contract rather than to each other, which is what makes
working in parallel possible instead of a constant renegotiation.

Changing the contract mid-flight is the one thing that breaks this setup. If a
change is needed, it goes in a handoff note and both sides adopt it together.

## Branches

    main                  integration; merge here
    chatgpt/<topic>        ChatGPT's work, applied by you
    claude/<topic>         Claude Code's work

Neither agent commits to `main` directly. Merge when a topic is done.

## The loop

**Sending context to ChatGPT:**

    ./collab/bundle.sh chatgpt

Writes `collab/outbox/context-<ts>.md` and copies it to your clipboard. The
bundle contains its assignment, the full contents of files it owns, shared
files as read-only context, a name-only listing of the rest, what changed since
its last bundle, and the output format it must use. Paste it into ChatGPT and
add your actual request underneath.

**Bringing ChatGPT's reply back:**

Copy its whole reply into a file, then:

    ./collab/apply.py collab/inbox/reply.md            # dry run: shows diffs
    ./collab/apply.py collab/inbox/reply.md --write     # apply

Prose is ignored; only `=== FILE: ... ===` blocks are read. Anything outside
ChatGPT's area is rejected rather than applied, so a stray edit to the UI
cannot land silently. Review with `git diff`, then commit on a `chatgpt/*`
branch so the authorship stays honest in the history.

**Claude Code's side:** it reads and writes the repo directly, on `claude/*`
branches.

## Handoffs

Each agent ends a work session with a HANDOFF note: what changed, what was
assumed, what the other side needs to know. Paste Claude's notes into ChatGPT
and vice versa — that, plus the diff, is the whole channel between them.

## What this is not

Neither agent sees the other's reasoning, and neither can wait on the other.
Treat them as two contractors sharing a repo and a written spec, not as a
conversation. Keep the contract current and the areas disjoint, and that is
enough.
