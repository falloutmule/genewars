from pathlib import Path
import struct
p=Path(__file__).resolve().parents[2]
w=p/'private/review/bootstrap-context/fbc3cdc5f63a4ebfa01d0e78c2cf5354/seed65535-players1-timer1000/frame14111/boundary'
a=(w/'actors.bin').read_bytes()[200:400]
print('actor1 bytes', {hex(n):a[n] for n in (6,0x16,0x18,0x19,0x49,0x4a,0x4f,0x52,0x53,0x55,0x57,0x70)})
print('actor1 words', {hex(n):struct.unpack_from('<h',a,n)[0] for n in (7,9,11,13,15,0x39,0x3b,0x3d,0x3f,0x41,0x45,0x47,0x49)})
print('height', {hex(n):struct.unpack_from('<i',a,n)[0] for n in (0x11,0x27)})
print('counter', (14111+struct.unpack_from('<h',a,15)[0])&31)
