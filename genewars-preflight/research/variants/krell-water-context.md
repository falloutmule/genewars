# Krell physics water-global binding audit

The captured serialized word at backing1C63 is816; dword1C67 is2000. They map to original globals11A9C3 and11A9C7 respectively. Current native mission_loading_physics derives1632 by doubling the former. The source callback oracle binds the latter directly from the captured serialized value2000.

The complete884D0..8887F listing is terrain-border table initialization and contains no write to either water global. A fresh read-only QueryReferences audit of11A9C7 in the verified Ghidra project returned exactly four direct references, all reads in6DF30:6DF3D,6E825,6E8B3,6E8F7. It found no direct write. This does not exclude indirect or bulk writes, but there is currently no evidence for a runtime assignment11A9C7=2*11A9C3. Do not project1632 into the source oracle as an established initialization fact.

11A9C7 controls source water comparisons and effect heights: initial above-water classification at6DF3D; water proximity plus320 at6E825; submerged threshold minus512 at6E8B3; and splash height max(ground,11A9C7) at6E8F7. It is therefore potentially observable near water.

The actual frame294 parent callback comparison passes all427369 bytes using raw2000 (private receipt5597f79ef70c465db1ffb4bfdeb10231). Its height5376 is safely above both candidate proximity thresholds; this pass does not establish native1632 as the correct runtime water global. Earlier high-ground passes should retain their recorded source inputs and not be silently relabeled. Near-water native fidelity needs resolution of this input discrepancy.

Evidence: private/ghidra/krell-water-context/reference-graph.json and instructions-0006df30.txt; private/ghidra/mission-target/instructions-000884d0.txt. Query ran readOnly/noanalysis and completed without project mutation or game execution.
