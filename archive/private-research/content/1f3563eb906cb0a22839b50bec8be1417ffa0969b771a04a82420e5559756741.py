"""Prepare a private LLVM dlltool from Rust's checksum-pinned tools archive.

LLVM's llvm-ar dispatches by executable basename. Copying the verified payload
as dlltool.exe supplies Rust GNU raw-dylib imports without a GNU assembler.
This never modifies the installed Rust toolchain or global PATH.
"""
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import subprocess
import tarfile
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
URL = 'https://static.rust-lang.org/dist/2026-10-01/llvm-tools-1.99.0-x86_64-pc-windows-gnu.tar.xz'
EXPECTED = '977aad42852c34b4d0489a8a3fbd997b5f3e71d907fd4d256daea8f5ecba70e7'
DEST = ROOT / 'private/tools/llvm-dlltool/1.99.0'


def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def put(path, data):
    if path.exists():
        if path.read_bytes() != data:
            raise ValueError('Refusing to replace different existing file: ' + str(path))
    else:
        with path.open('xb') as stream:
            stream.write(data)


def main():
    DEST.mkdir(parents=True, exist_ok=True)
    archive = DEST / URL.rsplit('/', 1)[-1]
    if not archive.exists():
        with urllib.request.urlopen(URL, timeout=60) as response, archive.open('xb') as out:
            while chunk := response.read(1024 * 1024):
                out.write(chunk)
    if sha(archive) != EXPECTED:
        raise ValueError('Official Rust LLVM tools archive checksum mismatch; nothing executed')
    extracted = []
    with tarfile.open(archive, 'r:xz') as package:
        members = package.getmembers()
        for member in members:
            name = Path(member.name).name
            if member.isfile() and (name == 'llvm-ar.exe' or name.lower().endswith('.dll') or name.startswith('LICENSE')):
                payload = package.extractfile(member).read()
                path = DEST / name
                put(path, payload)
                extracted.append({'archive_member': member.name, 'path': path.relative_to(ROOT).as_posix(),
                                  'size': len(payload), 'sha256': sha(path)})
    source = DEST / 'llvm-ar.exe'
    alias = DEST / 'dlltool.exe'
    put(alias, source.read_bytes())
    runtime = ROOT / 'private/tools/rust/rustup/toolchains/1.99.0-x86_64-pc-windows-gnu/bin'
    runtime_files = []
    for name in ('libgcc_s_seh-1.dll', 'libwinpthread-1.dll'):
        path = runtime / name
        put(DEST / name, path.read_bytes())
        runtime_files.append({'source': path.relative_to(ROOT).as_posix(), 'sha256': sha(path)})
    version = subprocess.run([str(source), '--version'], capture_output=True, text=True, timeout=15)
    help_result = subprocess.run([str(alias), '--help'], capture_output=True, text=True, timeout=15)
    if version.returncode != 0 or 'OVERVIEW: llvm-dlltool' not in help_result.stdout:
        raise ValueError('Verified LLVM payload did not provide working dlltool dispatch')
    # Harmless synthetic import declaration, not execution of the named DLL.
    definition = DEST / 'probe.def'
    library = DEST / 'probe.lib'
    put(definition, b'LIBRARY kernel32.dll\nEXPORTS\nGetTickCount\n')
    probe = subprocess.run([str(alias), '-d', str(definition), '-D', 'kernel32.dll',
                            '-l', str(library), '-m', 'i386:x86-64', '-f', '--64',
                            '--no-leading-underscore', '--temp-prefix', 'probe'],
                           capture_output=True, text=True, timeout=15, cwd=DEST)
    if probe.returncode != 0 or not library.read_bytes().startswith(b'!<arch>\n'):
        raise ValueError('LLVM import-library probe failed: ' + probe.stderr)
    receipt = {'completed_utc': datetime.now(timezone.utc).isoformat(), 'source_url': URL,
               'archive_sha256': EXPECTED, 'archive_bytes': archive.stat().st_size, 'extracted': extracted,
               'alias': {'path': alias.relative_to(ROOT).as_posix(), 'sha256': sha(alias),
                         'byte_identical_to': source.relative_to(ROOT).as_posix()},
               'version': version.stdout, 'dlltool_help': help_result.stdout,
               'dlltool_help_exit_code': help_result.returncode, 'runtime_dependencies': runtime_files,
               'import_library_probe': {'exit_code': probe.returncode, 'sha256': sha(library)},
               'scope': 'Private verified tool payload and basename alias only; no global configuration or Rust installation changes.',
               'rationale': 'Bundled GNU dlltool2.44 found but import generation fails CreateProcess; its accompanying GNU assembler is absent.',
               'documentation': ['https://github.com/rust-lang/rust/issues/140704',
                                 'https://github.com/rust-lang/rust/issues/85658',
                                 'https://raw.githubusercontent.com/llvm/llvm-project/main/llvm/tools/llvm-ar/llvm-ar.cpp']}
    (ROOT / 'research/integration/llvm-dlltool.json').write_text(json.dumps(receipt, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'archive_sha256': EXPECTED, 'alias_sha256': sha(alias), 'version': version.stdout}))


if __name__ == '__main__':
    main()
