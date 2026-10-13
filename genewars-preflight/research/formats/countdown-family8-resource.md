# Countdown family resource binding

## Finding

The family-8 resource base is bound to `data/ethereal.anb`. The static descriptor at mapped VA `0x103910` contains that NUL-terminated path; its pointer field at `+0x1C` is `0x0011681C`. The mapped input is `private/ghidra/actor-activation/memory-.object3.bin` (SHA-256 `aaab09272771ee7f3fbe71e9a3dc75add592fcca42fdd8d982e069629ac1ce87`, mapped from VA `0x100000`). This establishes the previously unresolved destination without relying on a filename guess.

The nearby descriptor at `0x1038E4` binds `data/eff.anb` to `0x116814`; its initializer writes the animation-table address to `0x116818`. The same 0x2C-byte descriptor stride is used by the generic loader at `0x154B0`/`0x15410`. The resulting family-8 resource hook at `0x10060` uses the initialized table at `0x116820`: selector byte `actor+0x55 == 1` selects that base, selectors 0 or 2 select base `+0x50`, and other values leave the existing resource pointer untouched.

The exact CD resource is manifest entry `root/2/disc/60` (`GAME/DATA/ETHEREAL.ANB;1`): packed SHA-256 `7660297d85773f47cacc07dcfdf96e8daa118dc52c8a38afc054c1ef9dfa20bc`, 52,085 bytes; decoded SHA-256 `318dcb26efa6968c34357e67c05b6514efe7d7a8b4052a4e811a851a2ba01bc4`, 91,123 bytes. The bounded RNC corpus reports method 1, successful header/CRC validation, and decoded CRC16 5624. The same packed/decoded identity occurs in `root/1/261` and `root/4/82`; these are aliases of the same bytes, not separate variants.

## Draw-path boundary

The family-8 draw entry is vtable `0x11242C + 0x24 = 0x10EA0`. It resolves animation/frame descriptors using the actor resource pointer, calls the shared shadow helper, then dispatches indexed sprite color through the existing normal/fade blitters. The current evidence closes this resource selection and common frame/color path, but does not establish all countdown-family state-specific decoration behavior. In particular, retain unknown selector behavior and caller-owned draw ordering; do not infer a complete countdown UI or animation policy from the resource binding alone.

## Pinned evidence

- `private/ghidra/mission-tail-init/instructions-00010060.txt`, SHA-256 `518f7e316012183b083623aca3e4b01b8feedee77b0476c56e24d6f76a7d441d` (resource-hook selector dispatch).
- `private/ghidra/countdown-expiry/instructions-00010ea0.txt`, SHA-256 `639fdb8a92c1342f206b3ddd7ebf563618988b4943ff53ca3c9bec9aed881dfb` (family-8 renderer entry).
- `research/ghidra/mission-load-evidence.json` pins the `0x10060` loader instruction listing; `research/ghidra/landing-evidence.json` pins the adjacent EFF descriptor and animation table destination.

This note records addresses, source identities, and bounded behavior only; it contains no decoded asset bytes.
