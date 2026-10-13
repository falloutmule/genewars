// @category GeneWars
// Explicit analysis-project correction for loader permissions, never input bytes.
import ghidra.app.script.GhidraScript;
public class RepairObjectPermissions extends GhidraScript {
 public void run() throws Exception {
  if(!currentProgram.getExecutableSHA256().equals("4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba"))throw new IllegalStateException("Wrong candidate");
  int[] flags={0x2045,0x45,0x2043,0x2043};
  for(int i=0;i<flags.length;i++) {
   var block=currentProgram.getMemory().getBlock(".object"+(i+1));
   if(block==null)throw new IllegalStateException("Missing object");
   block.setRead((flags[i]&1)!=0);block.setWrite((flags[i]&2)!=0);block.setExecute((flags[i]&4)!=0);
   println("Object "+(i+1)+" permissions restored from verified LE flags "+Integer.toHexString(flags[i]));
  }
 }
}
