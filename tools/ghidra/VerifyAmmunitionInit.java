import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.ByteBuffer;
import java.util.HexFormat;

public class VerifyAmmunitionInit extends GhidraScript {
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]);
        byte[] table=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data ID"));
        byte[] item=java.util.Arrays.copyOfRange(table,30000,30150);
        StringBuilder cases=new StringBuilder();
        for(long initial:new long[]{1,2,1860097651,1741770978,2147483646}) {
            EmulatorHelper cpu=new EmulatorHelper(currentProgram);
            try {
                for(String register:new String[]{"D0","D1","D2","D3","D4","D5","D6","D7","A0","A1","A2","A3","A4","A5","A6","XF","NF","ZF","VF","CF"}) cpu.writeRegister(register,0);
                cpu.writeRegister("PC",0x12d558);cpu.writeRegister("SP",0x700000);
                cpu.writeMemoryValue(toAddr(0x700000),4,0x7ffff0);
                cpu.writeMemoryValue(toAddr(0x700004),2,0);cpu.writeMemoryValue(toAddr(0x700006),2,0);
                cpu.writeMemoryValue(toAddr(0x442576),2,0);
                cpu.writeMemory(toAddr(0x41fc74),new byte[22]);cpu.writeMemoryValue(toAddr(0x41fc74),2,200);
                long seed=initial;StringBuilder draws=new StringBuilder();boolean returned=false;
                for(int step=0;step<100000;step++) {
                    long pc=cpu.getExecutionAddress().getOffset();
                    if(pc==0x7ffff0){returned=true;break;}
                    if(pc==0x130afe||pc==0x153ca4){
                        long sp=cpu.readRegister("SP").longValue();int argument=ByteBuffer.wrap(cpu.readMemory(toAddr(sp+4),2)).getShort();
                        if(pc==0x130afe){if(argument!=200)throw new Exception("Unexpected item");cpu.writeMemory(toAddr(0x45363c),item);cpu.writeRegister("D0",0);}
                        else {seed=seed*16807%2147483647;short raw=(short)seed;if(raw==Short.MIN_VALUE)raw=0;short result=(short)((Math.abs((int)raw)*argument/32768)+1);cpu.writeRegister("D0",result);if(draws.length()>0)draws.append(',');draws.append("{\"bound\":").append(argument).append(",\"raw\":").append(raw).append(",\"result\":").append(result).append('}');}
                        long ret=Integer.toUnsignedLong(ByteBuffer.wrap(cpu.readMemory(toAddr(sp),4)).getInt());cpu.writeRegister("SP",sp+4);cpu.writeRegister("PC",ret);
                    } else if(!cpu.step(monitor)) throw new Exception(cpu.getLastError());
                }
                if(!returned)throw new Exception("Ammunition initialization instruction budget");
                if(cases.length()>0)cases.append(',');cases.append("{\"seed\":").append(initial).append(",\"after_seed\":").append(seed).append(",\"draws\":[").append(draws).append("],\"record\":\"").append(HexFormat.of().formatHex(cpu.readMemory(toAddr(0x41fc74),22))).append("\"}");
            } finally {cpu.dispose();}
        }
        Files.writeString(root.resolve("evidence/fixtures/ammunition-init.json"),"{\"authority\":\"Centurions 1.3.2 randomitem p-code; item load and verified Random boundary intercepted\",\"item_id\":200,\"definition\":\""+HexFormat.of().formatHex(item)+"\",\"cases\":["+cases+"]}\n");
        println("Ammunition initialization execution exported");
    }
}
