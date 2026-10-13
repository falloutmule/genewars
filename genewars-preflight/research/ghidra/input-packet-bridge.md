# Mode-two mouse packets and custom arrow capture

`input-evidence.json` and `scripts/ghidra/validate_input_bridge.py` cover static provisional CD candidate SHA-256 `4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba`. They check six private instruction listings, all eight old-held/press-latch/current-held combinations, and 2,304 synthetic capture cases. These checks do not execute the original program.

Mouse initialization `0xA3164` registers DOS INT33 function0C with mask0x7F and callback `0xA3524`. The callback's event mask2 sets left-held byte `0x19CFB6` and press latch `0x19CFB3`; mask4 clears left-held. Mask8 sets right-held `0x19CFB7` and latch `0x19CFB4`; mask0x10 clears right-held. The [FreeDOS mouse project's interface documentation](https://github.com/FDOS/mouse/blob/master/int33.lst) corroborates those physical left/right event assignments. No driver source was copied. The first pending press also records coordinates; later presses while that latch remains set do not replace them. Release-coordinate latches are separate.

Packet producer `0x70E40` branches on the current player's mode byte; mode2 constructs mouse packets. It copies the low16 words of current x/y globals `0x19CF9C`/`0x19CFA0` into packet storage `0x118771`/`0x118773`. Consumers interpret these as signed16. Global `0x118970` points at `0x118771` after startup.

Packet byte+8 bits0/1/2 represent left press event/release event/packet-held state. Bits6/7 represent right press/release; byte+9 bit0 stores right packet-held state. For each button, old packet-held state, latched press, and current physical-held state determine the next packet:

| Condition, in priority order | Press | Release | Next packet-held |
|---|---:|---:|---:|
| Press latch set | 1 | old packet-held | 1 |
| No press latch; physical-held | 0 | 0 | 1 |
| No press latch; physically released | 0 | old packet-held | 0 |

The producer consumes the press latches and clears left/right latches again on its common return path. The packet's event bits persist until the next producer update; the generic UI manager does not consume or zero them. A press and release occurring between producer polls therefore yield a press packet first, then a release packet on the following poll. Re-pressing while old packet-held is set can yield both event bits in one packet. This is more specific than a conventional instantaneous button-edge adapter.

Caller `0x70B00` invokes producer `0x70E40` at `0x70B78`, then invokes the current screen's input callback through its vtable+8 at `0x70BCC`. This establishes order within that call, without establishing a wallclock poll rate. Reset helper `0x715A0` clears physical-held and press-latch globals for three buttons and the keyboard event byte; its observed caller `0x1BE50` is a display/reset path, not a per-poll reset. Callback movement uses mickey integration, scaling, and clamping; those mechanics remain outside this adapter contract.

For the two actual controls in `ui-control-bridge.md` (flags0x48, IDs27/28), initial delay byte `0x10AAFD` is1. Capture manager `0xA87D0` zero-extends it into u16 countdown `0x1A011C`. With no capture, packet bit0 plus the first enabled opaque hit in original list order stores capture and selected appearance and returns signed16 **-2**. Simultaneous press/release bits on that initial packet do not immediately activate: the uncaptured branch wins for that call. Packet bit6 cannot capture these two controls because mask16 is absent from flags0x48.

While captured with countdown<=1, packet bit1 activates the captured ID and clears capture-mask and draw-state even outside its pixels. A different control under the cursor cannot replace the captured control. With no matching release bit, the function returns signed16 **-1** and preserves capture and selected appearance. Thus -2 denotes newly captured, not every pending poll. Uncaptured no-hit also returns-1.

The original's countdown>1 branch decrements each manager call and, when it reaches1, emits the captured event and clears capture/mask without testing release. That behavior is outside the restricted initial-delay1 port. Other control flag combinations, hover/callback side effects, original async races, and the outer input loop's timing remain outside scope.

Root's `CustomArrowCapture` implementation was read directly and matches this restricted contract: bit0 capture, bit1 release, capture priority, -2 only at capture start, -1 while pending, and outside release. Its caller remains responsible for supplying the first opaque hit from the actual list/frame composition. The text-selection primitive's flag `0x19CFB6` is now established as the left physical-held byte; its numeric branch remains distinct from packet capture.
