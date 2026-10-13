"""Static CloneCD/ISO inventory. Uses only Python stdlib and Windows tar/libarchive.

No mounting, installer execution, content extraction by archive-supplied paths, or
network access. Scratch names are generated, and archive originals stay unchanged.
"""
from __future__ import annotations

import argparse
import configparser
import hashlib
import io
import json
import math
import re
import subprocess
import zlib
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TAR = Path(r"C:\Windows\System32\tar.exe")
SYNC = b"\0" + b"\xff" * 10 + b"\0"


def hash_file(path, start=0, length=None):
    h = hashlib.sha256()
    with path.open("rb") as f:
        f.seek(start)
        remaining = length
        while remaining is None or remaining:
            b = f.read(1024 * 1024 if remaining is None else min(1024 * 1024, remaining))
            if not b:
                if remaining:
                    raise ValueError("Unexpected EOF during bounded hash")
                break
            h.update(b)
            if remaining is not None:
                remaining -= len(b)
    return h.hexdigest()


def analyze(data, name):
    from inventory_common import analyze_bytes
    return analyze_bytes(data, re.sub(r";\d+$", "", name))


class Disc:
    def __init__(self, path, track_start, data_end, warnings):
        self.path = path
        self.f = path.open("rb")
        self.track_start = track_start
        self.data_end = data_end
        self.sectors = path.stat().st_size // 2352
        self.warnings = warnings
        self.checked = set()
        self.bad = set()

    def payload(self, lba):
        absolute = self.track_start + lba
        if absolute < self.track_start or absolute >= self.data_end:
            raise ValueError(f"LBA {lba} outside data track [{self.track_start}, {self.data_end})")
        self.f.seek(absolute * 2352)
        raw = self.f.read(2352)
        if len(raw) != 2352:
            raise ValueError(f"Short physical sector at {absolute}")
        self.checked.add(absolute)
        if raw[:12] != SYNC or raw[15] != 1:
            self.bad.add(absolute)
            raise ValueError(f"Sector {absolute} lacks MODE1 sync/mode header")
        return raw[16:2064]

    def extent(self, lba, size, volume_blocks):
        if lba < 0 or size < 0 or lba + math.ceil(size / 2048) > volume_blocks:
            raise ValueError(f"Extent {lba}+{size} exceeds volume ({volume_blocks} blocks)")
        if self.track_start + lba + math.ceil(size / 2048) > self.data_end:
            raise ValueError(f"Extent {lba}+{size} exceeds data-track boundary")
        return b"".join(self.payload(lba + n) for n in range(math.ceil(size / 2048)))[:size]


def both(data, offset, width, label):
    little = int.from_bytes(data[offset:offset + width], "little")
    big = int.from_bytes(data[offset + width:offset + 2 * width], "big")
    if little != big:
        raise ValueError(f"{label} endian copies disagree: {little} / {big}")
    return little


def record(data, namespace):
    if len(data) < 34 or len(data) != data[0]:
        raise ValueError("Invalid directory record length")
    n = data[32]
    if 33 + n > len(data):
        raise ValueError("Directory identifier exceeds record")
    raw_name = data[33:33 + n]
    name = {b"\0": ".", b"\1": ".."}.get(raw_name)
    if name is None:
        name = raw_name.decode("utf-16-be" if namespace == "joliet" else "ascii", errors="replace")
    return {
        "name": name,
        "identifier_hex": raw_name.hex(),
        "extent_lba": both(data, 2, 4, "extent"),
        "size": both(data, 10, 4, "data length"),
        "extended_attribute_blocks": data[1],
        "flags": data[25],
        "file_unit_size": data[26],
        "interleave_gap": data[27],
        "volume_sequence": both(data, 28, 2, "volume sequence"),
        "recording_time_raw": list(data[18:25]),
        "system_use_hex": data[33 + n + (1 if n % 2 == 0 else 0):].hex(),
    }


def inventory_iso(disc, image_name, entries, warnings):
    descriptors = []
    volumes = []
    terminated = False
    for lba in range(16, 272):
        data = disc.payload(lba)
        info = {"lba": lba, "type": data[0], "identifier": data[1:6].decode("ascii", "replace"), "version": data[6]}
        descriptors.append(info)
        if data[1:6] != b"CD001":
            warnings.append(f"Unsupported volume descriptor at LBA {lba}: {data[1:6]!r}")
            break
        if data[0] == 255:
            terminated = True
            break
        if data[0] in (1, 2):
            joliet = data[0] == 2 and data[88:91] in (b"%/@", b"%/C", b"%/E")
            if data[0] == 2 and not joliet:
                info["inspection_status"] = "blocked"
                warnings.append(f"Supplementary non-Joliet volume at LBA {lba} not traversed")
                continue
            namespace = "joliet" if joliet else "iso9660"
            encoding = "utf-16-be" if joliet else "ascii"
            clean = lambda b: b.decode(encoding, "replace").strip(" \0")
            info.update({
                "namespace": namespace,
                "system_identifier": clean(data[8:40]),
                "volume_identifier": clean(data[40:72]),
                "volume_blocks": both(data, 80, 4, "volume blocks"),
                "logical_block_size": both(data, 128, 2, "block size"),
                "volume_set_size": both(data, 120, 2, "volume set size"),
                "volume_sequence_number": both(data, 124, 2, "volume sequence"),
                "path_table_size": both(data, 132, 4, "path table size"),
                "little_endian_path_table_lba": int.from_bytes(data[140:144], "little"),
                "optional_little_endian_path_table_lba": int.from_bytes(data[144:148], "little"),
                "big_endian_path_table_lba": int.from_bytes(data[148:152], "big"),
                "optional_big_endian_path_table_lba": int.from_bytes(data[152:156], "big"),
                "volume_set_identifier": clean(data[190:318]),
                "publisher_identifier": clean(data[318:446]),
                "data_preparer_identifier": clean(data[446:574]),
                "application_identifier": clean(data[574:702]),
                "creation_time": data[813:830].decode("ascii", "replace"),
                "modification_time": data[830:847].decode("ascii", "replace"),
                "expiration_time": data[847:864].decode("ascii", "replace"),
                "effective_time": data[864:881].decode("ascii", "replace"),
            })
            if info["logical_block_size"] != 2048:
                info["inspection_status"] = "blocked"
                warnings.append(f"Unsupported logical block size {info['logical_block_size']}")
                continue
            root = record(data[156:156 + data[156]], namespace)
            info["root_directory"] = root
            volumes.append((namespace, info, root))
    if not terminated:
        warnings.append("ISO descriptor sequence did not reach terminator")

    checks = []
    container_probe_count = 0
    container_discoveries = []
    for namespace, info, root in volumes:
        base = image_name + "!/" + ("JOLIET/" if namespace == "joliet" else "")
        queue = [(base, root, None)]
        seen = set()
        path_names = set()
        dirs = 0
        files = 0
        structures = 0
        records_seen = 0
        blocked = 0
        directory_extents = set()
        all_extent_bounds_valid = True
        while queue:
            prefix, rec, parent_extent = queue.pop(0)
            identity = (rec["extent_lba"], rec["size"])
            entry = {"path": prefix, "kind": "directory", "size": rec["size"], "sha256": None,
                     "signature": None, "classification": "iso9660_directory", "inspection_status": "inspected", "iso9660": rec}
            entries.append(entry)
            dirs += 1
            if identity in seen:
                entry["inspection_status"] = "blocked"
                entry["inspection_error"] = "Repeated directory extent; traversal stopped to prevent cycle"
                warnings.append(f"Repeated directory extent: {prefix}")
                blocked += 1
                continue
            seen.add(identity)
            directory_extents.add(rec["extent_lba"])
            try:
                buf = disc.extent(rec["extent_lba"] + rec["extended_attribute_blocks"], rec["size"], info["volume_blocks"])
                entry["directory_data_sha256"] = hashlib.sha256(buf).hexdigest()
                directory_records = []
                pos = 0
                while pos < len(buf):
                    length = buf[pos]
                    if not length:
                        next_block = min(((pos // 2048) + 1) * 2048, len(buf))
                        if any(buf[pos:next_block]):
                            raise ValueError(f"Nonzero bytes after zero directory padding at offset {pos}")
                        pos = next_block
                        continue
                    if (pos % 2048) + length > 2048 or pos + length > len(buf):
                        raise ValueError(f"Directory record crosses block/end at offset {pos}")
                    child = record(buf[pos:pos + length], namespace)
                    child["record_offset"] = pos
                    directory_records.append(child)
                    pos += length
                entry["directory_record_count"] = len(directory_records)
                records_seen += len(directory_records)
                for child in directory_records:
                    if child["name"] in (".", ".."):
                        structures += 1
                        expected = rec["extent_lba"] if child["name"] == "." else parent_extent
                        if expected is not None and child["extent_lba"] != expected:
                            warnings.append(f"{prefix}{child['name']} extent disagrees with tree")
                        continue
                    child_path = prefix + child["name"]
                    if child_path in path_names:
                        # ISO multi-extents require concatenation; retain every record even when blocked.
                        warnings.append(f"Duplicate ISO path record: {child_path}")
                    path_names.add(child_path)
                    if child["flags"] & 2:
                        queue.append((child_path + "/", child, rec["extent_lba"]))
                        continue
                    files += 1
                    item = {"path": child_path, "kind": "file", "size": child["size"], "sha256": None,
                            "signature": None, "classification": "unreadable_iso9660_file", "inspection_status": "blocked", "iso9660": child}
                    entries.append(item)
                    try:
                        if child["flags"] & 0x80:
                            raise ValueError("Multi-extent file record requires assembly; not supported")
                        if child["file_unit_size"] or child["interleave_gap"]:
                            raise ValueError("Interleaved file not supported")
                        data = disc.extent(child["extent_lba"] + child["extended_attribute_blocks"], child["size"], info["volume_blocks"])
                        item.update(analyze(data, child["name"]))
                        item["inspection_status"] = "inspected"
                        container_probe_count += 1
                        known_prefixes = ((b"MSCF", "Microsoft cabinet"), (b"SZDD", "Microsoft compressed file"),
                                          (b"ISc(", "InstallShield cabinet"), (b"Rar!", "RAR"),
                                          (b"\x1f\x8b", "gzip"), (b"7z\xbc\xaf\x27\x1c", "7z"))
                        container_format = next((label for magic, label in known_prefixes if data.startswith(magic)), None)
                        if zipfile.is_zipfile(io.BytesIO(data)):
                            container_format = "ZIP (including possible self-extracting overlay)"
                        if container_format:
                            item["nested_container_inspection"] = {"format": container_format, "status": "blocked", "reason": "Nested container detected; this disc parser inventories ISO content only"}
                            container_discoveries.append({"path": child_path, "format": container_format})
                            warnings.append(f"Nested container remains unexpanded: {child_path}")
                    except Exception as exc:
                        item["inspection_error"] = str(exc)
                        warnings.append(f"{child_path}: {exc}")
                        blocked += 1
                        if "extent" in str(exc).lower():
                            all_extent_bounds_valid = False
            except Exception as exc:
                entry["inspection_status"] = "blocked"
                entry["inspection_error"] = str(exc)
                warnings.append(f"{prefix}: {exc}")
                blocked += 1
                if "extent" in str(exc).lower():
                    all_extent_bounds_valid = False

        def parse_path_table(lba, endian):
            raw = disc.extent(lba, info["path_table_size"], info["volume_blocks"])
            result = []
            p = 0
            while p < len(raw):
                n = raw[p]
                if not n or p + 8 + n + n % 2 > len(raw):
                    raise ValueError(f"Invalid path table record at {p}")
                ident = raw[p + 8:p + 8 + n]
                result.append({"extent_lba": int.from_bytes(raw[p + 2:p + 6], endian),
                               "parent_index": int.from_bytes(raw[p + 6:p + 8], endian), "identifier_hex": ident.hex()})
                p += 8 + n + n % 2
            return result

        path_table_check = {}
        try:
            little = parse_path_table(info["little_endian_path_table_lba"], "little")
            big = parse_path_table(info["big_endian_path_table_lba"], "big")
            path_table_check = {"little_big_tables_match": little == big, "path_table_directory_count": len(little),
                                "directory_tree_matches_path_table_extents": directory_extents == {r["extent_lba"] for r in little},
                                "entries": little}
            if little != big or not path_table_check["directory_tree_matches_path_table_extents"]:
                warnings.append(f"{namespace} path table comparison failed")
        except Exception as exc:
            path_table_check = {"inspection_status": "blocked", "error": str(exc)}
            warnings.append(f"{namespace} path table: {exc}")
        checks.append({"namespace": namespace, "directories": dirs, "files": files,
                       "directory_records": records_seen, "dot_and_dotdot_records": structures,
                       "member_record_accounting_matches": records_seen == files + dirs - 1 + structures,
                       "all_file_and_directory_extents_in_bounds": all_extent_bounds_valid,
                       "blocked_entries": blocked, "path_table_check": path_table_check})
    return {"volume_descriptors": descriptors, "descriptor_terminator_found": terminated,
            "namespace_checks": checks, "mode1_sector_headers_validated": len(disc.checked),
            "invalid_sector_headers": sorted(disc.bad),
            "nested_container_probe": {"files_checked": container_probe_count, "discoveries": container_discoveries,
                                       "checks": ["ZIP end-of-central-directory probe, including MZ overlays", "MSCF", "SZDD", "ISc(", "Rar!", "gzip", "7z"],
                                       "scope": "Known container signatures only; game resource compression is not recursively decoded."},
            "limitations": ["ECC/EDC and subchannel decoding not performed; validated MODE1 sync/mode headers only.",
                            "Filesystem timestamps and identifiers are recorded evidence, not independently established release identity."]}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--archive", type=Path, default=ROOT / "private/disc-package.7z")
    parser.add_argument("--output", type=Path, default=ROOT / "research/disc-inventory.json")
    parser.add_argument("--reuse-verified", action="store_true", help="Reuse scratch only when size and SHA-256 match this output's prior inventory and source archive hash matches")
    args = parser.parse_args()
    scratch = ROOT / "private/disc"
    scratch.mkdir(parents=True, exist_ok=True)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    commands = []
    extraction_actions = []
    warnings = []
    entries = []
    source_hash = hash_file(args.archive)
    cached = {}
    if args.reuse_verified and args.output.exists():
        previous = json.loads(args.output.read_text(encoding="utf-8"))
        if previous.get("source_archive_sha256") == source_hash:
            cached = {e["path"]: e for e in previous["entries"] if "!/" not in e["path"]}
    command = [str(TAR), "-tvf", str(args.archive)]
    commands.append(command)
    listing = subprocess.run(command, capture_output=True, check=True).stdout.decode("utf-8")
    members = []
    for line in listing.splitlines():
        match = re.match(r"^(?P<permissions>\S+)\s+\d+\s+\S+\s+\S+\s+(?P<size>\d+)\s+\S+\s+\d+\s+\S+\s+(?P<name>.+)$", line)
        if not match:
            raise ValueError(f"Unparsed archive listing line: {line!r}")
        members.append((match["name"], int(match["size"]), match["permissions"].startswith("d")))
    extracted = {}
    for index, (name, size, directory) in enumerate(members):
        item = {"path": name, "kind": "directory" if directory else "file", "size": size,
                "sha256": None, "signature": None, "classification": "archive_directory", "inspection_status": "inspected"}
        entries.append(item)
        if directory:
            continue
        dest = scratch / (f"member-{index:03d}" + Path(name).suffix.lower())
        command = [str(TAR), "-xOf", str(args.archive), name]
        reuse = (name in cached and dest.exists() and dest.stat().st_size == size
                 and hash_file(dest) == cached[name].get("sha256"))
        extraction_actions.append({"member": name, "scratch_path": str(dest), "action": "reused_after_source_and_member_hash_verification" if reuse else "extracted_with_tar_xOf", "command": command})
        print(f"{'Verified cached' if reuse else 'Extracting'} {name} -> {dest.name}", flush=True)
        if not reuse:
            commands.append(command)
            with dest.open("wb") as out:
                proc = subprocess.run(command, stdout=out, stderr=subprocess.PIPE)
            if proc.returncode:
                item.update({"inspection_status": "blocked", "classification": "unreadable_archive_member",
                             "inspection_error": proc.stderr.decode("utf-8", "replace")})
                warnings.append(f"Extraction failed: {name}")
                continue
        if dest.stat().st_size != size:
            item.update({"inspection_status": "blocked", "inspection_error": "Extracted size disagrees with listing"})
            continue
        extracted[Path(name).suffix.lower()] = (name, dest)
        if size < 32 * 1024 * 1024 and Path(name).suffix.lower() != ".sub":
            data = dest.read_bytes()
            item.update(analyze(data, name))
            if Path(name).suffix.lower() in (".cue", ".ccd", ".sfv"):
                item["text_content"] = data.decode("utf-8", "replace")
            if Path(name).suffix.lower() == ".log":
                item["selected_text_claims_unverified"] = [line for line in data.decode("utf-8", "replace").splitlines()
                                                           if any(key in line for key in ("Time stamp of this log", "Reading Mode:", "Source Info:", "Disc dumping completed"))]
        else:
            with dest.open("rb") as f:
                signature = f.read(32).hex()
            item.update({"sha256": hash_file(dest), "signature": {"prefix_hex": signature, "detected": "raw MODE1/audio sectors" if name.endswith(".img") else "opaque CD subchannel bytes"},
                         "classification": "raw_cd_image" if name.endswith(".img") else "raw_cd_subchannel_data"})
    image_metadata = {}
    try:
        cue = extracted[".cue"][1].read_text()
        ccd = configparser.ConfigParser()
        ccd.read(extracted[".ccd"][1])
        image_name, image_path = extracted[".img"]
        total_sectors, remainder = divmod(image_path.stat().st_size, 2352)
        if remainder:
            raise ValueError("Image size is not divisible by 2352-byte sectors")
        tracks = []
        current = None
        for line in cue.splitlines():
            match = re.match(r"\s*TRACK\s+(\d+)\s+(\S+)", line)
            if match:
                current = {"number": int(match[1]), "type": match[2]}
                tracks.append(current)
            match = re.match(r"\s*INDEX\s+(\d+)\s+(\d+):(\d+):(\d+)", line)
            if match and current is not None:
                current[f"index_{int(match[1])}_lba"] = (int(match[2]) * 60 + int(match[3])) * 75 + int(match[4])
        for n, track in enumerate(tracks):
            section = ccd[f"TRACK {track['number']}"]
            ccd_start = int(section["INDEX 1"])
            track["ccd_index_1_lba"] = ccd_start
            track["cue_ccd_index_1_match"] = ccd_start == track["index_1_lba"]
            track["ccd_mode"] = int(section["MODE"])
            track["mode_agrees"] = (track["type"] == "MODE1/2352" and track["ccd_mode"] == 1) or (track["type"] == "AUDIO" and track["ccd_mode"] == 0)
            track["ccd_index_0_lba"] = int(section["INDEX 0"]) if "INDEX 0" in section else None
            track["end_lba_exclusive"] = tracks[n + 1]["index_1_lba"] if n + 1 < len(tracks) else total_sectors
            track["raw_byte_offset"] = track["index_1_lba"] * 2352
            track["raw_byte_size"] = (track["end_lba_exclusive"] - track["index_1_lba"]) * 2352
            track["raw_sha256"] = hash_file(image_path, track["raw_byte_offset"], track["raw_byte_size"])
            track["classification"] = "filesystem_data_track_plus_following_pregap_if_any" if track["type"].startswith("MODE") else "red_book_audio_track_raw_samples"
        leadouts = [int(ccd[s]["PLBA"]) for s in ccd.sections() if s.startswith("Entry ") and ccd[s].get("Point", "").lower() == "0xa2"]
        image_metadata = {"raw_sector_size": 2352, "total_sectors": total_sectors, "ccd_leadout_lbas": leadouts,
                          "ccd_leadout_matches_image_length": leadouts == [total_sectors],
                          "tracks": tracks, "data_payload_offset": 16, "data_payload_bytes_per_sector": 2048,
                          "track_hash_partition_note": "Track hashes use consecutive CUE INDEX 01 boundaries and partition the whole raw image. CCD audio INDEX 00 pregap is separately noted and excluded from ISO data bounds."}
        sub_name, sub_path = extracted[".sub"]
        image_metadata["subchannel"] = {"path": sub_name, "bytes_per_sector": 96,
                                        "size_matches_96_bytes_per_image_sector": sub_path.stat().st_size == total_sectors * 96,
                                        "inspection_status": "hashed_opaque_not_decoded"}
        sfv_checks = []
        for line in extracted[".sfv"][1].read_text().splitlines():
            match = re.fullmatch(r"(.+?)\s+([0-9A-Fa-f]{8})", line)
            if match and not line.startswith(";"):
                target = extracted.get(Path(match[1]).suffix.lower())
                if target is None or Path(target[0]).name != match[1]:
                    sfv_checks.append({"name": match[1], "inspection_status": "blocked", "reason": "Referenced SFV member not found"})
                    warnings.append(f"SFV references unknown member {match[1]}")
                    continue
                crc = 0
                with target[1].open("rb") as f:
                    for chunk in iter(lambda: f.read(1024 * 1024), b""):
                        crc = zlib.crc32(chunk, crc)
                actual = f"{crc:08X}"
                sfv_checks.append({"name": match[1], "claimed_crc32": match[2].upper(), "observed_crc32": actual, "matches": actual == match[2].upper()})
                if actual != match[2].upper():
                    warnings.append(f"SFV CRC mismatch for {match[1]}")
        image_metadata["sfv_checks"] = sfv_checks
        if not all(t["cue_ccd_index_1_match"] and t["mode_agrees"] for t in tracks):
            raise ValueError("CUE and CCD track metadata disagree")
        if len(tracks) < 2 or tracks[0]["type"] != "MODE1/2352":
            raise ValueError("Only single initial MODE1/2352 data track followed by audio is supported")
        data_end = tracks[1]["ccd_index_0_lba"] or tracks[1]["index_1_lba"]
        image_metadata["data_track_end_before_audio_pregap_lba"] = data_end
        if tracks[1]["ccd_index_0_lba"] is not None:
            start = tracks[1]["ccd_index_0_lba"]
            length = tracks[1]["index_1_lba"] - start
            image_metadata["audio_pregap"] = {"start_lba": start, "sector_count": length, "raw_sha256": hash_file(image_path, start * 2352, length * 2352)}
        disc = Disc(image_path, tracks[0]["index_1_lba"], data_end, warnings)
        try:
            image_metadata["iso9660"] = inventory_iso(disc, image_name, entries, warnings)
            primary = next((d for d in image_metadata["iso9660"]["volume_descriptors"] if d.get("namespace") == "iso9660"), None)
            if primary:
                volume_end = tracks[0]["index_1_lba"] + primary["volume_blocks"]
                tail_sectors = data_end - volume_end
                image_metadata["outside_declared_iso_volume_before_audio_pregap"] = {
                    "start_lba": volume_end, "sector_count": tail_sectors,
                    "raw_sha256": hash_file(image_path, volume_end * 2352, tail_sectors * 2352) if tail_sectors >= 0 else None,
                    "classification": "raw_disc_tail_outside_declared_filesystem_not_interpreted"}
            iso_checks = image_metadata["iso9660"]
            complete = (iso_checks["descriptor_terminator_found"] and bool(iso_checks["namespace_checks"])
                        and not iso_checks["invalid_sector_headers"] and not warnings
                        and all(c["blocked_entries"] == 0 and c["member_record_accounting_matches"]
                                and c["all_file_and_directory_extents_in_bounds"]
                                and c["path_table_check"].get("little_big_tables_match")
                                and c["path_table_check"].get("directory_tree_matches_path_table_extents")
                                for c in iso_checks["namespace_checks"]))
            image_metadata["inspection_status"] = "inspected" if complete else "partially_inspected"
            image_metadata["inspection_scope"] = "Archive members hashed, raw track boundaries/lengths/hashes recorded, ISO9660 files structurally validated and hashed; audio and subchannels are opaque, not decoded."
        finally:
            disc.f.close()
    except Exception as exc:
        warnings.append(f"Disc filesystem inspection blocked: {exc}")
        image_metadata["inspection_status"] = "blocked"
        image_metadata["inspection_error"] = str(exc)

    final_source_hash = hash_file(args.archive)
    for entry in entries:
        if "iso9660" in entry:
            entry["normalized_iso_path"] = re.sub(r";\d+(?=/|$)", "", entry["path"].split("!/", 1)[1])
    if final_source_hash != source_hash:
        warnings.append("Source archive hash changed during inspection")
        image_metadata["inspection_status"] = "partially_inspected"
    output = {"schema_version": 1, "source_archive": str(args.archive), "source_archive_sha256": source_hash,
              "source_archive_size": args.archive.stat().st_size,
              "source_archive_unchanged": final_source_hash == source_hash,
              "archive_listing": listing, "entries": entries, "image_metadata": image_metadata,
              "warnings": warnings, "commands": commands,
              "extraction_actions": extraction_actions,
              "self_check": {"listed_archive_members": len(members), "inventoried_archive_members": sum("!/" not in e["path"] for e in entries),
                             "all_archive_members_accounted_for": len(members) == sum("!/" not in e["path"] for e in entries),
                             "file_count": sum(e["kind"] == "file" for e in entries),
                             "directory_count": sum(e["kind"] == "directory" for e in entries),
                             "blocked_entry_count": sum(e["inspection_status"] == "blocked" for e in entries),
                             "all_inspected_files_have_sha256": all(e.get("sha256") for e in entries if e["kind"] == "file" and e["inspection_status"] == "inspected")}}
    args.output.write_text(json.dumps(output, indent=2, ensure_ascii=False), encoding="utf-8")
    print(json.dumps(output["self_check"], indent=2), flush=True)
    print(f"Wrote {args.output}; warnings={len(warnings)}", flush=True)


if __name__ == "__main__":
    main()
