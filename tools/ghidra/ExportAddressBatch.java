// Decompile selected code addresses in an already analyzed Ghidra project. An address inside no
// function becomes a new function only when something refers to it or it follows a ret (near
// or far, with or without a byte count) or padding, so a call site is never made into a
// function that splits the real one. An address
// that fails is reported in the output and the batch goes on.
// Usage: analyzeHeadless PROJECT NAME -process PROGRAM -noanalysis -scriptPath tools/ghidra \
//   -postScript ExportAddressBatch.java OUTPUT.txt 0xADDRESS [0xADDRESS ...]
import ghidra.app.decompiler.*;
import ghidra.app.script.GhidraScript;
import ghidra.program.model.address.Address;
import ghidra.program.model.listing.Function;
import java.io.*;

public class ExportAddressBatch extends GhidraScript {
  @Override
  public void run() throws Exception {
    String[] args = getScriptArgs();
    File output = new File(args[0]);
    output.getParentFile().mkdirs();
    DecompInterface decompiler = new DecompInterface();
    decompiler.openProgram(currentProgram);
    try (PrintWriter writer = new PrintWriter(new BufferedWriter(new FileWriter(output)))) {
      for (int i = 1; i < args.length && !monitor.isCancelled(); i++) {
        try {
          one(args[i], decompiler, writer);
        } catch (Exception e) {   // one bad address must not stop the batch
          writer.println("===== ADDRESS " + args[i] + " ERROR " + e);
          writer.println();
        }
      }
    }
    decompiler.dispose();
    println("Address batch written to " + output);
  }

  private void one(String text, DecompInterface decompiler, PrintWriter writer) throws Exception {
    Address address = toAddr(text);
    if (address == null) throw new IllegalArgumentException("not an address in this program");
    Function function = currentProgram.getFunctionManager().getFunctionContaining(address);
    if (function == null) {
      if (!looksLikeStart(address)) {
        writer.println("===== ADDRESS " + address + " FUNCTION <none>");
        writer.println("// nothing refers to this address and it does not follow a ret or padding:"
          + " it may be inside a function Ghidra missed. Give that function's start instead.");
        writer.println();
        return;
      }
      if (currentProgram.getListing().getInstructionAt(address) == null) disassemble(address);
      function = createFunction(address, null);
    }
    writer.println("===== ADDRESS " + address + " FUNCTION "
      + (function == null ? "<none>" : function.getEntryPoint() + " " + function.getName(true)));
    if (function == null) {
      writer.println("// no code function could be created at this explicit callback address");
      writer.println();
      return;
    }
    DecompileResults result = decompiler.decompileFunction(function, 60, monitor);
    if (result.decompileCompleted()) writer.println(result.getDecompiledFunction().getC());
    else writer.println("// decompile failed: " + result.getErrorMessage());
    writer.println();
  }

  // A function may start here: something refers to the address (a call or a pointer in data),
  // or the bytes before it end another routine (ret C3, ret N C2 nn nn, and the far returns of
  // 16-bit code, retf CB and retf N CA nn nn) or pad (int3, nop).
  private boolean looksLikeStart(Address a) throws Exception {
    if (currentProgram.getReferenceManager().getReferenceCountTo(a) > 0) return true;
    int b1 = getByte(a.subtract(1)) & 0xff;
    int b3 = getByte(a.subtract(3)) & 0xff;
    return b1 == 0xc3 || b1 == 0xcb || b1 == 0xcc || b1 == 0x90 || b3 == 0xc2 || b3 == 0xca;
  }
}
