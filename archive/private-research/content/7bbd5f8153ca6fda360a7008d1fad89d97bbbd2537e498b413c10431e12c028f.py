"""Stage source corner patterns; the native runtime executes the884D0 builder."""
from pathlib import Path
import hashlib
import json

ROOT = Path(__file__).resolve().parents[1]
SOURCE_SHA = 'aaab09272771ee7f3fbe71e9a3dc75add592fcca42fdd8d982e069629ac1ce87'
PATTERN_SHA = 'd848e423b7a76a2c2dbf517fc595e33a789df7382697b56f382cc7606b18939e'

def stage():
    source = (ROOT/'private/ghidra/validated/memory-.object3.bin').read_bytes()
    assert hashlib.sha256(source).hexdigest() == SOURCE_SHA
    patterns = source[0x5060:0x52e0]
    assert len(patterns) == 640 and hashlib.sha256(patterns).hexdigest() == PATTERN_SHA
    output = ROOT/'private/integration/physics-tables/terrain-patterns.bin'
    output.parent.mkdir(parents=True, exist_ok=True)
    if output.exists():
        assert output.read_bytes() == patterns, 'refusing changed terrain patterns'
    else:
        with output.open('xb') as stream:
            stream.write(patterns)
    receipt = dict(source_sha256=SOURCE_SHA, source_va='0x105060', size=640,
                   patterns_sha256=PATTERN_SHA, destination=str(output.relative_to(ROOT)),
                   generated_table_sha256='422f124eac806197ba95d2438516284c3bf828a1db8e8aabe330661fff23ef8a',
                   evidence='research/campaign/bootstrap-details.json', original_execution=False)
    (ROOT/'research/integration/terrain-pattern-inputs.json').write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
    return receipt

if __name__ == '__main__':
    print(json.dumps(stage()))
