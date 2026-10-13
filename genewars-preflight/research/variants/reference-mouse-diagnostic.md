# Run003 mouse diagnosis from pinned source

Root observed visible menu labels after the 19 installation aliases, but the host pointer moved while the guest brown cursor remained centered. This is an operator observation; no raw event trace establishes why motion was absent.

The official release [mouse implementation](https://github.com/joncampbell123/dosbox-x/blob/dosbox-x-v2026.10.01/src/ints/mouse.cpp) accumulates relative INT33 mickeys only while user_cursor_locked is true. The later mouse_emulation policy controls reported position without removing that condition. Thus changing locked to always alone cannot establish cumulative-mickey support while unlocked. A captured-window motion test is needed for this application.

The pinned [SDL host source](https://github.com/joncampbell123/dosbox-x/blob/dosbox-x-v2026.10.01/src/gui/sdlmain.cpp) sets relative mode and updates the capture menu checkmark/title suffix when capturing. Focus loss releases capture; normal window focus gain does not automatically recapture. raw_mouse_input is startup-only, defaults false, and selects the warp backend when false. No supported live raw-input toggle was found. A title suffix/checkmark establishes requested DOSBox capture, though the code does not check SDL_SetRelativeMouseMode's return value.

The [official guide](https://dosbox-x.com/wiki/Guide:Mouse-support-in-DOSBox%E2%80%90X) documents Ctrl-F10 and Main→Capture mouse. Root can verify the actual lock indicator, then test a drag within the client without an intervening focus transfer. The [SDL2 hint documentation](https://wiki.libsdl.org/SDL2/SDL_HINT_MOUSE_RELATIVE_MODE_WARP) corroborates warp versus raw selection. Injected absolute movement under capture remains unverified; native winit cursor events do not prove SDL2's relative path. No configuration, GUI, integration device or original execution was changed by this reviewer.

Source snapshots and hashes are private review inputs, pinned to dosbox-x-v2026.10.01. They are not proof of a reproducible binary build. Actual tool behavior and operator observations remain the final evidence; no mouse fix is claimed.
