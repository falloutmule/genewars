"""Independent hash-specific BONES packed4 check; only our CLI is executed."""
from datetime import datetime, timezone
import importlib.util
import json
from pathlib import Path
import struct

ROOT = Path(__file__).resolve().parents[2]
MODEL = ROOT/'research/variants/creature-review.py'
spec = importlib.util.spec_from_file_location('independent_packed4_model',MODEL)
model = importlib.util.module_from_spec(spec)
spec.loader.exec_module(model)
model.SCRATCH = ROOT/'private/review/bones-corpus'


def main():
    model.SCRATCH.mkdir(parents=True,exist_ok=True)
    proof_path = ROOT/'research/ghidra/bones-evidence.json'
    corpus_path = ROOT/'research/integration/sprite-corpus.json'
    proof = json.loads(proof_path.read_bytes())
    corpus = json.loads(corpus_path.read_bytes())
    key = proof['source_sha256']
    assert key == '2106ce79deaafe53d5fbb6292c78a8c71fb6920b08d1a11b5c44ea3b0ab75e9f'
    aliases = [v for v in corpus['results'] if v['input_sha256']==key]
    assert {v['entry_id'] for v in aliases} == {v['entry_id'] for v in proof['source_aliases']}
    assert len(aliases) == 2
    for row in aliases:
        assert row['encoding'] == 'packed4-high-first' and row['status'] == 'passed'
    row = aliases[-1]
    full = (ROOT/row['private_database']).read_bytes()
    database = ROOT/row['private_bounded_database']
    data = database.read_bytes()
    table_path = ROOT/row['private_table']
    table = table_path.read_bytes()
    assert model.sha(full) == row['decoded_sha256'] == key
    limit = row['section_offsets'][0]
    assert limit == proof['data_end'] == 3721
    assert data == full[:limit] and table == full[limit:row['section_offsets'][1]]
    assert model.sha(table) == row['embedded_table_sha256']
    tool_before = model.record(model.EXE)
    totals = dict(active_records=0,empty_records=0,rows=0,short_rows=0,overshoot_rows=0,
                  opaque_pixels=0,odd_runs=0,zero_nibbles=0,full_native_pixel_mask_comparisons=0)
    results = []
    for index,(offset,width,height) in enumerate(struct.iter_unpack('<IBB',table)):
        pixels,mask,stats = model.decode(data,offset,width,height,0)
        cached = row['native_result']['records'][index]
        assert cached['consumed_bytes'] == stats['consumed_bytes']
        assert cached['opaque_pixels'] == stats['opaque_pixels']
        if not width or not height:
            totals['empty_records'] += 1
            assert not proof['records'][index]['active']
            continue
        expected = proof['records'][index]
        assert stats['consumed_bytes'] == expected['consumed']
        assert stats['opaque_pixels'] == expected['opaque']
        assert stats['end'] == expected['end']
        assert 18+offset == expected['start']
        assert model.sha(pixels) == expected['indices_sha256']
        assert model.sha(mask) == expected['mask_sha256']
        assert stats['short_rows'] == expected['short_rows']
        assert stats['max_row_sum'] == expected['maximum_row_width']
        native = model.native(database,table_path,index,0,key,pixels,mask,stats)
        bp,bm,bs = model.decode(data,offset,width,height,254)
        base254 = model.native(database,table_path,index,254,key,bp,bm,bs)
        totals['active_records'] += 1
        totals['full_native_pixel_mask_comparisons'] += 1
        for field in ('rows','short_rows','overshoot_rows','opaque_pixels','odd_runs','zero_nibbles'):
            totals[field] += stats[field]
        results.append(dict(record=index,width=width,height=height,start=18+offset,**stats,**native,base254=base254))
    assert totals['active_records'] == 10 and totals['rows'] == totals['short_rows'] == 378
    assert totals['overshoot_rows'] == 0 and totals['opaque_pixels'] == 3061
    assert results[-1]['end'] == limit
    listings = []
    for item in proof['private_listings']:
        check = model.record(ROOT/item['private_path'])
        assert check['sha256'] == item['sha256']
        listings.append(check)
    assert tool_before == model.record(model.EXE)
    receipt = dict(status='passed',checked_utc=datetime.now(timezone.utc).isoformat(),
        original_or_gui_execution=False,source_sha256=key,source_aliases=proof['source_aliases'],
        independent_model=model.record(MODEL),proof=model.record(proof_path),corpus=model.record(corpus_path),
        native_tool=tool_before,listing_hash_checks=listings,summary=totals,results=results,
        contract='Separate exact-hash admission. Unscaled raw high-first packed4 indices and explicit opacity. Sprite extent remains [18,section0); no byte or row-width bounds relaxed.',
        limits=['No palette-bank/lighting/translation/flip/scale admission','Next sprite offsets are observations, not parser bounds',
            'No original render parity or runtime state labels inferred'])
    Path(__file__).with_suffix('.json').write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(dict(status='passed',summary=totals)))


if __name__=='__main__': main()
