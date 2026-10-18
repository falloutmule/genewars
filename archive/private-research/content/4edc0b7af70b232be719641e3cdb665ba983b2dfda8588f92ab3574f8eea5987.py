"""Compare the native campaign plan with separately exported instruction vectors.

No original code is run. Names are DOS case-insensitive; source identities are
the exact CD manifest entries recorded by the static proof, never bare filenames.
"""
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def crc16(data):
    value = 0
    for byte in data:
        value ^= byte
        for _ in range(8):
            value = (value >> 1) ^ (0xA001 if value & 1 else 0)
    return value


def main():
    evidence_path = ROOT / 'research/ghidra/campaign-entry-evidence.json'
    evidence = json.loads(evidence_path.read_bytes())
    assert evidence['candidate_sha256'] == '4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba'
    native = ROOT / 'private/build/rust-target/debug/genewars-tools.exe'
    mapping = {
        'header': 'header', 'mission': 'mission', 'texture_atlas': 'texture',
        'campaign_background': 'background', 'campaign_palette': 'palette',
        'system_movie': 'movie', 'transition_background': 'transition_background',
        'transition_palette': 'transition_palette',
    }
    sources = {row['source'].upper(): row for row in evidence['assets']}
    assert len(sources) == len(evidence['assets'])
    identities = set()
    states = set()
    for vector in evidence['vectors']:
        state = (vector['progress'], vector['prior_system'])
        assert state not in states
        states.add(state)
        result = subprocess.run([str(native), 'campaign-plan', *map(str, state)],
                                capture_output=True, text=True, check=True, timeout=5)
        actual = json.loads(result.stdout)
        assert actual['gameplay'] is False
        for key in ('progress', 'local_planet', 'effective_system'):
            assert actual[key] == vector[key], (state, key)
        for native_key, proof_key in mapping.items():
            value, expected = actual[native_key], vector[proof_key]
            assert (value.upper() if value else value) == expected, (state, native_key)
            if value:
                identities.add(sources[value.upper()]['entry_id'])
        assert (actual['system_movie'] is not None) == vector['updates_system']
    assert states == {(progress, prior) for progress in range(24) for prior in range(4)}
    rejected = [(-1, 0), (24, 0), (-128, 0), (127, 0), (0, 4), (0, 255), (128, 0), (0, -1)]
    for progress, prior in rejected:
        result = subprocess.run([str(native), 'campaign-plan', str(progress), str(prior)],
                                capture_output=True, text=True, timeout=5)
        assert result.returncode == 2 and not result.stdout
    staged = json.loads((ROOT / 'research/integration/campaign-inputs.json').read_bytes())
    paths = {}
    for resource in staged['staged_resources']:
        path = (ROOT / resource['private_path']).resolve()
        assert path.is_relative_to((ROOT / 'private').resolve())
        assert path.stat().st_size == resource['staged_size'] and digest(path) == resource['staged_sha256']
        paths[resource['filename']] = path
    header_path, mission_path = paths['HDRCO1.DAT'], paths['CONQ001.DAT']
    header, mission = header_path.read_bytes(), mission_path.read_bytes()
    assert digest(header_path) == evidence['first_header']['sha256']
    assert digest(mission_path) == evidence['first_mission']['sha256']
    result = subprocess.run([str(native), 'campaign-inspect', str(header_path), str(mission_path)],
                            capture_output=True, text=True, check=True, timeout=5)
    title = header[32:header.index(0, 32)]
    end = evidence['first_mission']['direct_prefix_size']
    assert hashlib.sha256(mission[:end]).hexdigest() == evidence['first_mission']['direct_prefix_sha256']
    expected = {
        'header_bytes': 699, 'title_bytes': len(title), 'title_crc16': crc16(title),
        'mission_bytes': len(mission), 'tag': 0x55336,
        'prefix_bytes': end, 'prefix_crc16': crc16(mission[:end]),
        'tail_bytes': len(mission) - end, 'tail_crc16': crc16(mission[end:]),
        'opaque_region_bytes': 14 * 16384, 'opaque_region_crc16': crc16(mission[0x355E:end]),
        'initial_player_crc16': [crc16(mission[20 + i * 1805:20 + (i + 1) * 1805]) for i in range(4)],
        'gameplay': False,
    }
    assert json.loads(result.stdout) == expected
    record = {
        'status': 'passed', 'checked_utc': datetime.now(timezone.utc).isoformat(),
        'native_binary_sha256': digest(native), 'evidence_sha256': digest(evidence_path),
        'native_source_sha256': digest(ROOT / 'rust/crates/genewars-core/src/campaign.rs'),
        'states_compared': len(states), 'source_identities_selected': len(identities),
        'invalid_inputs_rejected': len(rejected), 'original_execution': False,
        'staged_resources_verified': len(paths), 'first_header_and_prefix_compared': expected,
        'limits': 'Bounded resource plan only; retained system is explicit. No mission execution, completion rule or general signed-byte behavior claim.',
    }
    (ROOT / 'research/integration/campaign-plan.json').write_text(json.dumps(record, indent=2) + '\n')
    print(json.dumps(record))


if __name__ == '__main__':
    main()
