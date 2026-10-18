# Mission outcomes — verified CD72ED0

Reference executable SHA-256: `4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba` (GENEWARS QA BETA V5). No retail equivalence or completed mission acceptance is claimed.

Observed: `mission_outcome::evaluate` preserves the original ordered player scan and enclosing eligibility predicates. The selected progress0 player's unsigned completed-selector6 count at606 reaches victory at3 only inside those gates. Other recovered rules remain shared: progress14/20 timer/limit checks, configured funds/terrain thresholds, completed terrain scoring, and last survivor. Player elimination byte6E3 updates follow source eligibility or the lazy805E0 capability predicate. These are original predicates, not injected mission-specific test objects.

The caller supplies exact live player records and explicit runtime context, and commits returned player records and flags atomically only after resolving the result request. `Running`, `Won` and `Lost` are separate from invalid context, capability errors and unsupported external branches. Existing flags80 preserve the original loss latch; flags40 cause an explicit already-terminal boundary because this callback cannot recover the winner identity after its early return. A caller that already holds a terminal result should retain it and stop evaluating.

| Context field | Exact source |
|---|---|
| flags, mode | runtime bytes118770,118968 |
| active, selected | runtime bytes11878C,11873D |
| frame | runtime dword11877E |
| mission_counter | backing1EC2 dword, global11AC22 |
| target_count | backing1EA6 unsigned word, global11AC06 |
| mission_limit | backing1C7C unsigned word, global11A9DC |
| mission_timer | backing355C unsigned word, global11C2BC |
| terrain_count, field_34b9 | backing34B8,34B9 bytes |
| terrain_limit | backing1EAA signed dword, global11AC0A |

In ordinary mode0, selected victory sets flags40 and returns `EnterVictoryModule185DFC`, the original2F810 request at7320F. Another winner or the selected player's elimination sets flags80; this callback issues no mode0 loss-module request. Campaign progress advancement, result-screen controls and next-level initialization are separate and are not inferred here. Mode1 terminal formatting/reveal/terrain/UI callbacks remain unsupported, as does deferred7F3B0 when the frame reaches a nonzero mission counter.

Reproduce: `python -B research/variants/mission-outcome-review.py`.

Observed proof:640 cases execute literal72ED0 and8AB30 against four complete player records, source flags and lazy capability query order. Ordinary terminal cases run through the full return;2F810 is captured as a result-request port.805E0 returns a supplied boolean or an explicit error boundary; its actual reconstruction and actor lookup have their existing independent `mission_status` proofs. The output compares all7220 playerbytes, flags, winner and request classification. Cases cover signed field limits, high unsigned completed counts, progress selectors, survivor ordering, deferred landing, terminal latch, mode1 boundaries and capability failure. Results:114 running,48 won,188 lost,100 already-terminal boundaries,17 deferred boundaries,160 mode1 boundaries and13 capability errors. Forty cases focus on the first campaign rule and enclosing gates; these are predicate fixtures, not ordinary-command mission acceptance.

No original program ran. No campaign transition or native interactive outcome acceptance is established by this numeric proof. Root owns host/world integration; startup and result progression remain with the campaign-contract lane.

Live adapter follow-up: `python -B research/variants/mission-outcome-world.py` replays the frozen ordinary Distributor sequence and reproduces the original1289/1489 captures byte-for-byte. The read-only adapter leaves the entire owned world unchanged. Literal72ED0 plus actual805E0, using captured actor blueprint bindings, matches all returned playerbytes/flags/query results; both worlds remain Running. Runtime flags3E/mode0/active1 are explicit development inputs. This is an integration predicate check, not authentic mission acceptance.
