// @category GeneWars
import ghidra.app.script.GhidraScript;
import ghidra.app.decompiler.*;
import ghidra.program.model.listing.*;
import java.io.*;
public class SurveyFunctions extends GhidraScript {
 public void run() throws Exception {
  File out=new File(getScriptArgs()[0]);out.mkdirs();
  DecompInterface dec=new DecompInterface();dec.openProgram(currentProgram);
  int count=0;
  try(PrintWriter w=new PrintWriter(new File(out,"small-leaf-functions.c"))) {
   var it=currentProgram.getFunctionManager().getFunctions(true);
   while(it.hasNext()&&!monitor.isCancelled()) {
    Function f=it.next();
    if(f.getBody().getNumAddresses()>110||f.getBody().getNumAddresses()<8||!f.getCalledFunctions(monitor).isEmpty())continue;
    var result=dec.decompileFunction(f,10,monitor);
    if(result.decompileCompleted()) {
     w.println("/* "+f.getEntryPoint()+" size "+f.getBody().getNumAddresses()+" */");
     w.println(result.getDecompiledFunction().getC());count++;
    }
   }
  }
  dec.dispose();println("PRIVATE survey exported "+count+" small leaf functions");
 }
}
