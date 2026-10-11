import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import ghidra.app.decompiler.DecompInterface;
import java.io.ByteArrayInputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.HexFormat;

public class VerifyRules extends GhidraScript {
    private EmulatorHelper emulator;
    private int rawRandom;
    private int execute(long address, int... words) throws Exception {
        emulator.writeRegister("SP", 0x700000);
        emulator.writeRegister("PC", address);
        emulator.writeRegister("A6", 0);
        emulator.writeMemoryValue(toAddr(0x700000), 4, 0x7ffff0);
        for (int i=0;i<words.length;i++) emulator.writeMemoryValue(toAddr(0x700004+i*2), 2, words[i]);
        for (int steps=0;steps<500;steps++) {
            long pc=emulator.getExecutionAddress().getOffset();
            if (pc==0x7ffff0) return emulator.readRegister("D0").shortValue();
            if (pc==0x153cb2) {
                emulator.writeMemoryValue(toAddr(emulator.readRegister("SP").longValue()), 2, rawRandom);
                emulator.writeRegister("PC",pc+2);
            } else if (!emulator.step(monitor)) throw new Exception(emulator.getLastError());
        }
        throw new Exception("Instruction limit reached");
    }
    public void run() throws Exception {
        String[] arguments=getScriptArgs();
        if(arguments.length<2) throw new IllegalArgumentException("Expected asset root and Macintosh ROM path");
        Path root=Path.of(arguments[0]);
        Path output=root.resolve("evidence/fixtures");
        Files.createDirectories(output);
        byte[] rom=Files.readAllBytes(Path.of(arguments[1]));
        var start=toAddr(0x802eb20);
        if (currentProgram.getMemory().getBlock(start)==null) {
            var block=currentProgram.getMemory().createInitializedBlock("QuickDrawRandom",start,
                new ByteArrayInputStream(rom,0x2eb20,128),128,monitor,false);
            block.setExecute(true);
            disassemble(start);
            createFunction(start,"QuickDrawRandom");
        }
        var decompiler=new DecompInterface();
        decompiler.openProgram(currentProgram);
        var decomp=decompiler.decompileFunction(getFunctionAt(start),30,monitor);
        if(decomp.decompileCompleted()) Files.writeString(root.resolve("analysis/ghidra/exports/QuickDrawRandom.c"),decomp.getDecompiledFunction().getC());
        decompiler.dispose();
        emulator=new EmulatorHelper(currentProgram);
        for (String register:new String[]{"D0","D1","D2","D3","D4","D5","D6","D7","A0","A1","A2","A3","A4","A5","A6","XF","NF","ZF","VF","CF"}) emulator.writeRegister(register,0);
        StringBuilder json=new StringBuilder("{\n  \"authority\": \"Centurions 1.3.2 CODE 1, Ghidra p-code execution; QuickDraw trap input injected\",\n  \"code_sha256\": \"366a280cdaa09094551642ae8e70ce057ad8276605c947bece231c6d1eeb680d\",\n  \"rand\": [\n");
        int[] bounds={-32768,-32767,-200,-1,0,1,2,7,100,32766,32767};
        int[] random={-32768,-32767,-16384,-1,0,1,16384,32766,32767};
        boolean first=true;
        for(int bound:bounds) for(int raw:random) {
            rawRandom=raw;
            int result=execute(0x153ca4,bound);
            if(!first) json.append(",\n"); first=false;
            json.append("    [").append(bound).append(',').append(raw).append(',').append(result).append(']');
        }
        json.append("\n  ],\n  \"range\": [\n"); first=true;
        int[] values={-32768,-1,0,1,7,32767};
        for(int value:values) for(int min:values) for(int max:values) {
            int result=execute(0x13846a,value,min,max);
            if(!first) json.append(",\n"); first=false;
            json.append("    [").append(value).append(',').append(min).append(',').append(max).append(',').append(result).append(']');
        }
        json.append("\n  ],\n  \"movement\": [\n"); first=true;
        int[] costs={-32768,-32767,-3,-2,-1,0,1,2,3,4,5,100,32767};
        for(int cost:costs) for(int dx:new int[]{0,1}) for(int dy:new int[]{0,-1}) {
            emulator.writeMemoryValue(toAddr(0x3167e),2,cost);
            emulator.writeMemoryValue(toAddr(0x42736),1,dx);
            emulator.writeMemoryValue(toAddr(0x42737),1,dy);
            int result=execute(0x153de6,0);
            if(!first) json.append(",\n"); first=false;
            json.append("    [").append(cost).append(',').append(dx).append(',').append(dy).append(',').append(result).append(']');
        }
        json.append("\n  ],\n  \"quickdraw\": [\n"); first=true;
        for(long seed:new long[]{0,1,2,16807,2147483646L,2147483647L,2147483648L,4294967295L,958682087L}) {
            emulator.writeRegister("A5",0x600000);
            emulator.writeMemoryValue(toAddr(0x600000),4,0x610000);
            emulator.writeMemoryValue(toAddr(0x60ff82),4,seed);
            execute(0x802eb20);
            long next=emulator.readMemory(toAddr(0x60ff82),4)[0]&255;
            for(int i=1;i<4;i++) next=(next<<8)|(emulator.readMemoryByte(toAddr(0x60ff82+i))&255);
            int result=(short)((emulator.readMemoryByte(toAddr(0x700004))&255)*256+(emulator.readMemoryByte(toAddr(0x700005))&255));
            if(!first) json.append(",\n"); first=false;
            json.append("    [").append(seed).append(',').append(next).append(',').append(result).append(']');
        }
        json.append("\n  ]\n}\n");
        Files.writeString(output.resolve("rules.json"),json.toString());
        emulator.dispose();
        println("Independent binary arithmetic cases exported");
    }
}
