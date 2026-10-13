// @category GeneWars
import ghidra.app.script.GhidraScript;
import ghidra.app.decompiler.*;
import ghidra.program.model.address.*;
import ghidra.program.model.listing.*;
import java.io.*;
import java.util.*;
import com.google.gson.*;
public class QueryReferences extends GhidraScript {
 public void run() throws Exception {
  String[] args=getScriptArgs();File out=new File(args[0]);out.mkdirs();
  List<Object> rows=new ArrayList<>();Set<Function> funcs=new HashSet<>();
  for(int ai=1;ai<args.length;ai++) {
   Address target=toAddr(args[ai]);Set<Address> visited=new HashSet<>();List<Address> queue=new ArrayList<>();queue.add(target);
   for(int depth=0;depth<4&&!queue.isEmpty();depth++) {
    List<Address> next=new ArrayList<>();
    for(Address addr:queue) {
     if(!visited.add(addr))continue;
     var refs=currentProgram.getReferenceManager().getReferencesTo(addr);
     while(refs.hasNext()) {
      var ref=refs.next();Function f=getFunctionContaining(ref.getFromAddress());
      Map<String,Object> row=new LinkedHashMap<>();row.put("root",target.toString());row.put("target",addr.toString());row.put("from",ref.getFromAddress().toString());row.put("type",ref.getReferenceType().toString());row.put("depth",depth);row.put("function",f==null?null:f.getEntryPoint().toString());rows.add(row);
      if(f!=null)funcs.add(f);else if(next.size()<100)next.add(ref.getFromAddress());
     }
    }queue=next;
   }
  }
  try(Writer w=new FileWriter(new File(out,"reference-graph.json"))) {new GsonBuilder().setPrettyPrinting().create().toJson(rows,w);}
  DecompInterface dec=new DecompInterface();dec.openProgram(currentProgram);
  for(Function f:funcs) {
   var result=dec.decompileFunction(f,60,monitor);
   if(result.decompileCompleted())try(Writer w=new FileWriter(new File(out,"decompile-"+f.getEntryPoint()+".c"))){w.write(result.getDecompiledFunction().getC());}
   try(PrintWriter w=new PrintWriter(new File(out,"instructions-"+f.getEntryPoint()+".txt"))) {
    var it=currentProgram.getListing().getInstructions(f.getBody(),true);while(it.hasNext()){var ins=it.next();w.println(ins.getAddress()+" "+ins);}
   }
  }
  dec.dispose();println("Exported "+rows.size()+" reference edges and "+funcs.size()+" function bodies privately");
 }
}
