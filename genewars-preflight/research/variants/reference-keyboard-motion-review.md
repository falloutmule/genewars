# Bounded built-in keyboard-to-mouse search

No supported relative-motion generator was found in the pinned DOSBox-X2026.10.01 keyboard mapper or menu implementation. This is a bounded negative result, not a claim about every possible emulator feature.

The official release [mapper source](https://github.com/joncampbell123/dosbox-x/blob/dosbox-x-v2026.10.01/src/gui/sdl_mapper.cpp) exposes mouse L/M/R events. Their handlers call mouse-button press/release only; there is no mouse-axis event or Mouse_CursorMoved call in this module. Its joystick-axis events are joystick outputs. The [menu source](https://github.com/joncampbell123/dosbox-x/blob/dosbox-x-v2026.10.01/src/gui/menu.cpp) and previously reviewed SDL host expose capture, sensitivity, axis reversal and wheel-to-key options, without a keyboard-to-relative-motion action.

The [official mapper overview](https://dosbox-x.com/wiki/Home) describes binding keys and joystick events. The [mouse guide](https://dosbox-x.com/wiki/Guide:Mouse-support-in-DOSBox%E2%80%90X) documents capture and mouse-wheel-to-key conversion; neither provides a relative-motion keyboard recipe. No such recipe was inferred from the inverse mapping.

Pinned private snapshots are hashed in the adjacent JSON. No mapper/config/UI change, helper injection, DOSBox patch or new original execution was performed. This search supplies no concrete basis for another reference attempt. Existing unresolved injected motion remains distinct from physical mouse support.
