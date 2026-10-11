import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.security.MessageDigest;
import java.util.HexFormat;

public class VerifyStartup extends GhidraScript {
    private EmulatorHelper emulator;
    private void execute(long entry) throws Exception {
        emulator.writeRegister("SP",0x700000);
        emulator.writeRegister("PC",entry);
        emulator.writeMemoryValue(toAddr(0x700000),4,0x7ffff0);
        for(int step=0;step<2000000;step++) {
            if(emulator.getExecutionAddress().getOffset()==0x7ffff0) return;
            if(!emulator.step(monitor)) throw new Exception(emulator.getLastError());
        }
        throw new Exception("Startup instruction limit reached");
    }
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]);
        byte[] data=Files.readAllBytes(root.resolve("analysis/extracted/resources/106/44415441_0.bin"));
        byte[] expected=Files.readAllBytes(root.resolve("analysis/ghidra/startup/a5-initial.bin"));
        emulator=new EmulatorHelper(currentProgram);
        for(String register:new String[]{"D0","D1","D2","D3","D4","D5","D6","D7","A0","A1","A2","A3","A4","A5","A6","XF","NF","ZF","VF","CF"}) emulator.writeRegister(register,0);
        emulator.writeMemory(toAddr(0x800000),data);
        emulator.writeMemory(toAddr(0x400000-0x1df0),new byte[expected.length]);
        emulator.writeMemoryValue(toAddr(0x700004),4,0x800004);
        emulator.writeMemoryValue(toAddr(0x700008),4,0x400000);
        execute(0x10010e);
        byte[] actual=emulator.readMemory(toAddr(0x400000-0x1df0),expected.length);
        if(!Arrays.equals(actual,expected)) throw new Exception("Startup initializer differs from Python decoder");
        emulator.writeRegister("A5",0x400000);
        emulator.writeMemoryValue(toAddr(0x400000-0x1de2),4,0x100000);
        emulator.writeRegister("A1",0x400000);
        execute(0x100286);
        emulator.writeRegister("A1",0x100000);
        execute(0x100286);
        long consumed=emulator.readRegister("A0").longValue()-0x800000;
        if(consumed!=data.length) throw new Exception("Relocations did not consume entire DATA resource");
        byte[] mapped=Files.readAllBytes(root.resolve("analysis/ghidra/startup/code1-mapped.bin"));
        byte[] mappedActual=emulator.readMemory(toAddr(0x100000),mapped.length);
        if(!Arrays.equals(mapped,mappedActual)) throw new Exception("Relocated CODE 1 differs from Python decoder");
        String hash=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(actual));
        Files.writeString(root.resolve("evidence/fixtures/startup.json"),"{\n  \"authority\": \"Unmodified CODE 1 startup helpers executed with Ghidra p-code\",\n  \"initialized_bytes\": "+actual.length+",\n  \"initialized_sha256\": \""+hash+"\",\n  \"data_consumed\": "+consumed+",\n  \"code_relocations_match\": true\n}\n");
        emulator.dispose();
        println("Full DATA initialization and CODE 1 relocation independently verified");
    }
}
