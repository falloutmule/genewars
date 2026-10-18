# Run004 raw relative mouse diagnostic

Fresh private staging contains the same 490 exact CD GAME files, generated 126-byte config.ini and 19 installer-proven English aliases. Run003 and both its attempts remain preserved. The candidate is provisional; neither installer nor patch runs. Existing direct human authorization explicitly covers necessary scoped reruns, recorded in reference-run-004-approval.json. Root alone launches, supervises and releases.

The sole emulator delta versus run003 is `[sdl] raw_mouse_input=true`. The packaged full reference omits this SDL2-only property; the pinned official release [source](https://github.com/joncampbell123/dosbox-x/blob/dosbox-x-v2026.10.01/src/gui/sdlmain.cpp) explicitly registers it startup-only with default false and selects the raw backend instead of mouse warping. The config contains 895 parameters, 20819 bytes, SHA256 `101d258bd061ca9f629637f7c1746426416b6b0155b7e16b968667ca76450baa`. Intro remains All, mouse_emulation remains always, autolock false and integration devices/network/host policies remain unchanged. This changes an input backend, not guest bytes or an OS security setting. Raw input may reject injected absolute movement too; no fix is claimed.

```text
python -B scripts/reference/run004.py launch
python -B scripts/reference/run004.py launch --open-approved-session --approval-note "Existing latest human full-access instruction authorizes this necessary scoped rerun; run004 tests raw_mouse_input=true with exact CD inputs and verified input-free gate, root supervised <=1200 host seconds."
python -B scripts/reference/run004.py gate
python -B scripts/reference/run004.py gate --release-approved-original
```

The first command is nonexecuting. Keep the actual launch's parent session alive. The same input-free helper is generated outside GAME and pinned to the successful empty probe. Fresh nonce markers, exact PID/image/creation time, C/D/Z mounts, secure-mode entry/refused E mount and effective always/raw=true queries must pass. There are ten startup commands, within the already reviewed command bound. Root must activate/capture the actual waiting window before release. No original autoruns. The unchanged host/emulator 1200-second cap includes gate wait.

Observe intro, labels, requested capture indicator and then guest cursor displacement independently. The documented capture control is necessary for cumulative mickeys; requested capture does not prove SDL success. If movement permits, observe first mission and optional brief save/reload/timing within the same bound. Do not infer outcomes from window existence or exitcode. Capture files are host-window observations unless a raw framebuffer method is explicitly used. No mission bytes or timings are presumed.

Abort an unreleased gate with `python -B scripts/reference/run004.py gate --abort`. Once released, root may use normal close, then verify exact PID/image/start before bounded termination if needed; preserve actual exit/end. Finish only after the final-process receipt with `python -B scripts/reference/run004.py finish --supervisor-observations <private-run004-json>`. The generalized checker inventories original/generated/copied inputs, all changes/additions/removals and helper/media/tool immutability. Preserve all private outputs and prior runs; cleanup here means no restaging or deletion of used evidence. DOSBox controls are not an OS sandbox; inherited host-user rights remain a limitation.
