import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.ByteBuffer;
import java.util.HexFormat;

public class VerifyNaturalPoison extends GhidraScript {
    int word(EmulatorHelper cpu,long a){return ByteBuffer.wrap(cpu.readMemory(toAddr(a),2)).getShort();}
    long pointer(EmulatorHelper cpu,long a){return Integer.toUnsignedLong(ByteBuffer.wrap(cpu.readMemory(toAddr(a),4)).getInt());}
    void ret(EmulatorHelper cpu){long sp=cpu.readRegister("SP").longValue();cpu.writeRegister("PC",pointer(cpu,sp));cpu.writeRegister("SP",sp+4);}
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]);HexFormat hex=HexFormat.of();StringBuilder cases=new StringBuilder();int count=0;
        String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};
        for(int monsterId:new int[]{2,3,4,5}) { byte[] enemy=java.util.Arrays.copyOfRange(Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data MD")),monsterId*288,(monsterId+1)*288);
        byte[] items=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data ID"));
        byte[] attacks=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data S"));
        for(String name:names)for(int attack=0;attack<3;attack++)for(int initialSeed=1;initialSeed<=16;initialSeed++)for(int variant=0;variant<(initialSeed<=2?8:1);variant++) {
            EmulatorHelper cpu=new EmulatorHelper(currentProgram);
            try {
                for(String reg:new String[]{"D0","D1","D2","D3","D4","D5","D6","D7","A0","A1","A2","A3","A4","A5","A6","XF","NF","ZF","VF","CF"})cpu.writeRegister(reg,0);
                byte[] input=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Warriors/"+name)),target=enemy.clone();input[0x9df]=1;ByteBuffer.wrap(input).putShort(0x50,(short)100).putShort(0x52,(short)100);
                ByteBuffer buf=ByteBuffer.wrap(input);buf.putShort(0x3c,(short)15).putShort(0x40,(short)(variant==4?-7:7));if(variant==1)buf.putShort(0x114,(short)100);if(variant==2)buf.putShort(0x64,(short)-1);if(variant==3)buf.putShort(0x64,(short)29);if(variant==5)buf.putShort(0x32,(short)3);if(variant==6)buf.putShort(0x64,(short)32767);if(variant==7)ByteBuffer.wrap(target).putShort(0x4e,(short)0);cpu.writeMemory(toAddr(0x448aa0),input);cpu.writeMemory(toAddr(0x439d3c),target);cpu.writeMemory(toAddr(0x4537fe),java.util.Arrays.copyOfRange(items,0,30000));cpu.writeMemory(toAddr(0x443788),attacks);
                cpu.writeMemoryValue(toAddr(0x44273a),1,0);cpu.writeMemoryValue(toAddr(0x4435d2),2,0);cpu.writeMemoryValue(toAddr(0x43565f),1,0);cpu.writeMemoryValue(toAddr(0x442576),2,0);cpu.writeMemoryValue(toAddr(0x447d96),2,0);cpu.writeMemoryValue(toAddr(0x4435cc),2,0);
                long sp=0x700000;cpu.writeMemory(toAddr(sp-8192),new byte[16384]);cpu.writeMemoryValue(toAddr(sp),4,0x900000);cpu.writeMemoryValue(toAddr(sp+4),2,10);cpu.writeMemoryValue(toAddr(sp+6),2,0);cpu.writeMemoryValue(toAddr(sp+8),2,attack);
                cpu.writeRegister("SP",sp);cpu.writeRegister("PC",0x101eb6);long seed=initialSeed,draws=0;boolean stopped=false;StringBuilder sounds=new StringBuilder();SaneBoundary sane=new SaneBoundary(cpu,currentProgram);
                for(int step=0;step<100000;step++){
                    long pc=cpu.getExecutionAddress().getOffset(),stack=cpu.readRegister("SP").longValue();
                    if(pc==0x900000){stopped=true;break;}
                    if(pc==0x400038){cpu.writeRegister("D0",Math.abs(word(cpu,stack+4)));ret(cpu);continue;}
                    if(pc==0x153ca4){int bound=word(cpu,stack+4);seed=seed*16807%2147483647;draws++;short raw=(short)seed;if(raw==Short.MIN_VALUE)raw=0;cpu.writeRegister("D0",Math.abs((int)raw)*bound/32768+1);ret(cpu);continue;}
                    if(pc==0x133a18||pc==0x16474a){if(sounds.length()>0)sounds.append(',');sounds.append(word(cpu,stack+4));ret(cpu);continue;}
                    if(pc==0x114c14||pc==0x158454||pc==0x105572||pc==0x13154c){ret(cpu);continue;}
                    int opcode=word(cpu,pc)&0xffff;
                    if(opcode==0xa873){cpu.writeRegister("SP",stack+4);cpu.writeRegister("PC",pc+2);continue;}
                    if(opcode==0xa9eb){sane.execute();continue;}
                    if(!cpu.step(monitor))throw new Exception("Name "+name+" PC "+Long.toHexString(pc)+": "+cpu.getLastError());
                }
                if(!stopped)throw new Exception("Natural attack instruction budget");
                if(cases.length()>0)cases.append(',');cases.append("{\"monster\":").append(monsterId).append(",\"name\":\"").append(name).append("\",\"initial_seed\":").append(initialSeed).append(",\"variant\":").append(variant).append(",\"attack\":").append(attack).append(",\"input\":\"").append(hex.formatHex(input)).append("\",\"target\":\"").append(hex.formatHex(target)).append("\",\"after\":\"").append(hex.formatHex(cpu.readMemory(toAddr(0x448aa0),3138))).append("\",\"after_target\":\"").append(hex.formatHex(cpu.readMemory(toAddr(0x439d3c),288))).append("\",\"sounds\":[").append(sounds).append("],\"seed\":").append(seed).append(",\"draws\":").append(draws).append('}');count++;
            } finally {cpu.dispose();}
        }
        } Files.writeString(root.resolve("evidence/fixtures/natural-poison.json"),"{\"authority\":\"Complete original attack2 including loadweapon, loadammo and resolvedamage; monsters2/3/4/5 natural attacks including condition5 poison against six starter warriors with 100 HP and controlled signed movement and action points. Variants isolate save100, negative poison immunity, counter29 cap, signed action division, Cyborg immunity and signed duration overflow. Native savevs and strength randrange execute; presentation intercepted\",\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyNaturalPoison.java"))+"\",\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\",\"cases\":["+cases+"]}\n");println(count+" required natural cases exported");
    }
}
