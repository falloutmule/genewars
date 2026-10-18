# Turret target tracking

Corroborated:256 literal/native cases cover target reuse, periodic search, alliance and state filters, wrapped positions, signed yaw, cooldown and RNG wrap. Successful complete outputs plus22 explicit projectile-boundary results compare441,112 bytes; successful callbacks execute2,159,450 literal instructions.

The review caught an instruction-level quirk: when a search finds no replacement,47383 clears the stored target kind while ESI retains the previously resolved target. Rotation and cooldown still use that old target for the rest of the callback. The production planner preserves this distinction between stored reference and current callback target.

The actual-world gate creates16 worlds around the exact mission Mule, across all four owners and four distances.44 successful callbacks match complete backing, native pools, players, RNG and handles (18,030,892 bytes;284,780 literal instructions). Each world then reaches the exact475D5 firing boundary; the native adapter rejects and preserves its entire beforeimage. Typed rotate/stop23 requests match source call sites, without executing audio hardware.

The composed gate additionally compares16 mixed full47F80 passes with acquired targets, incomplete neighbors, player history and shared RNG (6,556,688 bytes;323,699 literal instructions). A final-player projectile boundary occurs after three earlier callbacks; the native whole-pass transaction preserves its complete beforeimage.

`step_tracking_completed_selector1_building` requires the actual frame counter and hash-bound physics tables. It resolves existing references and reads intrusive map lists from the owned world. The legacy quiet-only API remains stricter. Projectile geometry, allocation, movement and damage are not implemented by this tracking slice; aligned zero-cooldown targets return `ProjectileRequired`.

Reproduce with turret-tracking-review.py and turret-tracking-world-review.py and turret-tracking-pass-review.py under research/variants. No original executable or GUI is run.
