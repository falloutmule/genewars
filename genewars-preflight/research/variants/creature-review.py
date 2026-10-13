"""Independent, caller-admitted packed4 audit. Runs only our native Rust tool.

Private source pixels and native outputs stay ignored; public receipt contains
counts/hashes only. No source extraction, emulator or original code execution.
"""
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import struct
import subprocess

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT/'research/variants/creature-review.json'
SCRATCH = ROOT/'private/review/creature-corpus'
EXE = ROOT/'private/build/rust-target/debug/genewars-tools.exe'


def sha(data):
    return hashlib.sha256(data).hexdigest()


def record(path):
    data = path.read_bytes()
    return dict(path=path.relative_to(ROOT).as_posix(),size=len(data),sha256=sha(data))


def decode(data, offset, width, height, base):
    pixels = bytearray(width*height)
    mask = bytearray(width*height)
    if not width or not height:
        return bytes(pixels),bytes(mask),dict(consumed_bytes=0,opaque_pixels=0,rows=0,
            odd_runs=0,zero_nibbles=0,overshoot_rows=0,short_rows=0,max_row_sum=0)
    start = cursor = 18+offset
    rowsums = []
    odd = zero = 0
    for y in range(height):
        x = 0
        while True:
            if cursor >= len(data):
                raise ValueError('control reaches metadata boundary')
            value = data[cursor]
            cursor += 1
            if value == 0:
                break
            if value >= 128:
                x += 256-value
                continue
            count = value
            end = cursor+(count+1)//2
            if end > len(data):
                raise ValueError('packed payload reaches metadata boundary')
            odd += count%2
            for n in range(count):
                packed = data[cursor+n//2]
                nibble = packed//16 if n%2 == 0 else packed%16
                zero += nibble == 0
                if x+n < width:
                    slot = y*width+x+n
                    pixels[slot] = (base+nibble)%256
                    mask[slot] = 1
            cursor = end
            x += count
        rowsums.append(x)
    return bytes(pixels),bytes(mask),dict(consumed_bytes=cursor-start,
        opaque_pixels=sum(mask),rows=height,odd_runs=odd,zero_nibbles=zero,
        overshoot_rows=sum(v>width for v in rowsums),short_rows=sum(v<width for v in rowsums),
        max_row_sum=max(rowsums),end=cursor,metadata_limit=len(data))


def native(database,table,index,base,key,pixels,mask,stats):
    prefix = SCRATCH/f'{key}-{index}-base{base}'
    indices = prefix.with_suffix('.indices')
    opaque = prefix.with_suffix('.mask')
    command = [str(EXE),'sprite4-decode',str(database),str(table),str(index),str(base),str(indices),str(opaque)]
    run = subprocess.run(command,capture_output=True,timeout=10)
    if run.returncode:
        raise ValueError(f'Native failed {key}/{index}: {run.stderr.decode(errors="replace")}')
    result = json.loads(run.stdout)
    assert indices.read_bytes() == pixels, (key,index,base,'pixels')
    assert opaque.read_bytes() == mask, (key,index,base,'mask')
    assert result['consumed_bytes'] == stats['consumed_bytes']
    assert result['opaque_pixels'] == stats['opaque_pixels']
    return dict(index_sha256=sha(pixels),mask_sha256=sha(mask),native_exit_code=run.returncode)


def main():
    SCRATCH.mkdir(parents=True,exist_ok=True)
    corpus_path = ROOT/'research/integration/sprite-corpus.json'
    evidence_path = ROOT/'research/ghidra/creature-evidence.json'
    corpus = json.loads(corpus_path.read_bytes())
    evidence = json.loads(evidence_path.read_bytes())
    admitted = set(evidence['packed4_source_sha256'])
    rows = {v['input_sha256']:v for v in corpus['results'] if v.get('encoding') == 'packed4-high-first'
            and v['input_sha256'] in admitted}
    assert set(rows) == admitted and len(rows) == 25
    tool_before = record(EXE)
    listing_checks = []
    for item in evidence['private_listings']:
        check = record(ROOT/item['private_path'])
        assert check['sha256'] == item['sha256']
        listing_checks.append(check)
    results = []
    totals = dict(inputs=25,active_streams=0,empty_records=0,rows=0,odd_runs=0,
                  zero_nibbles=0,overshoot_rows=0,short_rows=0,opaque_pixels=0,
                  native_full_pixel_mask_comparisons=0,base250_comparisons=0)
    for key,row in rows.items():
        assert row['status'] == 'passed'
        full = (ROOT/row['private_database']).read_bytes()
        database = ROOT/row['private_bounded_database']
        data = database.read_bytes()
        table_path = ROOT/row['private_table']
        table = table_path.read_bytes()
        limit = row['section_offsets'][0]
        assert sha(full) == row['decoded_sha256']
        assert data == full[:limit]
        assert table == full[limit:row['section_offsets'][1]]
        assert sha(table) == row['embedded_table_sha256']
        receipts = row['native_result']['records']
        source_results = []
        sampled_base = False
        for index,(offset,width,height) in enumerate(struct.iter_unpack('<IBB',table)):
            pixels,mask,stats = decode(data,offset,width,height,0)
            old = receipts[index]
            assert (old['width'],old['height']) == (width,height)
            assert old['consumed_bytes'] == stats['consumed_bytes']
            assert old['opaque_pixels'] == stats['opaque_pixels']
            if not width or not height:
                totals['empty_records'] += 1
                continue
            totals['active_streams'] += 1
            for field in ('rows','odd_runs','zero_nibbles','overshoot_rows','short_rows','opaque_pixels'):
                totals[field] += stats[field]
            item = dict(record=index,width=width,height=height,**stats,
                        **native(database,table_path,index,0,key,pixels,mask,stats))
            totals['native_full_pixel_mask_comparisons'] += 1
            if not sampled_base:
                bp,bm,bs = decode(data,offset,width,height,250)
                item['base250'] = native(database,table_path,index,250,key,bp,bm,bs)
                totals['base250_comparisons'] += 1
                sampled_base = True
            source_results.append(item)
        results.append(dict(input_sha256=key,decoded_sha256=row['decoded_sha256'],
            table_sha256=sha(table),metadata_boundary=limit,records=source_results))
        print(f'Checked {len(results)}/25 inputs, {totals["active_streams"]} active streams',flush=True)
    assert totals['active_streams'] == 2673
    # Independently specified vector: odd low padding ignored, opaque zero,
    # negative skip, row clipping, and unsigned palette wrap are all observable.
    payload = bytes([3,0x0f,0xaf,0xff,2,0x12,0, 0xfe,4,0x0f,0x12,0])
    database = SCRATCH/'synthetic-prefix.bin'
    table_path = SCRATCH/'synthetic.tab'
    database.write_bytes(bytes(18)+payload)
    table_path.write_bytes(struct.pack('<IBB',0,5,2))
    expected_mask = bytes([1,1,1,0,1, 0,0,1,1,1])
    fixtures = []
    for base,expected in [(0,bytes([0,15,10,0,1,0,0,0,15,1])),
                          (254,bytes([254,13,8,0,255,0,0,254,13,255]))]:
        pixels,mask,stats = decode(database.read_bytes(),0,5,2,base)
        assert pixels == expected and mask == expected_mask and stats['consumed_bytes'] == len(payload)
        fixtures.append(dict(base=base,expected_pixels=list(expected),expected_mask=list(expected_mask),
            **stats,**native(database,table_path,0,base,'synthetic',pixels,mask,stats)))
    truncated = SCRATCH/'synthetic-truncated.bin'
    truncated.write_bytes(database.read_bytes()[:-1])
    rejected = subprocess.run([str(EXE),'sprite4-verify',str(truncated),str(table_path),'0'],
                              capture_output=True,timeout=10)
    assert rejected.returncode != 0
    assert tool_before == record(EXE), 'Native tool changed during audit'
    receipt = dict(status='passed',checked_utc=datetime.now(timezone.utc).isoformat(),
        original_execution=False,corpus=record(corpus_path),admission_evidence=record(evidence_path),
        native_tool=tool_before,listing_checks=listing_checks,summary=totals,synthetic=fixtures,
        truncated_row_terminator_rejected=dict(exit_code=rejected.returncode),results=results,
        scope='Explicit 25-hash caller admission; complete-height unscaled high-first packed4, no translation table. Native masks and full pixels independently compared for every active stream.',
        exclusions=['BONES/GPF','runtime palette-bank binding','translation-table branch','scale/flip/lighting/timing','next sprite offset as hard boundary'])
    OUT.write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(dict(status='passed',summary=totals,output=OUT.relative_to(ROOT).as_posix())))


if __name__ == '__main__':
    main()
