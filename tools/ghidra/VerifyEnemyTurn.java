import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.ByteBuffer;
import java.util.HexFormat;

public class VerifyEnemyTurn extends GhidraScript {
    int word(EmulatorHelper cpu,long a){return ByteBuffer.wrap(cpu.readMemory(toAddr(a),2)).getShort();}
    long pointer(EmulatorHelper cpu,long a){return Integer.toUnsignedLong(ByteBuffer.wrap(cpu.readMemory(toAddr(a),4)).getInt());}
    void ret(EmulatorHelper cpu){long sp=cpu.readRegister("SP").longValue();cpu.writeRegister("PC",pointer(cpu,sp));cpu.writeRegister("SP",sp+4);}
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]);HexFormat hex=HexFormat.of();StringBuilder cases=new StringBuilder();int count=0;
        byte[] initial=java.util.Arrays.copyOfRange(Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data MD")),49*288,50*288);
        byte[] properties=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data Swamp BD"));
        for(int dx=-1;dx<=1;dx++)for(int dy=-1;dy<=1;dy++)if(dx!=0||dy!=0)for(int layout=0;layout<1;layout++)for(int points:new int[]{3,4})for(int initialSeed:new int[]{1,12,12345}) {
            EmulatorHelper cpu=new EmulatorHelper(currentProgram);
            try {
                for(String reg:new String[]{"D0","D1","D2","D3","D4","D5","D6","D7","A0","A1","A2","A3","A4","A5","A6","XF","NF","ZF","VF","CF"})cpu.writeRegister(reg,0);
                byte[] input=initial.clone();input[0x106]=24;input[0x109]=(byte)points;ByteBuffer.wrap(input).putShort(0x110,(short)8).putShort(0x112,(short)8).putShort(0x10c,(short)-1).putShort(0x11e,(short)1155);
                byte[] map=new byte[16200];ByteBuffer mw=ByteBuffer.wrap(map);for(int i=0;i<8100;i++)mw.putShort(i*2,(short)1155);
                if(layout==1 || layout==3)mw.putShort(((25+dx)*90+25+dy)*2,(short)(layout==3?1000:1061));
                if(layout==2)for(int x=24;x<=26;x++)for(int y=24;y<=26;y++)mw.putShort((x*90+y)*2,(short)1061);
                mw.putShort((25*90+25)*2,(short)10);mw.putShort(((25+dx*4)*90+25+dy*4)*2,(short)0);
                cpu.writeMemory(toAddr(0x4085b8),map);cpu.writeMemory(toAddr(0x439d3c),input);cpu.writeMemory(toAddr(0x43167c),java.util.Arrays.copyOfRange(properties,64,12064));
                cpu.writeMemoryValue(toAddr(0x442780),4,5);cpu.writeMemoryValue(toAddr(0x442784),4,5);cpu.writeMemoryValue(toAddr(0x446f0a),2,20);cpu.writeMemoryValue(toAddr(0x446f0c),2,20);cpu.writeMemoryValue(toAddr(0x44709a),2,20+dx*4);cpu.writeMemoryValue(toAddr(0x44709c),2,20+dy*4);
                cpu.writeMemoryValue(toAddr(0x447196),2,0);cpu.writeMemoryValue(toAddr(0x447194),2,1);cpu.writeMemoryValue(toAddr(0x4470ba),2,10);cpu.writeMemoryValue(toAddr(0x402f43),1,2);cpu.writeMemoryValue(toAddr(0x43550c),2,155);cpu.writeMemoryValue(toAddr(0x439568),2,0);cpu.writeMemory(toAddr(0x442bd6),new byte[1200]);
                byte[] warriors=new byte[3138*6];String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};for(int i=0;i<6;i++){byte[] w=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Warriors/"+names[i]));w[0x9df]=1;ByteBuffer.wrap(w).putShort(0x50,(short)100).putShort(0x52,(short)100);System.arraycopy(w,0,warriors,i*3138,3138);if(i>0){cpu.writeMemoryValue(toAddr(0x44709a+i*4),2,40+i);cpu.writeMemoryValue(toAddr(0x44709c+i*4),2,40);}}cpu.writeMemory(toAddr(0x448aa0),warriors);cpu.writeMemoryValue(toAddr(0x44273a),1,5);cpu.writeMemoryValue(toAddr(0x4435c4),2,1);cpu.writeMemoryValue(toAddr(0x43874a),2,1);
                long frame=0x701000;cpu.writeMemory(toAddr(frame-1024),new byte[2048]);cpu.writeRegister("A6",frame);long sp=0x700000;cpu.writeMemory(toAddr(sp-1024),new byte[2048]);cpu.writeMemoryValue(toAddr(sp),4,0x900000);cpu.writeRegister("SP",sp);cpu.writeRegister("PC",0x1145c8);long seed=initialSeed,draws=0;boolean stopped=false;StringBuilder sounds=new StringBuilder();SaneBoundary sane=new SaneBoundary(cpu,currentProgram);
                for(int step=0;step<1500000;step++){
                    long pc=cpu.getExecutionAddress().getOffset(),stack=cpu.readRegister("SP").longValue();
                    if(pc==0x115732){stopped=true;break;}
                    if(pc==0x400038){cpu.writeRegister("D0",Math.abs(word(cpu,stack+4)));ret(cpu);continue;}
                    if(pc==0x153ca4){int bound=word(cpu,stack+4);seed=seed*16807%2147483647;draws++;short raw=(short)seed;if(raw==Short.MIN_VALUE)raw=0;cpu.writeRegister("D0",Math.abs((int)raw)*bound/32768+1);ret(cpu);continue;}
                    if(pc==0x133a18){if(sounds.length()>0)sounds.append(',');sounds.append(word(cpu,stack+4));ret(cpu);continue;}
                    if(pc==0x120c14||pc==0x105572||pc==0x10e6b6||pc==0x133d46||pc==0x158454||pc==0x13154c||pc==0x11f1a8||pc==0x11edc4){ret(cpu);continue;}
                    int opcode=word(cpu,pc)&0xffff;if(opcode==0xa873){cpu.writeRegister("SP",stack+4);cpu.writeRegister("PC",pc+2);continue;}if(opcode==0xa9b4){cpu.writeRegister("PC",pc+2);continue;}if(opcode==0xa974){cpu.writeRegister("SP",stack-2);cpu.writeMemoryValue(toAddr(stack-2),2,0);cpu.writeRegister("PC",pc+2);continue;}if(opcode==0xa9eb){sane.execute();continue;}
                    if(!cpu.step(monitor))throw new Exception("PC "+Long.toHexString(pc)+": "+cpu.getLastError());
                }
                if(!stopped)throw new Exception("Enemy movement instruction budget");
                if(cases.length()>0)cases.append(',');cases.append("{\"dx\":").append(dx).append(",\"dy\":").append(dy).append(",\"layout\":").append(layout).append(",\"initial_seed\":").append(initialSeed).append(",\"input\":\"").append(hex.formatHex(input)).append("\",\"warriors\":\"").append(hex.formatHex(warriors)).append("\",\"after_warriors\":\"").append(hex.formatHex(cpu.readMemory(toAddr(0x448aa0),warriors.length))).append("\",\"after\":\"").append(hex.formatHex(cpu.readMemory(toAddr(0x439d3c),288))).append("\",\"tiles\":\"").append(hex.formatHex(cpu.readMemory(toAddr(0x4085b8),16200))).append("\",\"x\":").append(word(cpu,0x446f0a)).append(",\"y\":").append(word(cpu,0x446f0c)).append(",\"moved\":").append(true).append(",\"sounds\":[").append(sounds).append("],\"seed\":").append(seed).append(",\"draws\":").append(draws).append('}');count++;
            } finally {cpu.dispose();}
        }
        Files.writeString(root.resolve("evidence/fixtures/enemy-turn.json"),"{\"authority\":\"Original combat enemy decision branch through turn end, checkforenemy, attack2, cansee, movemonster and empty collide; presentation intercepted; size-zero monster49\",\"properties\":\""+hex.formatHex(properties)+"\",\"cases\":["+cases+"]}\n");println(count+" enemy movement cases exported");
    }
}
