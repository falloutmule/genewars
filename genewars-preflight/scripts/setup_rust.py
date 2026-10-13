"""Install a checksum-verified Rust GNU toolchain in the ignored project directory."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tomllib
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
TOOLS = ROOT / 'private/tools/rust'


def fetch(url):
    request = urllib.request.Request(url, headers={'User-Agent': 'GeneWars-local-research/1.0'})
    with urllib.request.urlopen(request, timeout=60) as response:
        return response.read()


def main():
    TOOLS.mkdir(parents=True, exist_ok=True)
    url = 'https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-gnu/rustup-init.exe'
    expected = fetch(url + '.sha256').decode().split()[0]
    data = fetch(url)
    actual = hashlib.sha256(data).hexdigest()
    if actual != expected:
        raise ValueError('rustup-init checksum mismatch')
    installer = TOOLS / 'rustup-init.exe'
    installer.write_bytes(data)
    channel_url = 'https://static.rust-lang.org/dist/channel-rust-stable.toml'
    channel_bytes = fetch(channel_url)
    channel_hash = fetch(channel_url + '.sha256').decode().split()[0]
    if hashlib.sha256(channel_bytes).hexdigest() != channel_hash:
        raise ValueError('stable manifest checksum mismatch')
    channel = tomllib.loads(channel_bytes.decode())
    version = channel['pkg']['rust']['version'].split()[0]
    toolchain = version + '-x86_64-pc-windows-gnu'
    env = os.environ.copy()
    env.update(RUSTUP_HOME=str(TOOLS / 'rustup'), CARGO_HOME=str(TOOLS / 'cargo'))
    args = [str(installer), '-y', '--no-modify-path', '--profile', 'minimal',
            '--default-host', 'x86_64-pc-windows-gnu', '--default-toolchain', toolchain]
    result = subprocess.run(args, env=env, capture_output=True, text=True, timeout=900)
    receipt = {'installer_url': url, 'installer_sha256': actual, 'checksum_url': url + '.sha256',
               'channel_url': channel_url, 'channel_sha256': channel_hash,
               'channel_date': channel['date'], 'toolchain': toolchain, 'command': args,
               'environment_overrides': {k: env[k] for k in ('RUSTUP_HOME', 'CARGO_HOME')},
               'exit_code': result.returncode, 'stdout': result.stdout, 'stderr': result.stderr,
               'configuration_scope': 'project-private Rust homes; --no-modify-path; no global configuration requested',
               'documentation': ['https://rust-lang.github.io/rustup/installation/windows.html',
                                 'https://rust-lang.github.io/rustup/installation/index.html']}
    (ROOT / 'research/integration/rust-toolchain.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print(json.dumps({'toolchain': toolchain, 'exit_code': result.returncode,
                      'stdout': result.stdout[-3000:], 'stderr': result.stderr[-3000:]}), flush=True)
    if result.returncode:
        raise SystemExit(result.returncode)


if __name__ == '__main__':
    main()
