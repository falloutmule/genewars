// Read-only listing export; never disassembles a possibly interior start byte.
// @category GeneWars
import ghidra.app.script.GhidraScript;
import java.io.*;
public class ExportExistingRange extends GhidraScript {
 public void run() throws Exception {
  String[] args=getScriptArgs();File out=new File(args[0]);out.getParentFile().mkdirs();
  var start=toAddr(args[1]);var end=toAddr(args[2]);
  try(PrintWriter w=new PrintWriter(out)) {
   var it=currentProgram.getListing().getInstructions(start,true);
   while(it.hasNext()){var ins=it.next();if(ins.getAddress().compareTo(end)>0)break;w.println(ins.getAddress()+" "+ins);}
  }
 }
}
