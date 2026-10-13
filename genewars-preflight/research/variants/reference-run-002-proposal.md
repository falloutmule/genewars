# Prepared run-002: generated configuration, separate approval required

Preparation is complete and no run-002 emulator or original has started. Run-001's approval was consumed by the configuration-failure observation. This document prepares one new bounded session for root's review; it neither requests nor grants execution approval. Root must obtain a new explicit human approval before changing `reference-run-002-approval.json` from false and invoking the guarded launch.

The change is a generated, 126-byte CRLF `GAME/config.ini`, SHA-256 `ff0c1921640bb81cd0b73fc4a995316ea9ff1b8d60ed288e5d52a1be4e7c0d5f`. Its exact independently generated contents are:

```text
CD="D:\GAME\"
InstallDrive="C:\GAME\"
Language="ENGLISH"
Intro="All"
Anims="All"
Sounds="All"
Levels="All"
Data="All"
```

Static evidence in `research/ghidra/config-evidence.json` and `config-bridge.md` establishes the quoted eight-key writer, accepted uppercase ENGLISH, trailing separators and the all-All setup branch. Each line is well below the 79-byte parser input bound, with no blank lines, section markers or spaces around `=`. All selects the installed prefix for that category. Mount letters are deliberate preparation choices, not original defaults. No sound.ini is generated: the CD has none, and static parser instructions install the observed binary fallback after missing-file retries. This establishes a supported preparation format, not successful installation or startup.

The fresh `private/reference-run/run-002/drive-c/GAME/` contains all 490 original CD GAME files, 186,368,166 bytes, independently reread through SourceReader and rehashed. It includes the unchanged GW.EXE SHA-256 `4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba` and DOS4GW.EXE `b8265123ac8a189637448618409ef3ecd2e9f3e1a47062c685a02240f688dec1`. A fresh exact CUE/IMG copy is in run-002/media; IMG SHA-256 is `3ae5f2154a84898a7d1652e550d741a0a431fe67fd0b3eb20121e3b11fd4e5de`, CUE is `ba85aeb6a808145b43998a34faf6904956866539625bd45ead6dba0c2fd10ee5`. Run-001 and its modified files/logs are preserved. Neither installer nor alternate launcher is run or used to synthesize state.

DOSBox-X is the same verified official portable 2026.10.01 SDL2 executable (`a82adcfa7af2784566cfeff74c077cefed08505396db2948b7e6aa7d4cee0832`). The complete 894-parameter private configuration is unchanged (`9dba4b2ff89fa91a891cff667616f23b1562ba909ec3ed6f98a143870eb58f78`). It retains the reviewed networking, host integration, clipboard, printing, serial/parallel, MIDI and microphone-silence policies. These are emulator policies, not OS isolation; DOSBox-X retains host-user permissions and still enumerates host microphones. The [official command-line reference](https://dosbox-x.com/wiki/DOSBox%E2%80%90X%E2%80%99s-Command%E2%80%90Line-Options), [image guide](https://dosbox-x.com/wiki/Guide:Managing-image-files-in-DOSBox%E2%80%90X) and [FAQ](https://dosbox-x.com/wiki/Home) support the previously validated startup/image/no-cache options. No new emulator download, installation or global setting is needed.

From the project directory, these commands are safe before approval:

```text
python -B scripts/reference/run002.py prepare          # verify only
python -B scripts/reference/run002.py launch           # print dry-run command; no process
```

Staging is reproducible with `run002.py prepare --stage`; it creates missing exact files and refuses changed files. The wrapper selects a fixed run-002 context only inside its Python process. It does not set a global environment variable or modify run-001. Generated config bytes and nine private instruction hashes are verified again. Readiness and before-files metadata pin all 491 game-tree inputs. The original source archive remains unmounted.

Proposed new approval scope: one root-supervised DOSBox-X session, at most 1,200 host seconds including the waiting gate, of those exact GW.EXE/DOS4GW hashes with this exact generated config. Observe startup; if it succeeds, capture menu and the first available mission, static left panel and a short timing sample, optionally one save/reload. Stop on installation/CD/layout rejection, startup crash, or required executable/config modification; do not run an installer, setup, launcher, patch, alternate variant or network play. No upload or unrelated host access. This is a direct reference observation of the provisional CD candidate, not canonical retail proof.

After a new human approval, root records its exact statement, time and scope in the pending approval file, retaining its run ID, candidate and generated-config hashes. Only then:

```text
python -B scripts/reference/run002.py launch --open-approved-session --approval-note "<actual new human statement and exact scope>"
```

Use normal tool approval for an interactive process outside the per-command sandbox, and retain the live execution session. The parent flushes PID, stays alive polling its exact child, captures stdout/stderr privately, explicitly requests SW_SHOWNORMAL, and kills that exact child at 1,200 host seconds. It uses the same eight reviewed CLI built-ins: MOUNT C private drive-c with `-nocachedir`; IMGMOUNT D copied CUE as ISO; MOUNT; CONFIG -SECUREMODE; refused MOUNT E; C:; CD GAME; STARTGW.BAT. The batch cannot call GW before its fresh random marker is released. All original-launch flags also check the separate run-002 approval and exact config hash.

Root must see and reliably control the actual DOSBox window, inspect the logged exact C/D/Z mounts and secure-mode E: refusal, and check the PID image/creation time before release:

```text
python -B scripts/reference/run002.py gate
python -B scripts/reference/run002.py gate --release-approved-original
```

The gate revalidates unchanged inputs, writes a private host-release receipt, then creates the exclusive single-use marker. No security settings or DOS shell typing through UI automation is necessary. The operator interacts only with the game's actual UI after release. Record observation times and distinguish configured host cycles from measured original behavior. Captures, saves and raw logs remain private.

Before release, `run002.py gate --abort` stops only the waiting gate. If a released process hangs, root verifies its exact recorded executable and creation time and terminates only that PID; automatic timeout remains in force. Preserve all run-002 outputs and hash the full game tree against before-files metadata after exit, including generated config, GW.ERR and any new files. Verify both original and copied media plus source archive again. Do not automatically delete or restage a used session. An unreleased failed attempt can be preserved with `run002.py archive --attempt-name <unique-name> --reason <actual-reason>` only after confirmed exit and proof of no release; it refuses any released attempt. New original execution after the one approved session requires another separately reviewed approval.
