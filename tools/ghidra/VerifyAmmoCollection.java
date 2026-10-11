import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.ByteBuffer;
import java.util.HexFormat;

public class VerifyAmmoCollection extends GhidraScript {
    int word(EmulatorHelper c,long a){return ByteBuffer.wrap(c.readMemory(toAddr(a),2)).getShort();}
    long pointer(EmulatorHelper c,long a){return Integer.toUnsignedLong(ByteBuffer.wrap(c.readMemory(toAddr(a),4)).getInt());}
    void ret(EmulatorHelper c){long s=c.readRegister("SP").longValue();c.writeRegister("PC",pointer(c,s));c.writeRegister("SP",s+4);}
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]);HexFormat hex=HexFormat.of();StringBuilder cases=new StringBuilder();
        byte[] table=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data ID"));
        byte[] definition=java.util.Arrays.copyOfRange(table,30000,30150);
        for(int reserve:new int[]{0,99,32768})for(int quantity:new int[]{1,32,32767})for(int initial:new int[]{1,12}) {
            EmulatorHelper c=new EmulatorHelper(currentProgram);
            try {
                for(String r:new String[]{"D0","D1","D2","D3","D4","D5","D6","D7","A0","A1","A2","A3","A4","A5","A6","XF","NF","ZF","VF","CF"})c.writeRegister(r,0);
                c.writeMemory(toAddr(0x45363c),definition);
                byte[] item=new byte[22];ByteBuffer.wrap(item).putShort(0,(short)200).putShort(2,(short)quantity);item[10]=26;
                c.writeMemory(toAddr(0x41fc74),item);c.writeMemoryValue(toAddr(0x42dbe4),4,reserve);
                c.writeRegister("A6",0x700100);c.writeRegister("SP",0x700000);c.writeRegister("PC",0x108ccc);
                c.writeMemoryValue(toAddr(0x700100-0x32),2,0);c.writeMemoryValue(toAddr(0x700100-0x6c),2,200);
                long seed=initial,draws=0;StringBuilder sounds=new StringBuilder();boolean stopped=false;
                for(int step=0;step<10000;step++) {
                    long pc=c.getExecutionAddress().getOffset(),s=c.readRegister("SP").longValue();
                    if(pc==0x10919c){stopped=true;break;}
                    if(pc==0x153ca4){int bound=word(c,s+4);seed=seed*16807%2147483647;draws++;short raw=(short)seed;if(raw==Short.MIN_VALUE)raw=0;c.writeRegister("D0",Math.abs((int)raw)*bound/32768+1);ret(c);continue;}
                    if(pc==0x133a18){if(sounds.length()>0)sounds.append(',');sounds.append(word(c,s+4));ret(c);continue;}
                    if(!c.step(monitor))throw new Exception("collection PC "+Long.toHexString(pc)+": "+c.getLastError());
                }
                if(!stopped)throw new Exception("Collection instruction budget");
                if(cases.length()>0)cases.append(',');cases.append("{\"input\":\"").append(hex.formatHex(item)).append("\",\"after_item\":\"").append(hex.formatHex(c.readMemory(toAddr(0x41fc74),22))).append("\",\"reserve\":").append(reserve).append(",\"after_reserve\":").append(pointer(c,0x42dbe4)).append(",\"initial_seed\":").append(initial).append(",\"seed\":").append(seed).append(",\"draws\":").append(draws).append(",\"sounds\":[").append(sounds).append("]}");
            }finally{c.dispose();}
        }
        Files.writeString(root.resolve("evidence/fixtures/ammunition-collection.json"),"{\"authority\":\"Original booty instructions 0x108ccc through 0x109198 including inclusive-range, stockpile addition and slot ID clear; sound and verified Random intercepted; selection and drawing outside slice\",\"definition\":\""+hex.formatHex(definition)+"\",\"cases\":["+cases+"]}\n");
    }
}
