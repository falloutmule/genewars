"""Bound first campaign mission to exact CD data and inspected instructions.

Static contract only: does not execute the game or claim mission acceptance.
"""
from pathlib import Path
import hashlib
import json
import struct

P = Path(__file__).resolve().parents[2]
G = P / 'private/ghidra'
sha = lambda b: hashlib.sha256(b).hexdigest()

def main():
    expected = {
        'GW-CD-candidate.exe': '4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba',
        'campaign-resources/152.dat': '72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2',
        'campaign-resources/190.dat': '5e1f382ea68f57f1cec03a3f7de2c4903ca127519481da98232a7f1c3a46aa11',
        'validated/memory-.object3.bin': 'aaab09272771ee7f3fbe71e9a3dc75add592fcca42fdd8d982e069629ac1ce87',
    }
    for name, digest in expected.items():
        assert sha((G / name).read_bytes()) == digest, name
    contracts = {
        'campaign-level/instructions-0007d090.txt': ['0007d416 MOVSX EAX,byte ptr [EAX + 0x11945b]', '0007d41d INC EAX', '0007d49e CALL 0x00071b70'],
        'campaign-header-consumer/instructions-0008ab70.txt': ['0008ab7c CMP EDX,0x5', '0008ab83 MOV ESI,dword ptr [EDX*0x4 + 0x1165a0]'],
        'font-menu-state/instructions-00072ed0.txt': ['00072fac MOV AL,byte ptr [EDX + 0x606]', '00072fb2 CMP EAX,0x3', '00072ff6 MOV AL,byte ptr [EDX + 0x6e7]', '0007300a JZ 0x00072faa', '000731d7 OR byte ptr [0x00118770],0x40', '0007320f CALL 0x0002f810'],
        'resource-helpers/instructions-00043f60.txt': ['00043f78 LEA EDI,[EAX + 0x4a1]', '00043f8f CMP DL,0x6', '00043f92 JZ 0x00043fdb', '00044039 MOVSX EAX,byte ptr [ESI + 0x2d]', '0004403d INC byte ptr [EDI + EAX*0x1 + 0x15f]'],
        'actor-panel-font-color/instructions-00024c50.txt': ['00024c83 MOVSX EDX,byte ptr [ESI + 0x2d]', '00024c8e SUB EAX,EDX', '00024c94 MOV AL,byte ptr [ESI + 0x2e]', '00024c97 ADD EAX,EDX', '00024c9e MOV EBP,dword ptr [EAX*0x4 + 0x1160bc]'],
        'mission-contract-refs/43960-range.txt': ['00043982 MOV AL,byte ptr [EAX + 0x104d88]', '0004398f JL 0x000439d2', '00043994 JGE 0x000439d2', '00043998 MOV AL,byte ptr [ECX + 0x104e74]', '000439a5 MOV AL,byte ptr [EBX + EAX*0x1 + 0x15f]', '000439ba MOV AX,word ptr [EBX + 0x16d]', '000439c8 TEST EAX,EBX'],
        'actor-name-trace/instructions-0002ab70.txt': ['0002abd0 MOV AL,byte ptr [ECX + 0x104d00]', '0002abe0 CALL 0x00043960'],
    }
    pins = {}
    for path, lines in contracts.items():
        b = (G / path).read_bytes()
        for line in lines:
            assert line in b.decode(), (path, line)
        pins[path] = sha(b)
    mission = (G / 'campaign-resources/152.dat').read_bytes()
    header = (G / 'campaign-resources/190.dat').read_bytes()
    mapped = (G / 'validated/memory-.object3.bin').read_bytes()
    text = (P / 'private/reference-run/run-006/drive-c/GAME/DATA/TEXT.DAT').read_bytes()
    assert sha(text) == '23e60305a9fa028a0f6541456ab453b7e7cc439f787a2b75e4b8d088e0b01786'
    briefing_index = (0x1165a0 - 0x115e90) // 4
    briefing = text.split(b'\0')[briefing_index]
    for token in [b'Engineer', b'Extractor', b'First Stage Distributor', b'3 Photon Processors', b'Geneticist', b'Mules']:
        assert token in briefing
    assert header[32:40] == b'Exceland'
    assert mission[0x52482] == 1
    assert mission[20 + 0x606] == 0
    assert struct.unpack_from('<H', mission, 0x1ea6)[0] == 0
    assert mission[0x34b8] == 0
    health = struct.unpack_from('<3h', mapped, 0x4d1e + 6 * 6)
    assert health == (80, 200, 360)
    manifest = json.loads((P / 'research/game-files-manifest.json').read_text())['entries']
    asset = next(e for e in manifest if e['entry_id'] == 'root/2/disc/309')
    assert asset['sha256'] == '26ac960d7e68d97bddf7b0c13740976af4cf955ecfe98a4bea51ee95c292958b'
    names = {11: 'Extractor', 3: 'First Stage Distributor', 6: 'Photon Processor'}
    dependencies = {11: 255, 3: 11, 6: 3}
    resources = {11: 'root/2/disc/303', 3: 'root/2/disc/306', 6: 'root/2/disc/309'}
    building_contracts = []
    for selector in [11, 3, 6]:
        assert text.split(b'\0')[139 + 3 * selector].decode() == names[selector]
        assert mapped[0x4d88 + selector * 3] == 0
        assert mapped[0x4e74 + selector] == dependencies[selector]
        resource = next(e for e in manifest if e['entry_id'] == resources[selector])
        building_contracts.append({'selector': selector, 'name': names[selector], 'grade': 0,
            'campaign_progress_threshold': 0, 'requires_completed_selector': None if selector == 11 else dependencies[selector],
            'base_cost_before_signed_skill_discount': struct.unpack_from('<i', mapped, 0x4e1c + selector * 4)[0],
            'health': struct.unpack_from('<h', mapped, 0x4d1e + selector * 6)[0],
            'footprint_parameter': mapped[0x4de0 + selector],
            'resource': {k: resource[k] for k in ['entry_id', 'normalized_iso_path', 'sha256', 'size']}})
    result = {
        'status': 'passed-static-contract', 'original_execution': False,
        'reference': 'Verified CD, volume label GENEWARS QA BETA V5; retail equivalence unestablished',
        'input_sha256': expected, 'listing_sha256': pins,
        'mission': {'progress': 0, 'number': 1, 'title': 'Exceland', 'header': 'LEVELS/HDRCO1.DAT', 'world': 'LEVELS/CONQ001.DAT', 'active_players': 1},
        'briefing': {'text_index': briefing_index, 'sha256': sha(briefing), 'required_actions': ['Engineer constructs Extractor', 'Engineer constructs First Stage Distributor', 'Engineer constructs three Photon Processors'], 'advice': 'Geneticist researches Mules'},
        'winning_predicate': {'campaign_progress': 0, 'selected_player_completed_selector6_count_at_0x606_minimum': 3, 'guards': '72ED0 campaign/selected-player eligible branch; terminal bit clear; complete earlier eligibility logic required', 'other_loaded_objective_fields': {'target_count': 0, 'terrain_object_count': 0}},
        'selector6': {'resource': 'DATA/BUILDING/BSOLA.ANB', 'entry_id': asset['entry_id'], 'size': asset['size'], 'sha256': asset['sha256'], 'health_per_grade': health, 'completion_count': 'player+0x4A1+0x15F+selector, for grade0 completion or loading', 'completion_audio_0x4d': False},
        'required_buildings': building_contracts,
        'availability': {'callee': '43960, called by native construction menu2AB70', 'rule': 'signed progress >= unsigned grade0 threshold and threshold<30; then prerequisite FF, completed prerequisite count>0, or own selector bit already set in player60E', 'first_unlocked_sequence': [11, 3, 6], 'cost_rule': 'base_cost - 2*signed engineer blueprint byte17; debit source player funds, not synthetic mission budget'},
        'outcome': {'selected_win': 'sets runtime118770 bit40 then calls2F810 when mode bit0 clear', 'failure': 'selected eliminated byte6E3 sets runtime80; complete predicates in72ED0 and805E0', 'transition': '2F810 internal result/continue behavior still requires recovery'},
        'unknowns': ['Required building running callbacks, source placement and power/economy behavior not all integrated', 'Complete native campaign startup and live RNG provenance', '2F810 result presentation and progression', 'Cross-variant mission equivalence'],
        'acceptance': {'automated_mission_complete': False, 'native_interactive_mission_complete': False},
    }
    out = P / 'research/campaign'
    out.mkdir(exist_ok=True)
    (out / 'mission-001-contract.json').write_text(json.dumps(result, indent=2) + '\n')
    print('PASS static Exceland contract: progress0, selector6 count>=3, briefing/resource/header pins; gameplay acceptance remains pending')

if __name__ == '__main__':
    main()
