import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.ByteBuffer;
import java.util.HexFormat;

public class VerifyDamage extends GhidraScript {
    int word(EmulatorHelper cpu,long a){return ByteBuffer.wrap(cpu.readMemory(toAddr(a),2)).getShort();}
    long pointer(EmulatorHelper cpu,long a){return Integer.toUnsignedLong(ByteBuffer.wrap(cpu.readMemory(toAddr(a),4)).getInt());}
    void ret(EmulatorHelper cpu){long sp=cpu.readRegister("SP").longValue();cpu.writeRegister("PC",pointer(cpu,sp));cpu.writeRegister("SP",sp+4);}
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]);HexFormat hex=HexFormat.of();StringBuilder cases=new StringBuilder();int count=0;
        byte[] warrior=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Warriors/Cindred"));
        byte[] enemy=java.util.Arrays.copyOfRange(Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data MD")),49*288,50*288);
        for(boolean monster:new boolean[]{false,true})for(int kind=0;kind<4;kind++)for(int flags=0;flags<16;flags++)for(int initialSeed:new int[]{1,12,12345}) {
            EmulatorHelper cpu=new EmulatorHelper(currentProgram);
            try {
                for(String reg:new String[]{"D0","D1","D2","D3","D4","D5","D6","D7","A0","A1","A2","A3","A4","A5","A6","XF","NF","ZF","VF","CF"})cpu.writeRegister(reg,0);
                byte[] input=(monster?enemy:warrior).clone(),packet=new byte[58];ByteBuffer p=ByteBuffer.wrap(packet),t=ByteBuffer.wrap(input);
                int body=monster?10:0;long target=monster?0x439d3c:0x448aa0;
                p.putShort(32,(short)1).putShort(34,(short)7);packet[9]=2;
                if(kind>=1){packet[11]=1;packet[12]=3;}
                if(kind>=2){for(int i=23;i<30;i++)packet[i]=(byte)(i-21);packet[42]=1;t.putShort(0x10,(short)2);}
                if(kind==3){p.putShort(40,(short)57);}
                t.putShort(monster?8:0x2c,(short)(kind%2==0?10:95));
                t.putShort(monster?0x56:0x24,(short)80).putShort(monster?0x58:0x26,(short)(kind%2==0?3:12));
                if(monster){input[0x10a]=2;input[0xb0]=0;java.util.Arrays.fill(input,0x8c,0x93,(byte)0);java.util.Arrays.fill(input,0xb2,0xb9,(byte)0);input[0xab]=0;}
                else {t.putShort(0x2a,(short)2);}
                cpu.writeMemory(toAddr(target),input);cpu.writeMemory(toAddr(0x44374e),packet);cpu.writeMemoryValue(toAddr(0x4435cc),2,2);
                cpu.writeMemoryValue(toAddr(0x41f710),2,0);cpu.writeMemoryValue(toAddr(0x4434b8),2,0);
                long sp=0x700000;cpu.writeMemory(toAddr(sp-1024),new byte[2048]);cpu.writeMemoryValue(toAddr(sp),4,0x900000);cpu.writeMemoryValue(toAddr(sp+4),2,body);
                for(int i=0;i<4;i++)cpu.writeMemoryValue(toAddr(sp+6+i*2),2,(flags>>i)&1);
                cpu.writeRegister("SP",sp);cpu.writeRegister("PC",0x15075a);long seed=initialSeed,draws=0;boolean stopped=false;StringBuilder sounds=new StringBuilder();
                for(int step=0;step<100000;step++){
                    long pc=cpu.getExecutionAddress().getOffset(),stack=cpu.readRegister("SP").longValue();
                    if(pc==0x900000){stopped=true;break;}
                    if(pc==0x400038){cpu.writeRegister("D0",Math.abs(word(cpu,stack+4)));ret(cpu);continue;}
                    if(pc==0x153ca4){int bound=word(cpu,stack+4);seed=seed*16807%2147483647;draws++;short raw=(short)seed;if(raw==Short.MIN_VALUE)raw=0;cpu.writeRegister("D0",Math.abs((int)raw)*bound/32768+1);ret(cpu);continue;}
                    if(pc==0x133a18){if(sounds.length()>0)sounds.append(',');sounds.append(word(cpu,stack+4));ret(cpu);continue;}
                    if(pc==0x10e6b6||pc==0x123d1c){ret(cpu);continue;}
                    if(pc==0x13a3ae)throw new Exception("Unexpected saving throw");
                    if(!cpu.step(monitor))throw new Exception("PC "+Long.toHexString(pc)+": "+cpu.getLastError());
                }
                if(!stopped)throw new Exception("Damage instruction budget");
                if(cases.length()>0)cases.append(',');cases.append("{\"monster\":").append(monster).append(",\"flags\":").append(flags).append(",\"initial_seed\":").append(initialSeed).append(",\"input\":\"").append(hex.formatHex(input)).append("\",\"packet\":\"").append(hex.formatHex(packet)).append("\",\"after\":\"").append(hex.formatHex(cpu.readMemory(toAddr(target),input.length))).append("\",\"total\":").append((short)cpu.readRegister("D0").intValue()).append(",\"sound\":").append(word(cpu,0x41f710)).append(",\"sounds\":[").append(sounds).append("],\"seed\":").append(seed).append(",\"draws\":").append(draws).append('}');count++;
            } finally {cpu.dispose();}
        }
        Files.writeString(root.resolve("evidence/fixtures/damage.json"),"{\"authority\":\"Complete original resolvedamage; QuickDraw-derived Rand boundary; presentation intercepted; elemental saving throws disabled by authored packet category\",\"cases\":["+cases+"]}\n");println(count+" damage cases exported");
    }
}
