import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.ByteBuffer;
import java.util.HexFormat;

public class VerifyPoisonRound extends GhidraScript {
    int word(EmulatorHelper cpu,long a){return ByteBuffer.wrap(cpu.readMemory(toAddr(a),2)).getShort();}
    long pointer(EmulatorHelper cpu,long a){return Integer.toUnsignedLong(ByteBuffer.wrap(cpu.readMemory(toAddr(a),4)).getInt());}
    void ret(EmulatorHelper cpu){long sp=cpu.readRegister("SP").longValue();cpu.writeRegister("PC",pointer(cpu,sp));cpu.writeRegister("SP",sp+4);}
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]);HexFormat hex=HexFormat.of();
        byte[] initial=new byte[6*3138];String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};
        for(int i=0;i<6;i++) {System.arraycopy(Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Warriors/"+names[i])),0,initial,i*3138,3138);initial[i*3138+0x9df]=1;}
        byte[] monster=java.util.Arrays.copyOfRange(Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data MD")),49*288,50*288);
        ByteBuffer.wrap(monster).putShort(0x110,(short)8).putShort(0x112,(short)8);
        StringBuilder cases=new StringBuilder();
        for(int body:new int[]{1})for(int health:new int[]{1,20,100})for(int poison:new int[]{-1,1,2,29})for(int regen:new int[]{0,5})for(int disease:new int[]{0,2}) {
            EmulatorHelper cpu=new EmulatorHelper(currentProgram);
            try {
                for(String reg:new String[]{"D0","D1","D2","D3","D4","D5","D6","D7","A0","A1","A2","A3","A4","A5","A6","XF","NF","ZF","VF","CF"})cpu.writeRegister(reg,0);
                byte[] warriors=initial.clone(),enemy=monster.clone();for(int i=0;i<6;i++){ByteBuffer w=ByteBuffer.wrap(warriors);w.putShort(i*3138+0x50,w.getShort(i*3138+0x52)).putShort(i*3138+0x20,(short)(i+1)).putShort(i*3138+0x18,(short)0);}
                ByteBuffer.wrap(warriors).putShort(0x50,(short)health).putShort(0x64,(short)poison).putShort(0x96,(short)disease).putShort(0x66,(short)regen);
                ByteBuffer.wrap(enemy).putShort(0x10c,(short)0).putShort(0x11e,(short)1155);
                byte[] map=new byte[16200];ByteBuffer mw=ByteBuffer.wrap(map);for(int i=0;i<8100;i++)mw.putShort(i*2,(short)1155);for(int i=0;i<6;i++)mw.putShort(((25+i)*90+25)*2,(short)i);mw.putShort((35*90+35)*2,(short)10);
                byte[] order=new byte[220];ByteBuffer ow=ByteBuffer.wrap(order);for(int i=0;i<110;i++)ow.putShort(i*2,(short)-1);for(int i=0;i<6;i++)ow.putShort((i+1)*2,(short)i);ow.putShort(14,(short)10);ow.putShort(2,(short)1);ow.putShort(4,(short)0);
                ByteBuffer.wrap(warriors).putShort(0x20,(short)2).putShort(3138+0x20,(short)1);
                cpu.writeMemory(toAddr(0x4085b8),map);cpu.writeMemory(toAddr(0x4470b8),order);cpu.writeMemory(toAddr(0x440edc),new byte[6*288]);cpu.writeMemory(toAddr(0x442562),new byte[20]);
                cpu.writeMemoryValue(toAddr(0x442780),4,5);cpu.writeMemoryValue(toAddr(0x442784),4,5);cpu.writeMemoryValue(toAddr(0x438776),2,1);cpu.writeMemoryValue(toAddr(0x43874a),2,1);cpu.writeMemoryValue(toAddr(0x44257c),2,0);
                for(int i=0;i<6;i++){cpu.writeMemoryValue(toAddr(0x44709a+i*4),2,20+i);cpu.writeMemoryValue(toAddr(0x44709c+i*4),2,20);cpu.writeMemoryValue(toAddr(0x437a6e+i*2),2,1155);}
                cpu.writeMemoryValue(toAddr(0x446f0a),2,30);cpu.writeMemoryValue(toAddr(0x446f0c),2,30);

                cpu.writeMemory(toAddr(0x448aa0),warriors);cpu.writeMemory(toAddr(0x439d3c),enemy);
                cpu.writeMemoryValue(toAddr(0x44273a),1,5);cpu.writeMemoryValue(toAddr(0x4435c4),2,1);cpu.writeMemoryValue(toAddr(0x44374c),2,0);
                cpu.writeMemoryValue(toAddr(0x447194),2,109);cpu.writeMemoryValue(toAddr(0x4470b8),2,-1);cpu.writeMemoryValue(toAddr(0x4470ba),2,body);
                cpu.writeMemoryValue(toAddr(0x43800c),2,0);cpu.writeMemoryValue(toAddr(0x447196),2,0);cpu.writeMemoryValue(toAddr(0x4435d2),2,0);
                cpu.writeMemory(toAddr(0x435664),new byte[]{12,14,12,14,10,12,10,10,14,16,0,12});
                long frame=0x701000;cpu.writeMemory(toAddr(frame-512),new byte[1024]);cpu.writeRegister("PC",0x125ca0);cpu.writeRegister("SP",0x700000);cpu.writeRegister("A6",frame);
                SaneBoundary sane=new SaneBoundary(cpu,currentProgram);boolean stopped=false;long seed=1,draws=0;StringBuilder sounds=new StringBuilder(),tickSounds=new StringBuilder();
                for(int step=0;step<500000;step++) {
                    long pc=cpu.getExecutionAddress().getOffset(),sp=cpu.readRegister("SP").longValue();
                    if(pc==0x127090){stopped=true;break;}
                    if(pc==0x400038){cpu.writeRegister("D0",(short)Math.abs(word(cpu,sp+4)));ret(cpu);continue;}
                    if(pc==0x161b80||pc==0x13a2c4||pc==0x1205f8||pc==0x105572||pc==0x1382a8||pc==0x158454||pc==0x15f66c||pc==0x400038||pc==0x1600fc||pc==0x13a498||pc==0x12382a||pc==0x13154c||pc==0x123d1c||pc==0x160a8e||pc==0x160cca||pc==0x160baa||pc==0x160fb2||pc==0x11f1a8||pc==0x16043e||pc==0x10e6b6){ret(cpu);continue;}
                    if(pc==0x12946c){cpu.writeRegister("D0",1);ret(cpu);continue;}
                    if(pc==0x153ca4){int bound=word(cpu,sp+4);seed=seed*16807%2147483647;draws++;short raw=(short)seed;if(raw==Short.MIN_VALUE)raw=0;cpu.writeRegister("D0",Math.abs((int)raw)*bound/32768+1);ret(cpu);continue;}
                    int opcode=word(cpu,pc)&0xffff;if(opcode==0xa8a7){cpu.writeRegister("SP",sp+12);cpu.writeRegister("PC",pc+2);continue;}if(opcode==0xa8a8){cpu.writeRegister("SP",sp+8);cpu.writeRegister("PC",pc+2);continue;}if(opcode==0xa873){cpu.writeRegister("SP",sp+4);cpu.writeRegister("PC",pc+2);continue;}
                    if(opcode==0xaa1e){cpu.writeMemoryValue(toAddr(sp+2),4,0);cpu.writeRegister("SP",sp+2);cpu.writeRegister("PC",pc+2);continue;}
                    if(pc==0x133a18){if(sounds.length()>0)sounds.append(',');sounds.append(word(cpu,sp+4));if(pointer(cpu,sp)==0x125f0a){if(tickSounds.length()>0)tickSounds.append(',');tickSounds.append(word(cpu,sp+4));}ret(cpu);continue;}
                    if(opcode==0xa9eb){sane.execute();continue;}
                    if(!cpu.step(monitor))throw new Exception("Body "+body+" PC "+Long.toHexString(pc)+": "+cpu.getLastError());
                }
                if(!stopped)throw new Exception("Turn instruction budget");
                if(cases.length()>0)cases.append(',');cases.append("{\"health\":").append(health).append(",\"poison\":").append(poison).append(",\"regen\":").append(regen).append(",\"before_map\":\"").append(hex.formatHex(map)).append("\",\"before_order\":\"").append(hex.formatHex(order)).append("\",\"map\":\"").append(hex.formatHex(cpu.readMemory(toAddr(0x4085b8),16200))).append("\",\"order\":\"").append(hex.formatHex(cpu.readMemory(toAddr(0x4470b8),220))).append("\",\"sounds\":[").append(sounds).append("],\"tick_sounds\":[").append(tickSounds).append("],\"body\":").append(body).append(",\"warriors\":\"").append(hex.formatHex(warriors)).append("\",\"enemy\":\"").append(hex.formatHex(enemy)).append("\",\"after_warriors\":\"").append(hex.formatHex(cpu.readMemory(toAddr(0x448aa0),initial.length))).append("\",\"after_enemy\":\"").append(hex.formatHex(cpu.readMemory(toAddr(0x439d3c),288))).append("\",\"seed\":").append(seed).append(",\"draws\":").append(draws).append('}');
            } finally {cpu.dispose();}
        }
        Files.writeString(root.resolve("evidence/fixtures/poison-round.json"),"{\"authority\":\"getup round transition including complete original reduce, heal, poison tick, killbody, bodyfield, bleeding and next warrior turn; six original warriors on controlled flat Swamp arena with mixed disease0/2 and poison duration -1,1,2,29; HP1,20,100; regen0/5. Empty collision/script queue, presentation intercepted and bounded SANE model.\",\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyPoisonRound.java"))+"\",\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\",\"cases\":["+cases+"]}\n");
        println("48 poison round cases exported");
    }
}
