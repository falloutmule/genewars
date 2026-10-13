// @category GeneWars
import ghidra.app.script.GhidraScript;
import java.io.*;
public class ExportRange extends GhidraScript {
 public void run() throws Exception {
  String[] a=getScriptArgs();File out=new File(a[0]);out.getParentFile().mkdirs();
  var start=toAddr(a[1]);var end=toAddr(a[2]);
  if(getInstructionAt(start)==null)disassemble(start);
  try(PrintWriter w=new PrintWriter(out)) {
   var it=currentProgram.getListing().getInstructions(start,true);
   while(it.hasNext()){var i=it.next();if(i.getAddress().compareTo(end)>0)break;w.println(i.getAddress()+" "+i);}
  }
 }
}
