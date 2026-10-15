# Owned Mule movement source boundary

Observed against pinned QA BETA V5 source: the source-created owner0 Mule accepts opcode0x11 and preserves command7/target1536,1536. The first callback enters state8. An additional64 literal callbacks (frames2..65) do not change position1024,1024. This is **blocked at the callback cap**, not movement acceptance or a demonstrated Rust defect.

The wait timer is not a simple nineteen-frame delay: it decrements on scheduled frames and resets during path attempts. Frame15 changes18 to22 and consumes36 RNG draws. Final RNG: seed64996/ring1/count304. State8, flags0x8018 and command7 remain. No first movement or arrival was observed.

This existing synthetic null-parent factory fixture has flat height100, terrain class byte5=0, no other actors and no Genepod ejection overwrite. It is not an established walkable Krell route. Do not modify terrain/timer/state to manufacture movement. Existing literal listings5B9F0,74A50,305F0 and6D230 close encountered listing gaps. Full per-callback before/after memory and traces: `private/review/krell-owned-mule-walk/2341ef2fdde94364b5190375f7f7fb55`. JSON pins source and generator. Presentation ports are inherited from the command probe.

Shared creature_world.rs still explicitly requires owner4 in initial/action/wait paths. Pure mule_owned_command proves command installation and guarded cleanup only; it does not justify removing those gates. Next: compare a real terminal factory/ejection output on actual terrain, then implement only observed owned branches with complete source afterimage/RNG checks.

Native_input supplied `private/review/genepod-terminal-world-002/success-after`: source-compared terminal adapter output, newbornslot1/owner0/quality50/elevated ejectionheight, on authentic CONQ001 terrain. Its completed Genepod/work1 were synthetic. This is a stronger next callback fixture, still not ordinary Krell production or native acceptance. Receipt: `research/variants/genepod-terminal-world-review.json`.
