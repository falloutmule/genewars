# Human Genepod command producer: bounded UI contract

Reference QA BETA V5. This is recovered source behavior, not yet native UI acceptance. The human producer is21E20 action8, not814A0 itself. Packet opcode0C (decimal12) is written at2256C(start) and2266D(cancel), with speciesword0, buildingslotword2, startword4=1 or0.

## Selection and navigation

Panel initialization21A10 binds the selected building at panel+4 and sets panel7D from building3C (21B19..21B1C). It installs the Genepod control chain106D6C via106BAA (21ADC..21AE7). Previous/next actions9/10 use panel7D as an index in selected-player list6AE; selected species is signed byte player[6AE+index], read2249B..224AD. List count is player6E0, read21FF2. Previous is disabled at index0 or positive work; next is disabled at index>=count-1 or positive work (21FAA..22032). The arrow actions set panel7E=-1/+1 and panel7F=0; rendering transition22FD0 applies the index change when animation7F reaches4 at233A8..233CA. These transitions use separate UI RNG1169A0, not shared gameplay RNG.

Mapped control records,40 bytes each: previous106D6C action9 flags9008 rectangle[18,277,30,53]; next106D94 action10 flags9008 rectangle[79,277,30,53]. Production106DBC action8 flags9128 origin[26,344], raw descriptor offset6 (six-byte record index1; A8320 treats this as a byte offset); its mapped width/height are zero and need the original resource-bound control dimensions, not an invented rectangle. The chain at106DE4 links back106BDC. Global mouse19FF70 bit8 is the accepted source input-event condition here; bits8001 suppress activation while held/canceled. Exact ordinary hit testing must continue through the existing original control dispatcher; this report does not identify an invented keyboard shortcut.

## Start eligibility and feedback

At21E55..21E83 the panel admits owned or allied buildings; enemy selection requests closing the panel. Generic panel enable flags also depend on ownership/completion (21E96..21F0A), so the initial allied allowance alone is not proof of every control being enabled for allies.

Action8 enters223E3. Start path requires building animationframe1B==0 and signed work36<=0. Panelstate8 must be>=2; inputword19FF70&8001 must be0 and bit8 set. Buildingflags31 must have bit0001, lack0040, and lack0100 (22443..22487).

Species comes from the selected player's list, as above. Signed wrapped funds4A3+4A7 must be>=mapped basecost10461C+species*45 (224C7..224E3), even though the handler charges incrementally. Grade0 permits only species<5; blocked higher species additionally requests message76620 id2B. A nonzero grade bypasses that species<5 restriction. Selectedplayer research dword61A+species*4 must be nonzero (22528..22542); no greater threshold occurs here. Passing emits sound2C and the start packet; failing funds/research emits sound26, not a gameplay loss. Mule species4/basecost75 therefore requires total funds>=75 and researchentry4 nonzero for ordinary UI initiation.

No creature-capacity check occurs in this start branch. Capacity/finalfactory failure belongs to subsequent production, not an invented UI rejection. Positive work prevents starting another job through this producer even though the raw814A0 handler accepts replacing work.

## Cancel eligibility

When start-idle conditions fail,225BC enters the cancel path. It repeats buildingflags admission and requires signedwork36>0. Input8001clear/bit8set is required; selectedplayer11873D must equal building3A (the recorded payer), not merely buildingowner. Success writes current buildingspecies33 andslot0F, startword0. Nonpayer uses denial feedback at22689. Start and cancel do not directly mutate production work; they emit the ordinary packet consumed by1C120/814A0.

## Remaining smallest integration dependencies

Recover resource-bound action8 hit dimensions and confirm the generic panel enable/hit chain for the intended owned grade0 Genepod. Read actual Krell player's species list/count and researchentry4 before offering Mule; do not substitute species4 for list index4. A future native producer should share these gates and packet dispatch with automated commands. Source-controlled producer tests and native hit testing are still not run. Positive production/finalspawn remains separately unsupported until its own proof, irrespective of usable start/cancel.

Sources: building-cost-refs/instructions-00021E20 (existing lowercase filename), plant-command/instructions-00021a10, actor-panel-font-color/instructions-00022fd0, portrait-detail-refs/instructions-00022760, and mappedobject3 control records. Source pins accompany this report.

Descriptor binding corroboration: A836B adds control rawoffset to sprite table; A8376..A838A reads descriptor bytes4/5 as width/height.2BD93..2BDA4 binds106D6C through MAINPANL+F3 generatedfrom+EB; mapped resource descriptor105698 identifies data/genepanl.anb with destination185EE7=185DFC+EB. Thus rawoffset6 selects six-byte descriptor1, not descriptor6.

Follow-up research/campaign/krell-mule-research-command.md/.json establishes originalGENEPANL.ANB sprite1 dimensions29x29,665opaque pixels and center14,14 hit via literal98B60 eight-case proof. This resolves the resource dimensions boundary above; native input acceptance remains separate.
