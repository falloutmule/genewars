# Incomplete building scheduler review

Corroborated:64 complete owned-world passes match literal47F80,451F0,7AC00 and437A0 bodies (26,226,752 bytes;133,349 instructions). Cases cover player counts0/1/2/4, all selected players, dense and sparse0/1/3/8-building partitions (including slot63), and incomplete progress0/1/62/119. The comparison includes complete backing, actors, creatures, buildings, players, RNG and typed resource bindings. Compilation uses a pinned private source snapshot, independent of later edits. Sparse fixtures use a private-only initial slot/link permutation; it does not change the callback implementation.

An initial review misread the skip target and proposed a dense-prefix restriction. The source's inactive/newborn branches jump48003, skipping48002's decrement; only an admitted active callback consumes the signed remaining count. A sparse active partition is therefore compatible when declared and actual counts agree. The restriction is removed and the expanded complete-world fixtures pass. Newborn scheduling remains separately unsupported.

Audio, queue, formatting and UI bodies are explicit ports. Completed turret searches, newborn rescanning, destruction and complete gameplay are excluded.

Reproduce with `python -B research/variants/building-scheduler-review.py`.
