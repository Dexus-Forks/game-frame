// Write a disassembly listing (address, bytes, instruction) from an analysed project.
// Usage: analyzeHeadless PROJECT NAME -process PROGRAM -noanalysis -scriptPath tools/ghidra \
//   -postScript ExportDisasm.java OUTPUT.txt [ADDRESS | START-END ...]
//   (or scripts/ghidra-run.sh BINARY disasm OUTPUT [ADDRESS | START-END ...])
//
// ADDRESS        the function that contains it; one is made there if Ghidra has none (code that
//                only a table or a run-time call reaches)
// START-END      every instruction from START up to END, decoded in order whether or not it is in
//                a function: blocks of short entry stubs that jump into shared code, data the
//                analysis left between functions
// no arguments   every function of the program
//
// For checking arithmetic, flags and instruction counts against a port, where pseudocode is not
// exact enough. Undecodable bytes are listed as "??". It does not save the project unless it made
// a function. Keep the output in analysis/ (git-ignored): it reproduces the program's code.
import ghidra.app.script.GhidraScript;
import ghidra.program.model.address.Address;
import ghidra.program.model.listing.*;
import java.io.*;

public class ExportDisasm extends GhidraScript {
  private Listing listing;
  private PrintWriter w;

  private void line(Instruction ins) {
    StringBuilder hex = new StringBuilder();
    try {
      for (byte b : ins.getBytes()) hex.append(String.format("%02x", b & 0xff));
    } catch (Exception e) {
      hex.append("?");
    }
    w.printf("%s  %-16s %s%n", ins.getAddress(), hex, ins);
  }

  private void function(Function f) {
    w.println("// ==== " + f.getEntryPoint() + " " + f.getName());
    for (Instruction ins : listing.getInstructions(f.getBody(), true)) line(ins);
  }

  private void range(Address a, Address end) throws Exception {
    w.println("// ==== " + a + "-" + end);
    while (a.compareTo(end) < 0) {
      if (listing.getInstructionAt(a) == null) disassemble(a);
      Instruction ins = listing.getInstructionAt(a);
      if (ins == null) {
        w.printf("%s  %02x               ??%n", a, getByte(a) & 0xff);
        a = a.add(1);
        continue;
      }
      line(ins);
      a = a.add(ins.getLength());
    }
  }

  @Override
  public void run() throws Exception {
    String[] args = getScriptArgs();
    File output = new File(args[0]);
    output.getParentFile().mkdirs();
    listing = currentProgram.getListing();
    try (PrintWriter out = new PrintWriter(new BufferedWriter(new FileWriter(output)))) {
      w = out;
      if (args.length == 1) {
        for (Function f : currentProgram.getFunctionManager().getFunctions(true)) function(f);
      }
      for (int i = 1; i < args.length; i++) {
        String arg = args[i];
        int dash = arg.indexOf('-');
        if (dash > 0) {
          range(toAddr(arg.substring(0, dash)), toAddr(arg.substring(dash + 1)));
          continue;
        }
        Address a = toAddr(arg);
        Function f = currentProgram.getFunctionManager().getFunctionContaining(a);
        if (f == null) {
          if (listing.getInstructionAt(a) == null) disassemble(a);
          f = createFunction(a, null);
        }
        if (f == null) w.println("// ==== " + a + " <no function could be made>");
        else function(f);
      }
    }
    println("Disassembly written to " + output);
  }
}
