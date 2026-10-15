"""Source4FF2C actor+C0 blueprint pointers from actual input, never expected output.
Only player0's typed roster with opaque zero C0 pointers is supported.5D700
reads category+16/boost+17 through these pointers. Restore input C0 only if
the callback kept the reconstructed native pointer unchanged.
"""
import struct
def bind(state,buffers,g):
 actors=buffers['actors.bin'];players=buffers['players.bin'];count=players[3]
 if count>5:raise ValueError('unsupported player0 specialist roster extent')
 slots=list(players[4:4+count])
 if len(set(slots))!=len(slots):raise ValueError('duplicate specialist roster')
 rows=[]
 for slot in slots:
  if slot>=5:raise ValueError('foreign player0 roster actor')
  r=actors[slot*200:(slot+1)*200];selector=r[0x74]
  if r[6]!=3 or r[24]!=0 or struct.unpack_from('<h',r,15)[0]!=slot or selector>=32 or any(r[0xc0:0xc4]):
   raise ValueError('requires typed player0 actor with opaque zero blueprint pointer')
  offset=0x16+selector*36;address=0x118d74+offset
  g.k.put(state,0x16360e+slot*200+0xc0,4,address)
  rows.append({'slot':slot,'owner':0,'selector':selector,'address':address,'category':players[offset+0x16],'boost':players[offset+0x17]})
 return rows
def restore(result,buffers,rows):
 actors=bytearray(result['actors.bin'])
 for row in rows:
  at=row['slot']*200+0xc0
  if int.from_bytes(actors[at:at+4],'little')!=row['address']:
   raise ValueError('source changed bound actor blueprint pointer')
  actors[at:at+4]=buffers['actors.bin'][at:at+4]
 result['actors.bin']=bytes(actors)
