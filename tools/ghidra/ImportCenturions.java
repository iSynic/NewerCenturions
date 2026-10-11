// Import resource-relative 68k segments; synthetic bases are not runtime addresses.
import ghidra.app.script.GhidraScript;
import ghidra.program.model.address.Address;
import ghidra.program.model.address.AddressSet;
import ghidra.program.model.listing.Function;
import ghidra.program.model.symbol.SourceType;
import java.io.ByteArrayInputStream;
import java.nio.file.Files;
import java.nio.file.Path;

public class ImportCenturions extends GhidraScript {
    public void run() throws Exception {
        String[] args = getScriptArgs();
        Path root = Path.of(args[0]);
        for (int segment = 2; segment <= 3; segment++) {
            byte[] bytes = Files.readAllBytes(root.resolve("analysis/extracted/resources/106/434f4445_" + segment + ".bin"));
            var block = currentProgram.getMemory().createInitializedBlock("CODE_" + segment,
                toAddr(segment * 0x100000L), new ByteArrayInputStream(bytes), bytes.length, monitor, false);
            block.setExecute(true);
            block.setWrite(false);
        }
        currentProgram.getMemory().getBlock(toAddr(0x100000)).setName("CODE_1");
        currentProgram.getMemory().getBlock(toAddr(0x100000)).setExecute(true);
        // Startup entry is the one CODE 0 jump-table target, after the near header.
        disassemble(toAddr(0x100004));
        createFunction(toAddr(0x100004), "segment_loader");
        for (String line : Files.readAllLines(root.resolve("analysis/ghidra/seeds.tsv"))) {
            if (line.isBlank()) continue;
            String[] fields = line.split("\t");
            long segment = Long.parseLong(fields[0], 16);
            long offset = Long.parseLong(fields[1], 16);
            long end = Long.parseLong(fields[2], 16);
            Address start = toAddr(segment * 0x100000 + offset);
            Address finish = toAddr(segment * 0x100000 + end - 1);
            disassemble(start);
            Function existing = getFunctionAt(start);
            if (existing == null) {
                currentProgram.getFunctionManager().createFunction(fields[3], start,
                    new AddressSet(start, finish), SourceType.USER_DEFINED);
            } else {
                existing.setName(fields[3], SourceType.USER_DEFINED);
            }
        }
        println("Centurions segments and embedded-name seeds imported");
    }
}
