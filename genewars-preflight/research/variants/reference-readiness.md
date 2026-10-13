# Scoped reference run completed at configuration check

Observed on 2026-10-12 local (2026-10-13 UTC): the human approved the scoped reference run and root supervised one execution of the exact CD GW.EXE through the verified DOS gate. It reported `Can't read config.ini!` and `Please install GENEWARS properly to play!`, then returned. No menu, mission, gameplay inputs, save/reload or framebuffer capture occurred. The approval has been used; no additional original or installer execution is authorized. The result and input/output hashes are in `reference-run-result.json`; readiness now truthfully records `original_execution=true`.

Root released PID 31916 at 00:58:39.250972 UTC; DOSBox-X exited with code 0 at 00:58:41.747427 UTC. The 2.496455-second interval includes emulator shutdown and is an upper bound rather than gameplay timing. The emulator's zero exit status does not establish successful game startup. BIOS INT15 unknown-call warnings were also logged; their causal role is unknown.

Post-run verification found 489 of the 490 staged GAME files unchanged. Only GAME/GW.ERR changed, from 45 to 47 bytes. C:\\ADDRESS.TXT is a new 24-byte output; STARTGW.BAT is the expected gate created by our own script. No files were removed. The source archive, original and copied disc media, complete emulator package and configuration still match their reviewed hashes. All private logs and outputs remain preserved. The two earlier unreleased attempts were preserved separately with exact hashes, and neither started an original. `python -B scripts/reference/finish.py` performs the post-run read-only checks; the pre-execution `prepare.py` verifier correctly rejects the now-modified runtime tree.

Before this approved execution, hidden empty DOSBox-X fixtures validated only built-in commands and the host-release handshake. Those checks remain separate evidence, retained in `reference-gate-validation.json` and readiness metadata. The preparation details below document the inputs and bounded procedure actually reviewed.

## Verified inputs and commands

The official [DOSBox-X 2026.10.01 portable archive](https://github.com/joncampbell123/dosbox-x/releases/download/dosbox-x-v2026.10.01/dosbox-x-vsbuild-win64-2026.10.01-portable.zip) is private under `private/tools/dosbox-x/`. Its 44,575,207 bytes match published SHA-256 `3c50915e63aefe35777d65ebb4391a06b4a2c2342e4c75cbee6bbda4c6916e33`. All 311 files extracted without executing package scripts match their ZIP members. The SDL2 executable is 25,836,544 bytes, SHA-256 `a82adcfa7af2784566cfeff74c077cefed08505396db2948b7e6aa7d4cee0832`; its startup log reports version 2026.10.01 commit b6abbd5. The packaged GPL v2 COPYING remains private with the tool; no tool code/configuration is redistributed with the reconstruction.

The established SourceReader revalidated the original archive and private disc, then read the targeted 490-file CD GAME tree (186,368,166 bytes). Every staged file matches its existing manifest hash; unexpected extra/missing files are rejected. `private/reference-run/run-001/drive-c/GAME/GW.EXE` remains SHA-256 `4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba`, and bundled DOS4GW.EXE remains `b8265123ac8a189637448618409ef3ecd2e9f3e1a47062c685a02240f688dec1`. The copied 630,947,520-byte IMG and 238-byte CUE match their original hashes. Before-files metadata is private under `run-001/logs/`.

The complete private configuration pins 894 parameters from the selected package's full reference. SHA-256: `9dba4b2ff89fa91a891cff667616f23b1562ba909ec3ed6f98a143870eb58f78`. Empty executable sections are `[config]`, `[autoexec]`, `[4dos]` and `[devices]`; startup JavaScript and boot/reset binaries are blank. Networking, automatic mounts, clipboard APIs/device, VMware API, host START translation, MCP server, nine serial ports, nine parallel ports, printing and MIDI passthrough are disabled. Guest recording source is explicitly silence. Config/data/source hashes are checked again by the launch script. No global tool configuration was changed.

From the project directory:

```text
python -B scripts/reference/prepare.py --stage   # local staging/verification, no execution
python -B scripts/reference/prepare.py           # verification only
python -B scripts/reference/launch.py            # print command only; no process
python -B scripts/reference/launch.py --validate-empty  # hidden built-in-only validation
```

`prepare.py --download` obtains the same pinned official ZIP if absent and implies staging; normal network approval applies. Extraction rejects traversal, symlinks, reparse points and filename collisions. It refuses changed cached files, unexpected C: contents and staging after a recorded session approval. It does not clean up or alter source files.

The default launch dry-run supplies the exact executable/config paths and working directory. Its argument list contains `-conf`, `-noconfig`, `-noautoexec`, `-defaultmapper`, `-nopromptfolder`, `-savedir states`, `-time-limit 1200`, and `-showrt`. There is no game filename, startup original command or hostrun switch. These flags are documented in the official [command-line reference](https://dosbox-x.com/wiki/DOSBox%E2%80%90X%E2%80%99s-Command%E2%80%90Line-Options).

The approved invocation used `launch.py --open-approved-session --approval-note "<record of the actual human approval and scope>"`. Eight reviewed CLI commands establish the private C: and read-only image D:, log the mount table, enter secure mode, test refusal of E:, and enter STARTGW.BAT's fresh-marker waiting loop. Root inspected the runtime evidence and actual visible window before `gate.py --release-approved-original` signaled the single-use marker. The interactive child explicitly uses SW_SHOWNORMAL and the launcher remains alive to poll that exact child up to 1,200 seconds, with stdout/stderr and final exit status captured privately. No installer or alternate launcher was used.

## Actual empty-runtime checks and limits

Three empty validation batches passed 15 effective setting queries. Each observed only internal Z:, entered secure mode, refused `MOUNT E .`, reached EXIT, and returned zero with no rejected/unknown configuration values. The checked settings cover automount, network redirector, START, VMware, both clipboard interfaces, MCP server, NE2000/its backend, IPX, serial/parallel first ports, printer, MIDI and silence recording source. The pinned configuration also disables the remaining serial/parallel ports. The earlier overlong command sequence stopped processing before its lock probe; final validation limits each batch to ten built-in commands and verifies the final lock/refusal/EXIT rather than trusting process return code.

The empty checks alone do not establish interactive display, CD mounting or game startup. The subsequent approved run observed its mounted CD, visible waiting window and DOS/4GW startup only; save behavior, gameplay CPU timing and framebuffer fidelity remain unknown. CD multi-track mounting follows the official [image guide](https://dosbox-x.com/wiki/Guide:Managing-image-files-in-DOSBox%E2%80%90X); fixed 30000 cycles is an experimental baseline, not original hardware speed.

DOSBox-X still enumerates/selects a host microphone at startup even with guest recording source `silence`. No guest capture was requested. Silence is an emulated input policy, not proof that the process cannot reach the host microphone. Its logs also include benign dummy-display/taskbar/codepage warnings. Documented emulator controls are not an OS sandbox; the process retains the user's host permissions. A disposable VM would require a separately reviewed setup. No system firewall policy was changed. Private captures and runtime outputs must remain private.
