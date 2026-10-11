import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.ByteBuffer;
import java.util.HexFormat;

public class VerifyBleeding extends GhidraScript {
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
        for(int body:new int[]{1})for(int carry:new int[]{0})for(int bleedingHealth:new int[]{0,-8,-9,-10}) {
            EmulatorHelper cpu=new EmulatorHelper(currentProgram);
            try {
                for(String reg:new String[]{"D0","D1","D2","D3","D4","D5","D6","D7","A0","A1","A2","A3","A4","A5","A6","XF","NF","ZF","VF","CF"})cpu.writeRegister(reg,0);
                byte[] warriors=initial.clone(),enemy=monster.clone();for(int i=0;i<6;i++){ByteBuffer w=ByteBuffer.wrap(warriors);w.putShort(i*3138+0x50,(short)(w.getShort(i*3138+0x52)-carry));}

                ByteBuffer.wrap(warriors).putShort(0x50,(short)bleedingHealth).putInt(0xa62,1);warriors[0x9de]=1;warriors[0x9df]=0;cpu.writeMemory(toAddr(0x448aa0),warriors);cpu.writeMemory(toAddr(0x439d3c),enemy);
                cpu.writeMemoryValue(toAddr(0x44273a),1,5);cpu.writeMemoryValue(toAddr(0x4435c4),2,1);cpu.writeMemoryValue(toAddr(0x44374c),2,0);
                cpu.writeMemoryValue(toAddr(0x447194),2,109);cpu.writeMemoryValue(toAddr(0x4470b8),2,-1);cpu.writeMemoryValue(toAddr(0x4470ba),2,body);
                cpu.writeMemoryValue(toAddr(0x43800c),2,0);cpu.writeMemoryValue(toAddr(0x447196),2,0);cpu.writeMemoryValue(toAddr(0x4435d2),2,0);
                cpu.writeMemory(toAddr(0x435664),new byte[]{12,14,12,14,10,12,10,10,14,16,0,12});
                long frame=0x701000;cpu.writeMemory(toAddr(frame-512),new byte[1024]);cpu.writeRegister("PC",0x125ca0);cpu.writeRegister("SP",0x700000);cpu.writeRegister("A6",frame);
                SaneBoundary sane=new SaneBoundary(cpu,currentProgram);boolean stopped=false;long seed=1,draws=0;StringBuilder sounds=new StringBuilder(),bleedingSounds=new StringBuilder();
                for(int step=0;step<500000;step++) {
                    long pc=cpu.getExecutionAddress().getOffset(),sp=cpu.readRegister("SP").longValue();
                    if(pc==0x127090){stopped=true;break;}
                    if(pc==0x1600fc||pc==0x13a498||pc==0x12382a||pc==0x13154c||pc==0x123d1c||pc==0x160a8e||pc==0x160cca||pc==0x160baa||pc==0x160fb2||pc==0x11f1a8||pc==0x16043e||pc==0x10e6b6){ret(cpu);continue;}
                    if(pc==0x12946c){cpu.writeRegister("D0",1);ret(cpu);continue;}
                    if(pc==0x153ca4){int bound=word(cpu,sp+4);seed=seed*16807%2147483647;draws++;short raw=(short)seed;if(raw==Short.MIN_VALUE)raw=0;cpu.writeRegister("D0",Math.abs((int)raw)*bound/32768+1);ret(cpu);continue;}
                    int opcode=word(cpu,pc)&0xffff;if(opcode==0xa8a7){cpu.writeRegister("SP",sp+12);cpu.writeRegister("PC",pc+2);continue;}if(opcode==0xa8a8){cpu.writeRegister("SP",sp+8);cpu.writeRegister("PC",pc+2);continue;}if(opcode==0xa873){cpu.writeRegister("SP",sp+4);cpu.writeRegister("PC",pc+2);continue;}
                    if(pc==0x133a18){if(sounds.length()>0)sounds.append(",");sounds.append(word(cpu,sp+4));if(pointer(cpu,sp)==0x125f0a){if(bleedingSounds.length()>0)bleedingSounds.append(",");bleedingSounds.append(word(cpu,sp+4));}ret(cpu);continue;}
                    if(pc==0x15f66c||pc==0x161b80){ret(cpu);continue;}
                    if(opcode==0xa9eb){sane.execute();continue;}
                    if(!cpu.step(monitor))throw new Exception("Body "+body+" PC "+Long.toHexString(pc)+": "+cpu.getLastError());
                }
                if(!stopped)throw new Exception("Turn instruction budget");
                if(cases.length()>0)cases.append(',');cases.append("{\"body\":").append(body).append(",\"warriors\":\"").append(hex.formatHex(warriors)).append("\",\"enemy\":\"").append(hex.formatHex(enemy)).append("\",\"after_warriors\":\"").append(hex.formatHex(cpu.readMemory(toAddr(0x448aa0),initial.length))).append("\",\"after_enemy\":\"").append(hex.formatHex(cpu.readMemory(toAddr(0x439d3c),288))).append("\",\"seed\":").append(seed).append(",\"draws\":").append(draws).append(",\"sounds\":[").append(sounds).append("],\"bleeding_sounds\":[").append(bleedingSounds).append("]}");
            } finally {cpu.dispose();}
        }
        Files.writeString(root.resolve("evidence/fixtures/bleeding-round.json"),"{\"authority\":\"getup round transition including reduce and bleeding; warrior0 unconscious with HP0,-8,-9,-10; warrior1 active; death sound and draw retained; presentation intercepted; empty spell queue and bounded SANE model\",\"cases\":["+cases+"]}\n");
        println("Bleeding round cases exported");
    }
}
