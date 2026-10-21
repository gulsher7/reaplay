# Choosing the first test game

Nothing here has been run. These are candidates selected against the criteria
in the project brief; each one's compatibility is a claim to be tested, not a
result. No game has been downloaded and nothing has been purchased.

## The key finding

**The first test does not need DirectX.** Every candidate below renders through
OpenGL or SDL, so it needs a Wine runtime and nothing else — no D3DMetal, no
DXMT, no Game Porting Toolkit. That matters, because the DirectX translation
layers are exactly the components with restrictive licences and Gatekeeper
problems (see `HANDOFF.md`).

Put differently: the licensing and Gatekeeper decision can be deferred past the
first launch. Plain Wine is enough to prove ReaPlay works end to end.

## Test in this order

Each rung isolates one new variable, so a failure says something specific
rather than "a game didn't start".

### 1. Anarch — process lifecycle only

Public-domain shooter by drummyfish. Software-rendered, no GPU API, famously
tiny and dependency-free.

Proves: the executable is found and validated, the process starts, a window
appears, keyboard input arrives, the process exits cleanly, and ReaPlay's logs
capture all of it. A failure here is a ReaPlay or Wine problem and *cannot* be
a graphics-translation problem — which is precisely what makes it the right
first rung.

### 2. Battle for Wesnoth — audio and a real game loop

Open-source turn-based strategy, SDL-based, 2D, long-standing and
well-documented.

Adds: audio output, mouse input, a substantial asset load, and a process that
runs long enough to observe memory behaviour on a 16 GB M4.

### 3. OpenArena — 3D graphics translation

Community deathmatch FPS built on the ioquake3 engine. OpenGL.

Adds: real-time 3D rendering and sustained frame output. This is the first rung
where a graphics translation layer is genuinely exercised.

## Other candidates noted

- **SuperDrome Bugs** — small free adventure game; a lighter alternative to
  Wesnoth.
- **Autopeli** — free split-screen racer; useful later for controller input.
- **Metanet Hunter G4** — $1 on itch.io. Paid, so out of scope without
  approval.

A broader pool is itch.io's free DRM-free Windows listing, which carries
several hundred titles.

## What to record for each attempt

The brief asks that a supported game be an empirical result. For every attempt,
record: macOS version, hardware, Wine/provider version, game version, the exact
settings used, and the observed outcome for each of — executable found,
environment detected, process started, window appeared and rendered, keyboard
and mouse worked, audio worked, clean exit with no stuck processes, logs
inspectable, and memory and performance on the M4.

ReaPlay's diagnostic export (Settings → Generate) captures the host half of
that automatically, with home paths redacted.

## Deliberately excluded

- Anything with anti-cheat, DRM or account authentication. The brief forbids
  bypassing these, and they are the most common cause of failure that cannot
  be fixed.
- Anything large or demanding. A long download and a heavy GPU load only make
  the first failure harder to read.
- **Bourbon 26**, a third-party tool advertising one-click management of the
  whole Windows gaming stack. Its claims come from its own marketing and it
  would install the proprietary components on your behalf — the opposite of
  the explicit, user-consented setup this project calls for.
