# Independent ANB sprite extent audit

Observed on 2026-10-12 local: the superseded full-file validation policy marked 260 ancestry records / 164 unique ANBs passed, but 480 nonempty sprite streams in 25 of those unique inputs read metadata beyond the first section boundary. These correspond to 50 ancestry records. This contradicts a claim that all former passes were clean bounded sprite decodes; it does not establish corrupted source assets.

The earlier full-file corpus was recovered from the Git index, restoring CRLF line endings, and preserved byte-for-byte as `full-file-sprite-corpus-superseded.json`, SHA-256 `e37e4ff54416f90d9192b8ba5de0f650770be9fd087bd2cffff6c62c02a931f2` (3,320,915 bytes). Its native executable hash remains pinned there. Root has since superseded that policy with a first-section prefix bound: 210 of 263 ancestry records pass, 53 are blocked, and 8,292 embedded sprite records are accepted. The 50 newly blocked records are the affected creature ANBs; the three earlier failures remain separate.

## Reproducible checks

`sprite-boundary-review.py` reads the preserved receipt and existing private cached databases/tables. It revalidates decoded/database hashes, embedded table hashes, actual header section offsets and table contents. For each six-byte descriptor it computes start=`18+relative_offset` and end=`start+native_consumed_bytes`. Empty dimensions consume no bytes and are not treated as reads, even if an otherwise unused offset were unusual. Every active interval is checked against [18,section0), independently of any next sprite offset.

An independently written control walker reproduced all 10,990 native consumed-byte and opaque-pixel receipts. It also checked the same 10,346 nonempty streams that stay within the actual data extent using section0 as a hard read limit. There are 164 empty descriptors, no nonempty starts already in metadata, and 480 streams starting in data but ending after section0. The largest excess is 1,486 bytes. Six valid streams end exactly at section0; end equality is allowed.

Run from the project directory:

```text
python -B research/variants/sprite-boundary-review.py
python -B research/variants/sprite-boundary-review.py --diagnostic-models
```

The first command intentionally exits 1 when it reproduces the historical boundary violations. The second appends diagnostic hypotheses to the owned JSON; it does not change the parser or declare a correct grammar. No archive extraction, native tool or original execution occurs in this audit.

## Concrete failure and unaffected UI

CD `GAME/DATA/CREATURE/DINO.ANB;1`, `root/2/disc/325`, is identical to RePack `root/1/231`; SHA-256 `0c9d9dbbcb29e39ab29aac17eca97da9a404791febdb79123b6c0eb44af78767`. Descriptor 48 is width49/height48 with relative offset22174. Under the reviewed base18 policy it starts22192 and the native receipt consumes21949 bytes through exclusive end44141. Sprite data ends44137, so four metadata bytes were consumed. A bounded independent traversal completes only44 rows before requesting control byte44137. Across those completed rows, run widths range1..2513 with median368, implausible as an ordinary width49 row interpretation. This is diagnostic evidence of a grammar/path mismatch, not a repaired decoder.

All 25 affected unique inputs are the BIRD/CRAB/DINO/FROG/MULE creature files and their named pair combinations, in the CD and matching RePack ancestry. The existing BONES failure is separately blocked. Exact CD IFCUSTOM (`1f69d8bca8bed9d9e15ee1cc79e0ce0856bb4cb6d7e3791e704b31d02b0006b4`) remains clean under the bound: 36 active streams, largest end35495, section0=35496. Its UI display bindings therefore do not depend on the unsafe animal passes.

The historical model produces 47,186 overlapping distinct-start interval pairs. These are reported as relationships, never automatic failures: adjacent offsets are not proven hard stream boundaries, and sharing/overlap may be valid. No duplicate nonempty start occurs in this corpus. Overlap here is also contaminated by the demonstrably wrong creature interpretation, so its magnitude is not evidence of intentional sharing.

## Alternative diagnostics and action

| Group/model | Active streams | Boundary failures | Completed rows with run sum beyond target |
|---|---:|---:|---:|
| Other139 inputs, base18, zero row ending | 8153 | 0 | 0 / 306953 |
| Affected25 inputs, base18, zero row ending | 2673 | 480 | 50162 |
| Affected25 inputs, base0, zero row ending | 2673 | 448 | 47498 |
| Affected25 inputs, base18, stop at width | 2673 | 93 | 84825 |
| Affected25 inputs, base18, width plus optional zero | 2673 | 103 | 89470 |

Half/double width and half/double row-count hypotheses were also measured. None fully resolves the animal set. A width stop can merely stop sooner inside a wrongly interpreted stream: DINO48 finishes at25514, while the next distinct sprite start is22706, and 46 of its48 rows overshoot width. That next offset is a comparison statistic here, not a validity rule. Base0 likewise does not resolve DINO48's zero-row failure. Changing base or row termination based only on improved pass counts would be unsupported.

Keep strict metadata bounds now. For the base18 decoder, pass `anb[:section0]` as the database, preserving the header-relative arithmetic; an ANB-aware library API should enforce the same extent. External TAB/DAT callers require a separately proven data extent and base. Recover the actual creature loader/consumer, scaling path, control grammar and row advancement from direct instructions before accepting these inputs under any different interpretation. SOL-01 is tracing creature consumer VA0x9A660 against the UI VA0xA7730 path; this audit makes no claim that their formats are interchangeable.
