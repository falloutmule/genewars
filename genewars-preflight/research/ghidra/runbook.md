# Isolated static-analysis runbook

Run commands from `genewars-preflight`. Ghidra and its JDK are pinned and local. `run_headless.py` routes home/settings/cache/temp and Java preferences into `private/ghidra`; it launches Ghidra only. Do not launch GW.EXE or its installer. Private dumps, original-byte listings and decompilation must remain Git ignored. Safe research contains behavior descriptions, hashes, addresses, provenance, and synthetic vectors only.

```
python -B scripts/ghidra/prepare_local.py
python -B scripts/ghidra/run_headless.py -import private/ghidra/GW-CD-candidate.exe -overwrite -loader LeLoader -analysisTimeoutPerFile 300 -scriptPath scripts/ghidra -postScript RepairObjectPermissions.java -log private/ghidra/le-import.log -scriptlog private/ghidra/le-import-script.log
python -B scripts/ghidra/run_headless.py -process GW-CD-candidate.exe -noanalysis -scriptPath scripts/ghidra -postScript ExportEvidence.java private/ghidra/validated 0x73690 0x737f0 0x97910 0x432c0 0x8d580 0x8d5f0 -log private/ghidra/validated.log -scriptlog private/ghidra/validated-script.log
python -B scripts/ghidra/validate_le.py
python -B scripts/ghidra/validate_primitives.py
python -B scripts/ghidra/validate_renderer_bridge.py
python -B scripts/ghidra/validate_ui_bridge.py
python -B scripts/ghidra/validate_creature_bridge.py
python -B scripts/ghidra/validate_input_bridge.py
python -B scripts/ghidra/validate_list_layout.py
python -B scripts/ghidra/validate_config_bridge.py
python -B scripts/ghidra/validate_bones_bridge.py
python -B scripts/ghidra/validate_construction_bridge.py
python -B scripts/ghidra/validate_planet_texture_bridge.py
```

The import command replaces the local analysis database; preserve existing discoveries before deliberately reimporting. Normal continued work uses `-process ... -noanalysis`, not reimport. `RepairObjectPermissions.java` corrects the loader's all-read-only default from independently verified LE flags; it changes no bytes. `RecoverPanelFunctions.java` repairs the rejected speculative boundary and creates two function-pointer targets. Reapply it after a fresh import before exporting panel evidence.

`ExportEvidence.java` receives private output folder followed by explicit function addresses. It exports initialized objects, complete relocation metadata, function/call metadata and the requested listings/decompilation. It may try to create a missing function, so supply supported boundaries only. `QueryReferences.java` receives a private output folder and data/function addresses, exporting xrefs and containing functions privately. `SurveyFunctions.java` is a bounded private leaf-function survey. Decompiler C types are not authoritative register contracts.

Tool verification receipts: `tool-verification.json` and `jdk-release.json`. Pinned Ghidra 12.0.1 and matching lx-loader v12.0.1 are installed; JDK Temurin 21.0.12.1+1 is compatible. Archives remain private for reproducibility. Helper repository tree metadata is in `helpers-tree.json`; its tree has no license and no symbol map for this CD candidate, so no helper bytes were copied or applied.

Setup failures are preserved privately: the first import without the extension fell back to DOS MZ and is quarantined under `initial`; it supplies no accepted evidence. Windows Java preferences initially attempted a registry access and failed in sandbox; the original in-memory preferences adapter resolves this without registry changes. Initial missing `LeLoader` display-name and one Java API compile error were corrected. Restricted network DNS required normal reviewed `curl.exe` escalation for pinned archive downloads; installed archive hashes match release metadata. No user-wide environment or Git setting was changed.

Accepted loader evidence is `loader-validation.json`: all 244 pages, 26,883 internal fixup records, 24,497 cross-object instances and all four reconstructed object images compare byte-for-byte. Entry object 1 + 0x96E40 maps to VA 0xA6E40. Function tranche and renderer bridge are bounded static behavior specifications; synthetic models/tests are not original execution traces.

Later bounded receipts are `ui-evidence.json`, `creature-evidence.json`, `input-evidence.json`, `list-layout-evidence.json`, `config-evidence.json`, and `bones-evidence.json`. Their matching bridge documents distinguish actual UI assets, caller-selected packed4 pixels, mode-two mouse packets/capture, bounded list-name storage, and exact CD configuration. Earlier general ANB indexed8 assumptions are superseded by the creature codec proof; never choose a codec by parse success. The creature receipt admits 25 exact input hashes/50 CD+root1 aliases. BONES is admitted separately by its actual entity/frame consumer trace and exact source hash; GPF remains blocked.

Configuration analysis preserved the existing game program and added a separate static installer program in the same local Ghidra project. The exact CD installer original remains untouched. The loader's partial-page read failure is documented in `config-bridge.md`; an analysis-only zero-padded copy was independently checked against every original LE page/fixup. No installer execution is involved.

Later simulation evidence is `construction-evidence.json` and `construction-cost-bridge.md`: a stage-three debit with signed skill adjustment, two wrapping signed32 pools, explicit readiness prerequisite and separately gated refill. `planet-texture-evidence.json` and `planet-texture-bridge.md` establish numbered PL1..4 texture tiles and index-zero transparency. PLANETS.DAT itself remains consumer-unproven; its byte count is not a planet-record schema.

Next work: identify animation timing/flags and behavior beyond the bounded custom controls and transactions. Root supervises the separately authorized reference workflow and preserves runtime receipts and variant hashes. This Ghidra lane continues static analysis and does not launch the original executable; static evidence receipts do not themselves grant runtime authorization.
