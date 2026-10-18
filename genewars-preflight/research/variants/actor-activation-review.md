# Independent actor activation review

The native Rust planner passes 262,144 placement cases (all 65,536 seeds at four boundary centers) and 144,187 activation/admission cases. The comparison covers 20,860,319 output bytes, including complete actor and blueprint before/after images, raw link plans, candidate retry order and rejection results. See `actor-activation-review.json` for exact source, harness and instruction pins. No original executable ran.

Review corrected three substantive issues before acceptance: the sine lookup must use the shifted angle index; the byte+0x1B write targets the blueprint, preserving the actor frame; and a nonnull old map head requires a resolved record identity. Eligible records are now restricted to the proved type 3 path, with actor-ID and partition-slot agreement checked. The safe admission rejects inconsistent old-head identities rather than copying an unverified raw reference.

The native harness and independent Python oracle were authored in the Sol review lane; root completed the resolved-head cases after that lane hit model capacity. Both source files are retained alongside this note. Errors leave every caller input unchanged. Extra environment entries after the first accepted candidate are not consumed.

This is a per-slot plan with explicit live seed, terrain/blocker observations and resolved-head inputs. It does not resolve actual world arrays, apply the five-slot player counter increment, derive the landing seed, run associated-type5 placement, or establish an original first controllable frame. The blueprint/resource handle must be bound to the selected player's actual metadata before world integration.
