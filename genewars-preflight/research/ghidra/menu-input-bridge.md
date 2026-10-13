# Main-menu capture, hit testing, and activation

This extends `font-menu-bridge.md` for exact CD candidate SHA-256 `4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba`. The contract is restricted to its four flags-`0x1048` menu records with callback pointer zero. It does not generalize the shared UI manager to arbitrary controls.

The original list order is ID 0, 1, 3, 2. The normal three-button branch disables ID 1 and positions the others at (140,115), (140,186), and (140,257). Each uses animation zero from the exact IFTITLE database. Its first frame is normal frame zero; its last frame is selected frame two. All control callback pointers at record+`0xE` and tooltip pointers at +`0x22` are zero. Scene input callback `0x7C870` calls manager `0xA87D0`, stores its signed16 result at `0x1169A2`, and acts on a returned control only when shared input mask `0x19FF70` contains `0x08`. ID 0 additionally requires the enabled-state gate documented in `campaign-entry-bridge.md`.

## Opaque hit footprint

Flags `0x40` bypass the control's rectangular hit-test branch. Routine `0xA86D0` always fetches the animation's **first frame**, including while selected artwork is being displayed. Frame hit routine `0x98C40` visits components from last to first and returns the first opaque component index, or -1. The menu can therefore use the boolean union of the normal components' opacity masks.

| Normal sprite | Component position | Raster dimensions |
|---|---|---|
| 1 | (0,0) | 116×75 |
| 2 | (116,11) | 120×50 |
| 3 | (236,3) | 110×71 |

All three actual component flags are zero. Generic frame hit testing mirrors local x with `width-1-x` for component bit zero, and local y with `height-1-y` for bit one. The bounded menu implementation need only admit these zero-flag components. `0x98B60` first requires `0 <= local_x < sprite_width` and `0 <= local_y < sprite_height`. It treats positive RLE literal runs as opaque without inspecting their color bytes; literal index zero is opaque. Negative runs and the transparent remainder after a row terminator do not hit.

The union's bounding extent is 346×75. At the first button its outer coordinates are `[140,486) × [115,190)`, but transparent interior pixels and corners also miss. Normal-mask opaque count is 18,827, mask SHA-256 `a9a7ad36a9255a84d90c9d9e4d70b9a17f3f9b6f7273ac7f1d018d3c6295814e`. Local (173,37) and (116,11) hit; (0,0), (345,74), and (236,3) miss. The mask is stored privately, with metadata only in the receipt.

Manager `0xA87D0` clears origin words `0x1A011E` and `0x1A0120` at the start of each call. These menu flags do not modify them. Packet coordinates are signed16 words at pointer `0x118970` offsets zero and two. Native host coordinates require the separately proven viewport mapping before narrowing to those coordinates.

## Restricted packet state machine

Packet byte+8 bits zero/one/two are left press, release, and packet-held state; byte+9 bit zero is right packet-held state. Their actual producer/latch contract is in `input-packet-bridge.md`. The following behavior assumes menu-owned capture and its reachable countdown values zero or one. The capture global is `0x19FF88`, capture-mask byte record+`0x26`, selected bit record+`0x27`, and countdown is u16 `0x1A011C`.

| State and input | Returned ID | Input-mask write | State change |
|---|---|---|---|
| Uncaptured, left press and first enabled normal opaque hit | Hit ID | 0 | Capture hit, mask8, selected=true, countdown1 |
| Uncaptured, no qualifying hit/press | -1 | No write | Remain uncaptured |
| Captured, left release inside captured normal opaque pixels | Captured ID | 8 | Clear capture, mask, and selected appearance |
| Captured, left release outside captured normal opaque pixels | -1 | 8 | Same clear |
| Captured, no release, hit state unchanged | -1 | Held mask | Preserve capture; selected/countdown reflect hit |
| Captured, no release, pointer leaves | Captured ID | 1 | Preserve capture; selected=false, countdown0 |
| Captured, no release, pointer enters | Captured ID | Held mask | Preserve capture; selected=true, countdown1 |

Held mask is `(byte8 bit2 ? 2 : 0) | (byte9 bit0 ? 4 : 0)`. It is written on **every captured no-release call**, before testing whether hit changed. Leaving overwrites it with one. Uncaptured no-event calls leave the previous global mask untouched, so an API should represent absence of a write explicitly. Return ID and input mask must be considered together; returning ID on press or hover change does not activate the menu action.

An uncaptured packet containing both press and release captures without activating because the uncaptured branch wins. A captured packet containing both bits takes the release branch. A new press while already captured cannot transfer capture to another control. Right press/release alone cannot capture or release these records because their admitted capture mask is eight. A release clears selected state but leaves the countdown global unchanged; its old value is irrelevant after capture clears. Selected artwork uses frame two when the captured hit is true; it returns to normal frame zero when the pointer leaves, without changing the normal-frame hit footprint.

No per-control callback is scheduled for these zero callback pointers. The scene's input callback processes the manager result, while draw callback `0x7CA70` invokes `0xA85D0`, which passes selected bit zero into `0xA84D0` and chooses the normal/selected frame. Input production precedes the scene input callback through `0x70B00`; these static calls do not establish a wallclock cadence.

Run `python -B scripts/ghidra/validate_menu_input_bridge.py`. It checks six instruction contracts, raw record flags/callbacks, all 25,950 normal-frame pixel hit queries against decoded opacity, 8,192 bounded packet cases, and 17 explicit transition vectors. `menu-input-evidence.json` is the safe receipt. Proprietary mask bytes remain at `private/ghidra/menu-input/normal-hit-mask.bin`; no original executable is run.
