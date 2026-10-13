# Run005 outcome

The fresh supervised run tested a specific new mouse hypothesis: `mouse_emulation=never`, `raw_mouse_input=false`, and `autolock=false`. The official DOSBox-X guide and pinned emulator source show that this route maps host absolute position into the guest driver position. The runtime startup log confirmed both changed parameters before the original was released.

The original displayed the animated introduction and labelled main menu. A click aimed at Single Player entered Options again. The brown guest cursor remained near the Options center while the host cursor moved. Escape did not visibly exit Options; an in-client drag toward the visible OK button also left the guest cursor stationary. This hypothesis did not resolve automated guest motion. No mission, gameplay, save/reload, raw framebuffer, or gameplay timing comparison was observed.

Some captures while the emulator was occluded showed the foreground app instead. The supervisor reactivated the returned DOSBox window and re-observed it before subsequent input. The retained Options image is a host-window capture. It is not a raw game framebuffer.

PID36328 was released at 2026-10-13T08:47:19.642552Z and stopped after exact executable and process creation-time verification. Exit15 is operator SIGTERM, not a game failure code. The process lifetime was514.108 seconds without watchdog timeout. The session is closed.

Integrity passed:489 of490 original files unchanged, with disposable GW.ERR alone changed. All19 English aliases, generated configuration, game executables, source archive, original/copied media and emulator inputs remained unchanged. Three additions were ADDRESS.TXT, the generated gate batch and input-free YIELD.COM. No files were removed. The timestamped private post-run receipt and `reference-run-005-result.json` preserve the details.

New static work indicates the game consumes relative mickey counters rather than callback absolute coordinates. That distinction needs a separate empty emulator input probe before another original attempt; this run itself does not prove the cause or establish physical-mouse incompatibility.
