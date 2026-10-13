"""Audit cached native ANB sprite receipts against the actual data-section end.

Reads private cached bytes; writes only metadata under research/variants.
Does not run tools, originals, or re-extract the source archive.
"""
import hashlib
import bisect
import json
from pathlib import Path
import struct
import sys

ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / 'research/variants/full-file-sprite-corpus-superseded.json'
OUTPUT = Path(__file__).with_suffix('.json')


def sha(data):
    return hashlib.sha256(data).hexdigest()


def consumed(data, start, width, height, limit):
    """Independent control traversal: no pixel copying or guessed next-offset bound."""
    if not width or not height:
        return 0, 0
    cursor, opaque = start, 0
    for _ in range(height):
        x = 0
        while True:
            if cursor >= limit:
                raise ValueError(f'control at {cursor} reaches/exceeds limit {limit}')
            control = data[cursor]
            cursor += 1
            if control == 0:
                break
            if control < 128:
                if cursor + control > limit:
                    raise ValueError(f'literal range [{cursor},{cursor+control}) exceeds limit {limit}')
                opaque += min(control, max(0, width - x))
                cursor += control
                x = min(width, x + control)
            else:
                x = min(width, x + 256 - control)
    return cursor - start, opaque


def diagnostic_walk(data, start, width, height, limit, policy):
    """Hypotheses only: these policies are not accepted decoder semantics."""
    target = width
    rows_target = height
    if policy == 'width-half':
        target = (width + 1) // 2
    elif policy == 'width-double':
        target = width * 2
    elif policy == 'zero-height-half':
        rows_target = (height + 1) // 2
    elif policy == 'zero-height-double':
        rows_target = height * 2
    cursor, overshoot_rows, completed, max_sum = start, 0, 0, 0
    try:
        for _ in range(rows_target):
            x = 0
            while True:
                if cursor >= limit:
                    raise ValueError('control-boundary')
                value = data[cursor]
                cursor += 1
                if value == 0:
                    break
                run = value if value < 128 else 256 - value
                if value < 128:
                    if cursor + run > limit:
                        raise ValueError('literal-boundary')
                    cursor += run
                x += run
                if policy.startswith('width') and x >= target:
                    if policy == 'width-optional-zero' and cursor < limit and data[cursor] == 0:
                        cursor += 1
                    break
            completed += 1
            overshoot_rows += x > target
            max_sum = max(max_sum, x)
        return dict(success=True, end=cursor, rows=completed, overshoot_rows=overshoot_rows,
                    max_row_sum=max_sum)
    except ValueError as error:
        return dict(success=False, end=cursor, rows=completed, overshoot_rows=overshoot_rows,
                    max_row_sum=max_sum, reason=str(error))


def diagnostic_models():
    baseline = json.loads(SOURCE.read_bytes())
    audit = json.loads(OUTPUT.read_text(encoding='utf-8'))
    affected = {v['input_sha256'] for v in audit['violations']}
    unique = {r['input_sha256']:r for r in baseline['results'] if r['status']=='passed'}
    policies = ('zero', 'width', 'width-optional-zero', 'width-half', 'width-double',
                'zero-height-half', 'zero-height-double')
    summaries, examples = {}, []
    for key,row in unique.items():
        data = (ROOT/row['private_database']).read_bytes()
        table = (ROOT/row['private_table']).read_bytes()
        assert sha(data)==row['decoded_sha256'] and sha(table)==row['embedded_table_sha256']
        records = [struct.unpack_from('<IBB',table,index*6) for index in range(len(table)//6)]
        for base in (0,18):
            starts = sorted({base+offset for offset,w,h in records if w and h})
            for policy in policies:
                group = 'affected-animal-inputs' if key in affected else 'other-passed-inputs'
                summary_key = f'{group}/base{base}/{policy}'
                s = summaries.setdefault(summary_key,dict(streams=0, bounded_success=0,
                      bounded_failure=0, end_equals_next_start=0,end_before_next_start=0,
                      end_after_next_start=0, rows=0,overshoot_rows=0,max_row_sum=0))
                for index,(offset,w,h) in enumerate(records):
                    if not w or not h:
                        continue
                    start = base+offset
                    result = diagnostic_walk(data,start,w,h,row['section_offsets'][0],policy)
                    s['streams']+=1
                    s['bounded_success']+=result['success']
                    s['bounded_failure']+=not result['success']
                    s['rows']+=result['rows']
                    s['overshoot_rows']+=result['overshoot_rows']
                    s['max_row_sum']=max(s['max_row_sum'],result['max_row_sum'])
                    position = bisect.bisect_right(starts,start)
                    next_start = starts[position] if position<len(starts) else row['section_offsets'][0]
                    if result['success']:
                        s['end_equals_next_start']+=result['end']==next_start
                        s['end_before_next_start']+=result['end']<next_start
                        s['end_after_next_start']+=result['end']>next_start
                    if key=='0c9d9dbbcb29e39ab29aac17eca97da9a404791febdb79123b6c0eb44af78767' and index==48:
                        examples.append(dict(input_sha256=key,record=index,base=base,policy=policy,
                                             width=w,height=h,start=start,next_distinct_start=next_start,
                                             sprite_data_end=row['section_offsets'][0],**result))
    audit['diagnostic_models'] = dict(status='hypotheses only; no parser policy change',
         grouping='25 affected animal inputs versus 139 other formerly passed inputs',
         next_start_use='Comparison statistic only, never a hard validity boundary.',
         summaries=summaries, dino_record48=examples)
    OUTPUT.write_text(json.dumps(audit,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(audit['diagnostic_models'],indent=2))


def main():
    raw = SOURCE.read_bytes()
    corpus = json.loads(raw)
    rows = [r for r in corpus['results'] if r['status'] == 'passed']
    unique, aliases = {}, {}
    for row in rows:
        key = row['input_sha256']
        aliases.setdefault(key, []).append(row['entry_id'])
        if key in unique:
            assert unique[key]['native_result'] == row['native_result']
            assert unique[key]['decoded_sha256'] == row['decoded_sha256']
        unique[key] = row
    details, violations = [], []
    totals = dict(sprite_records=0, nonempty_streams=0, empty_records=0,
                  empty_offsets_outside_data=0, shared_start_records=0,
                  overlapping_distinct_start_pairs=0, ends_exactly_at_section=0,
                  independent_full_receipts_matched=0, independent_bounded_streams_passed=0)
    for key, row in sorted(unique.items()):
        data = (ROOT / row['private_database']).read_bytes()
        table = (ROOT / row['private_table']).read_bytes()
        assert len(data) == row['decoded_size'] and sha(data) == row['decoded_sha256']
        assert sha(table) == row['embedded_table_sha256']
        _, *offsets = struct.unpack_from('<HIIII', data)
        assert offsets == row['section_offsets']
        begin, end = 18, offsets[0]
        assert begin <= end <= offsets[1] <= offsets[2] <= offsets[3] <= len(data)
        assert data[offsets[0]:offsets[1]] == table and len(table) % 6 == 0
        native = row['native_result']['records']
        assert len(native) == len(table) // 6
        intervals, local_violations = [], []
        empty_outside, empty_count, exact_end = 0, 0, 0
        for index, report in enumerate(native):
            relative, width, height = struct.unpack_from('<IBB', table, index * 6)
            assert (report['record'], report['width'], report['height']) == (index, width, height)
            start = begin + relative
            length, opaque = consumed(data, start, width, height, len(data))
            assert (length, opaque) == (report['consumed_bytes'], report['opaque_pixels'])
            totals['independent_full_receipts_matched'] += 1
            if not width or not height:
                assert length == 0
                empty_count += 1
                empty_outside += not (begin <= start < end)
                continue
            stop = start + length
            intervals.append((start, stop, index))
            exact_end += stop == end
            issue = None
            if start < begin or start >= end:
                issue = 'nonempty-start-outside-sprite-data'
            elif stop > end:
                issue = 'nonempty-stream-crosses-into-metadata'
            if issue:
                item = dict(record=index, relative_offset=relative, start=start,
                            end_exclusive=stop, consumed_bytes=length,
                            sprite_data_end=end, bytes_beyond_data=max(0, stop-end),
                            width=width, height=height, issue=issue)
                local_violations.append(item)
                violations.append(dict(input_sha256=key, entry_ids=aliases[key], **item))
            else:
                assert consumed(data, start, width, height, end) == (length, opaque)
                totals['independent_bounded_streams_passed'] += 1
        sorted_intervals = sorted(intervals)
        starts = {}
        distinct_overlaps = 0
        overlap_examples = []
        for i, (start, stop, index) in enumerate(sorted_intervals):
            starts[start] = starts.get(start, 0) + 1
            for later_start, later_stop, later_index in sorted_intervals[i+1:]:
                if later_start >= stop:
                    break
                if later_start != start:
                    distinct_overlaps += 1
                    if len(overlap_examples) < 8:
                        overlap_examples.append(dict(first_record=index, second_record=later_index,
                                                     first_span=[start,stop], second_span=[later_start,later_stop]))
        shared = sum(count - 1 for count in starts.values())
        detail = dict(input_sha256=key, decoded_sha256=row['decoded_sha256'],
                      entry_ids=aliases[key], sprite_data_begin=begin, sprite_data_end=end,
                      section_offsets=offsets, sprite_records=len(native),
                      nonempty_streams=len(intervals), empty_records=empty_count,
                      empty_offsets_outside_data=empty_outside, shared_start_records=shared,
                      overlapping_distinct_start_pairs=distinct_overlaps,
                      overlap_examples=overlap_examples, ends_exactly_at_section=exact_end,
                      largest_nonempty_end=max((span[1] for span in intervals), default=begin),
                      violations=local_violations)
        details.append(detail)
        for field in ('sprite_records','nonempty_streams','empty_records','empty_offsets_outside_data',
                      'shared_start_records','overlapping_distinct_start_pairs','ends_exactly_at_section'):
            totals[field] += detail[field]
    affected = {v['input_sha256'] for v in violations}
    result = dict(schema_version=1, source_receipt_path=SOURCE.relative_to(ROOT).as_posix(),
                  source_receipt_sha256=sha(raw), source_policy='Superseded full-file decoder extent; historical audit input preserved from Git index.',
                  source_decoder_executable_sha256=corpus['decoder_executable_sha256'],
                  passed_ancestry_records=len(rows), unique_passed_inputs=len(unique),
                  original_execution=False, summary=totals,
                  metadata_traversal_streams=len(violations), metadata_traversal_unique_inputs=len(affected),
                  metadata_traversal_ancestry_records=sum(len(aliases[k]) for k in affected),
                  violations=violations, unique_inputs=details,
                  interpretation='Consumed spans are checked against [18,section0); next sprite offsets are not treated as hard bounds. Empty descriptors consume no bytes. Sharing/overlap alone is not invalidity.',
                  recommendation='Pass anb[:section0] as the decoder database while retaining the 18-byte data base, or provide an ANB-aware API enforcing that section boundary. Generic external-table callers must supply their own proven extent.')
    OUTPUT.write_text(json.dumps(result, indent=2)+'\n', encoding='utf-8')
    print(json.dumps({k:v for k,v in result.items() if k not in ('unique_inputs','violations')}))
    if violations:
        print(json.dumps({'first_violations': violations[:10]}))
    return bool(violations)


if __name__ == '__main__':
    if sys.argv[1:] == ['--diagnostic-models']:
        diagnostic_models()
    else:
        raise SystemExit(main())
