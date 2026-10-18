"""Static byte inspection only. No target code is loaded or executed."""
from __future__ import annotations

import hashlib
import re
import struct
from pathlib import PurePosixPath


def u16(data: bytes, offset: int) -> int:
    return struct.unpack_from('<H', data, offset)[0]


def u32(data: bytes, offset: int) -> int:
    return struct.unpack_from('<I', data, offset)[0]


def executable_details(data: bytes) -> dict:
    size = len(data)
    result = {'format': 'DOS MZ', 'validation': [], 'file_size': size}
    if size < 64:
        result['validation'].append('truncated MZ header')
        return result
    pages, last_page = u16(data, 4), u16(data, 2)
    dos_size = (pages - 1) * 512 + (last_page or 512) if pages else 0
    result.update(mz_header_bytes=u16(data, 8) * 16,
                  mz_relocation_table_offset=u16(data, 24),
                  mz_declared_image_bytes=dos_size,
                  bytes_after_mz_image=max(0, size - dos_size),
                  mz_entry_cs=u16(data, 22), mz_entry_ip=u16(data, 20))
    off = u32(data, 60)
    result['new_header_offset_candidate'] = off
    if 64 <= off <= size - 2:
        magic = data[off:off + 4]
        result['new_header_signature_hex'] = magic.hex()
        if magic[:2] in (b'LE', b'LX'):
            result['format'] = magic[:2].decode('ascii')
            result['new_header_offset'] = off
            if off + 176 > size:
                result['validation'].append('truncated LE/LX header')
            else:
                result.update(byte_order=data[off + 2], word_order=data[off + 3],
                              format_level=u32(data, off + 4), cpu_type=u16(data, off + 8),
                              os_type=u16(data, off + 10), module_version=u32(data, off + 12),
                              module_flags=u32(data, off + 16), module_pages=u32(data, off + 20),
                              entry_object=u32(data, off + 24), entry_offset=u32(data, off + 28),
                              stack_object=u32(data, off + 32), stack_offset=u32(data, off + 36),
                              page_size=u32(data, off + 40),
                              last_page_bytes_or_page_shift=u32(data, off + 44),
                              object_table_relative_offset=u32(data, off + 64),
                              object_count=u32(data, off + 68),
                              object_page_map_relative_offset=u32(data, off + 72),
                              fixup_page_table_relative_offset=u32(data, off + 104),
                              fixup_record_table_relative_offset=u32(data, off + 108),
                              data_pages_absolute_offset=u32(data, off + 128),
                              nonresident_names_absolute_offset=u32(data, off + 136),
                              nonresident_names_bytes=u32(data, off + 140))
                table = off + result['object_table_relative_offset']
                count = result['object_count']
                if count == 0 or count > 4096 or table < off + 176 or table + count * 24 > size:
                    result['validation'].append('object table overlaps header, outside file, or implausible count')
                else:
                    result['objects'] = [dict(zip(
                        ('virtual_size', 'relocation_base', 'flags', 'page_map_index', 'page_count', 'reserved'),
                        struct.unpack_from('<6I', data, table + i * 24))) for i in range(count)]
                if result['entry_object'] == 0:
                    result['entry_interpretation'] = 'zero entry object; no conventional entry point declared (library/driver semantics not validated)'
                elif result['entry_object'] > count:
                    result['validation'].append('entry object is not in object table')
                if result.get('objects') and 1 <= result['entry_object'] <= count:
                    obj = result['objects'][result['entry_object'] - 1]
                    result['entry_virtual_address_candidate'] = obj['relocation_base'] + result['entry_offset']
                    if result['entry_offset'] >= obj['virtual_size']:
                        result['validation'].append('entry offset exceeds object virtual size')
                if result['data_pages_absolute_offset'] >= size:
                    result['validation'].append('data pages start outside file')
                if result['byte_order'] or result['word_order']:
                    result['validation'].append('non-little-endian LE/LX fields need separate validation')
                # Header/table sanity does not prove relocation semantics or successful loading.
                result['validation_scope'] = 'header and object-table bounds only; relocations not applied'
        elif magic == b'PE\x00\x00':
            result.update(format='PE', new_header_offset=off)
            if off + 24 <= size:
                result.update(machine=u16(data, off + 4), section_count=u16(data, off + 6),
                              coff_timestamp=u32(data, off + 8))
        elif magic[:2] == b'NE':
            result.update(format='NE', new_header_offset=off)
        else:
            result['new_header_interpretation'] = 'candidate does not point to a recognized new executable header'
    else:
        result['new_header_interpretation'] = 'candidate out of bounds; not a valid new-header pointer'
    markers = (b'DOS/4G', b'DOS/32A', b'PMODE', b'PKLITE', b'LZEXE', b'UPX!',
               b'Rar!\x1a\x07', b'PK\x03\x04', b'InstallShield', b'WATCOM', b'Watcom')
    result['markers'] = [{'marker': m.decode('ascii', 'backslashreplace'), 'offset': data.find(m)}
                         for m in markers if data.find(m) >= 0]
    indicators = []
    for match in re.finditer(rb'[\x20-\x7e]{6,200}', data):
        text = match.group().decode('ascii')
        if re.search(r'(?i)genewars|gene wars|bullfrog|version\s*[0-9]|watcom|dos/4g', text):
            indicators.append({'offset': match.start(), 'text': text[:200]})
            if len(indicators) >= 40:
                break
    result['build_strings_unverified'] = indicators
    return result


def analyze_bytes(data: bytes, name: str) -> dict:
    suffix = PurePosixPath(name).suffix.lower()
    base = PurePosixPath(name).name.lower()
    signature = {'prefix_hex': data[:16].hex(), 'detected': 'unknown'}
    classification = 'opaque_data'
    result = {'size': len(data), 'sha256': hashlib.sha256(data).hexdigest(),
              'signature': signature, 'classification': classification}
    if data.startswith(b'MZ'):
        details = executable_details(data)
        signature['detected'] = 'MZ/' + details['format'] if details['format'] != 'DOS MZ' else 'DOS MZ'
        result['executable'] = details
        role = ('game_executable_candidate' if base == 'gw.exe' else
                'launcher_or_game_entry_candidate' if base == 'genewars.exe' else
                'dos_extender_candidate' if base in ('dos4gw.exe', 'dos32a.exe') else
                'sound_setup_candidate' if base == 'setsound.exe' else
                'library_or_driver_candidate' if suffix in ('.dll', '.drv', '.vxd', '.386') else
                'patcher_candidate' if 'patch' in base or base == 'gwp.exe' else 'executable_unknown_role')
        result['classification'] = role
        result['role_evidence'] = 'filename and static header; program not executed'
    elif data.startswith((b'PK\x03\x04', b'PK\x05\x06', b'PK\x07\x08')):
        signature['detected'] = 'ZIP'; result['classification'] = 'archive'
    elif data.startswith(b'7z\xbc\xaf\x27\x1c'):
        signature['detected'] = '7z'; result['classification'] = 'archive'
    elif data.startswith(b'%PDF-'):
        signature['detected'] = 'PDF'; result['classification'] = 'manual_or_document'
    elif data[:3] == b'SMK':
        signature['detected'] = 'Smacker'; result['classification'] = 'video'
    elif data[:3] == b'RNC' and len(data) >= 4:
        signature['detected'] = 'RNC/method-byte-' + str(data[3])
        result['classification'] = 'compressed_resource_not_decoded'
    elif data.startswith(b'RIFF'):
        signature['detected'] = 'RIFF/' + data[8:12].decode('ascii', 'replace')
        result['classification'] = 'audio' if data[8:12] == b'WAVE' else 'riff_container'
    elif data.startswith(b'MThd'):
        signature['detected'] = 'MIDI'; result['classification'] = 'audio'
    elif data.startswith(b'FORM'):
        signature['detected'] = 'IFF/' + data[8:12].decode('ascii', 'replace'); result['classification'] = 'iff_container'
    elif suffix in ('.txt', '.nfo', '.diz', '.log', '.sfv'):
        result['classification'] = 'text_document'
    elif suffix in ('.ini', '.cfg', '.cue', '.ccd'):
        result['classification'] = 'configuration_or_disc_descriptor'
    elif suffix == '.bat':
        result['classification'] = 'batch_script_not_executed'
    elif suffix in ('.c', '.h', '.rc', '.dev', '.o'):
        result['classification'] = 'source_or_build_material'
    elif suffix in ('.dat', '.res', '.spe', '.tab', '.pal', '.anb', '.spr', '.gla', '.bio'):
        result['classification'] = 'game_resource_not_decoded'
    elif suffix in ('.sav', '.rst'):
        result['classification'] = 'save_or_state_candidate'
    elif suffix in ('.mdi', '.dig', '.ad', '.opl', '.sbk', '.xmi', '.mid', '.wav'):
        result['classification'] = 'audio_or_driver_candidate'
    elif suffix in ('.img', '.bin', '.iso'):
        result['classification'] = 'disc_image_candidate'
    elif suffix == '.sub':
        result['classification'] = 'disc_subchannel_candidate'
    if result['classification'] != 'opaque_data' and signature['detected'] == 'unknown':
        result['classification_basis'] = 'filename extension; format not fully decoded'
    return result
