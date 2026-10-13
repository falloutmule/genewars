# Scoped isolated CD reference observation — completed

Observed result: root supervised the single approved run, released the gate at 2026-10-13 00:58:39.250972 UTC, and the original returned after reporting `Can't read config.ini!` / `Please install GENEWARS properly to play!`. No menu or mission was reached. The run is finished and its approval does not cover another original, launcher, installer or setup run. `reference-run-result.json` records actual hashes and limits; `reference-readiness.md` describes the outcome. The protocol below is preserved as the reviewed procedure, not a pending authorization request.

Prepared 2026-10-12 local. **The human has explicitly replied “Approve this scoped reference run”; that separate approval is recorded in reference-approval.json. Root supervises the single approved session. This review lane has only run hidden empty fixtures and must not start another original session.** The approval covers the exact candidate and scope below, not installers, patches or alternate executables.

## Requested scope

One supervised local reference session, maximum 20 minutes, running the unmodified CD `GAME/GW.EXE;1` and its bundled DOS/4GW runtime in DOSBox-X. Observe startup, menu, first selectable campaign mission, the static left panel and a short timing baseline; optionally create one game save and reload it. Preserve all captures/saves privately. No network play, patches, installers, sound setup programs, cheats, executable modification, host-drive mounts, uploads, or other executable variants. If direct startup rejects the prepared layout, stop and report the exact error; do not silently run the launcher, installer or patcher.

Approval wording for root to offer: “Approve one supervised, at most 20-minute DOSBox-X session of the exact CD GW.EXE and DOS4GW.EXE hashes listed in this proposal, using only a private disposable game copy and read-only disc-image mount, with networking and host integrations disabled, to capture menus, the first available mission, panel appearance, timing observations and one optional save/reload.” Approval must come from the human; this proposal is not approval. Emulator acquisition and private staging are complete; original execution remains the separate approval boundary.

## Exact inputs and private mount layout

Project `P = C:\Users\fallo\Documents\ChatGPT\Freecad\genewars-preflight`.

| Input | Source/expected SHA-256 |
|---|---|
| Original source archive (never mounted) | `8153f36276f46acfb356f3199c7c2e89c9fa936b2afdf1b37039f143e2657906` |
| CD `GAME/GW.EXE;1`, 1,265,994 bytes | `4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba` |
| CD `GAME/DOS4GW.EXE;1`, 265,396 bytes | `b8265123ac8a189637448618409ef3ecd2e9f3e1a47062c685a02240f688dec1` |
| Existing `private/disc/member-004.img`, 630,947,520 bytes | `3ae5f2154a84898a7d1652e550d741a0a431fe67fd0b3eb20121e3b11fd4e5de` |
| Existing `private/disc/member-003.cue`, 238 bytes | `ba85aeb6a808145b43998a34faf6904956866539625bd45ead6dba0c2fd10ee5` |

Prepared and hash-verified `private/reference-run/run-001/` contains:

```text
run-001/
  drive-c/GAME/       writable disposable copy of the 490 inventoried CD GAME files
  media/GENEWARS.img  private copy of member-004.img, hash unchanged
  media/GENEWARS.cue  private copy of member-003.cue, hash unchanged
  captures/          private host capture output, not DOS-mounted
  states/            private emulator state path, not DOS-mounted
  logs/              configuration, tool/input hashes, timestamps and file changes
  dosbox-x.conf      reviewed full configuration, empty autoexec/config sections
```

Use the existing manifest predicate `!/GENEWARS.img!/GAME/` and its per-file hashes/ISO logical offsets to copy only that known tree. Remove terminal ISO `;1` from DOS names, preserve relative directories and bytes, reject path traversal/reparse points, and validate all 490 hashes. This is one bounded staging copy for the proposed runtime, not another inventory/extraction campaign. Do not use the patched small RIP to fill gaps. `C:` exposes only `drive-c`; saves/config/error logs may modify this disposable copy. Preserve a before/after file-hash manifest. `D:` is the copied CUE image; its existing relative `GENEWARS.img` reference resolves inside `media`. The multi-track descriptor must be retained rather than flattening the disc to a data-only ISO. Official DOSBox-X documentation supports CUE/BIN multi-track CD mounts and read-only CD image semantics. [Image guide](https://dosbox-x.com/wiki/Guide:Managing-image-files-in-DOSBox%E2%80%90X)

CD volume label `GENEWARS QA BETA V5` remains observed provenance, not retail proof. Running this candidate cannot itself establish a canonical edition.

## Emulator pin and configuration gate

Pinned package: official [DOSBox-X 2026.10.01 release](https://github.com/joncampbell123/dosbox-x/releases/tag/dosbox-x-v2026.10.01), published `2026-10-02T02:29:52Z` according to release API metadata retrieved for this proposal. `dosbox-x-vsbuild-win64-2026.10.01-portable.zip`, 44,575,207 bytes, has verified published SHA-256 `3c50915e63aefe35777d65ebb4391a06b4a2c2342e4c75cbee6bbda4c6916e33`. [Exact asset](https://github.com/joncampbell123/dosbox-x/releases/download/dosbox-x-v2026.10.01/dosbox-x-vsbuild-win64-2026.10.01-portable.zip). All 311 extracted files match the archive. Selected SDL2 executable SHA-256 is `a82adcfa7af2784566cfeff74c077cefed08505396db2948b7e6aa7d4cee0832`; the empty runtime reports 2026.10.01 commit b6abbd5. No system-wide installer or default-profile changes.

A complete 894-parameter private configuration has been generated from that package's full reference, applying the selected overrides below and additional restrictions in `reference-readiness.md`. The upstream [reference configuration](https://github.com/joncampbell123/dosbox-x/blob/master/dosbox-x.reference.full.conf) also identifies version 2026.10.01. Empty-runtime checks verify effective settings and the secure-mode lock; they do not prove an OS sandbox. The tracked preparation script records the exact overrides, including disabled MCP server, empty startup script, nine disabled serial ports, disabled VMware API, and silence for guest audio recording.

```ini
[sdl]
fullscreen=false
output=surface
autolock=false
clip_mouse_button=none
mapperfile=logs/mapper.map
[dosbox]
machine=svga_s3
memsize=64
captures=captures
[render]
frameskip=0
aspect=false
scaler=none
[cpu]
core=normal
cputype=auto
cycles=fixed 30000
[dos]
automount=false
automountall=false
automount drive directories=false
network redirector=false
startcmd=false
dos clipboard device enable=false
dos clipboard api=false
[ipx]
ipx=false
[ne2000]
ne2000=false
backend=none
[serial]
serial1=disabled
serial2=disabled
serial3=disabled
serial4=disabled
[parallel]
parallel1=disabled
parallel2=disabled
parallel3=disabled
parallel4=disabled
parallel5=disabled
parallel6=disabled
parallel7=disabled
parallel8=disabled
parallel9=disabled
[printer]
printer=false
[midi]
mpu401=none
mididevice=none
[config]
[autoexec]
```

Leave ordinary emulated audio hardware at pinned defaults for this first observation, record those settings, and do not invoke the original sound-setup program. MIDI passthrough is disabled. DOS clipboard APIs/device are explicitly disabled; secure mode also disables those communication paths. [Clipboard guide](https://dosbox-x.com/wiki/Guide:Clipboard-support-in-DOSBox%E2%80%90X)

`cycles=fixed 30000` is an experimental baseline, not measured original hardware speed or cycle-accurate emulation. If the host cannot maintain realtime, record the slowdown and stop timing claims; do not adjust speed within a measurement. [Official CPU guide](https://dosbox-x.com/wiki/Guide:CPU-settings-in-DOSBox%E2%80%90X)

## Approved supervisor launch and host release

The final workflow uses CLI startup commands and a host-controlled waiting gate, avoiding terminal typing through computer-use automation. Root runs from P:

```text
python -B scripts/reference/launch.py --open-approved-session --approval-note "User explicitly approved this scoped reference run: one <=20-minute CD GW.EXE/DOS4GW session in prepared run-001, approved observation/save scope only."
python -B scripts/reference/gate.py
```

The launch rechecks hashes, generates a fresh single-use eight-hex-digit marker name, refuses existing gate/marker/session records, and starts the visible SDL2 emulator with private stdout/stderr files. Its eight `-c` commands establish C with `-nocachedir`, mount copied CUE as D, list mounts, enable secure mode, probe refused E mount, switch to C/GAME and invoke our STARTGW.BAT. The batch waits without executing GW until the supervisor releases its marker. Directory caching must be disabled on C so host-created markers become visible; the official [DOSBox-X FAQ](https://dosbox-x.com/wiki/Home) documents per-drive `-nocachedir`. The full configuration hash is unchanged.

Root inspects the actual private log and the gate checker output. Only C/D/Z may exist; both reviewed paths, secure-mode success, refused E mount and the fresh waiting token must be present. The checker also verifies PID executable/creation time, the unchanged batch, session age, startup warnings and all original/media hashes. Then root may run:

```text
python -B scripts/reference/gate.py --release-approved-original
```

This records the inspected log hash before exclusively creating the marker. The batch consumes it and launches GW exactly once. No original command is run by launch.py before this release. `gate.py --abort` stops a waiting batch; after release, use the ordinary visible quit or terminate only the verified recorded emulator PID. The release/abort handshake was tested in hidden empty-drive fixtures using an ECHO sentinel, with no game files mounted or executed; see reference-gate-validation.json. No additional original reference session is authorized by these tests.

The following is the equivalent reviewed DOS sequence for audit, now supplied through CLI rather than typed:

Run from `P\private\reference-run\run-001` as the ordinary non-admin user, with the verified SDL2 executable's absolute path. Use:

```text
<verified-dosbox-x.exe> -conf dosbox-x.conf -noconfig -noautoexec -defaultmapper -nopromptfolder -savedir states -time-limit 1200 -showrt
```

These documented options select the private configuration, skip startup commands, use default mappings, direct state output, impose the session timeout and show emulation speed. Do not pass `-hostrun`, `-userconf`, a game filename or arbitrary command-line `-c` content. Do **not** add command-line `-securemode` here: it disables mounts before the two intended mounts can be established. [Official command-line options](https://dosbox-x.com/wiki/DOSBox%E2%80%90X%E2%80%99s-Command%E2%80%90Line-Options)

At the built-in `Z:` prompt, type only this reviewed sequence (the operator fills `P` with the exact absolute path):

```text
MOUNT C "P\private\reference-run\run-001\drive-c" -nocachedir
IMGMOUNT D "P\private\reference-run\run-001\media\GENEWARS.cue" -t iso
MOUNT
CONFIG -SECUREMODE
MOUNT E "."
C:
CD \GAME
```

Check the mount list before locking: only internal `Z:`, disposable `C:`, and copied CD `D:` may exist. After locking, the harmless `MOUNT E "."` probe must be refused; if it succeeds, close the emulator before starting the game. Confirm clipboard APIs/host launch integrations are disabled and no serial/Ethernet/IPX interface is enabled in effective startup settings/logs. `CONFIG -SECUREMODE` disables subsequent mounting, booting and configuration export; it persists until emulator restart. [Official DOSBox manual, CONFIG security behavior](https://www.dosbox.com/DOSBoxManual.html). Its usage is also present in DOSBox-X's official generated [shell documentation](https://dosbox-x.com/doxygen/html/shell_8cpp_source.html); verify the pinned runtime's response during the empty-emulator restriction check.

Only after those checks, launch `GW.EXE`. This is the explicit original-execution boundary. Verify `C:\GAME\GW.EXE` and `DOS4GW.EXE` hashes on the host immediately beforehand. Do not run `GENEWARS.EXE` (separate launcher hash `964103130ff485fbb375aae7c7f2fb12913376cbad24373cf8b12341c89b8c1a`) if direct launch refuses; that is a separately traceable executable and scope adjustment.

## Observation sequence and evidence

Use a monotonic host stopwatch and a private CSV/JSON action log. Every row includes run ID, candidate/tool/config hashes, wall offset in milliseconds, exact key or click coordinates, visible menu label, screenshot filename and uncertainty. Record framebuffer PNG dimensions from the file; do not infer resolution from the window size. Use the emulator's **Capture → Take screenshot** item or its default Windows host-key shortcut `F11+P`. These capture the current DOS display as PNG. [Official capture menu documentation](https://dosbox-x.com/wiki/DOSBox%E2%80%90X%E2%80%99s-Drop%E2%80%90Down-Menus)

1. **Startup:** start stopwatch at Enter after `GW.EXE`. Record first visible screen and time to stable menu; capture `01-menu`. Allow at most 120 seconds. Record every message and code if startup fails. Do not patch missing-install/CD checks or run bundled installers.
2. **Menu:** capture before input and log visible labels. Select the visible single-player/campaign entry and first currently selectable mission by its actual displayed label; no invented menu coordinates. Capture mission/briefing selection and note whether a saved profile is required. If no mission is available, capture that state and stop.
3. **Mission:** log Enter/click that starts the mission, then capture the first stable mission frame as `03-mission-start`. Move the cursor into a neutral part of the display before capturing. Record source screenshot dimensions, left panel boundary and any mode change. Do not claim the 128×480 panel is the entire frame or compare animated overlay pixels to the bare resource.
4. **Panel comparison:** capture a stationary frame with no selected entity overlay if the UI permits. Compare the static leftmost 128-pixel region with the current raw `GENEPANL.DAT` reconstruction, recording cropping, occlusion and palette effects. Report exact pixel comparison only after establishing the emulator's color expansion policy; normalized Rust RGB conversion is currently a presentation convention.
5. **Timing baseline:** from the first stable mission state, provide no gameplay input for 30 wall seconds; capture near offsets 0, 10, 20 and 30 seconds and transcribe any visible counters. Log observed animation/counter changes and the emulator realtime indicator. These are wall-time observations, not inferred simulation ticks. Pause/unpause once using only a manual-supported or visibly identified control; record the actual state and whether counters stop. Do not guess a key if no control is known.
6. **One command if supported:** select one visibly identified controllable unit and issue one manual-documented move to a nearby unobstructed point. Record selection and destination screen coordinates, visible path/response and wall delay. Do not turn screen displacement into world speed without a coordinate transform.
7. **Optional save/reload:** if a save UI is visible, create slot `REF001` in the disposable game tree, capture before/after, reload that slot and compare visible counters/locations. If absent or disabled, record unknown; do not overwrite existing original saves. Save success is not save-format parity.
8. **Exit:** use the visible quit/menu action. If hung, close only this emulator instance. Record end time, exit reason and output hashes. A second run requires a fresh disposable copy and a separately logged run ID; the first approval does not authorize a new variant or installer.

## Containment limits and cleanup

Only private copies are DOS-mounted; network adapters, serial paths, UNC redirector, clipboard, host START and printing are disabled, then mounts are locked. The emulator process still runs with the ordinary host user's OS permissions. These documented controls are **not a proof of an OS security sandbox** and do not eliminate emulator vulnerabilities or all host file access. No firewall/global policy is changed by this proposal. If stronger host containment is required, use an independently approved disposable VM with its network/clipboard/shares disabled; this DOSBox-only protocol does not provision or claim such a VM.

After exit, verify source archive and original private disc inputs remain unchanged using existing hashes; compare the disposable GAME before/after manifest, including any new executable files, and retain it privately. Keep screenshots, saves, logs and copied game files under ignored `private/`; publish only safe factual metadata. Do not automatically open generated executables or documents. Cleanup should first move the completed private run directory to a private archival name after resolving and checking its absolute path remains under `P\private\reference-run`; optional deletion is a separate explicit operator action. Never recursively delete a computed unverified path, original disc source, or source archive. No public upload/release is part of this run.

## Readiness and stops

Completed: scoped human approval, pinned portable download/digest verification and extraction, all 490 GAME file hashes, unchanged CUE/IMG copies, complete private configuration, empty-runtime effective-option/secure-mode checks and tested CLI host gate. See `reference-readiness.json`, `reference-readiness.md`, and `reference-gate-validation.json`. Original observations and interactive display/capture verification remain for the supervisor's session receipt. Any mismatch, extra mount, unknown-option warning affecting containment, failed secure-mode probe, unexpected host prompt, required installer/launcher or changed candidate hash stops the run. Preserve failure evidence and continue static/Rust work.
