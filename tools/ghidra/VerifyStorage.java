import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.MessageDigest;
import java.util.HexFormat;
import java.util.Arrays;
import java.nio.ByteBuffer;

public class VerifyStorage extends GhidraScript {
    public void run() throws Exception {
        StringBuilder output = new StringBuilder("{\"authority\":\"Centurions 1.3.2 mapped CODE 1 p-code execution; sound call intercepted\",\"code_sha256\":\"366a280cdaa09094551642ae8e70ce057ad8276605c947bece231c6d1eeb680d\",\"cases\":[");
        boolean first=true;
        byte[] item=new byte[22];
        for(int i=2;i<22;i++) item[i]=(byte)i;
        item[1]=7;
        for(boolean crate:new boolean[]{true,false}) for(int hole:new int[]{0,517,999,-1}) {
            EmulatorHelper cpu=new EmulatorHelper(currentProgram);
            try {
                for(String register:new String[]{"D0","D1","D2","D3","D4","D5","D6","D7","A0","A1","A2","A3","A4","A5","A6","XF","NF","ZF","VF","CF"}) cpu.writeRegister(register,0);
                long storage=crate?0x4285f4:0x41fc74;
                byte[] before=new byte[22000];
                for(int i=0;i<1000;i++) { System.arraycopy(item,0,before,i*22,22);before[i*22+1]=9; }
                if(hole>=0) Arrays.fill(before,hole*22,hole*22+22,(byte)0);
                cpu.writeMemory(toAddr(storage),before);
                cpu.writeRegister("SP",0x700000);
                cpu.writeRegister("PC",crate?0x100a38:0x100a9a);
                cpu.writeMemoryValue(toAddr(0x700000),4,0x7ffff0);
                cpu.writeMemory(toAddr(0x700004),item);
                int sound=0;
                boolean returned=false;
                for(int step=0;step<100000;step++) {
                    long pc=cpu.getExecutionAddress().getOffset();
                    if(pc==0x7ffff0) {returned=true;break;}
                    if(pc==0x133a18) {
                        long sp=cpu.readRegister("SP").longValue();
                        sound=ByteBuffer.wrap(cpu.readMemory(toAddr(sp+4),2)).getShort();
                        long ret=Integer.toUnsignedLong(ByteBuffer.wrap(cpu.readMemory(toAddr(sp),4)).getInt());
                        cpu.writeRegister("SP",sp+4);
                        cpu.writeRegister("PC",ret);
                    } else if(!cpu.step(monitor)) throw new Exception(cpu.getLastError());
                }
                if(!returned) throw new Exception("Storage instruction limit");
                byte[] after=cpu.readMemory(toAddr(storage),22000);
                String hash=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(after));
                if(!first) output.append(',');first=false;
                output.append("{\"crate\":").append(crate).append(",\"hole\":").append(hole).append(",\"result\":").append(cpu.readRegister("D0").shortValue())
                    .append(",\"sound\":").append(sound).append(",\"sha256\":\"").append(hash).append("\"}");
            } finally {cpu.dispose();}
        }
        output.append("]}\n");
        Files.writeString(Path.of(getScriptArgs()[0]).resolve("evidence/fixtures/storage.json"),output.toString());
        println("Eight independent storage cases exported");
    }
}
