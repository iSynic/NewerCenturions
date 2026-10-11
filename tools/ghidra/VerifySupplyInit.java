import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.ByteBuffer;
import java.util.HexFormat;

public class VerifySupplyInit extends GhidraScript {
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]);
        byte[] table=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data ID"));
        StringBuilder cases=new StringBuilder();StringBuilder definitions=new StringBuilder();
        for(int itemId:new int[]{60,1,200,460,440,600,637,307,290,239,2,180,219,247,201,680,681,231,232}) {
        byte[] item=java.util.Arrays.copyOfRange(table,itemId*150,(itemId+1)*150);
        if(definitions.length()>0)definitions.append(','); definitions.append("\"").append(itemId).append("\":\"").append(HexFormat.of().formatHex(item)).append("\"");
        for(long initial=1;initial<=128;initial++) {
            EmulatorHelper cpu=new EmulatorHelper(currentProgram);
            try {
                for(String register:new String[]{"D0","D1","D2","D3","D4","D5","D6","D7","A0","A1","A2","A3","A4","A5","A6","XF","NF","ZF","VF","CF"}) cpu.writeRegister(register,0);
                cpu.writeRegister("PC",0x12d558);cpu.writeRegister("SP",0x700000);
                cpu.writeMemoryValue(toAddr(0x700000),4,0x7ffff0);
                cpu.writeMemoryValue(toAddr(0x700004),2,0);cpu.writeMemoryValue(toAddr(0x700006),2,0);
                cpu.writeMemoryValue(toAddr(0x442576),2,0);
                cpu.writeMemory(toAddr(0x41fc74),new byte[22]);cpu.writeMemoryValue(toAddr(0x41fc74),2,itemId);
                cpu.writeMemory(toAddr(0x46225e),java.util.Arrays.copyOfRange(table,30000,60000));
                long seed=initial;StringBuilder draws=new StringBuilder();boolean returned=false;
                for(int step=0;step<100000;step++) {
                    long pc=cpu.getExecutionAddress().getOffset();
                    if(pc==0x7ffff0){returned=true;break;}
                    if(pc==0x130afe||pc==0x153ca4){
                        long sp=cpu.readRegister("SP").longValue();int argument=ByteBuffer.wrap(cpu.readMemory(toAddr(sp+4),2)).getShort();
                        if(pc==0x130afe){if(argument!=itemId)throw new Exception("Unexpected item");cpu.writeMemory(toAddr(0x45363c),item);cpu.writeRegister("D0",0);}
                        else {seed=seed*16807%2147483647;short raw=(short)seed;if(raw==Short.MIN_VALUE)raw=0;short result=(short)((Math.abs((int)raw)*argument/32768)+1);cpu.writeRegister("D0",result);if(draws.length()>0)draws.append(',');draws.append("{\"bound\":").append(argument).append(",\"raw\":").append(raw).append(",\"result\":").append(result).append('}');}
                        long ret=Integer.toUnsignedLong(ByteBuffer.wrap(cpu.readMemory(toAddr(sp),4)).getInt());cpu.writeRegister("SP",sp+4);cpu.writeRegister("PC",ret);
                    } else if(!cpu.step(monitor)) throw new Exception(cpu.getLastError());
                }
                if(!returned)throw new Exception("Ammunition initialization instruction budget");
                if(cases.length()>0)cases.append(',');cases.append("{\"item_id\":").append(itemId).append(",\"seed\":").append(initial).append(",\"after_seed\":").append(seed).append(",\"draws\":[").append(draws).append("],\"record\":\"").append(HexFormat.of().formatHex(cpu.readMemory(toAddr(0x41fc74),22))).append("\"}");
            } finally {cpu.dispose();}
        }
        }
        Files.writeString(root.resolve("evidence/fixtures/supply-init.json"),"{\"authority\":\"Original randomitem execution for the authored treasure 30 item IDs; item load and verified Random intercepted; original ammunition table loaded\",\"ammunition_table\":\""+HexFormat.of().formatHex(java.util.Arrays.copyOfRange(table,30000,60000))+"\",\"definitions\":{"+definitions+"},\"cases\":["+cases+"]}\n");
        println("Supply initialization execution exported");
    }
}
