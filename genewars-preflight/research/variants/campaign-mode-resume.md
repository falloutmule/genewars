# Campaign mode resume

The mode2 pause left by victory is cleared by the shared view dispatcher before successor mission gameplay. This is source-inspected behavior, not dynamic transition acceptance.

After old-view exit at1BCC3 and new-view entry at1BCD1,1BCE0 tests mode118968 bit2.1BCEA tests newly current module byte+6 bit1. When mode2 is set and module bit1 is clear,1BCF2 loads savedframe118786,1BCF7 clears modebit2,1BCFA restores frame11877E, and1BCFF storesmode.

For Campaign186F19, mapped111F34 is9D1 and base186548+9D1=186F19. Mapped module bytes startzero, includingflags186F1F. Constructor15B10 installs metadata/vtables withoutsettingflag6; entry15B90 sets currentmodule at15CB8 withoutsettingthatflag ormode. Therefore Victory-to-Campaign entry satisfies the dispatcher resume predicate: mode2 becomes0 and frame temporarily restores savedvictoryframe.

The later planet-view exit7D020 resetsframe0.71B70 loads CONQ002 runtime frame0 andsavedframe0.7D090 inherits resumedmode0. MAINPANLconstructor2BA8A sets its moduleflag6 bit1, but no pendingmode2 remains at thattransition. Do not implement unconditionalmodeclear in25150,7D090 orMAINPANLentry.

Exact source/data hashes and addresses are in campaign-mode-resume.json. Only existing exported listings and mapped initialdata were inspected; no originalexecution or newtest was performed.
