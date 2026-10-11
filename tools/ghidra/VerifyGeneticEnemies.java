import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.ByteBuffer;
import java.util.HexFormat;
import java.util.regex.Pattern;

public class VerifyGeneticEnemies extends GhidraScript {
    int word(EmulatorHelper cpu,long a){return ByteBuffer.wrap(cpu.readMemory(toAddr(a),2)).getShort();}
    long pointer(EmulatorHelper cpu,long a){return Integer.toUnsignedLong(ByteBuffer.wrap(cpu.readMemory(toAddr(a),4)).getInt());}
    void ret(EmulatorHelper cpu){long sp=cpu.readRegister("SP").longValue();cpu.writeRegister("PC",pointer(cpu,sp));cpu.writeRegister("SP",sp+4);}
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]);HexFormat hex=HexFormat.of();
        String suffix="-opening";
        var matcher=Pattern.compile("\\\"tiles\\\"\\s*:\\s*\\\"([0-9a-f]+)\\\"").matcher(Files.readString(root.resolve("evidence/fixtures/formation"+suffix+".json")));
        if(!matcher.find())throw new Exception("Formation fixture missing");byte[] map=hex.parseHex(matcher.group(1));
        Path scenario=root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0");
        byte[] battles=Files.readAllBytes(scenario.resolve("Data BD"));StringBuilder battleRecords=new StringBuilder();
        byte[] monsters=Files.readAllBytes(scenario.resolve("Data MD"));
        byte[] properties=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data Swamp BD"));
        StringBuilder cases=new StringBuilder();
        for(int battleId:new int[]{8,11,16,19,22,25}) {
            byte[] battle=java.util.Arrays.copyOfRange(battles,battleId*1372,(battleId+1)*1372);
            if(battleRecords.length()>0)battleRecords.append(',');battleRecords.append("{\"id\":").append(battleId).append(",\"record\":\"").append(hex.formatHex(battle)).append("\"}");
            for(int base:new int[]{-4})for(long initial:new long[]{12,1}) {
            EmulatorHelper cpu=new EmulatorHelper(currentProgram);
            try {
                for(String reg:new String[]{"D0","D1","D2","D3","D4","D5","D6","D7","A0","A1","A2","A3","A4","A5","A6","XF","NF","ZF","VF","CF"})cpu.writeRegister(reg,0);
                cpu.writeMemory(toAddr(0x4085b8),map);cpu.writeMemory(toAddr(0x43979c),battle);
                cpu.writeMemory(toAddr(0x439d3c),new byte[100*288]);cpu.writeMemoryValue(toAddr(0x43874a),2,0);cpu.writeMemoryValue(toAddr(0x4435c4),2,0);cpu.writeMemoryValue(toAddr(0x43877c),2,0);
                cpu.writeMemory(toAddr(0x43167c),java.util.Arrays.copyOfRange(properties,64,12064));
                cpu.writeMemoryValue(toAddr(0x442780),4,-34);cpu.writeMemoryValue(toAddr(0x442784),4,-10);
                cpu.writeMemoryValue(toAddr(0x439582),2,0);cpu.writeMemoryValue(toAddr(0x442576),2,0);
                long frame=0x701000;cpu.writeRegister("PC",0x11e0fe);cpu.writeRegister("SP",0x700000);cpu.writeRegister("A6",frame);cpu.writeRegister("D5",base);cpu.writeRegister("D6",base);
                cpu.writeMemoryValue(toAddr(frame-8),2,5);cpu.writeMemoryValue(toAddr(frame-6),2,5);cpu.writeMemoryValue(toAddr(frame-4),2,85);cpu.writeMemoryValue(toAddr(frame-2),2,85);
                SaneBoundary sane=new SaneBoundary(cpu,currentProgram);boolean stopped=false;long seed=initial,draws=0,offset=0;
                for(int step=0;step<1500000;step++) {
                    long pc=cpu.getExecutionAddress().getOffset(),sp=cpu.readRegister("SP").longValue();
                    if(pc==0x11eb86){stopped=true;break;}
                    if(pc==0x136cfa){ret(cpu);continue;}
                    if(pc==0x400070){cpu.writeRegister("A0",0x600000);ret(cpu);continue;}
                    if(pc==0x400078){offset=pointer(cpu,sp+8);if(word(cpu,sp+12)!=0)throw new Exception("Non-absolute seek");cpu.writeRegister("D0",0);ret(cpu);continue;}
                    if(pc==0x400048){int size=(int)pointer(cpu,sp+8),count=(int)pointer(cpu,sp+12);if(size!=288||count!=1||offset+size>monsters.length)throw new Exception("Monster read boundary");cpu.writeMemory(toAddr(pointer(cpu,sp+4)),java.util.Arrays.copyOfRange(monsters,(int)offset,(int)offset+size));offset+=size;cpu.writeRegister("D0",1);ret(cpu);continue;}
                    if(pc==0x400038){cpu.writeRegister("D0",Math.abs(word(cpu,sp+4)));ret(cpu);continue;}
                    if(pc==0x153ca4){int bound=word(cpu,sp+4);seed=seed*16807%2147483647;draws++;short raw=(short)seed;if(raw==Short.MIN_VALUE)raw=0;cpu.writeRegister("D0",Math.abs((int)raw)*bound/32768+1);ret(cpu);continue;}
                    int opcode=word(cpu,pc)&0xffff;
                    if(opcode==0xa9eb){sane.execute();continue;}
                    if(opcode==0xa8ad){long r=pointer(cpu,sp);int y=word(cpu,sp+4),x=word(cpu,sp+6);boolean inside=y>=word(cpu,r)&&x>=word(cpu,r+2)&&y<word(cpu,r+4)&&x<word(cpu,r+6);cpu.writeMemoryValue(toAddr(sp+8),1,inside?1:0);cpu.writeRegister("SP",sp+8);cpu.writeRegister("PC",pc+2);continue;}
                    if(!cpu.step(monitor))throw new Exception("PC "+Long.toHexString(pc)+": "+cpu.getLastError());
                }
                if(!stopped)throw new Exception("Enemy setup instruction budget");
                int count=word(cpu,0x43874a);
                if(cases.length()>0)cases.append(',');cases.append("{\"battle_id\":").append(battleId).append(",\"monster_id\":").append(battleId==8?42:battleId==11?43:battleId==16?44:battleId==19?45:battleId==22?46:47).append(",\"battle\":\"").append(hex.formatHex(battle)).append("\",\"base\":").append(base).append(",\"seed\":").append(initial).append(",\"seed_before\":").append(initial).append(",\"seed_after\":").append(seed).append(",\"after_seed\":").append(seed).append(",\"draws\":").append(draws).append(",\"count\":").append(count).append(",\"records\":\"").append(hex.formatHex(cpu.readMemory(toAddr(0x439d3c),count*288))).append("\",\"tiles\":\"").append(hex.formatHex(cpu.readMemory(toAddr(0x4085b8),16200))).append("\",\"positions\":\"").append(hex.formatHex(cpu.readMemory(toAddr(0x446f0a),count*4))).append("\"}");
            }finally{cpu.dispose();}
        }
        }
        String identity="\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyGeneticEnemies.java"))+"\",\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\",\"definitions_sha256\":\""+ActionOracleHarness.sha256(scenario.resolve("Data MD"))+"\",\"battles_sha256\":\""+ActionOracleHarness.sha256(scenario.resolve("Data BD"))+"\",\"properties_sha256\":\""+ActionOracleHarness.sha256(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data Swamp BD"))+"\",\"formation_fixture_sha256\":\""+ActionOracleHarness.sha256(root.resolve("evidence/fixtures/formation-opening.json"))+"\",\"ghidra\":\"12.1.2\"";
        Files.writeString(root.resolve("evidence/fixtures/genetic-enemies.json"),"{\"authority\":\"Original combatsetup enemy slice11e0fe..11eb86, inline enemy initialization, original placemonster and bodyfield with exact BD8/11/16/19/22/25 and MD definitions. Uses independently verified opening formation map and Swamp properties; file, Rand, rectangle containment and SANE services modeled. Initial base-4 and bounds5..85 are synthetic controlled setup inputs. No complete battle/live route claim.\",\"cache_identity\":{"+identity+"},\"properties\":\""+hex.formatHex(properties)+"\",\"battles\":["+battleRecords+"],\"definitions\":\""+hex.formatHex(monsters)+"\",\"map\":\""+hex.formatHex(map)+"\",\"cases\":["+cases+"]}\n");
        println("Twelve genetic enemy setup cases exported");
    }
}
