# CD configuration parser and setup writer

Scope: static analysis of the provisional CD GW.EXE candidate (SHA-256 `4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba`) and exact CD `INSTALL/DOS/INST.EXE` (SHA-256 `570059e6b64f289e740491814fa775d4c87d53b4e2a61d53cea5e7cd7106feb7`). Neither program was executed for this analysis. No configuration file was generated. The separately authorized reference session ended at missing config.ini; another original session requires separate authorization.

The two observed startup messages come only from GW function `0x150F0` taking its fopen-failure branch at `0x15110 -> 0x153D5`. It opens the relative filename `config.ini`, mode `r`, directly through `0x99D41/0x99D08`. This establishes a current-working-directory file requirement, not an executable-directory lookup. The failure precedes parsing, so it says nothing about required key validation. Successful EOF returns true even if keys were omitted; that behavior does not make omitted keys safe for later consumers.

The exact CD inventory contains neither a config.ini nor a sound.ini member. DOS setup writes CONFIG.INI at `0x1AE48`: chosen install directory + backslash + filename, opened with `wt`. This is a generated file, not a CD template. Its writer emits these eight records, each followed by LF in the format string (text-mode output may translate LF to CRLF):

| Key | Writer value source / consumer contract |
| --- | --- |
| CD | Setup current directory, uppercased and truncated at `\\INSTALL`, followed by `\\GAME\\`. Game stores the complete prefix and computes one-based drive number from its first letter. |
| InstallDrive | Chosen install directory followed by an appended `\\`. Game stores complete prefix and first-letter drive number. |
| Language | Selected language string; game accepts ENGLISH, FRENCH, GERMAN, SPANISH, SWEDISH, ITALIAN case insensitively, mapping to byte values 0..5 in that order. |
| Intro | First value byte exactly uppercase `A` selects installed prefix; any other first byte selects CD prefix. |
| Anims | Same selection, category 1. |
| Sounds | Same selection, category 2. |
| Levels | Same selection, category 3. |
| Data | Same selection, category 4. |

All writer values are enclosed in double quotes, with no spaces around `=`. InstallDrive's format appends the trailing backslash; CD's constructed prefix already includes it. These are prefixes to the game content directory, not just drive letters. The game joins prefix + resource filename using `%s%s` without adding a separator (`0x150B0`, `0x154F0`, `0x155F0`). `0x154F0/0x155F0` bypass prefixing when a filename contains a colon; the registered callback `0x150B0` bypasses names beginning with `*` instead.

Setup category choices are instruction backed: branch `0x136E3` assigns `All` to all five; `0x1454F` assigns `All,All,Min,Min,Min`; `0x153BB` assigns `Min` to all five. These are observed branch assignments, not a claim about which choice the UI defaults to. `All` reads that category from the installed prefix and requires corresponding content there; `Min` reads it from the CD prefix. Copying only GW.EXE and choosing All does not establish a complete installation. Drive letters and mount locations remain preparation choices requiring explicit review.

Parser details at `0x150F0`: an 80-byte fgets buffer reads at most 79 bytes per call; tokenization splits on the individual characters `=` and `"`. It performs no whitespace trimming, INI section handling, escaping or comment stripping. Keys and language names use ASCII case-insensitive comparisons; resource flag values do not. Thus `data="All"` is recognized, `Data ="All"` is ignored, and `Data="all"` selects CD. Blank/empty-token lines and recognized keys without a value reach unchecked token pointers. Keep the eight generated-style records within the line bound, with no blank or section lines. Quoted values avoid a trailing newline becoming part of the value. Unknown language values preserve prior state; this is not a supported default.

At EOF, ECX is explicitly incremented from the config-object base `0x1187A4` to `0x1187A5` before subsequent stores. Accounting for that instruction, flags at `0x118965/0x118966` become 1/0, and resource category `0x118967` becomes 5. The five category flags occupy `0x118960..0x118964`. The install prefix is `0x1187BC`, CD prefix `0x11883C`. The accumulated number of uppercase-A category lines is compared with 7 at `0x15384`; duplicate lines count again. With exactly five category lines, that comparison is false. Its purpose is not inferred.

The next file is installed-prefix `sound.ini`, loaded as raw bytes into settings at `0x1187A5`, not parsed as text. `0x9954C` reads the whole file (and supports decompression); no ten-byte input length guard was found on this call. After missing-file retries, `0x154F0` logs and flushes its error stream through `0x9A2B8`; it returns -1. This is **not** a fatal-exit call. The parser then installs fallback settings `{u32 127, u32 50, u8 1, u8 1}` (little-endian bytes `7f000000320000000101`). `0x15050` consumes the first dword, the low byte at +4, and flags +8/+9; `0x99AD0` accepts the first value only within 0..127 and `0x992B0` clamps its byte argument to 127. Exact game audio settings UI/save behavior is outside this tranche. No guessed sound.ini is necessary to explain the current failure.

Installer loading note: lx-loader rejected the original partial final page by attempting a 4096-byte read beyond EOF. A separate analysis-only copy adds 863 zero bytes; original extracted bytes and their hash are preserved. An independent reconstruction from the untouched original verifies all 232 pages and 7,845 internal fixups, with every byte of all three mapped objects equal to Ghidra. Installer permissions remain the loader's analysis-only read-only setting; no execution is involved. Raw listings and extracted bytes remain under `private/ghidra/config-*`.

Reproduce receipt and bounded synthetic parser tests with `python -B scripts/ghidra/validate_config_bridge.py`. The tests establish static contracts and representative parser edge cases, not successful original startup or sound playback.
