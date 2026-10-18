# Campaign carried-state teardown — bounded72E50

Observed against the verified QA BETA V5 CD. `result_carry::plan_campaign_carry` produces transition data, not an updated playable old world. The source clears each player's building bookkeeping but does not retire the old building pool; next-world loading is therefore a required separate step.

The mode0 projection follows four original player passes. For each listed actor,80D40 clears the active-link flag and unlinks the actor through8ACC0, then80360 clears the associated blueprint's low two runtime flags, shifts the signed-byte actor list, clears the actor's C4 field and retires its kind. Active stored blueprint runtime bytes1C..1F are zeroed.42B50 clears player4A1..619 while preserving owner byte4A1.80E90 clears its other mission counters, elimination and alliance fields, and restores player identity. Progress6E7 and retained genetic blueprint data remain source-preserved.

72E50 also masks each of300 external two-byte entries at184D7C to low3 bits plusFF, clears runtimebits2/4, and zeros frame. The surrounding victory-exit7DD40 selects player0 before this call and performs additional presentation cleanup afterwards. These surrounding operations must not be assumed to have occurred merely because a carry projection was returned.

Reproduce: `python -B research/variants/result-carry-review.py`.

Two tests use the exact ordinary-command worlds1289/1489 reproduced by the outcome-adapter probe. Literal72E50/80E90/80D40/80360/42B50/8ACC0 matches all7220 playerbytes,4000 actorbytes,229376 mapbytes,600 external-record bytes and flags/frame. Actor blueprint pointers are restored from captured logical bindings then normalized for comparison. Each world removes/unlinks two actors. The external record table is a separately supplied synthetic metadata fixture, not a claimed capture of the original process. Standard memset/memmove are explicit interpreter primitives, pinned to their source listings; selection-service25150 is returned as a required native UI action.

Admission is deliberately bounded: ordinary mode0, explicit selected player0..3, valid four player and bookkeeping-owner identities, signed actor counts0..5, valid owner-local blueprint bindings, and isolated actor cell heads. Unsupported mode, stale bindings or nonisolated links return errors before any mutation of inputs. Two explicit rejection checks protect mode1 and nonisolated links. Further general link handling should reuse the shared entity registry rather than duplicate a second world implementation.

Remaining: consume the selected-actor UI reset, bind the external record table, implement nonisolated links as required, complete victory-exit presentation and initialize the next original world using retained campaign state. Neither supplied fixture is a completed mission, and this does not count as a campaign transition or mission acceptance.

Focused ownership refinement compares source selectedplayer0 and1 for both captures: the selected-actor UI reset occurs only for its owning selected player; carried players and map remain identical. The API takes selected_player explicitly rather than encoding the first mission owner.
