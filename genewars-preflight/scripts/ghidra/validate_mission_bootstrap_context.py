"""Pin mission-loop inputs separately from the native development route.

This is a static instruction/data contract, not a whole-startup execution.
Commercial bytes and raw listings remain private.
"""
import hashlib
import json
from pathlib import Path
import struct

ROOT = Path(__file__).resolve().parents[2]
GHIDRA = ROOT / 'private/ghidra'
EXE_SHA = '4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba'
MISSION_SHA = '72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2'


def digest(data):
    return hashlib.sha256(data).hexdigest()


def main():
    assert digest((GHIDRA / 'GW-CD-candidate.exe').read_bytes()) == EXE_SHA
    source = (GHIDRA / 'campaign-resources/152.dat').read_bytes()
    assert digest(source) == MISSION_SHA
    listings = {}

    def pin(path, lines):
        raw = (GHIDRA / path).read_bytes()
        text = raw.decode('utf-8')
        for line in lines:
            assert line in text, (path, line)
        listings[path] = digest(raw)

    pin('campaign-level/instructions-00071b70.txt', [
        '00072340 MOV EBX,0x34', '00072345 MOV EDX,0x118770',
        '0007234c CALL 0x0009a420'])
    pin('terrain-camera-refs/instructions-0007d090.txt', [
        '0007d4a3 TEST byte ptr [0x00118770],0x20',
        '0007d4aa JNZ 0x0007d4b4', '0007d4af MOV [0x0011878c],AL',
        '0007d4b4 MOV EDX,0x3e8', '0007d4bb MOV word ptr [0x0011c2bc],DX',
        '0007d4c2 CALL 0x000884d0', '0007d4c7 OR byte ptr [0x00118770],0x2',
        '0007d6b4 MOV AL,[0x0011878c]', '0007d6bb JLE 0x0007d76a'])
    pin('campaign-actions/instructions-00016170.txt', [
        '00016212 AND byte ptr [0x00118968],0xfe',
        '00016219 OR byte ptr [0x00118770],0x20'])
    pin('font-menu-state/instructions-0001bbc0.txt', [
        '0001bd3b CALL 0x0001c120', '0001bd40 CALL 0x0008dfe0',
        '0001bd4a CALL 0x00040c60', '0001bd4f INC dword ptr [0x0011877e]'])
    pin('campaign-refs/instructions-0001c120.txt', [
        '0001c73b TEST byte ptr [0x00118770],0x4', '0001c742 JZ 0x0001c7e2',
        '0001c748 TEST byte ptr [0x00118968],0x2', '0001c74f JNZ 0x0001c7e2',
        '0001c76d CALL 0x00058780', '0001c778 CALL 0x00047f80',
        '0001c785 CALL 0x0006a540', '0001c790 CALL 0x0006d0b0',
        '0001c79d CALL 0x00013780', '0001c7ca CALL 0x00079220',
        '0001c7d4 CALL 0x00072ed0'])
    pin('entity-order/instructions-0002bbe0.txt', [
        '0002bdb9 CMP dword ptr [EBP + 0xc8],0x0', '0002bdc0 JNZ 0x0002be41',
        '0002be19 CMP EAX,0x4', '0002be1c JL 0x0002be2d',
        '0002be35 CALL 0x00025150', '0002be3a OR byte ptr [0x00118770],0x4'])
    pin('mission-array-refs/instructions-00058780.txt', ['00058795 MOV AL,[0x0011878c]'])
    pin('mission-array-refs/instructions-00047f80.txt', ['000482d3 MOV AL,[0x0011878c]'])
    pin('mission-array-refs/instructions-00013780.txt', ['00013840 MOV AL,[0x0011878c]'])
    pin('mission-array-refs/instructions-0006a540.txt', [
        '0006a562 MOVZX ESI,byte ptr [0x0011878c]', '0006a5b4 CMP ESI,0x4'])
    pin('terrain-map-refs/instructions-00072ed0.txt', [
        '00072efe MOV EDX,dword ptr [ESI + 0x1ec2]', '00072f08 MOV EAX,[0x0011877e]',
        '00072f13 MOV AL,[0x0011873d]', '00072f28 CALL 0x0007f3b0'])

    raw_flags, raw_frame, active_count = source[0x52466], struct.unpack_from('<I', source, 0x52474)[0], source[0x52482]
    countdown = struct.unpack_from('<H', source, 0x355c)[0]
    deferred_frame = struct.unpack_from('<I', source, 0x1ec2)[0]
    assert (raw_flags, raw_frame, active_count, countdown, deferred_frame) == (0x38, 0, 1, 0, 0)
    record = {
        'schema_version': 1,
        'status': 'passed-static-mission-bootstrap-context-contract',
        'original_execution': False,
        'whole_startup_executed': False,
        'candidate_sha256': EXE_SHA, 'mission_sha256': MISSION_SHA,
        'listing_sha256': listings,
        'source_fields': {
            'runtime_flags_118770': raw_flags,
            'frame_counter_11877e': raw_frame,
            'active_player_count_11878c': active_count,
            'serialized_countdown_11c2bc': countdown,
            'deferred_event_frame_backing_1ec2': deferred_frame,
        },
        'corroborated_instruction_facts': [
            '71B70 reads the 52-byte serialized tail into118770; it includes flags, frame counter and active-player count.',
            '7D090 campaign branch preserves loaded active count, writes countdown1000, calls884D0 and then ORs flags with2.',
            '16170 clears only mode118968 bit0; it does not establish that the whole mode byte is zero.',
            '1C120 performs simulation callbacks only with flags bit4 set and mode bit2 clear.',
            'The conditional initial MAINPANL path through2BBE0 sets flags bit4 after25150; object+C8 and prior module context remain required.',
            '1BBC0 increments11877E after1C120 and presentation calls. Its value is a frame counter; selected player is11873D.',
            '58780/47F80/13780 use the active-player count;6A540 uses it for player maintenance while creature owner4 is independently neutral.',
            '72ED0 compares backing1EC2 to the frame counter before its deferred selected-player event.',
        ],
        'native_development_route_gaps': [
            'SharedMission currently supplies4 to player schedulers, though this mission file contains1.',
            'The development route starts its countdown from serialized backing rather than executing the7D090 countdown1000 write.',
            'The route advances frame context before callbacks; original1BBC0 increments after them. Module/startup timing before the first world pass remains unestablished.',
            'Runtime mode, module gates, inherited native priors, seed generation and all transitive startup effects are not a completed original-startup reconstruction.',
        ],
        'next_step': 'Admit explicit source-backed bootstrap context and close its transitive gates before treating the development route as original campaign entry or enabling72ED0 outcomes.',
    }
    destination = ROOT / 'research/ghidra/mission-bootstrap-context-evidence.json'
    destination.write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
    print(f'PASS: {len(listings)} pinned listings; mission frame/count/countdown distinguished from development settings')


if __name__ == '__main__':
    main()
