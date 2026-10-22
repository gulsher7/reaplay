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

## Result: Anarch ran, 2026-10-22

Anarch 1.01 is the first Windows game ReaPlay has launched end to end.

| | |
|---|---|
| Source | `bin/Anarch_winshitxp_sdl_1-01.zip` in the author's own repo, <https://gitlab.com/drummyfish/anarch> |
| sha256 (zip) | `b8d109102e7cd012f34ba59ef4a89f07e6db9e5edfe9ed476b63158b19141a67` |
| Licence | Unlicense (code), CC0 (assets) — public domain, so nothing is redistributed under restriction |
| Binary | `PE32 executable (console) Intel 80386` — **32-bit x86** |
| Runtime | CrossOver 26.3, bottle `ReaPlay` (`win10_64`) |
| Host | macOS 26.6.2, aarch64 |

It rendered its title screen and menu, and was tracked and terminated from
ReaPlay. The prediction in this document held: no DirectX, no D3DMetal and no
Game Porting Toolkit were needed. Plain Wine-via-CrossOver was enough.

### Two things worth knowing

**The game is 32-bit.** That was not planned, and it turned out to be a
stronger test than a 64-bit binary: it exercises CrossOver's 32-bit support on
Apple Silicon, which is the part most likely to disappear. macOS raised a
CrossOver notice during the run — *"This version of CrossOver includes a
component that will not work with a future release of macOS"* — which refers to
exactly that. A 64-bit game should be the next rung, so the first failure of
the 32-bit path does not read as a ReaPlay regression.

**"MALWARE OS DETECTED" is not an error.** Anarch prints it on purpose when it
believes it is running on Windows. It is the author's joke, not a diagnostic.

### Command ReaPlay builds

```
/Applications/CrossOver.app/Contents/SharedSupport/CrossOver/bin/wine
  --bottle ReaPlay
  --no-gui          # errors go to the launch log, not a modal nobody dismisses
  --wait            # so the recorded exit code is the game's
  <native path to the .exe>
```

Still an argument array, never a shell string.

## Phase 5 verification, 2026-10-22

| Aspect | Result |
|---|---|
| Graphics | Title screen and menu render correctly |
| Input | Confirmed by playing it |
| Audio | Confirmed working |
| Exit behaviour | Clean exit records `Exited`; a stop records `Terminated` |
| Logs | Program, argument array, exit code and captured output recorded |
| Resource usage | ~25–35% of one core, 239 MB RSS, 18 threads (measured on the game process) |

Total footprint including CrossOver's helpers (`wineserver`, two `winedevice`,
the wrapper) was about 545 MB across 6 processes.

### A bug this found: terminate did not stop the game

Measuring resource usage exposed a real defect, because it forced the question
*which* process is the game. It is not the one ReaPlay spawns.

```
44633  wine (Perl wrapper)      <- what ReaPlay spawned and tracked
44634  winewrapper.exe          ppid 44633
44658  Y:\...\Anarch...exe      ppid 1, pgid 44658   <- the actual game
```

Wine reparents the game to init **and** puts it in its own process group, so
neither killing the spawned child nor killing its process group reaches it.
Killing the wrapper was verified to leave the game running at 19% CPU with its
window still on screen, while ReaPlay recorded `Terminated`. The app was
reporting something that had not happened.

The fix is `StopCommand`, a shutdown command the provider supplies and the
registry runs after killing the child. CrossOver supplies `wineserver -k`
scoped to the bottle by `WINEPREFIX`.

Two consequences worth stating plainly:

- **Shutdown is bottle-wide.** Stopping one game stops everything else in that
  bottle. One bottle per game makes it exact, which is what per-game profiles
  are for. The fix is narrower scope, not a wider kill.
- **A non-zero `wineserver -k` is not a failure.** It exits 1 when no server is
  running, which means there was nothing left to stop.

### Why the first version of the test did not catch it

The test asserted the *record* said `Terminated`. That was true, and useless.
It now asserts the game process is actually gone, and it was verified to fail
when the shutdown command is removed, so it tests the fix rather than
decorating it.

Two further traps, both of which produced a passing test that proved nothing:

- Searching for the bare file name matched the `wine` wrapper, whose arguments
  contain the game's native path, so the test "found the game" before the game
  existed. It now matches the Windows path separator (`\game.exe`).
- A stray game from a previous run satisfied every assertion. The test now
  refuses to start if one is already running.
