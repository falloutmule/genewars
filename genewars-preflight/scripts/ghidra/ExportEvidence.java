// Original local interoperability metadata exporter. No proprietary source embedded.
// @category GeneWars
import ghidra.app.script.GhidraScript;
import ghidra.app.decompiler.*;
import ghidra.program.model.listing.*;
import ghidra.program.model.mem.*;
import ghidra.program.model.symbol.*;
import ghidra.program.model.reloc.*;
import java.io.*;
import java.util.*;
import com.google.gson.*;

public class ExportEvidence extends GhidraScript {
  public void run() throws Exception {
    String[] args=getScriptArgs();
    File out=new File(args[0]); out.mkdirs();
    Map<String,Object> root=new LinkedHashMap<>();
    root.put("executable_format",currentProgram.getExecutableFormat());
    root.put("language",currentProgram.getLanguageID().toString());
    root.put("compiler",currentProgram.getCompilerSpec().getCompilerSpecID().toString());
    root.put("image_base",currentProgram.getImageBase().toString());
    root.put("sha256",currentProgram.getExecutableSHA256());
    List<Object> blocks=new ArrayList<>();
    for(MemoryBlock b:currentProgram.getMemory().getBlocks()) {
      Map<String,Object> item=new LinkedHashMap<>();
      item.put("name",b.getName());item.put("start",b.getStart().toString());item.put("end",b.getEnd().toString());
      item.put("size",b.getSize());item.put("read",b.isRead());item.put("write",b.isWrite());item.put("execute",b.isExecute());item.put("initialized",b.isInitialized());
      blocks.add(item);
      if(b.isInitialized()) {
        byte[] bytes=new byte[(int)b.getSize()]; currentProgram.getMemory().getBytes(b.getStart(),bytes);
        try(OutputStream stream=new FileOutputStream(new File(out,"memory-"+b.getName()+".bin"))) {stream.write(bytes);}
      }
    }
    root.put("blocks",blocks);
    List<String> entries=new ArrayList<>();
    var entryIt=currentProgram.getSymbolTable().getExternalEntryPointIterator();
    while(entryIt.hasNext()) entries.add(entryIt.next().toString());
    root.put("entry_points",entries);
    List<Object> relocations=new ArrayList<>(); int count=0;
    var relocIt=currentProgram.getRelocationTable().getRelocations();
    while(relocIt.hasNext()) {
      Relocation r=relocIt.next();count++;
      if(true) {
        Map<String,Object> item=new LinkedHashMap<>();
        item.put("address",r.getAddress().toString());item.put("type",r.getType());item.put("status",r.getStatus().toString());item.put("values",r.getValues());
        item.put("current_int",Integer.toUnsignedLong(currentProgram.getMemory().getInt(r.getAddress())));
        relocations.add(item);
      }
    }
    root.put("relocation_count",count);root.put("relocation_samples",relocations);
    List<Object> funcs=new ArrayList<>();
    var fit=currentProgram.getFunctionManager().getFunctions(true);
    while(fit.hasNext()) {
      Function f=fit.next();Map<String,Object> item=new LinkedHashMap<>();
      item.put("entry",f.getEntryPoint().toString());item.put("name",f.getName());item.put("size",f.getBody().getNumAddresses());
      Set<String> called=new TreeSet<>();
      for(Function c:f.getCalledFunctions(monitor)) called.add(c.getEntryPoint().toString());
      item.put("calls",called);funcs.add(item);
    }
    root.put("function_count",funcs.size());root.put("functions",funcs);
    Gson gson=new GsonBuilder().setPrettyPrinting().create();
    try(Writer w=new FileWriter(new File(out,"program-metadata.json"))) {gson.toJson(root,w);}
    if(args.length>1) {
      DecompInterface dec=new DecompInterface();dec.openProgram(currentProgram);
      for(int i=1;i<args.length;i++) {
        var addr=toAddr(args[i]);Function f=getFunctionAt(addr);
        if(f==null) {disassemble(addr);f=createFunction(addr,null);}
        if(f==null) {println("No function "+addr);continue;}
        DecompileResults result=dec.decompileFunction(f,60,monitor);
        try(Writer w=new FileWriter(new File(out,"decompile-"+addr+".c"))) {
          w.write("/* PRIVATE original binary analysis: "+currentProgram.getExecutableSHA256()+" @ "+addr+" */\n");
          if(result.decompileCompleted()) w.write(result.getDecompiledFunction().getC());
          else w.write(result.getErrorMessage());
        }
        try(PrintWriter w=new PrintWriter(new File(out,"instructions-"+addr+".txt"))) {
          var it=currentProgram.getListing().getInstructions(f.getBody(),true);
          while(it.hasNext()) {var ins=it.next();w.println(ins.getAddress()+" "+ins);}
        }
      }
      dec.dispose();
    }
    println("EXPORTED metadata: "+funcs.size()+" functions; "+count+" relocation entries");
  }
}
