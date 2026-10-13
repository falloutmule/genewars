// @category GeneWars
// Reconstruct code annotations only; no source/mapped byte changes.
import ghidra.app.script.GhidraScript;
public class RecoverPanelFunctions extends GhidraScript {
 public void run() throws Exception {
  // Discard one exploratory mid-instruction candidate; never use its decompile.
  removeFunctionAt(toAddr("24a80"));clearListing(toAddr("24a60"),toAddr("24acf"));
  disassemble(toAddr("24a60"));createFunction(toAddr("24a60"),null);
  // Starts supported by function pointer slots at 0x112848 and 0x11284C.
  disassemble(toAddr("23a80"));createFunction(toAddr("23a80"),null);
  disassemble(toAddr("23cb0"));createFunction(toAddr("23cb0"),null);
 }
}
