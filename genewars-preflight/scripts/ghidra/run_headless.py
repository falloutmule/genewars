"""Launch only Ghidra, with settings/cache/home/logs rooted inside private/ghidra."""
from pathlib import Path
import subprocess, sys
P=Path(__file__).resolve().parents[2]
tool=P/'private/tools/ghidra/ghidra_12.0.1_PUBLIC'
jdk=next((P/'private/tools/jdk').glob('jdk-*'))
scratch=P/'private/ghidra'
for name in ['home','settings','cache','tmp','projects']:
    (scratch/name).mkdir(exist_ok=True)
classes=scratch/'launcher-classes'
classes.mkdir(exist_ok=True)
subprocess.run([str(jdk/'bin/javac.exe'),'-d',str(classes),str(Path(__file__).with_name('MemoryPreferencesFactory.java'))],check=True)
args=[str(jdk/'bin/java.exe'),'-Xmx2G','-Xshare:off','-Djava.system.class.loader=ghidra.GhidraClassLoader','-Djava.util.prefs.PreferencesFactory=MemoryPreferencesFactory',
      '-Dfile.encoding=UTF8','-Duser.language=en','-Duser.country=US','-Dlog4j.skipJansi=true',
      '-Djavax.xml.accessExternalDTD=','-Djavax.xml.accessExternalSchema=','--enable-native-access=ALL-UNNAMED']
for key,name in [('user.home','home'),('application.settingsdir','settings'),('application.cachedir','cache'),('application.tempdir','tmp'),('java.io.tmpdir','tmp')]:
    args.append('-D'+key+'='+str(scratch/name))
args += ['-cp',str(classes)+';'+str(tool/'Ghidra/Framework/Utility/lib/Utility.jar'),'ghidra.Ghidra',
         'ghidra.app.util.headless.AnalyzeHeadless',str(scratch/'projects'),'GW_CD_Candidate']+sys.argv[1:]
print('Launching pinned isolated Ghidra; game bytes are read-only analysis input.',flush=True)
sys.exit(subprocess.call(args,cwd=P))
