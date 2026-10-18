# Victory result and campaign transition boundary

Observed against verified CD executable SHA-256 `4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba`.

The first victory does **not** immediately increment campaign progress.72ED0 sets runtime40 and calls2F810(receiver185DFC), which prepares the result widget. The recovered ordinary Enter scan1C creates packet21;1C120 dispatches it to82590. With campaign20 and victory40 set,82590 queues view1863DC and preserves the previous view in11697C.

Constructor8DCEB..8DD12 and mapped vtables1127E8/112808 resolve that view's entry thunk8DED0 to7DBC0. Its entry performs presentation services, clears runtimeflag4, sets modebit2, saves runtimeframe to118786, zeros runtimeframe and increments the selected player's progress byte6E7 with byte wrapping. No other player bytes change in the bounded entry comparison. This increment belongs to entering the victory view after Continue, not to a synthetic mission-completion shortcut.

The view's update thunk8DEB0→7DDD0 tests runtime input byte118779 bit1. It queues campaign view186F19 when that edge is present and progress is not24. Progress24 takes a separate ending sequence; this is evidence of a source branch, **not a verified campaign-length denominator**. Binding this flag to a native action must preserve the input producer; the branch alone does not prove the entire input route.70E40's type2 input path derives it from19CFB3; no second Enter mapping is claimed.

`result_transition.rs` exposes bounded plans for these numeric operations. `VictoryEntryPlan.service_calls` lists the required original presentation services in order; these are obligations, not services that the Rust planner has executed. The initial video lock depth0/successful presentation response is the admitted service contract. The full32-case source comparison uses explicit typed ports for presentation, compares all7220 playerbytes and runtime flags/mode/stage/frame/savedframe, and verifies the ordered call addresses.82590 continuation and7DDD0 nonfinal view routing are separately source executed. Reproduce with `python -B research/variants/result-transition-review.py`.

## Remaining required carried-state work

Leaving the victory view runs7DD40→72E50. It first sets selectedplayer0, then processes all four player records via80E90; afterwards it masks the300 two-byte entries at184D7C, calls80820 (RET) for each player, zeros frame and clears runtimebits2/4.80E90 removes live actors through80D40, clears runtime bytes1C..1F in active stored blueprints, invokes42B50 building cleanup, clears multiple mission counters/elimination/alliance fields and restores each record's player identity. This is an actual world cleanup boundary; keeping the whole old world or discarding its state arbitrarily is unsupported. Mode1 adds further cleanup and is outside the ordinary campaign admission.

Required next proof:80D40 actor removal and42B50 building cleanup over a captured completed mission world, followed by72E50 whole-world equality and next campaign-view initialization. Presentation assets/input acceptance, campaign state serialization and loading the next original map remain separate integration requirements. No mission or campaign transition is accepted by these numeric plans alone.

Follow-up: `research/variants/result-carry.md` documents the now-proven bounded72E50 carried-state projection on actual ordinary-command captures. It includes actor unlink/retirement and exact retained playerbytes, with nonisolated links and new-world integration still explicit boundaries.
