"""Independently authored, input-free, bounded 8086 DOS.COM polling helper.

This is a guest-instruction yield opportunity, not a calibrated wall-clock delay.
It neither changes IF nor reads/generates keyboard input. Host watchdog required.
"""

CODE = bytes.fromhex('1e b8 40 00 8e d8 8b 1e 6c 00 b9 ff ff '
                     '3b 1e 6c 00 75 02 e2 f8 1f b8 00 4c cd 21')

ASSEMBLY = '''; Independently authored 8086 DOS.COM; ORG 0100h
; Assumes normal DOS COM stack. Read-only BDA word, no input/ports/files.
org 0100h
push ds
mov ax,0040h
mov ds,ax
mov bx,[006ch]
mov cx,0ffffh
wait_tick:
cmp bx,[006ch]
jne done
loop wait_tick
done:
pop ds
mov ax,04c00h
int 021h
'''


def verified_probe(root):
    """Bind the actual empty supervised test before using this in an original gate."""
    import json,hashlib
    path=root/'research/variants/reference-input-free-probe-result.json'
    result=json.loads(path.read_bytes())
    assert result['original_execution'] is False
    assert result['gate_released'] and result['sentinel_observed'] and not result['aborted']
    assert result['final_process']['exit_code']==0 and not result['final_process']['host_timeout']
    assert result['helper']['sha256']==hashlib.sha256(CODE).hexdigest()
    observed=result['observations'];private=root/observed['file']['path']
    assert private.resolve().is_relative_to((root/'private/reference-run/input-free-gate-probe-002').resolve())
    assert hashlib.sha256(private.read_bytes()).hexdigest()==observed['file']['sha256']
    assert observed['data']['activation']=='succeeded' and observed['data']['waiting_nonce_visible']
    return dict(path=path.relative_to(root).as_posix(),size=path.stat().st_size,
                sha256=hashlib.sha256(path.read_bytes()).hexdigest())


def interpret(ticks, ds=0x1234):
    """Independent small instruction interpreter for the complete helper bytes.

    ticks supplies successive BDA word reads; last value persists. This models
    the exact branch bytes and register widths; it does not emulate DOS/IRQs.
    """
    values = iter(ticks)
    last = next(values)
    def read_tick():
        nonlocal last
        value = last
        last = next(values, last)
        return value & 0xffff
    registers = {'ax':0, 'bx':0, 'cx':0, 'ds':ds}
    stack=[];pc=0;zero=False;polls=0;steps=0
    while True:
        steps+=1
        assert steps < 200000, 'Helper escaped finite bound'
        op=CODE[pc]
        if op==0x1e:stack.append(registers['ds']);pc+=1
        elif op in (0xb8,0xb9):
            registers['ax' if op==0xb8 else 'cx']=int.from_bytes(CODE[pc+1:pc+3],'little');pc+=3
        elif CODE[pc:pc+2]==b'\x8e\xd8':registers['ds']=registers['ax'];pc+=2
        elif CODE[pc:pc+4]==b'\x8b\x1e\x6c\x00':
            assert registers['ds']==0x40;registers['bx']=read_tick();pc+=4
        elif CODE[pc:pc+4]==b'\x3b\x1e\x6c\x00':
            assert registers['ds']==0x40;zero=registers['bx']==read_tick();polls+=1;pc+=4
        elif op==0x75:
            displacement=int.from_bytes(CODE[pc+1:pc+2],'little',signed=True)
            pc+=2+(displacement if not zero else 0)
        elif op==0xe2:
            registers['cx']=(registers['cx']-1)&0xffff
            displacement=int.from_bytes(CODE[pc+1:pc+2],'little',signed=True)
            pc+=2+(displacement if registers['cx']!=0 else 0)
        elif op==0x1f:registers['ds']=stack.pop();pc+=1
        elif CODE[pc:pc+2]==b'\xcd\x21':
            assert registers['ax']==0x4c00 and registers['ds']==ds and not stack
            return dict(polls=polls,steps=steps,exit_code=0,ds_restored=True)
        else:raise AssertionError(('Unexpected instruction',pc,op))


def validate():
    assert len(CODE)==27
    unchanged=interpret([0x1234]);assert unchanged['polls']==65535
    wrap=interpret([0xffff,0]);assert wrap['polls']==1
    late=interpret([7]*65535+[8]);assert late['polls']==65535
    for tick in range(65536):
        assert interpret([tick,(tick+1)&0xffff])['polls']==1
    return dict(size=len(CODE),unchanged_tick=unchanged,low16_wrap=wrap,
                last_poll_change=late,exhaustive_successor_ticks=65536,
                model_limitations='Only listed guest instructions modeled; no IRQ, DOS or SDL execution')


if __name__=='__main__':
    import json
    print(json.dumps(validate(),indent=2))
