"""Manifest-bound next-campaign resources, using the shared RNC stager."""
from pathlib import Path
from result_inputs import stage as stage_rows
ROOT=Path(__file__).resolve().parents[1]
ROWS=[['LEVSCR01.DAT', 'root/2/disc/93', 'fa70fb20318dceab350402fe962744248f9127bb91b64cb0b11526f9bb7f88d6', 105717, '761701f542363a867b8330b9a63f54f8659e6195b96f26824bf301b967a2f8c3', 307200], ['LEVSCR01.PAL', 'root/2/disc/94', '678f226e78ab61eb7ec30afa6c54f2686dca550850e0267a4824e79e7ec1e44d', 706, '9d08b13c6d0864a48a3a6c37dc9c5766b7443e65549328763d17e264e69cfbf2', 768], ['CONQ002.DAT', 'root/2/disc/153', '4042c6df3b161132c12a424dace7b9fcbf253a197261f1cf76e0db4af41ea27a', 343958, '4042c6df3b161132c12a424dace7b9fcbf253a197261f1cf76e0db4af41ea27a', 343958], ['HDRCO2.DAT', 'root/2/disc/201', '5a42310026365e8c36ddce8ecba48fe099c42accc6c878933d2360c1ebb6e80c', 699, '5a42310026365e8c36ddce8ecba48fe099c42accc6c878933d2360c1ebb6e80c', 699]]
def stage():return stage_rows(ROWS,ROOT/"private/integration/campaign-inputs")
if __name__=="__main__":print(stage())
