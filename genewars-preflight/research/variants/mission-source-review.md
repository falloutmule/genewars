# Independent source parser review

The native borrowed parser passed byte-for-byte comparisons on CONQ001/002/003/024/040, pinned to their CD manifest hashes, plus three opaque generated files. This covered 10,944 records and all 2,751,664 source bytes. Every one of the 343,958 truncation lengths rejects; 64 independent tag-bit changes and two unsupported longer lengths reject. Target/group/record count and `usize::MAX` bounds, borrowed pointer offsets and source immutability also passed. See [the receipt](mission-source-review.json).

No defect was found in the reviewed source view. Exact size and tags admit a layout version, without requiring a particular file hash or interpreting opaque records as live entities. Native pointer gaps and family-2 runtime fields are absent from serialized records. The flags/count fields and raw byte 6 remain uninterpreted; arbitrary values in generated files remain accessible without invented policy checks. No original executable or GUI ran.
