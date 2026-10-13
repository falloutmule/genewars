# PLANETS.DAT bounded reconnaissance

## Input and decode evidence

The selected input is manifest entry `root/1/246`, `Gene_Wars_RePack/DATA/PLANETS.DAT`, read with `scripts/source_member.py` into ignored `private/assets/planets.dat`. It is 50,679 bytes with SHA-256 `1e3b70c2b6856c645af01db2a98f8a71f99b7c823324c24f6406bce734c5e23c`. A targeted `SourceReader` reread of `root/1/246`, `root/4/146`, and CD entry `root/2/disc/122` verified each file's size and SHA-256 against the manifest; all three yielded the same exact bytes.

`research/integration/rnc-corpus.json` records a passed method-1 RNC decode for this exact input hash: 245,760 output bytes, output SHA-256 `ff3da7b5de7e26508691e3e1c13f01ced3c6a0ed8cd5cede8ed2f266682e4b65`, decoded CRC16 23279. The corpus output was reused, not re-decoded, and copied into ignored `private/assets/planets.dat.decoded` for the bounded inspection. Its original corpus path is `private/integration/rnc-decoded/1e3b70c2b6856c645af01db2a98f8a71f99b7c823324c24f6406bce734c5e23c.bin`.

## Byte-level observation

The decoded size is divisible by 24 and by 10,240: it can be partitioned into 24 equal 10,240-byte slices. All 24 slice hashes differ. Each slice has a distinct mix of zero and nonzero bytes; several late slices also contain long runs of ASCII space (`0x20`). This partition is a measurement only. The manual describes a 24-planet single-player campaign (`research/manual/manual-index.json`, “Campaign and missions”, PDF page 6 / printed page 9), but that count does not prove that the file consists of one fixed-size record per planet.

No field widths, names, offsets, coordinate system, map dimensions, or terrain interpretation are established by these measurements. No raster or gameplay representation has been inferred. The targeted CD trace found no literal `PLANETS.DAT` reference in the four mapped objects and did not establish a consumer for this file; that does not prove the resource is dead or exclude all computed references. No parser is implemented. The separate numbered planet texture path is documented in [planet-texture-atlas.md](planet-texture-atlas.md).

## Consumer-trace redirection

Sol01's targeted CD trace found no `PLANETS.DAT` filename reference in the mapped objects. The matching descriptor at VA `0x102E58` names `data/planets.anb`. The planet-selection initializer at VA `0x15B90` formats `data/pl%d.dat` from a system index, reads the selected raster through a path at VA `0x154F0`, and loads the `PLANETS.ANB` resource through descriptor VA `0x154B0`. This is evidence that the planet-selection flow uses `PL%d.DAT` and `PLANETS.ANB`, not proof that `PLANETS.DAT` consists of planet records. The descriptor loader at VA `0x15410` and raster extent semantics are still under targeted review; no parser contract is asserted here yet. Sol01's private evidence exports are `private/ghidra/planets-refs/instructions-00015b90.txt` and `private/ghidra/planets-load/instructions-000154b0.txt`.

For the exact CD variant, manifest entry `root/2/disc/117` (`PL1.DAT;1`) is 35,346 bytes, SHA-256 `4283af614d54519e2a9f6c59a881ba58ad356b8844c9e489a51501c51e2aeeac`; matching RIP entry `root/1/200` has the same digest. The RNC corpus reports 163,840 decoded bytes, SHA-256 `476090285016a3f458c175e2aae84c30d1f33a23817ee5556d99dd96ef1aceee`. CD entry `root/2/disc/121` (`PLANETS.ANB;1`) is 5,666 bytes, SHA-256 `e1a167148f02580ce2a170743502dc91460277a086f83bb9af83d26242cb1801`; its RNC output is 21,068 bytes, SHA-256 `f066895683616c0014548237d072a3bb2b2bc897d1a28d45fd7f2cd869123c88`. These exact CD inputs and existing corpus outputs have been staged under ignored `private/assets/` for the targeted follow-up. The 5,555-byte `PLANETS.ANB` from `root/4/145` is a different variant and is not substituted.
