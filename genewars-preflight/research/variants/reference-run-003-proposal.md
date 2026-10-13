# Prepared run003: English installation aliases and mouse diagnostic

Attempt001 is preserved under `private/reference-run/run-003/attempts/attempt001-unreleased-ui`; it never released an original. Fresh attempt002 uses the independently authored input-free YIELD.COM gate after empty probe002 passed actual target activation/capture, release/sentinel and exit0 in 38.343 seconds. See `reference-input-free-probe-result.json` and `reference-run-003-yield-integration.json`. This supersedes the earlier tight IF/GOTO wait. The same 510 game-tree baseline files remain; launch adds the generated 27-byte helper and nonce batch outside GAME. Their hashes are checked before release and the helper is audited after exit. The helper polls BIOS ticks for at most65535 comparisons per call; it generates no keyboard input and is not a calibrated delay. The 1200-second host watchdog remains authoritative.

Ready for root-supervised execution under the latest human authorization recorded verbatim in `reference-run-003-approval.json`. No new question is required. Preparation, the actual default launch dry-run and seven mocked refusal cases started no emulator or original. Run001 and run002 remain preserved. The provisional CD analysis candidate is not established canonical retail behavior.

The fresh private drive has 510 baseline files: 490 unchanged CD GAME files (186,368,166 bytes), the same generated 126-byte config.ini, and 19 byte-identical English aliases (19,202,716 bytes). The exact source IDs, destinations, lengths and hashes are separately recorded in readiness/before-files metadata and `research/ghidra/english-install-evidence.json` (SHA256 `541c851fe27e78c78b2bebe83b1c33e848fc39c40aa82ef014c1e16a02f47970`). Independent review checked five instruction listings, three installer lists, mapped format strings and all 19 manifest tuples; destinations are case-insensitively collision-free. Entry592 becomes DATA/TEXT.DAT; entries420..437 become SPEECH/basename. Nested English sources remain intact. No installer was run, and no sound.ini was invented.

GW.EXE remains SHA256 `4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba`; DOS4GW.EXE remains `b8265123ac8a189637448618409ef3ecd2e9f3e1a47062c685a02240f688dec1`. Config.ini remains `ff0c1921640bb81cd0b73fc4a995316ea9ff1b8d60ed288e5d52a1be4e7c0d5f`: CD=D:\GAME\, InstallDrive=C:\GAME\, Language=ENGLISH and all five categories All, using the same quoted CRLF writer format. The complete alias-set binding is `55c352cd19e90344f4c07c97b3dc65b70c437ae763a6555c78b42d869ca3f0f6`.

The only emulator configuration difference from run002 is `[sdl] mouse_emulation=always` instead of locked. The complete 894-parameter, 20,796-byte file is SHA256 `a3275d1cf47a7305764698352de4ed6f03fef2ecea192d7f4c45fdd7cb17107b`. The [official mouse guide](https://dosbox-x.com/wiki/Guide:Mouse-support-in-DOSBox%E2%80%90X) documents locked as emulating while locked and always as emulating every time. Autolock remains false; CPU integration device and PNP remain false. This is a diagnostic input setting, with no host security or global setting change. It is not yet evidence that the guest cursor will move. English flattening and mouse emulation are two independent hypotheses; observe and report each separately.

The same official portable DOSBox-X 2026.10.01 SDL2 executable is pinned at `a82adcfa7af2784566cfeff74c077cefed08505396db2948b7e6aa7d4cee0832`. Archive, original and private CUE/IMG, tool members and full private config are rehashed before launch and release. Only private drive-c and copied disc image are mounted. Networking, host command/clipboard/integration, serial/parallel, printing and MIDI policies remain disabled. These are emulator restrictions, not an OS sandbox: the process retains host-user rights and still enumerates microphones despite guest recording silence. Empty probe002 checked C/Z, secure-mode entry, refused E mount and mouse_emulation always using this exact config. Its C-only scope does not prove the original session's C/D/Z mounts; fresh runtime log verification remains a release prerequisite.

From the project directory, root may verify with `python -B scripts/reference/run003.py prepare` and print the nonexecuting command with `python -B scripts/reference/run003.py launch`. The actual supervised launch is:

```text
python -B scripts/reference/run003.py launch --open-approved-session --approval-note "Latest human instruction authorizes necessary scoped reruns with full access; supervised exact CD GW.EXE/DOS4GW, <=1200 seconds, 19 installer-proven English aliases, mouse_emulation=always; no installer or patch."
```

Use the normal full-access tool review for the visible interactive process and keep its parent execution session alive. Fresh stdout/stderr files avoid inherited tool pipes. SW_SHOWNORMAL is explicit. The parent polls its exact child and enforces 1,200 host seconds including gate wait, then writes actual PID, exit code, end UTC and timeout status. The nine CLI startup built-ins mount C, mount the copied CUE as D, list mounts, enable secure mode, attempt a refused E mount, query mouse_emulation, select C, CD GAME and enter STARTGW.BAT. A fresh exclusive nonce marker gates GW.EXE; no DOS shell typing through UI automation is required.

Before release, root must see and reliably control the actual DOSBox window and run:

```text
python -B scripts/reference/run003.py gate
python -B scripts/reference/run003.py gate --release-approved-original
```

The gate checks process image/creation time, exact C/D/Z mounts, secure-mode entry and E refusal, waiting nonce, no configuration warnings, log query value always, all staged hashes and human bindings. It then writes a host-release receipt and exclusive marker. If these checks or visible UI fail, use `gate --abort` before release. Preserve the exited unreleased attempt with the guarded archive action before a necessary retry; it rejects released attempts.

Observe startup and menu labels, then guest cursor response separately. If usable, enter the first available single-player mission, capture its panel, record a short bounded timing observation and optionally save/reload once. Stop on install/CD rejection, crash, required binary/config change or host bound. No installer, patcher, alternate variant, network play or upload is included. Keep raw logs, screenshots and saves private. Distinguish host window captures from raw framebuffer dumps, and operator termination from a game failure or natural exit. If normal exit fails, verify the recorded executable and creation time before terminating only that PID; the parent's exact-child timeout remains active.

After the parent writes final-process.json and root writes actual supervisor observations, use:

```text
python -B scripts/reference/run003.py finish --supervisor-observations private/reference-run/run-003/logs/supervisor-observations.json
```

The generalized checker binds all 510 before-files, reports original/generated/copied inputs separately, preserves actual added/changed/removed files and nonzero/negative exits, rechecks media/tools/archive/config, and hashes captures without inferring their contents. Missing optional observations remain unknown. Four synthetic cases, including 19 copied aliases and exit15, passed. Checkers grant no execution authorization; they record and enforce the existing human scope.
