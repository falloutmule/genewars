"""Root-side family checker for the GeneWars Mule family (F1).

Implements the strengthened comparison policy in
private/integration/development-policy-20261017.md:

* a positive gate must show EXECUTION SUCCESS and identity - Rust rc 0,
  result.json status == passed, the intended frame/slot, input roundtrip true and no
  callback error - BEFORE any byte comparison;
* all eight buffers must be compared at their EXACT pinned lengths
  (actors 4000, backing 343958, buildings 17152, creatures 37200, effects 16200,
  family8 1632, players 7220, rng 7; total 427369) before equality is credited;
* zero byte differences, the expected RNG draw count, and the rng.bin write
  (seed + ring = draw order) must match the interpreter output;
* negative gates are reported SEPARATELY and must prove the expected rejection/error
  AND complete transactional preservation (output buffers byte-identical to input);
  rollback-produced equality is never credited as a success.

Usage:
  C:/Python314/python.exe -B scripts/root_family_check.py --tag <unique> [--head <sha>]
        [--campaign <replay.txt>] [--no-focused] [--no-suite]
Each run writes private/integration/<tag>/summary.json and never reuses an output dir.
"""
import argparse
import json
import os
import subprocess
import time
from pathlib import Path

P = Path(__file__).resolve().parents[1]
PY = 'C:/Python314/python.exe'
BUFS = ['actors.bin', 'backing.bin', 'buildings.bin', 'creatures.bin', 'effects.bin',
        'family8.bin', 'players.bin', 'rng.bin']
SIZES = {'actors.bin': 4000, 'backing.bin': 343958, 'buildings.bin': 17152,
         'creatures.bin': 37200, 'effects.bin': 16200, 'family8.bin': 1632,
         'players.bin': 7220, 'rng.bin': 7}
TOTAL = 427369
FOCUSED = r'C:/Users/fallo/Documents/Codex/2026-10-15/9-router-is-installed-to-use/run195_focused_fresh.py'

# mode 'direct': byte-compare the port output against the interpreter's own output dir.
# mode 'oracle': run the owned/neutral oracle with --expected against the port output.
POSITIVE = [
    dict(id='R05-neutral-completed-attack', frame=5914, slot=5, draws=9, mode='direct',
         fixture='private/review/synthetic-source-5914-slot-5-before',
         expected='private/review/neutral-mule-captured-width/5683a83b87bb4ddcb207b4a26da2b84a'),
    dict(id='R06-owned-hit-hold', frame=5916, slot=1, draws=0, mode='direct',
         fixture='private/review/gw5914w9-campaign-20261017T064630/capture/frame-5916-slot-1-blocked-before-owned',
         expected='private/review/owned-mule-captured-width/567ca731e64c4ba092a31cc5e2a8ab87'),
    dict(id='R07-owned-hit-complete', frame=5922, slot=1, draws=0, mode='direct',
         fixture='private/review/gw5922-source-fixture/frame-5922-slot-1-blocked-before-owned',
         expected='private/review/owned-mule-captured-width/4036d5daf3834570827bfa7d1bd15c5b'),
    dict(id='R08-owned-command-ai', frame=5924, slot=1, draws=2, mode='direct',
         fixture='private/review/gw5922-attempt11-campaign/fixture/frame-5924-slot-1-blocked-before-owned',
         expected='private/review/owned-mule-captured-width/723f2066ee5748d4821f4d2de8c68c50'),
    dict(id='R09-neutral-zero-reserve-complete', frame=5938, slot=5, draws=8, mode='direct',
         fixture='private/review/gw5924-attempt12-campaign/fixture/frame-5938-slot-5-blocked-before',
         expected='private/review/neutral-mule-captured-width/54ce64d726194635840afa00f6f0e6bd'),
    # historical focused captures: compare through the oracle (--expected) because the
    # interpreter output dirs for these batches are not retained in the checkout.
    dict(id='R01-owned-light-4801', frame=4801, slot=1, draws=0, mode='oracle', oracle='owned',
         fixture='private/review/authentic-krell-capture4802/frame-4801-slot-1-owned-before',
         instructions=2224),
    dict(id='R02-owned-light-5893', frame=5893, slot=1, draws=0, mode='oracle', oracle='owned',
         fixture='private/review/authentic-krell-194/frame-5892-slot-1-blocked-before-owned',
         instructions=1714),
    dict(id='R03-owned-light-5939', frame=5939, slot=1, draws=0, mode='oracle', oracle='owned',
         fixture='private/review/authentic-krell-195/frame-5939-slot-1-blocked-before-owned',
         instructions=1503),
    # F1-R04 intentionally expanded (2026-10-17): the frame-5940 owned FULL-path
    # transition is now independently source-proved by the owned oracle run on
    # private/review/attempt13-fx-5940 (status source_prepared, 11594
    # instructions, 5 draws, output 606d12841ae14e6b87a2bd999f44e63a, slot1-only
    # deltas 0x19/0x1b..0x1e/0x4b/0x5e/0x66/0x68/0x6c); the former negative gate
    # becomes this direct positive comparison per the family policy's
    # "source-proved and intentionally expanded" condition.
    dict(id='R04-owned-full-path-5940', frame=5940, slot=1, draws=5, mode='direct',
         fixture='private/review/attempt13-fx-5940',
         expected='private/review/owned-mule-captured-width/606d12841ae14e6b87a2bd999f44e63a'),
]
NEGATIVE = []


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--tag', required=True)
    ap.add_argument('--head', default=None, help='expected HEAD sha (skips the assertion when omitted)')
    ap.add_argument('--campaign', default=None, help='replay.txt to run headless')
    ap.add_argument('--no-focused', action='store_true')
    ap.add_argument('--no-suite', action='store_true')
    a = ap.parse_args()

    run = P / f'private/integration/{a.tag}'
    run.mkdir(parents=True, exist_ok=False)
    env = {k: v for k, v in os.environ.items() if not k.startswith('GENEWARS_CAPTURED_MULE_')}
    env.update(CARGO_BUILD_JOBS='2', CARGO_PROFILE_TEST_OPT_LEVEL='1')
    results = []

    def sh(args, logname, extra=None):
        e = dict(env)
        if extra:
            e.update(extra)
        log = run / logname
        with log.open('wb') as f:
            rc = subprocess.run(args, cwd=P, env=e, stdout=f, stderr=subprocess.STDOUT).returncode
        return rc, log.read_text(errors='replace')

    def rec(step, ok, detail, kind='check'):
        results.append({'step': step, 'ok': bool(ok), 'kind': kind, 'detail': detail})
        print(('PASS ' if ok else 'FAIL ') + step + ' :: ' + json.dumps(detail)[:420], flush=True)

    out_root = run / 'out'
    out_root.mkdir()

    def captured(fixture, frame, slot, outdir, expect=None):
        e = {'GENEWARS_CAPTURED_MULE_INPUT': str(fixture), 'GENEWARS_CAPTURED_MULE_OUTPUT': str(outdir),
             'GENEWARS_CAPTURED_MULE_FRAME': str(frame), 'GENEWARS_CAPTURED_MULE_SLOT': str(slot)}
        if expect:
            e['GENEWARS_CAPTURED_MULE_EXPECT'] = expect
        return sh([PY, '-B', 'scripts/cargo.py', 'test', '-p', 'genewars-runtime', 'captured_mule_callback',
                   '--offline', '--locked', '--', '--ignored', '--nocapture'],
                  f'{outdir.name}.log', e)

    def geometry(port_dir, expected_dir=None):
        """Exact-length gate over the port output (and, when given, the interpreter output)."""
        got, bad = {}, []
        for b in BUFS:
            p = port_dir / b
            if not p.exists():
                bad.append(f'{b}:missing'); continue
            n = p.stat().st_size
            got[b] = n
            if n != SIZES[b]:
                bad.append(f'{b}:{n}!={SIZES[b]}')
            if expected_dir is not None:
                q = expected_dir / b
                if not q.exists() or q.stat().st_size != n:
                    bad.append(f'{b}:expected-mismatch')
        tot = sum(got.values())
        if tot != TOTAL:
            bad.append(f'total:{tot}!={TOTAL}')
        return got, bad

    def diffs(port_dir, expected_dir):
        out = {}
        for b in BUFS:
            p, q = port_dir / b, expected_dir / b
            if not (p.exists() and q.exists()):
                out[b] = 'missing'; continue
            x, y = p.read_bytes(), q.read_bytes()
            if len(x) != len(y):
                out[b] = f'length {len(x)}!={len(y)}'; continue
            n = sum(1 for i in range(len(x)) if x[i] != y[i])
            if n:
                first = [i for i in range(len(x)) if x[i] != y[i]][:4]
                out[b] = f'{n} diffs first@{first}'
        return out

    rc, _ = sh([PY, '-B', 'scripts/cargo.py', 'test', '--workspace', '--offline', '--locked', '--no-run'], 'compile.log')
    rec('compile-workspace', rc == 0, {'rc': rc})

    for c in POSITIVE:
        fixture = P / c['fixture']
        expected = P / c['expected'] if c.get('expected') else None
        outdir = out_root / c['id']
        rc, _ = captured(fixture, c['frame'], c['slot'], outdir)
        res = json.loads((outdir / 'result.json').read_text()) if (outdir / 'result.json').exists() else None
        # 1) execution/identity gate
        exec_ok = (rc == 0 and res is not None and res.get('status') == 'passed'
                   and res.get('frame') == c['frame'] and res.get('slot') == c['slot']
                   and res.get('input_roundtrip') is True and not res.get('error'))
        rec(f"{c['id']}-execution", exec_ok, {'rc': rc, 'rust': res}, kind='positive')
        got, bad = geometry(outdir, expected if c['mode'] == 'direct' else None)
        rec(f"{c['id']}-exact-lengths", not bad, {'lengths': got, 'problems': bad}, kind='positive')
        pd = diffs(outdir, expected) if expected is not None else {}  # oracle mode: the oracle compares
        rng_in = (fixture / 'rng.bin').read_bytes().hex() if (fixture / 'rng.bin').exists() else None
        rng_out = (outdir / 'rng.bin').read_bytes().hex() if (outdir / 'rng.bin').exists() else None
        rng_exp = (expected / 'rng.bin').read_bytes().hex() if expected is not None and (expected / 'rng.bin').exists() else None
        draws_ok = res is not None and res.get('rng_draws') == c['draws']
        if c['mode'] == 'direct':
            rec(f"{c['id']}-rng+bytes", (not pd) and draws_ok and rng_out == rng_exp,
                {'draws': None if res is None else res.get('rng_draws'), 'expected_draws': c['draws'],
                 'rng_in': rng_in, 'rng_out': rng_out, 'rng_expected': rng_exp, 'buffer_diffs': pd}, kind='positive')
        else:
            rc2, txt2 = sh([PY, '-B', f"research/variants/{c['oracle']}-mule-captured-width-review.py",
                            '--before', str(fixture), '--expected', str(outdir),
                            '--frame', str(c['frame']), '--slot', str(c['slot'])], f"{c['id']}-oracle.log")
            try:
                d = json.loads(txt2)
            except Exception as ex:
                d = {'parse_error': repr(ex), 'tail': txt2[-200:]}
            ok = (rc2 == 0 and d.get('status') == 'passed' and d.get('compared_bytes') == TOTAL
                  and not d.get('differences') and d.get('draws') == c['draws'] and draws_ok)
            rec(f"{c['id']}-oracle+bytes", ok,
                {k: d.get(k) for k in ('status', 'draws', 'instructions', 'compared_bytes', 'differences', 'parse_error')}
                | {'expected_instructions': c.get('instructions'), 'rust_draws': None if res is None else res.get('rng_draws')},
                kind='positive')

    for c in NEGATIVE:
        fixture = P / c['fixture']
        outdir = out_root / c['id']
        rc, _ = captured(fixture, c['frame'], c['slot'], outdir, expect='blocked')
        res = json.loads((outdir / 'result.json').read_text()) if (outdir / 'result.json').exists() else None
        rec(f"{c['id']}-rejection", rc == 0 and res is not None and res.get('status') == 'blocked'
            and c['expect_error'] in (res.get('error') or ''), {'rc': rc, 'rust': res}, kind='negative')
        got, bad = geometry(outdir)
        pd = diffs(outdir, fixture)
        rec(f"{c['id']}-transactional-preservation", (not bad) and (not pd),
            {'lengths': got, 'problems': bad, 'buffer_diffs_vs_input': pd}, kind='negative')

    if not a.no_focused:
        rc, txt = sh([PY, '-B', FOCUSED], 'focused195.log')
        rec('focused-batch-9of9', rc == 0 and '"passed": 9' in txt,
            {'rc': rc, 'tail': txt.strip().splitlines()[-4:]})
    if not a.no_suite:
        rc, txt = sh([PY, '-B', 'scripts/cargo.py', 'test', '-p', 'genewars-runtime', '--offline', '--locked'], 'suite.log')
        lines = [l for l in txt.splitlines() if l.startswith('test result:')]
        rec('runtime-suite', rc == 0 and any('260 passed' in l and '0 failed' in l for l in lines), {'results': lines})

    if a.campaign:
        exe = P / 'private/build/rust-target/release/genewars-game.exe'
        rc, _ = sh([PY, '-B', 'scripts/cargo.py', 'build', '--release', '-p', 'genewars-desktop',
                    '--features', 'genewars-runtime/callback-captures', '--offline', '--locked'], 'release.log')
        rec('release-build', rc == 0, {'rc': rc, 'exe': str(exe)})
        camp = run / 'campaign'
        camp.mkdir()
        e = {'GENEWARS_MULE_CALLBACK_CAMPAIGN_PROGRESS': '1', 'GENEWARS_MULE_CALLBACK_FIXTURE': str(camp / 'capture')}
        (camp / 'capture').mkdir()
        rc, _ = sh([str(exe), str(P / 'private/integration/game-runtime'), '--reference-startup', 'fresh',
                    '--headless', str(P / a.campaign), '--report', str(camp / 'report.json'),
                    '--bmp', str(camp / 'frontier.bmp')], 'campaign.log', e)
        rep = json.loads((camp / 'report.json').read_text()) if (camp / 'report.json').exists() else {}
        m = rep.get('mission', {})
        rec('campaign-run', rc == 0 or bool(m),
            {'rc': rc, 'steps': m.get('steps'), 'blocked': m.get('blocked'),
             'world_sha256': m.get('world_sha256'), 'rgb_sha256': m.get('rgb_sha256'),
             'campaign_progress': m.get('campaign_progress')})

    if a.head:
        head = subprocess.run(['git', 'rev-parse', 'HEAD'], cwd=P.parent, capture_output=True, text=True).stdout.strip()
        rec('head-unchanged', head.startswith(a.head), {'head': head, 'expected': a.head})
    staged = subprocess.run(['git', 'diff', '--cached', '--name-only'], cwd=P.parent,
                            capture_output=True, text=True).stdout.strip()
    rec('staged-empty', staged == '', {'staged': staged.splitlines()})

    pos = [r for r in results if r['kind'] == 'positive']
    neg = [r for r in results if r['kind'] == 'negative']
    summary = {
        'tag': a.tag, 'run': str(run.relative_to(P)), 'head': subprocess.run(
            ['git', 'rev-parse', 'HEAD'], cwd=P.parent, capture_output=True, text=True).stdout.strip(),
        'positive': {'passed': sum(1 for r in pos if r['ok']), 'total': len(pos)},
        'negative': {'passed': sum(1 for r in neg if r['ok']), 'total': len(neg)},
        'checks': {'passed': sum(1 for r in results if r['ok']), 'total': len(results)},
        'all_ok': all(r['ok'] for r in results),
        'steps': results,
    }
    (run / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n', encoding='utf-8')
    print('===== FAMILY SUMMARY =====')
    print(json.dumps({k: summary[k] for k in ('positive', 'negative', 'checks', 'all_ok')}))


if __name__ == '__main__':
    main()
