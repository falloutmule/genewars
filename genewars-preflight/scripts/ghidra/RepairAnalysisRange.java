// Repair only analysis code units in an explicitly bounded range; source bytes remain unchanged.
// @category GeneWars
import ghidra.app.script.GhidraScript;
public class RepairAnalysisRange extends GhidraScript {
 public void run() throws Exception {
  String[] args=getScriptArgs();var start=toAddr(args[0]);var end=toAddr(args[1]);
  if(end.subtract(start)<0||end.subtract(start)>256)throw new IllegalArgumentException("Repair range must be <=257 bytes");
  clearListing(start,end);disassemble(start);
 }
}
