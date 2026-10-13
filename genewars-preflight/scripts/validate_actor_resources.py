"""Validate the typed actor ANB catalog against pinned source and sprite receipts."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import subprocess
import struct
import tempfile

from actor_inputs import ROOT, stage


SCRATCH = ROOT / 'private/integration/actor-resource-validation'
HARNESS = SCRATCH / 'harness'
RESULTS = SCRATCH / 'native.tsv'
RECEIPT = ROOT / 'research/integration/actor-resources-validation.json'
RUNTIME_SOURCE = ROOT / 'rust/crates/genewars-runtime/src/actor_resources.rs'
ACTOR_RENDER_SOURCE = ROOT / 'rust/crates/genewars-runtime/src/actor_render.rs'
ACTOR_DRAW_LISTINGS = [
    ROOT / 'private/ghidra/actor-draw/instructions-00056f10.txt',
    ROOT / 'private/ghidra/actor-draw/instructions-00057050.txt',
    ROOT / 'private/ghidra/actor-draw/instructions-00057070.txt',
    ROOT / 'private/ghidra/actor-draw/instructions-0008f660.txt',
    ROOT / 'private/ghidra/actor-draw/instructions-00096870.txt',
    ROOT / 'private/ghidra/actor-draw/instructions-00096990.txt',
    ROOT / 'private/ghidra/actor-draw/instructions-00095390.txt',
    ROOT / 'private/ghidra/actor-draw/instructions-00095e40.txt',
]


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def prepare_harness() -> None:
    (HARNESS / 'src').mkdir(parents=True, exist_ok=True)
    source = RUNTIME_SOURCE.as_posix()
    render_source = ACTOR_RENDER_SOURCE.as_posix()
    cargo_toml = '''[package]\nname = "actor-resource-validation"\nversion = "0.1.0"\nedition = "2021"\n\n[dependencies]\ngenewars-assets = { path = "../../../../rust/crates/genewars-assets" }\ngenewars-runtime = { path = "../../../../rust/crates/genewars-runtime" }\n'''
    main = f'''use std::fs;
use std::path::Path;
mod actor_initialization {{ pub use genewars_runtime::actor_initialization::{{ActorResourceTableKey,BlueprintRecordHandle}}; }}
mod identity {{ pub use genewars_runtime::identity::sha256_hex; }}
mod actor_pool {{ pub use genewars_runtime::actor_pool::{{ActorPool,ActorPoolError,ActorHandleBindings}}; }}
mod player_state {{ pub use genewars_runtime::player_state::{{PlayerState,ActorOriginSample,PLAYER_RECORD_BYTES}}; }}
#[path = "{source}"] mod actor_resources;
#[path = "{render_source}"] mod actor_render;
use player_state::{{PlayerState,ActorOriginSample}};
use actor_resources::{{ActorResourceCatalog,ActorResourceError}};
fn main() {{
  let root = std::env::args().nth(1).expect("input dir");
  let catalog = ActorResourceCatalog;
  for spec in catalog.specs() {{
    let path = Path::new(&root).join(format!("key-{{:02}}.anb", spec.key.flat_index));
    let bytes = fs::read(path).expect("read staged ANB");
    let db = catalog.load(spec.key, bytes).expect("strict catalog load");
    let borrowed = catalog.load_borrowed(spec.key, db.bytes()).expect("borrowed strict catalog load");
    assert_eq!(borrowed.bytes(), db.bytes());
    assert!(matches!(db.animation(db.animation_count()), Err(ActorResourceError::AnimationOutOfBounds{{..}})));
    assert!(matches!(db.frame(0, usize::MAX), Err(ActorResourceError::FrameOutOfBounds{{..}})));
    assert!(matches!(db.frame(0, db.frame_count(0).unwrap()), Err(ActorResourceError::FrameOutOfBounds{{..}})));
    assert!(db.component(0, 0, usize::MAX).is_err());
    assert!(db.decode_sprite_indexed8(db.sprite_record_count()).is_err());
    assert!(matches!(catalog.load(spec.key, db.bytes()[..db.bytes().len()-1].to_vec()), Err(ActorResourceError::WrongLength{{..}})));
    let mut altered = db.bytes().to_vec(); altered[0] ^= 1;
    assert!(matches!(catalog.load(spec.key, altered), Err(ActorResourceError::WrongHash{{..}})));
    assert!(matches!(catalog.spec(actor_initialization::ActorResourceTableKey{{style:spec.key.style,category:spec.key.category,flat_index:spec.key.flat_index ^ 1}}), Err(ActorResourceError::InvalidKey(_))));
    println!("P\\t{{}}\\t{{}}\\t{{}}\\t{{}}\\t{{}}\\t{{}}\\t{{}}\\t{{}}\\t{{}}\\t{{}}\\t{{}}\\t{{}}", spec.key.flat_index, spec.key.style, spec.key.category, spec.descriptor_va, spec.database_slot_va, spec.animation_slot_va, spec.source_name, spec.entry_id, spec.source_sha256, spec.decoded_sha256, spec.decoded_bytes, spec.animation_count);
    println!("R\\t{{}}\\t{{}}\\t{{}}\\t{{}}", spec.key.flat_index, db.animation_count(), db.sprite_record_count(), identity::sha256_hex(db.bytes()));
    for animation in 0..db.animation_count() {{
      let record = db.animation(animation).expect("animation");
      let frame_count = db.frame_count(animation).expect("frame count");
      println!("A\\t{{}}\\t{{}}\\t{{}}\\t{{}}\\t{{}}", spec.key.flat_index, animation, record.first_frame_offset, record.last_frame_offset, frame_count);
      for frame_index in 0..frame_count {{
        let frame = db.frame(animation, frame_index).expect("frame");
        println!("F\\t{{}}\\t{{}}\\t{{}}\\t{{}}\\t{{}}\\t{{}}\\t{{}}", spec.key.flat_index, animation, frame_index, frame.width_raw, frame.height_raw, frame.component_count, frame.component_offset);
        for component in 0..usize::from(frame.component_count) {{
          let c = db.component(animation, frame_index, component).expect("component");
          println!("C\\t{{}}\\t{{}}\\t{{}}\\t{{}}\\t{{}}\\t{{}}\\t{{}}\\t{{}}\\t{{}}", spec.key.flat_index, animation, frame_index, component, c.sprite_offset, c.x_raw, c.y_raw, c.flags, c.extra);
        }}
      }}
    }}
    for (record, desc) in db.sprite_descriptors().iter().enumerate() {{
      let sprite = db.decode_sprite_indexed8(record).expect("indexed8 sprite decode");
      let opaque = sprite.opaque.iter().filter(|v| **v).count();
      println!("S\\t{{}}\\t{{}}\\t{{}}\\t{{}}\\t{{}}\\t{{}}\\t{{}}", spec.key.flat_index, record, desc.width, desc.height, sprite.consumed_bytes, opaque, sprite.indices.len());
    }}
  }}
  let key0 = catalog.specs()[0].key;
  let db0 = catalog.load(key0, fs::read(Path::new(&root).join("key-00.anb")).unwrap()).unwrap();
  let packed = vec![0u8; 20 * 140];
  let native = [[0u8; 200]; 20];
  let mut pool = actor_pool::ActorPool::from_packed_inactive(&packed, &native, [actor_pool::ActorHandleBindings::empty(); 20]).unwrap();
  let mut player = PlayerState::fresh_campaign(0, [20000,12000], b"TEST\\0").unwrap();
  let mut player_bytes = *player.bytes();
  player_bytes[0x16] = b'P'; player_bytes[0x16+0x15] = 0; player_bytes[0x16+0x16] = 0;
  player = PlayerState::from_record(&player_bytes).unwrap();
  let partition = pool.partition(0).unwrap();
  let prepared = player.prepare_fresh_actor_partition(&partition, 1, ActorOriginSample {{ position:[321,-456], height:10, shade:4 }}, [777,-888]).unwrap();
  let expected_player = player.clone();
  pool.commit_prepared_partition(&mut player, &expected_player, &partition, &prepared).unwrap();
  let view = actor_render::extract_actor_render_record(&pool, 0, &catalog, &db0).unwrap();
  let decoded = actor_render::decode_actor_components_indexed8(&view, &db0).unwrap();
  assert_eq!(decoded.len(), view.components.len());
  assert!(decoded.iter().all(|c| c.sprite.indices.len() == c.sprite.opaque.len()));
  assert_eq!(view.actor_id, 0); assert_eq!(view.owner, 0); assert_eq!(view.resource_key, key0);
  assert_eq!(view.world_position, [777,-888]);
  assert_eq!(view.animation_index, 8); assert_eq!(view.frame_index, 0);
  let db1 = catalog.load(catalog.specs()[1].key, fs::read(Path::new(&root).join("key-01.anb")).unwrap()).unwrap();
  assert!(matches!(actor_render::extract_actor_render_record(&pool,0,&catalog,&db1), Err(actor_render::ActorRenderError::ResourceBindingMismatch{{..}})));
  println!("X\\t{{}}\\t{{}}\\t{{}}\\t{{}}\\t{{}}\\t{{}}\\t{{}}\\t{{}}", view.actor_id, view.owner, view.resource_key.flat_index, view.world_position[0], view.world_position[1], view.animation_index, view.frame_index, view.components.len());
}}
'''
    for path, data in ((HARNESS / 'Cargo.toml', cargo_toml.encode()), (HARNESS / 'src/main.rs', main.encode())):
        if path.exists() and path.read_bytes() != data:
            path.write_bytes(data)
        elif not path.exists():
            path.write_bytes(data)


def run_native() -> list[list[str]]:
    prepare_harness()
    args = [
        'run', '--offline', '--manifest-path', str(HARNESS / 'Cargo.toml'), '--',
        str((ROOT / 'private/integration/actor-resources').resolve()),
    ]
    cargo = ROOT / 'scripts/cargo.py'
    done = subprocess.run([os.environ.get('PYTHON', 'python'), '-B', str(cargo), *args],
                          cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                          text=True, timeout=180)
    if done.returncode:
        raise RuntimeError('native actor-resource validation failed:\n' + done.stdout[-12000:])
    lines = [line.split('\t') for line in done.stdout.splitlines() if line.startswith(('P\t','R\t','A\t','F\t','C\t','S\t','X\t'))]
    if not lines:
        raise RuntimeError('native actor-resource harness returned no result rows')
    RESULTS.parent.mkdir(parents=True, exist_ok=True)
    RESULTS.write_text('\n'.join('\t'.join(line) for line in lines) + '\n', encoding='utf-8', newline='\n')
    return lines


def validate() -> dict:
    staged = stage()
    evidence = json.loads((ROOT / 'research/ghidra/blueprint-init-evidence.json').read_bytes())
    corpus = json.loads((ROOT / 'research/integration/sprite-corpus.json').read_bytes())
    native = run_native()
    by_key = {int(row[1]): row for row in native if row[0] == 'R'}
    proof_by_key = {int(row['key']): row for row in evidence['resources']}
    sprite_rows = {r['entry_id']: r for r in corpus['results']}
    actual_sprites = {}
    animation_rows = 0
    frame_rows = 0
    component_rows = 0
    render_row = next((row for row in native if row[0] == 'X'), None)
    if render_row is None or render_row[:8] != ['X', '0', '0', '0', '777', '-888', '8', '0']:
        raise ValueError(f'actor render extraction disagrees with initialized typed source fields: {render_row}')
    key0_frame = next((row for row in native if row[:4] == ['F', '0', '8', '0']), None)
    if key0_frame is None or render_row[8] != key0_frame[6]:
        raise ValueError(f'actor render component references disagree with selected frame: {render_row}, {key0_frame}')
    for item in staged['resources']:
        key = item['key']
        proof_item = proof_by_key[key]
        p_row = next((row for row in native if row[0] == 'P' and int(row[1]) == key), None)
        expected_p = ['P', str(key), str(proof_item['style']), str(proof_item['category']),
                      str(int(proof_item['descriptor_va'], 16)), str(int(proof_item['raw_slot'], 16)),
                      str(int(proof_item['animation_slot'], 16)), proof_item['source_name'],
                      proof_item['entry_id'], proof_item['input_sha256'], proof_item['decoded_sha256'],
                      str(proof_item['decoded_size']), str(proof_item['animation_count'])]
        if p_row != expected_p:
            raise ValueError(f'production catalog/evidence mismatch for key {key}: {p_row} != {expected_p}')
        resource_row = by_key.get(key)
        if resource_row is None or int(resource_row[2]) != item['animation_count']:
            raise ValueError(f'native animation database mismatch for key {key}')
        if sha((ROOT / item['private_path']).read_bytes()) != item['decoded_sha256']:
            raise ValueError(f'staged resource hash mismatch for key {key}')
        sprite = sprite_rows[item['entry_id']]
        if sprite['encoding'] != 'indexed8' or sprite['native_result']['status'] != 'verified':
            raise ValueError(f'indexed8 corpus admission changed for key {key}')
        data = (ROOT / item['private_path']).read_bytes()
        independent = parse_anb_metadata(data, key)
        expected_anim = [row for row in native if row[0] == 'A' and int(row[1]) == key]
        expected_frames = [row for row in native if row[0] == 'F' and int(row[1]) == key]
        expected_components = [row for row in native if row[0] == 'C' and int(row[1]) == key]
        if expected_anim != independent['animations']:
            raise ValueError(f'independent animation-table mismatch for key {key}')
        if expected_frames != independent['frames']:
            raise ValueError(f'independent frame-table mismatch for key {key}')
        if expected_components != independent['components']:
            raise ValueError(f'independent component-table mismatch for key {key}')
        expected = sprite['native_result']['records']
        got = [row for row in native if row[0] == 'S' and int(row[1]) == key]
        if len(got) != len(expected):
            raise ValueError(f'sprite record count differs for key {key}: {len(got)} != {len(expected)}')
        for output, reference in zip(got, expected, strict=True):
            observed = (int(output[3]), int(output[4]), int(output[5]), int(output[6]), int(output[7]))
            wanted = (reference['width'], reference['height'], reference['consumed_bytes'], reference['opaque_pixels'], reference['width'] * reference['height'])
            if observed != wanted:
                raise ValueError(f'indexed8 sprite stats differ at key {key} record {output[1]}: {observed} != {wanted}')
        actual_sprites[key] = len(got)
    for row in native:
        if row[0] == 'A': animation_rows += 1
        elif row[0] == 'F': frame_rows += 1
        elif row[0] == 'C': component_rows += 1
    if animation_rows != sum(r['animation_count'] for r in evidence['resources']):
        raise ValueError('native animation range enumeration incomplete')
    modules = [RUNTIME_SOURCE, ROOT / 'rust/crates/genewars-runtime/src/actor_initialization.rs',
               ROOT / 'rust/crates/genewars-assets/src/lib.rs']
    return {
        'schema_version': 1,
        'scope': 'Sixteen source-bound actor resource keys, parsed ANB animation/frame/component metadata, and indexed8 sprite-record comparison to the existing corpus. A synthetic initialized actor exercises metadata extraction and selected-component indexed8 decoding. The bounded 0x57050/0x57070 component compositor now covers its source-proved wrapper visibility/56F10 flag policy, indexed8 opacity, owner recolor, explicit fade/blend tables, flips, clipping, and transactional failure. World projection, normal-path shadow execution, high component flag 0x80, table acquisition, and draw ordering remain caller-owned or unsupported.',
        'candidate_sha256': evidence['candidate_sha256'],
        'evidence_sha256': sha((ROOT / 'research/ghidra/blueprint-init-evidence.json').read_bytes()),
        'actor_draw_contract_sha256': sha((ROOT / 'research/ghidra/actor-draw-contract.json').read_bytes()),
        'catalog_sha256': sha((ROOT / 'research/integration/actor-resource-catalog.tsv').read_bytes()),
        'sprite_corpus_sha256': sha((ROOT / 'research/integration/sprite-corpus.json').read_bytes()),
        'runtime_sources': {p.relative_to(ROOT).as_posix(): sha(p.read_bytes()) for p in modules + [ACTOR_RENDER_SOURCE]},
        'actor_draw_listing_sha256': {p.relative_to(ROOT).as_posix(): sha(p.read_bytes()) for p in ACTOR_DRAW_LISTINGS},
        'resource_count': len(staged['resources']),
        'animation_records': animation_rows,
        'frame_records': frame_rows,
        'component_records': component_rows,
        'sprite_records': sum(actual_sprites.values()),
        'render_record_checks': 1,
        'render_component_decode_checks': 1,
        'per_key_sprite_records': actual_sprites,
        'codec_evidence': 'The actor wrapper at 0x57070 dispatches both color helpers through 0x96870/0x96990, which call the indexed8 RLE routine 0x95390/0x95E40. Creature Packed4 evidence is a different consumer and is not applied here. Actor owner recolor precedes optional fade lookup; blend and fade tables are caller-supplied and strictly bounded. The 56F10 inputs are read through the typed blueprint handle.',
        'render_limitations': ['screen origin and virtual height are explicit inputs', 'ordinary world path requires a separate shadow pass before color composition', 'component flags with high 0x80 bit are rejected pending clipping proof', 'fade/blend table bytes are not acquired from a proprietary executable by this validator'],
        'negative_checks': ['wrong-key tuple', 'same-length wrong-hash bytes', 'truncated wrong-length bytes', 'animation index at count', 'frame index at count and usize::MAX', 'component index usize::MAX', 'sprite index at count'],
        'native_harness_sha256': sha((HARNESS / 'src/main.rs').read_bytes()),
        'native_output_sha256': sha(RESULTS.read_bytes()),
        'resources': staged['resources'],
        'original_execution': False,
    }


def parse_anb_metadata(data: bytes, key: int) -> dict:
    """Independent byte-offset model for the observed ANB record widths."""
    if len(data) < 18:
        raise ValueError('ANB shorter than the observed header')
    header_word = int.from_bytes(data[0:2], 'little')
    offsets = [int.from_bytes(data[i:i+4], 'little') for i in (2,6,10,14)]
    if offsets[0] < 18 or offsets != sorted(offsets) or offsets[-1] > len(data):
        raise ValueError('ANB section offsets outside the source bytes')
    table, components, frames, animations = offsets
    if (components-table) % 6 or (frames-components) % 10 or (animations-frames) % 14 or len(data)-animations != header_word*10:
        raise ValueError('ANB observed table section widths do not divide exactly')
    records = [struct.unpack_from('<IhhBB', data, components + i*10) for i in range((frames-components)//10)]
    frame_raw = []
    frame_count_total = (animations-frames)//14
    for i in range(frame_count_total):
        at = frames + i*14
        component_offset = int.from_bytes(data[at:at+4], 'little')
        width_raw = int.from_bytes(data[at+4:at+6], 'little', signed=True)
        height_raw = int.from_bytes(data[at+6:at+8], 'little', signed=True)
        component_count = data[at+11]
        if component_offset % 10 or component_offset + component_count*10 > frames-components:
            raise ValueError(f'frame component slice invalid at frame {i}')
        frame_raw.append((i, width_raw, height_raw, component_count, component_offset))
        first = component_offset//10
        for local in range(component_count):
            index = first+local
            sprite_offset, x_raw, y_raw, flags, extra = records[index]
            if sprite_offset % 6 or sprite_offset + 6 > components-table:
                raise ValueError(f'component sprite reference invalid at component {index}')
    animations_out = []
    expected_frames = []
    expected_components = []
    for i in range(header_word):
        at = animations + i*10
        first = int.from_bytes(data[at:at+4], 'little')
        last = int.from_bytes(data[at+4:at+8], 'little')
        if first > last or (last-first) % 14 or last >= animations-frames:
            raise ValueError(f'animation frame range invalid at animation {i}')
        count = (last-first)//14+1
        animations_out.append((i, first, last, count))
        for frame_index in range(count):
            global_frame = first//14 + frame_index
            _, width, height, component_count, component_offset = frame_raw[global_frame]
            expected_frames.append(['F', str(key), str(i), str(frame_index), str(width), str(height), str(component_count), str(component_offset)])
            start_component = component_offset//10
            for local in range(component_count):
                sprite_offset, x_raw, y_raw, flags, extra = records[start_component+local]
                expected_components.append(['C', str(key), str(i), str(frame_index), str(local), str(sprite_offset), str(x_raw), str(y_raw), str(flags), str(extra)])
    return {
        'animations': [['A', str(key), str(k), str(first), str(last), str(count)] for k,first,last,count in animations_out],
        'frames': expected_frames,
        'components': expected_components,
    }


if __name__ == '__main__':
    result = validate()
    payload = (json.dumps(result, indent=2) + '\n').encode('utf-8')
    RECEIPT.parent.mkdir(parents=True, exist_ok=True)
    if not RECEIPT.exists() or RECEIPT.read_bytes() != payload:
        fd, temporary = tempfile.mkstemp(dir=RECEIPT.parent, prefix=RECEIPT.name + '-', suffix='.tmp')
        try:
            with os.fdopen(fd, 'wb') as stream:
                stream.write(payload)
            os.replace(temporary, RECEIPT)
        finally:
            if os.path.exists(temporary):
                os.unlink(temporary)
    print(json.dumps({k: result[k] for k in ('resource_count','animation_records','frame_records','component_records','sprite_records','native_output_sha256')}, indent=2))
