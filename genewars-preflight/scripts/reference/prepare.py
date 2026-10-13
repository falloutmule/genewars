"""Stage the reviewed private reference environment. Never executes any binary.

Default action is verification only. --stage performs bounded local copies;
--download additionally obtains the pinned official emulator archive.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import shutil
import stat
import sys
import urllib.request
import zipfile

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'scripts'))
from source_member import SourceReader, digest

VERSION = '2026.10.01'
ASSET = f'dosbox-x-vsbuild-win64-{VERSION}-portable.zip'
URL = f'https://github.com/joncampbell123/dosbox-x/releases/download/dosbox-x-v{VERSION}/{ASSET}'
PACKAGE_SHA = '3c50915e63aefe35777d65ebb4391a06b4a2c2342e4c75cbee6bbda4c6916e33'
TOOLS = ROOT / 'private/tools/dosbox-x'
EXTRACT = TOOLS / VERSION
SDL2 = EXTRACT / 'bin/x64/Release SDL2'
RUN = ROOT / 'private/reference-run/run-001'
RECORD = ROOT / 'research/variants/reference-readiness.json'
SESSION = 'run-001'
HUMAN_APPROVAL = ROOT / 'research/variants/reference-approval.json'
EXTRA_GAME_INPUTS = {}
COPIED_GAME_INPUTS = {}
SESSION_WRAPPERS = {'run-002':'run002.py','run-003':'run003.py','run-004':'run004.py','run-005':'run005.py'}
# Independently reviewed five listings, mapped strings/tables, and all19aliases.
TEXT_INSTALL_PROOF_PATH = 'research/ghidra/english-install-evidence.json'
TEXT_INSTALL_PROOF_SHA256 = '541c851fe27e78c78b2bebe83b1c33e848fc39c40aa82ef014c1e16a02f47970'
ENGLISH_TEXT_ENTRY = 'root/2/disc/592'
ENGLISH_TEXT_SHA256 = '23e60305a9fa028a0f6541456ab453b7e7cc439f787a2b75e4b8d088e0b01786'


def select_session(name):
    """Select a fixed reviewed context inside this Python process only."""
    global SESSION, RUN, RECORD, HUMAN_APPROVAL, EXTRA_GAME_INPUTS, COPIED_GAME_INPUTS
    if name not in SESSION_WRAPPERS:
        raise ValueError('Only explicitly reviewed session contexts can be selected')
    SESSION = name
    RUN = ROOT / 'private/reference-run' / name
    RECORD = ROOT / 'research/variants' / ('reference-'+name+'-readiness.json')
    HUMAN_APPROVAL = ROOT / 'research/variants' / ('reference-'+name+'-approval.json')
    COPIED_GAME_INPUTS = {}
    if name in ('run-003','run-004','run-005'):
        proof_path = ROOT / TEXT_INSTALL_PROOF_PATH
        if TEXT_INSTALL_PROOF_SHA256 is None or not proof_path.exists() or digest(proof_path) != TEXT_INSTALL_PROOF_SHA256:
            raise ValueError('Run003 text-copy staging awaits independently reviewed installer evidence')
        proof = json.loads(proof_path.read_bytes())
        for item in proof['private_listings']:
            if digest(ROOT/item['private_path']) != item['sha256']:
                raise ValueError('Text-install instruction evidence changed')
        COPIED_GAME_INPUTS = {'GAME/DATA/TEXT.DAT':dict(source_entry_id=ENGLISH_TEXT_ENTRY,
            sha256=ENGLISH_TEXT_SHA256,size=36610,evidence=file_record(proof_path))}
        manifest = json.loads((ROOT/'research/game-files-manifest.json').read_bytes())
        speech = [entry for entry in manifest['entries'] if entry['kind']=='file'
            and entry['entry_id'].startswith('root/2/disc/')
            and entry.get('normalized_iso_path','').startswith('GAME/SPEECH/ENG/')]
        if len(speech)!=18 or sum(entry['size'] for entry in speech)!=19166106:
            raise ValueError('Unexpected exact English speech source set')
        for entry in speech:
            source = PurePosixPath(entry['normalized_iso_path'])
            if len(source.parts)!=4:
                raise ValueError('Installer wildcard admits direct English speech files only')
            destination = 'GAME/SPEECH/'+source.name
            if destination.casefold() in {name.casefold() for name in COPIED_GAME_INPUTS}:
                raise ValueError('Flattened runtime alias collision')
            COPIED_GAME_INPUTS[destination] = dict(source_entry_id=entry['entry_id'],
                sha256=entry['sha256'],size=entry['size'],evidence=file_record(proof_path))
        admitted = {(row['entry_id'],'GAME/'+row['destination'],row['size'],row['sha256']) for row in proof['copies']}
        prepared = {(row['source_entry_id'],destination,row['size'],row['sha256']) for destination,row in COPIED_GAME_INPUTS.items()}
        if admitted != prepared:
            raise ValueError('Runtime aliases differ from the reviewed installer receipt')
    # Independent generated text following the exact setup writer's eight
    # quoted fields. These are preparation paths, not inferred game defaults.
    pairs = [('CD','D:\\GAME\\'),('InstallDrive','C:\\GAME\\'),('Language','ENGLISH'),
             ('Intro','All'),('Anims','All'),('Sounds','All'),('Levels','All'),('Data','All')]
    evidence = json.loads((ROOT/'research/ghidra/config-evidence.json').read_bytes())
    if [key for key,value in pairs] != evidence['keys'] or ['All']*5 not in evidence['category_choices']:
        raise ValueError('Static configuration contract changed; review before preparation')
    for item in evidence['private_listings']:
        if digest(ROOT/item['private_path']) != item['sha256']:
            raise ValueError('Configuration instruction evidence changed')
    text = ''.join(f'{key}="{value}"\r\n' for key,value in pairs).encode('ascii')
    if any(len(line) >= 79 for line in text.splitlines()) or len(text.splitlines()) != 8:
        raise ValueError('Generated configuration violates parser line limit')
    EXTRA_GAME_INPUTS = {'GAME/config.ini': text}


def wrapper_command(action):
    wrapper = SESSION_WRAPPERS.get(SESSION)
    if wrapper is None:
        return 'python -B scripts/reference/'+action+'.py'
    return 'python -B scripts/reference/'+wrapper+' '+action


def effective_overrides():
    overrides = {section:dict(values) for section,values in OVERRIDES.items()}
    if SESSION in ('run-003','run-004'):
        overrides['sdl']['mouse_emulation'] = 'always'
    if SESSION == 'run-004':
        overrides['sdl']['raw_mouse_input'] = 'true'
    if SESSION == 'run-005':
        overrides['sdl']['mouse_emulation'] = 'never'
        overrides['sdl']['raw_mouse_input'] = 'false'
    return overrides


def runtime_alias_binding(rows):
    """Bind the complete source/destination set, independent of session folder."""
    normalized = []
    for row in rows:
        relative = row.get('runtime_relative_path')
        if relative is None:
            relative = row['path'].split('/drive-c/',1)[1]
        normalized.append(dict(source_entry_id=row['source_entry_id'],sha256=row['sha256'],size=row['size'],runtime_relative_path=relative))
    normalized.sort(key=lambda row:row['runtime_relative_path'].casefold())
    return __import__('hashlib').sha256(json.dumps(normalized,sort_keys=True,separators=(',',':')).encode()).hexdigest()


OVERRIDES = {
    'sdl': {'fullscreen': 'false', 'output': 'surface', 'autolock': 'false',
            'clip_mouse_button': 'none', 'mapperfile': 'logs/mapper.map',
            'mapperfile_sdl2': 'logs/mapper.map', 'waitonerror': 'false'},
    'log': {'logfile': 'logs/dosbox-x.log'},
    'dosbox': {'machine': 'svga_s3', 'memsize': '64', 'captures': 'captures',
               'mcp_server': '0', 'call binary on reset': '', 'call binary on boot': '',
               'video debug at startup': 'false', 'working directory option': 'noprompt',
               'working directory default': ''},
    'script': {'startup.js': ''},
    'render': {'frameskip': '0', 'aspect': 'false', 'scaler': 'none'},
    'video': {'enable supermegazeux tweakmode': 'false'},
    'cpu': {'core': 'normal', 'cputype': 'auto', 'cycles': 'fixed 30000'},
    'dos': {'automount': 'false', 'automountall': 'false',
            'automount drive directories': 'false', 'network redirector': 'false',
            'startcmd': 'false', 'starttranspath': 'false', 'vmware': 'false',
            'dos clipboard device enable': 'false', 'dos clipboard api': 'false',
            'log console': 'true'},
    'ipx': {'ipx': 'false'},
    'ne2000': {'ne2000': 'false', 'backend': 'none'},
    'ethernet, slirp': {'restricted': 'true', 'disable_host_loopback': 'true',
                       'tcp_port_forwards': '', 'udp_port_forwards': ''},
    'serial': {f'serial{i}': 'disabled' for i in range(1, 10)},
    'parallel': {f'parallel{i}': 'disabled' for i in range(1, 10)},
    'printer': {'printer': 'false'},
    'midi': {'mpu401': 'none', 'mididevice': 'none'},
    'sblaster': {'recording source': 'silence', 'listen to recording source': 'false'},
    'sblaster2': {'recording source': 'silence', 'listen to recording source': 'false'},
}


def checked(path: Path, base: Path) -> Path:
    """Reject traversal and existing junctions/symlinks before writes."""
    path, base = path.absolute(), base.absolute()
    if not path.is_relative_to(base):
        raise ValueError(f'Path escapes owned directory: {path}')
    for part in (path, *path.parents):
        if part.exists():
            flags = getattr(part.lstat(), 'st_file_attributes', 0)
            if part.is_symlink() or flags & stat.FILE_ATTRIBUTE_REPARSE_POINT:
                raise ValueError(f'Reparse point forbidden: {part}')
        if part == ROOT:
            break
    if not path.resolve().is_relative_to(base.resolve()):
        raise ValueError('Resolved path escapes owned directory')
    if not base.resolve().is_relative_to((ROOT / 'private').resolve()):
        raise ValueError('Owned base must stay beneath project private/')
    return path


def ensure_bytes(path, data, base):
    path = checked(path, base)
    if path.exists():
        if path.read_bytes() != data:
            raise ValueError(f'Refusing to overwrite changed file: {path}')
    else:
        path.parent.mkdir(parents=True, exist_ok=True)
        with path.open('xb') as stream:
            stream.write(data)


def file_record(path):
    return {'path': path.relative_to(ROOT).as_posix(), 'size': path.stat().st_size,
            'sha256': digest(path)}


def prepare_tool(download, stage):
    archive = checked(TOOLS / ASSET, TOOLS)
    if download and not archive.exists():
        TOOLS.mkdir(parents=True, exist_ok=True)
        temporary = checked(TOOLS / (ASSET + '.partial'), TOOLS)
        with urllib.request.urlopen(URL, timeout=60) as response, temporary.open('xb') as out:
            shutil.copyfileobj(response, out, 1024 * 1024)
        if digest(temporary) != PACKAGE_SHA:
            raise ValueError('Downloaded archive digest mismatch; partial retained for inspection')
        temporary.rename(archive)
    if archive.stat().st_size != 44575207 or digest(archive) != PACKAGE_SHA:
        raise ValueError('Pinned official archive size/digest mismatch')
    with zipfile.ZipFile(archive) as package:
        members = package.infolist()
        if sum(m.file_size for m in members) > 128 * 1024 * 1024:
            raise ValueError('Unexpected package expansion size')
        destinations = set()
        for member in members:
            rel = PurePosixPath(member.filename)
            if rel.is_absolute() or any(p in ('..', '.') or ':' in p or '\\' in p for p in rel.parts):
                raise ValueError('Unsafe ZIP member')
            if stat.S_ISLNK(member.external_attr >> 16):
                raise ValueError('ZIP symlink forbidden')
            target = checked(EXTRACT.joinpath(*rel.parts), TOOLS)
            if member.is_dir():
                continue
            folded = str(target).casefold()
            if folded in destinations:
                raise ValueError('Case-insensitive ZIP collision')
            destinations.add(folded)
            data = package.read(member)
            if stage:
                ensure_bytes(target, data, TOOLS)
            elif not target.exists() or target.read_bytes() != data:
                raise ValueError('Tool extraction missing or changed')
    return {'version': VERSION, 'url': URL, 'archive': file_record(archive),
            'executable': file_record(SDL2 / 'dosbox-x.exe'),
            'reference_config': file_record(SDL2 / 'dosbox-x.reference.full.conf'),
            'license': file_record(EXTRACT / 'COPYING'), 'members_verified': len(destinations),
            'license_note': 'Packaged COPYING is GNU GPL v2 text. Private tool use only; no redistribution or source copying into reconstruction.'}


def make_config():
    """Pin every packaged parameter; strip comments and executable sections."""
    sections, current = {}, None
    for raw in (SDL2 / 'dosbox-x.reference.full.conf').read_text(encoding='utf-8-sig').splitlines():
        line = raw.strip()
        if not line or line.startswith(('#', ';')):
            continue
        if line.startswith('[') and line.endswith(']'):
            current = line[1:-1].lower()
            if current in sections:
                raise ValueError('Duplicate reference section')
            sections[current] = {}
        elif current not in ('autoexec', 'config', '4dos', 'devices'):
            key, sep, value = line.partition('=')
            if not sep or current is None or key.strip().lower() in sections[current]:
                raise ValueError('Unexpected reference configuration syntax')
            sections[current][key.strip().lower()] = value.strip()
    for section, settings in effective_overrides().items():
        for key, value in settings.items():
            if key not in sections[section]:
                # Packaged full reference omits this SDL2-only key. The pinned
                # release sdlmain.cpp6950 explicitly registers it, defaultfalse,
                # OnlyAtStart; admit exactly this reviewed run004 diagnostic.
                if not (section=='sdl' and key=='raw_mouse_input' and
                        ((SESSION=='run-004' and value=='true') or (SESSION=='run-005' and value=='false'))):
                    raise ValueError(f'Unsupported override {section}/{key}')
            sections[section][key] = value
    lines = ['# Private pinned reference setup. No startup/mount/game commands.']
    for section, settings in sections.items():
        lines.append(f'[{section}]')
        lines.extend(f'{key} = {value}' for key, value in settings.items())
        lines.append('')
    return ('\n'.join(lines) + '\n').encode('utf-8'), sum(map(len, sections.values()))


def prepare_run(stage):
    if stage and (RUN / 'logs/human-approval.json').exists():
        raise ValueError('Run has approval/session record; preserve it and stage a separately reviewed new run')
    rows = []
    copied = []
    with SourceReader() as reader:
        entries = [e for e in reader.entries.values() if e['kind'] == 'file'
                   and e.get('normalized_iso_path', '').startswith('GAME/')
                   and e['entry_id'].startswith('root/2/disc/')]
        if len(entries) != 490 or sum(e['size'] for e in entries) != 186368166:
            raise ValueError('Expected exactly the established 490-file CD GAME tree')
        names = set()
        for entry in entries:
            rel = PurePosixPath(entry['normalized_iso_path'])
            if rel.is_absolute() or '..' in rel.parts or any(':' in p or '\\' in p for p in rel.parts):
                raise ValueError('Unsafe normalized ISO path')
            if str(rel).casefold() in names:
                raise ValueError('DOS filename collision')
            names.add(str(rel).casefold())
            target = checked(RUN / 'drive-c' / Path(*rel.parts), RUN)
            if stage:
                ensure_bytes(target, reader.read(entry['entry_id']), RUN)
            if target.stat().st_size != entry['size'] or digest(target) != entry['sha256']:
                raise ValueError(f'Staged file mismatch: {entry["entry_id"]}')
            rows.append({'entry_id': entry['entry_id'], **file_record(target)})
        for rel,contract in COPIED_GAME_INPUTS.items():
            if rel.casefold() in names:
                raise ValueError('Flattened alias collides with an original CD path')
            names.add(rel.casefold())
            entry = reader.entries[contract['source_entry_id']]
            if (entry['size'],entry['sha256']) != (contract['size'],contract['sha256']):
                raise ValueError('Reviewed runtime-copy source changed')
            target = checked(RUN/'drive-c'/rel,RUN)
            if stage:
                ensure_bytes(target,reader.read(entry['entry_id']),RUN)
            if target.stat().st_size != entry['size'] or digest(target) != entry['sha256']:
                raise ValueError('Reviewed flattened text copy changed')
            copied.append(dict(source_entry_id=entry['entry_id'],runtime_relative_path=rel,**file_record(target),
                provenance='Byte-identical source copy to statically proved installed runtime path',evidence=contract['evidence']))
        media = []
        for entry_id, source, name in (
            ('root/2/disc/3', ROOT / 'private/disc/member-003.cue', 'GENEWARS.cue'),
            ('root/2/disc/4', ROOT / 'private/disc/member-004.img', 'GENEWARS.img')):
            entry = reader.entries[entry_id]
            if source.stat().st_size != entry['size'] or digest(source) != entry['sha256']:
                raise ValueError('Original private disc source mismatch')
            target = checked(RUN / 'media' / name, RUN)
            if stage and not target.exists():
                target.parent.mkdir(parents=True, exist_ok=True)
                with source.open('rb') as inp, target.open('xb') as out:
                    shutil.copyfileobj(inp, out, 1024 * 1024)
            if target.stat().st_size != entry['size'] or digest(target) != entry['sha256']:
                raise ValueError('Copied media mismatch')
            media.append({'entry_id': entry_id, **file_record(target)})
        archive = {'sha256': reader.manifest['source']['sha256_before'],
                   'revalidated_by': 'SourceReader constructor'}
    # Only the exact expected files may be visible in C: before approval.
    expected = {row['path'] for row in rows+copied}
    generated = []
    for rel,data in EXTRA_GAME_INPUTS.items():
        target = checked(RUN/'drive-c'/rel,RUN)
        if stage:
            ensure_bytes(target,data,RUN)
        if target.read_bytes() != data:
            raise ValueError('Generated game configuration changed')
        info = file_record(target)
        info['provenance'] = 'Independently generated from exact static setup writer/parser contract'
        generated.append(info)
        expected.add(info['path'])
    approval_path = RUN / 'logs/human-approval.json'
    if approval_path.exists():
        approval = json.loads(approval_path.read_text(encoding='utf-8'))
        gate_info = approval.get('gate')
        if gate_info:
            gate_path = checked(ROOT / gate_info['path'], RUN)
            if digest(gate_path) != gate_info['sha256']:
                raise ValueError('Session gate changed')
            expected.add(gate_info['path'])
        helper_info = approval.get('yield_helper')
        if helper_info:
            from input_free_yield import CODE
            helper_path = checked(ROOT / helper_info['path'], RUN)
            if helper_path != RUN/'drive-c/YIELD.COM' or helper_path.read_bytes()!=CODE or file_record(helper_path)!=helper_info:
                raise ValueError('Input-free yield helper changed')
            expected.add(helper_info['path'])
    actual = {checked(p, RUN).relative_to(ROOT).as_posix()
              for p in (RUN / 'drive-c').rglob('*') if p.is_file()}
    if actual != expected:
        raise ValueError('Unexpected extra/missing file in DOS C: tree')
    config, count = make_config()
    if stage:
        for folder in ('captures', 'states', 'logs'):
            checked(RUN / folder, RUN).mkdir(parents=True, exist_ok=True)
        config_path = checked(RUN / 'dosbox-x.conf', RUN)
        if config_path.exists() and config_path.read_bytes() != config:
            # Permit only review-time regeneration of our own unchanged config.
            prior = json.loads(RECORD.read_text(encoding='utf-8'))
            if prior['original_execution'] or digest(config_path) != prior['runtime']['config']['sha256']:
                raise ValueError('Refusing to replace a changed/used configuration')
            config_path.write_bytes(config)
        else:
            ensure_bytes(config_path, config, RUN)
        ensure_bytes(RUN / 'logs/before-files.json', (json.dumps(rows+generated+copied, indent=2)+'\n').encode(), RUN)
    if (RUN / 'dosbox-x.conf').read_bytes() != config:
        raise ValueError('Pinned config mismatch')
    return {'files': rows, 'file_count': len(rows), 'game_bytes': sum(r['size'] for r in rows),
            'generated_game_inputs': generated,
            'copied_game_inputs': copied,
            'media': media, 'source_archive': archive, 'config': file_record(RUN / 'dosbox-x.conf'),
            'config_parameter_count': count, 'config_overrides': effective_overrides(),
            'executable_sections_empty': ['autoexec', 'config', '4dos', 'devices']}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--stage', action='store_true', help='Create/verify private local copies; never execute')
    parser.add_argument('--download', action='store_true', help='Download official pinned archive if absent; implies --stage')
    args = parser.parse_args()
    stage = args.stage or args.download
    tool = prepare_tool(args.download, stage)
    runtime = prepare_run(stage)
    empty_receipt_path = ROOT / 'private/reference-run/empty-validation/logs/receipt.json'
    validation = None
    if empty_receipt_path.exists():
        receipt = json.loads(empty_receipt_path.read_text())
        if (receipt['config']['sha256'] == runtime['config']['sha256']
                and receipt['tool']['sha256'] == tool['executable']['sha256']
                and receipt.get('effective_queries_passed')
                and all(digest(ROOT / b['startup_log']['path']) == b['startup_log']['sha256']
                        for b in receipt['batches'])):
            validation = receipt
    record = {'schema_version': 1, 'status': 'staged-and-hash-verified-no-original-executed',
              'session': SESSION,
              'original_execution': False, 'emulator_execution': validation is not None,
              'emulator_runtime_restrictions_tested': validation is not None,
              'empty_emulator_validation': validation, 'tool': tool, 'runtime': runtime,
              'scope': 'One supervised <=20-minute direct GW.EXE/DOS4GW session only after separate human approval.',
              'security_limit': 'DOSBox controls are not an OS sandbox. Process retains host user permissions.',
              'reproduce': 'python -B scripts/reference/prepare.py --stage',
              'verify_only': 'python -B scripts/reference/prepare.py'}
    if SESSION in SESSION_WRAPPERS:
        record.update(reproduce=wrapper_command('prepare')+' --stage',
                      verify_only=wrapper_command('prepare'),
                      config_evidence=file_record(ROOT/'research/ghidra/config-evidence.json'),
                      human_approval_record=HUMAN_APPROVAL.relative_to(ROOT).as_posix())
        pending = {'run_id':SESSION,'approved':False,'human_statement':None,
                   'candidate_sha256':'4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba',
                   'generated_config_sha256':runtime['generated_game_inputs'][0]['sha256'],
                   'scope':'One separately approved supervised session of exact CD GW.EXE/DOS4GW with generated static-evidence config.ini. No installer/launcher/patch/other variant.',
                   'note':'Preparation does not grant permission. Supervisor must record the applicable human-approved scope.'}
        if runtime['copied_game_inputs']:
            pending['flattened_text_sha256'] = runtime['copied_game_inputs'][0]['sha256']
            pending['runtime_aliases_sha256'] = runtime_alias_binding(runtime['copied_game_inputs'])
            record['text_install_evidence'] = runtime['copied_game_inputs'][0]['evidence']
        if stage and not HUMAN_APPROVAL.exists():
            with HUMAN_APPROVAL.open('x',encoding='utf-8') as stream:
                json.dump(pending,stream,indent=2)
    if stage:
        RECORD.write_text(json.dumps(record, indent=2)+'\n', encoding='utf-8')
    print(json.dumps({'status': record['status'], 'tool': tool, 'file_count': runtime['file_count'],
                      'game_bytes': runtime['game_bytes'], 'media': runtime['media'],
                      'config': runtime['config'], 'config_parameter_count': runtime['config_parameter_count'],
                      'empty_validation_passed': validation is not None}))


if __name__ == '__main__':
    main()
