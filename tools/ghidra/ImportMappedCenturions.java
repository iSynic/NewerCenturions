import ghidra.app.script.GhidraScript;
import ghidra.program.model.address.AddressSet;
import ghidra.program.model.symbol.SourceType;
import java.io.ByteArrayInputStream;
import java.nio.file.Files;
import java.nio.file.Path;

public class ImportMappedCenturions extends GhidraScript {
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]);
        for(int segment=2;segment<=3;segment++) {
            byte[] bytes=Files.readAllBytes(root.resolve("analysis/extracted/resources/106/434f4445_"+segment+".bin"));
            var block=currentProgram.getMemory().createInitializedBlock("CODE_"+segment,toAddr(segment*0x100000L),new ByteArrayInputStream(bytes),bytes.length,monitor,false);
            block.setExecute(true);
        }
        byte[] globals=Files.readAllBytes(root.resolve("analysis/ghidra/startup/a5-mapped.bin"));
        var data=currentProgram.getMemory().createInitializedBlock("A5_DATA",toAddr(0x400000-0x1df0),new ByteArrayInputStream(globals),globals.length,monitor,false);
        data.setWrite(true);
        for(int offset=0x28;offset<0x400;offset+=8) {
            if(currentProgram.getMemory().getShort(toAddr(0x400000+offset))!=(short)0x4ef9) break;
            var address=toAddr(0x400000+offset);
            disassemble(address);
            createFunction(address,"jump_"+Integer.toHexString(offset));
        }
        for(String line:Files.readAllLines(root.resolve("analysis/ghidra/seeds.tsv"))) {
            if(line.isBlank()) continue;
            String[] fields=line.split("\t");
            long base=Long.parseLong(fields[0],16)*0x100000;
            var start=toAddr(base+Long.parseLong(fields[1],16));
            var end=toAddr(base+Long.parseLong(fields[2],16)-1);
            disassemble(start);
            var function=getFunctionAt(start);
            if(function==null) currentProgram.getFunctionManager().createFunction(fields[3],start,new AddressSet(start,end),SourceType.USER_DEFINED);
            else function.setName(fields[3],SourceType.USER_DEFINED);
        }
        println("Mapped global operands and resolved analysis-only segment thunks imported");
    }
}
