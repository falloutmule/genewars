# Krell Architect return: current fixture correction

The frame1005 callback fixture already contains the next Distributor command. Actor0 is state8, task82=1, selector83=3, pending flags57=1/58=2, moving from [2638,15952] to [2943,15999]. Its associated entity kind4F is zero. The sole building is Extractor11, completed at health280 with flags1. The adjacent JSON pins these exact observations.

The earlier frame960 state9 return is stale evidence for the current blocker. Do not reset the actor or indefinitely wait for idle: the new command has already been admitted. Retained work stage88=5 does not establish that this actor is still constructing the old building; state and current task selector must be read together.

The existing construction scheduler explicitly handles state8 with pending57/58 and task82/7E as ordinary queued construction movement. Previous return proof (`research/ghidra/construction-return-saved-fixture-source.json`) already compares source movement/whole-callback afterimages on an older return fixture. No new source defect is demonstrated here, and repeating that proof would not unblock the current wildlife callback failure.

This is a saved-state diagnosis, not proof of a completed Distributor or accepted Krell mission. Continue the current callback-boundary repair and observe the already-issued Distributor order.
