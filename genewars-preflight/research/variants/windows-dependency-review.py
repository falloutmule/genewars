"""Audit actual Windows-GNU normal/build closure using only pinned local cache."""
from datetime import datetime, timezone
import hashlib
import json
import re
from pathlib import Path
import subprocess
import sys
import tarfile
import tomllib

ROOT = Path(__file__).resolve().parents[2]
TARGET = 'x86_64-pc-windows-gnu'
REGISTRY = ROOT/'private/tools/rust/cargo/registry'
PRIVATE = ROOT/'private/review/windows-dependencies'


def sha(data):
    return hashlib.sha256(data).hexdigest()


def recorded(path):
    data = path.read_bytes()
    return dict(path=path.relative_to(ROOT).as_posix(),size=len(data),sha256=sha(data))


def main():
    PRIVATE.mkdir(parents=True,exist_ok=True)
    lock_path = ROOT/'rust/Cargo.lock'
    lock_before = recorded(lock_path)
    command = [sys.executable,'-B',str(ROOT/'scripts/cargo.py'),'metadata','--offline','--locked',
               '--format-version','1','--filter-platform',TARGET]
    run = subprocess.run(command,capture_output=True,timeout=60)
    assert run.returncode == 0, run.stderr.decode(errors='replace')
    metadata = json.loads(run.stdout)
    (PRIVATE/'cargo-metadata.json').write_bytes(run.stdout)
    packages = {v['id']:v for v in metadata['packages']}
    nodes = {v['id']:v for v in metadata['resolve']['nodes']}
    host = next(v for v in packages.values() if v['name']=='genewars-desktop')
    roots = [v for v in packages.values() if (v['name'],v['version']) in [('winit','0.30.13'),('softbuffer','0.4.8')]]
    assert len(roots) == 2
    def closure(start):
        reached, pending = set(),list(start)
        while pending:
            key = pending.pop()
            if key in reached:
                continue
            reached.add(key)
            for dep in nodes[key]['deps']:
                if any(v['kind'] in (None,'build') for v in dep['dep_kinds']):
                    pending.append(dep['pkg'])
        return reached
    all_ids = closure([host['id']])
    new_ids = closure([v['id'] for v in roots])
    # metadata can unify optional features from other workspace/target graphs.
    # The actual target-specific Cargo tree is authoritative for this build.
    tree_command = [sys.executable,'-B',str(ROOT/'scripts/cargo.py'),'tree','--offline','--locked',
        '-p','genewars-desktop','--target',TARGET,'-e','normal,build','--prefix','none','--format','{p}|{f}']
    tree = subprocess.run(tree_command,capture_output=True,timeout=60)
    assert tree.returncode == 0,tree.stderr.decode(errors='replace')
    (PRIVATE/'cargo-tree.txt').write_bytes(tree.stdout)
    active = {}
    for line in tree.stdout.decode('utf-8').splitlines():
        identity,features = line.split('|',1)
        match = re.match(r'([\w-]+) v([^ ]+)',identity)
        assert match, line
        if features.strip() == '(*)':
            continue
        active[match.groups()] = sorted(filter(None,features.split(',')))
    all_ids = {key for key in all_ids if (packages[key]['name'],packages[key]['version']) in active}
    assert {(packages[k]['name'],packages[k]['version']) for k in all_ids} == set(active)
    lock = tomllib.loads(lock_path.read_text(encoding='utf-8'))
    pins = {(v['name'],v['version']):v for v in lock['package'] if 'checksum' in v}
    results = []
    for key in sorted(all_ids):
        pkg = packages[key]
        if not pkg['source']:
            continue
        pin = pins[pkg['name'],pkg['version']]
        directory = Path(pkg['manifest_path']).parent
        manifest = tomllib.loads((directory/'Cargo.toml').read_text(encoding='utf-8'))
        assert manifest['package']['license'] == pkg['license']
        assert pkg['license'], (pkg['name'],'missing declared license')
        archives = list((REGISTRY/'cache').glob(f'*/{pkg["name"]}-{pkg["version"]}.crate'))
        assert len(archives) == 1
        archive = archives[0]
        assert sha(archive.read_bytes()) == pin['checksum']
        checksum_path = directory/'.cargo-checksum.json'
        if checksum_path.exists():
            checksum = json.loads(checksum_path.read_bytes())
            assert checksum['package'] == pin['checksum']
        file_count = byte_count = 0
        with tarfile.open(archive,'r:gz') as tar:
            for member in tar:
                if member.isdir():
                    continue
                assert member.isfile(), 'Unexpected archive link or special entry'
                parts = Path(member.name).parts
                assert parts[0] == f'{pkg["name"]}-{pkg["version"]}' and '..' not in parts
                cached = directory.joinpath(*parts[1:])
                content = tar.extractfile(member).read()
                assert cached.read_bytes() == content, (pkg['name'],member.name,'modified extracted source')
                file_count += 1
                byte_count += len(content)
        licenses = [p for p in directory.rglob('*') if p.is_file() and
                    p.name.upper().startswith(('LICENSE','LICENCE','COPYING','NOTICE','COPYRIGHT','UNLICENSE'))]
        license_records = []
        for path in sorted(licenses):
            raw = path.read_bytes()
            text = raw.decode('utf-8',errors='replace')
            markers = []
            for label,needle in [('Apache-2.0','Apache License'),('MIT','Permission is hereby granted'),
                                 ('BSD','Redistribution and use'),('Zlib','This software is provided')]:
                if needle in text:
                    markers.append(label)
            license_records.append(dict(relative_path=path.relative_to(directory).as_posix(),
                size=len(raw),sha256=sha(raw),text_family_markers=markers))
        results.append(dict(name=pkg['name'],version=pkg['version'],declared_license=pkg['license'],
            repository=pkg.get('repository'),new_host_dependency_closure=key in new_ids,
            active_features=active[pkg['name'],pkg['version']],registry_checksum=pin['checksum'],
            cargo_checksum_sidecar_present=checksum_path.exists(),
            license_payload_present=bool(licenses),
            archive=recorded(archive),source_files_matched=file_count,source_bytes_matched=byte_count,
            license_files=license_records))
    assert lock_before == recorded(lock_path),'Lock changed during audit'
    gaps = [v['name']+' '+v['version'] for v in results if not v['license_payload_present']]
    output = dict(status='archive-and-source-checks-passed',checked_utc=datetime.now(timezone.utc).isoformat(),
        target=TARGET,dependency_kinds=['normal','build'],network_used=False,
        originals_or_gui_executed=False,cargo_lock=lock_before,
        desktop_manifest=recorded(ROOT/'rust/crates/genewars-desktop/Cargo.toml'),
        private_metadata=recorded(PRIVATE/'cargo-metadata.json'),
        actual_windows_tree=recorded(PRIVATE/'cargo-tree.txt'),
        registry_packages=len(results),new_backend_closure_packages=sum(v['new_host_dependency_closure'] for v in results),
        license_payload_gaps=gaps,
        packages=results,limits=['MSVC target not validated; windows_x86_64_msvc 0.52.6 absent from local cache',
            'License statements and license payloads are metadata checks, not a source-wide legal opinion',
            'Retain applicable license/notice files when distributing third-party components',
            'This receipt does not establish GUI/event behavior or non-Windows target compatibility'])
    path = ROOT/'research/variants/windows-dependency-review.json'
    path.write_text(json.dumps(output,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(dict(status=output['status'],registry_packages=len(results),new_backend_closure_packages=output['new_backend_closure_packages'],license_payload_gaps=gaps)))


if __name__=='__main__':
    main()
