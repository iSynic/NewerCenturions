import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.ByteBuffer;
import java.util.HexFormat;

public class VerifyEnemyRangedTurn extends GhidraScript {
    int word(EmulatorHelper cpu,long a){return ByteBuffer.wrap(cpu.readMemory(toAddr(a),2)).getShort();}
    long pointer(EmulatorHelper cpu,long a){return Integer.toUnsignedLong(ByteBuffer.wrap(cpu.readMemory(toAddr(a),4)).getInt());}
    void ret(EmulatorHelper cpu){long sp=cpu.readRegister("SP").longValue();cpu.writeRegister("PC",pointer(cpu,sp));cpu.writeRegister("SP",sp+4);}
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]);HexFormat hex=HexFormat.of();StringBuilder cases=new StringBuilder();int count=0;
        byte[] definitions=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data ID"));byte[] attacks=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data S"));
        byte[] initial=java.util.Arrays.copyOfRange(Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data MD")),49*288,50*288);
        byte[] properties=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data Swamp BD"));
        for(int dx:new int[]{1,4})for(int dy:new int[]{0})for(int layout=0;layout<1;layout++)for(int points:new int[]{3,4})for(int initialSeed:new int[]{1,3,12345})for(int weapon:new int[]{680,681}) {
            EmulatorHelper cpu=new EmulatorHelper(currentProgram);
            try {
                for(String reg:new String[]{"D0","D1","D2","D3","D4","D5","D6","D7","A0","A1","A2","A3","A4","A5","A6","XF","NF","ZF","VF","CF"})cpu.writeRegister(reg,0);
                byte[] input=java.util.Arrays.copyOfRange(Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data MD")),(weapon==680?22:23)*288,(weapon==680?23:24)*288);ByteBuffer.wrap(input).putShort(0x26,(short)weapon).putShort(0x4c,(short)0);cpu.writeMemory(toAddr(0x4537fe),java.util.Arrays.copyOfRange(definitions,0,30000));cpu.writeMemory(toAddr(0x46225e),java.util.Arrays.copyOfRange(definitions,30000,60000));cpu.writeMemory(toAddr(0x46978e),java.util.Arrays.copyOfRange(definitions,90000,120000));cpu.writeMemory(toAddr(0x443788),attacks);input[0x106]=24;input[0x109]=(byte)points;ByteBuffer.wrap(input).putShort(0x110,(short)8).putShort(0x112,(short)8).putShort(0x10c,(short)-1).putShort(0x11e,(short)1155);
                byte[] map=new byte[16200];ByteBuffer mw=ByteBuffer.wrap(map);for(int i=0;i<8100;i++)mw.putShort(i*2,(short)1155);
                if(layout==1 || layout==3)mw.putShort(((25+dx)*90+25+dy)*2,(short)(layout==3?1000:1061));
                if(layout==2)for(int x=24;x<=26;x++)for(int y=24;y<=26;y++)mw.putShort((x*90+y)*2,(short)1061);
                mw.putShort((25*90+25)*2,(short)10);mw.putShort(((25+dx*4)*90+25+dy*4)*2,(short)0);
                cpu.writeMemory(toAddr(0x4085b8),map);cpu.writeMemory(toAddr(0x439d3c),input);cpu.writeMemory(toAddr(0x43167c),java.util.Arrays.copyOfRange(properties,64,12064));
                cpu.writeMemoryValue(toAddr(0x442780),4,5);cpu.writeMemoryValue(toAddr(0x442784),4,5);cpu.writeMemoryValue(toAddr(0x446f0a),2,20);cpu.writeMemoryValue(toAddr(0x446f0c),2,20);cpu.writeMemoryValue(toAddr(0x44709a),2,20+dx*4);cpu.writeMemoryValue(toAddr(0x44709c),2,20+dy*4);
                cpu.writeMemoryValue(toAddr(0x447196),2,0);cpu.writeMemoryValue(toAddr(0x447194),2,1);cpu.writeMemoryValue(toAddr(0x4470ba),2,10);cpu.writeMemoryValue(toAddr(0x402f43),1,2);cpu.writeMemoryValue(toAddr(0x43550c),2,155);cpu.writeMemoryValue(toAddr(0x439568),2,0);cpu.writeMemory(toAddr(0x442bd6),new byte[1200]);
                byte[] warriors=new byte[3138*6];String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};for(int i=0;i<6;i++){byte[] w=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Warriors/"+names[i]));w[0x9df]=1;ByteBuffer.wrap(w).putShort(0x50,(short)100).putShort(0x52,(short)100);System.arraycopy(w,0,warriors,i*3138,3138);if(i>0){cpu.writeMemoryValue(toAddr(0x44709a+i*4),2,20+dx*4+i);cpu.writeMemoryValue(toAddr(0x44709c+i*4),2,20+dy*4);mw.putShort(((25+dx*4+i)*90+25+dy*4)*2,(short)i);}}cpu.writeMemory(toAddr(0x4085b8),map);cpu.writeMemory(toAddr(0x448aa0),warriors);cpu.writeMemoryValue(toAddr(0x44273a),1,5);cpu.writeMemoryValue(toAddr(0x4435c4),2,1);cpu.writeMemoryValue(toAddr(0x43874a),2,1);
                long frame=0x701000;cpu.writeMemory(toAddr(frame-1024),new byte[2048]);cpu.writeRegister("A6",frame);long sp=0x700000;cpu.writeMemory(toAddr(sp-1024),new byte[2048]);cpu.writeMemoryValue(toAddr(sp),4,0x900000);cpu.writeRegister("SP",sp);cpu.writeRegister("PC",0x1145c8);long seed=initialSeed,draws=0;boolean stopped=false;StringBuilder sounds=new StringBuilder();SaneBoundary sane=new SaneBoundary(cpu,currentProgram);
                for(int step=0;step<1500000;step++){
                    long pc=cpu.getExecutionAddress().getOffset(),stack=cpu.readRegister("SP").longValue();
                    if(pc==0x115732 && (word(cpu,0x439d84)==0 || (byte)cpu.readMemory(toAddr(0x439e45),1)[0]<2 || word(cpu,0x438780)>1)){stopped=true;break;}
                    if(pc==0x4001a0){sane.extended(pointer(cpu,stack+4),Math.sqrt(sane.extended(stack+8)));ret(cpu);continue;}
                    if(pc==0x400038){cpu.writeRegister("D0",Math.abs(word(cpu,stack+4)));ret(cpu);continue;}
                    if(pc==0x153ca4){int bound=word(cpu,stack+4);seed=seed*16807%2147483647;draws++;short raw=(short)seed;if(raw==Short.MIN_VALUE)raw=0;cpu.writeRegister("D0",Math.abs((int)raw)*bound/32768+1);ret(cpu);continue;}
                    if(pc==0x133a18){if(sounds.length()>0)sounds.append(',');sounds.append(word(cpu,stack+4));ret(cpu);continue;}
                    if(pc==0x10d4fe){cpu.writeMemoryValue(toAddr(0x438776),2,1);cpu.writeMemory(toAddr(0x4434e0),new byte[220]);int n=word(cpu,0x4435c0);for(int t=0;t<n;t++){int body=word(cpu,0x443722+t*6);if(word(cpu,0x443726+t*6)!=1)throw new Exception("Unexpected cast target");cpu.writeMemoryValue(toAddr(0x4434e0+body*2),2,1);}cpu.writeRegister("PC",0x151022);continue;}
                    if(pc==0x16043e||pc==0x159192||pc==0x15a1f6||pc==0x15edfa||pc==0x13154c||pc==0x123d1c||pc==0x1107cc||pc==0x15b2f6||pc==0x1205f8||pc==0x12e4e4||pc==0x10ff54||pc==0x15879c||pc==0x13154c||pc==0x12f040||pc==0x120c14||pc==0x105572||pc==0x10e6b6||pc==0x133d46||pc==0x158454||pc==0x13154c||pc==0x11f1a8||pc==0x11edc4){ret(cpu);continue;}
                    int opcode=word(cpu,pc)&0xffff;if(opcode==0xa873){cpu.writeRegister("SP",stack+4);cpu.writeRegister("PC",pc+2);continue;}if(opcode==0xa9b4){cpu.writeRegister("PC",pc+2);continue;}if(opcode==0xa974){cpu.writeRegister("SP",stack-2);cpu.writeMemoryValue(toAddr(stack-2),2,0);cpu.writeRegister("PC",pc+2);continue;}if(opcode==0xa9eb){sane.execute();continue;}
                    if(!cpu.step(monitor))throw new Exception("PC "+Long.toHexString(pc)+": "+cpu.getLastError());
                }
                if(!stopped)throw new Exception("Enemy movement instruction budget");
                if(cases.length()>0)cases.append(',');cases.append("{\"weapon_id\":").append(weapon).append(",\"dx\":").append(dx).append(",\"dy\":").append(dy).append(",\"layout\":").append(layout).append(",\"initial_seed\":").append(initialSeed).append(",\"input\":\"").append(hex.formatHex(input)).append("\",\"warriors\":\"").append(hex.formatHex(warriors)).append("\",\"after_warriors\":\"").append(hex.formatHex(cpu.readMemory(toAddr(0x448aa0),warriors.length))).append("\",\"after\":\"").append(hex.formatHex(cpu.readMemory(toAddr(0x439d3c),288))).append("\",\"tiles\":\"").append(hex.formatHex(cpu.readMemory(toAddr(0x4085b8),16200))).append("\",\"x\":").append(word(cpu,0x446f0a)).append(",\"y\":").append(word(cpu,0x446f0c)).append(",\"moved\":").append(true).append(",\"sounds\":[").append(sounds).append("],\"seed\":").append(seed).append(",\"draws\":").append(draws).append('}');count++;
            } finally {cpu.dispose();}
        }
        Files.writeString(root.resolve("evidence/fixtures/enemy-ranged-turn.json"),"{\"authority\":\"Original combat enemy decision through terminal turn end for chemical weapons680/681; cansee, getrange, movement, resist, resolvedamage and savevs execute; targeted cast redirects into resolvespell with selected flags; presentation intercepted; six100HP warriors; SANE modeled\",\"properties\":\""+hex.formatHex(properties)+"\",\"cases\":["+cases+"]}\n");println(count+" enemy movement cases exported");
    }
}
