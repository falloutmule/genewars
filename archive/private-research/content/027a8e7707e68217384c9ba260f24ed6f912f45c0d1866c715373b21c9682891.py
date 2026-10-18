"""Copy only the nested disc archive to a seekable private file, verifying bytes."""
import argparse
import hashlib
import json
from pathlib import Path
import zipfile

from build_manifest import DISC_NAME, EXPECTED_SOURCE_SHA256, hash_file


def prepare(source, root):
    original_hash = hash_file(source)
    if original_hash != EXPECTED_SOURCE_SHA256:
        raise ValueError('Source differs from the supplied archive; do not reuse its preflight results')
    private = root.resolve() / 'private'
    private.mkdir(parents=True, exist_ok=True)
    target = private / 'disc-package.7z'
    staged = private / 'disc-package.7z.part'
    assert target.resolve().is_relative_to(root.resolve())
    assert staged.resolve().is_relative_to(root.resolve())
    digest, count = hashlib.sha256(), 0
    with zipfile.ZipFile(source) as z:
        info = z.getinfo(DISC_NAME)
        with z.open(info) as src, staged.open('wb') as dst:
            while chunk := src.read(1024 * 1024):
                dst.write(chunk)
                digest.update(chunk)
                count += len(chunk)
    if count != info.file_size or hash_file(staged) != digest.hexdigest():
        raise ValueError('Seekable copy did not verify')
    if hash_file(source) != original_hash:
        raise ValueError('Original archive changed during copy')
    staged.replace(target)
    print(json.dumps({'source_sha256_unchanged': original_hash, 'copy': str(target),
                      'size': count, 'sha256': digest.hexdigest()}))


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--source', type=Path, default=Path(r'C:\Users\fallo\Downloads\genewars.zip'))
    parser.add_argument('--root', type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args()
    prepare(args.source, args.root)
