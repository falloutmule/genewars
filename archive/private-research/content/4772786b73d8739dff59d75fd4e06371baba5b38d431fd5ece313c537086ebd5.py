"""Static/model and mocked launch guards; never creates an emulator process."""
from pathlib import Path
import sys,json,contextlib,io,types
from unittest.mock import patch
import input_free_yield as helper
import input_free_probe as probe
from prepare import ROOT,file_record


def main():
    assert not (probe.PROBE/'logs/launch-process.json').exists(), 'Prelaunch-only mutation test: preserve a used probe'
    package=ROOT/'private/review/input-free-yield/capstone'
    sys.path.insert(0,str(package))
    # Capstone 5's optional setuptools resource locator is absent in this
    # private Python runtime. Supply only that locator, without a global install.
    shim=types.ModuleType('pkg_resources')
    shim.resource_filename=lambda package_name,relative:str(package/'capstone'/relative)
    with patch.dict(sys.modules,{'pkg_resources':shim}):
        import capstone
    engine=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_16)
    instructions=[dict(address=i.address,bytes=i.bytes.hex(),mnemonic=i.mnemonic,operands=i.op_str)
                  for i in engine.disasm(helper.CODE,0x100)]
    assert sum(len(bytes.fromhex(i['bytes'])) for i in instructions)==27
    assert [(x['address'],x['operands']) for x in instructions if x['mnemonic'] in ('jne','loop')]==[(0x111,'0x115'),(0x113,'0x10d')]
    receipt=probe.prepare()
    with patch('subprocess.Popen',side_effect=AssertionError('Must not launch')),patch('sys.argv',['input_free_probe.py','launch']),contextlib.redirect_stdout(io.StringIO()):
        probe.main()
    log=('Switched to secure mode.\nThis operation is not permitted in secure mode.\nREFERENCE_GATE_WAITING '+receipt['token']+
         '\nDOS CON: Z:\\>CONFIG -GET sdl "mouse_emulation"\nDOS CON: always\nDOS CON: C   '+str(probe.PROBE/'drive-c')+'\nDOS CON: Z   Internal Virtual Drive\n')
    assert not probe.issues(log,receipt['token'],10)[0]
    for bad in [log.replace('Switched to secure mode.',''),log+'DOS CON: D   bad\n',log.replace(receipt['token'],'BAD')]:
        assert probe.issues(bad,receipt['token'],10)[0]
    assert probe.issues(log,receipt['token'],180)[0]
    binary=probe.PROBE/'drive-c/YIELD.COM'
    binary.write_bytes(helper.CODE+b'X')
    try:
        try:probe.prepare()
        except AssertionError:pass
        else:raise AssertionError('Changed helper admitted')
    finally:binary.write_bytes(helper.CODE)
    assert probe.prepare()==receipt
    model=helper.validate()
    result=dict(status='prepared-not-executed',original_execution=False,reviewer_emulator_execution=False,
        helper=file_record(binary),assembly=file_record(probe.PROBE/'YIELD.asm'),validation=model,disassembly=instructions,
        independent_review='SOL-01 checked branch targets, stack balance, input absence and 65535 poll bound',
        guards=['default launch never Popen','changed helper rejected','missing secure mode rejected','extra mount rejected','wrong nonce rejected','expired watchdog rejected'],
        capstone=dict(version=capstone.__version__,wheel_sha256='1bfa5c81e6880caf41a31946cd6d2d069c048bcc22edf121254b501a048de675',license='BSD3; verified private wheel LICENSE.TXT before extraction',private_locator_shim=True),
        sources=[file_record(x) for x in (ROOT/'private/review/input-free-yield').glob('*.cpp')]+[file_record(ROOT/'private/review/input-free-yield/include-bios.h')],
        prepared=file_record(probe.PROBE/'logs/prepared.json'),
        scripts=[file_record(Path(__file__).parent/n) for n in ['input_free_yield.py','input_free_probe.py','validate_input_free_probe.py']],
        gate=file_record(probe.PROBE/'drive-c/YIELDGAT.BAT'),config=file_record(probe.PROBE/'dosbox-x.conf'),
        no_game_or_disc_mounted=True,root_only_gui_launch=True)
    (ROOT/'research/variants/reference-input-free-probe-readiness.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps(dict(status=result['status'],helper=result['helper'],guards=result['guards'],instructions=len(instructions),model=model)))


if __name__=='__main__':main()
