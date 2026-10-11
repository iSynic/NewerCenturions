import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.ByteBuffer;
import java.util.HexFormat;

public class VerifyVictory extends GhidraScript {
    int word(EmulatorHelper cpu,long a){return ByteBuffer.wrap(cpu.readMemory(toAddr(a),2)).getShort();}
    long pointer(EmulatorHelper cpu,long a){return Integer.toUnsignedLong(ByteBuffer.wrap(cpu.readMemory(toAddr(a),4)).getInt());}
    void ret(EmulatorHelper cpu){long sp=cpu.readRegister("SP").longValue();cpu.writeRegister("PC",pointer(cpu,sp));cpu.writeRegister("SP",sp+4);}
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]);HexFormat hex=HexFormat.of();StringBuilder cases=new StringBuilder();
        String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};
        byte[] initial=java.util.Arrays.copyOfRange(Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data MD")),49*288,50*288);
        for(int body:new int[]{5,6,8})for(int health:new int[]{0,1}) {
            EmulatorHelper cpu=new EmulatorHelper(currentProgram);
            try {
                for(String reg:new String[]{"D0","D1","D2","D3","D4","D5","D6","D7","A0","A1","A2","A3","A4","A5","A6","XF","NF","ZF","VF","CF"})cpu.writeRegister(reg,0);
                byte[] warriors=new byte[3138*6];
                for(int i=0;i<6;i++){byte[] w=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Warriors/"+names[i]));w[0x9df]=1;ByteBuffer.wrap(w).putShort(0x20,(short)(i+1)).putShort(0x18,(short)body);if(i==0&&health==1){ByteBuffer.wrap(w).putShort(0x50,(short)0);w[0x9df]=0;}System.arraycopy(w,0,warriors,i*3138,3138);cpu.writeMemoryValue(toAddr(0x44709a+i*4),2,20+i);cpu.writeMemoryValue(toAddr(0x44709c+i*4),2,20);cpu.writeMemoryValue(toAddr(0x437a6e+i*2),2,1155);}
                byte[] m=initial.clone();m[0x106]=24;m[0xb4]=2;m[0xc9]=1;ByteBuffer.wrap(m).putShort(0x110,(short)0).putShort(0x112,(short)8).putShort(0x10c,(short)body).putShort(0x11e,(short)1155);
                byte[] map=new byte[16200];ByteBuffer mw=ByteBuffer.wrap(map);for(int i=0;i<8100;i++)mw.putShort(i*2,(short)1155);for(int i=0;i<6;i++)mw.putShort(((25+i)*90+25)*2,(short)i);mw.putShort((35*90+35)*2,(short)10);
                byte[] order=new byte[220];ByteBuffer ow=ByteBuffer.wrap(order);for(int i=0;i<110;i++)ow.putShort(i*2,(short)-1);for(int i=0;i<6;i++)ow.putShort((i+1)*2,(short)i);ow.putShort(14,(short)10);
                cpu.writeMemory(toAddr(0x448aa0),warriors);for(int j=0;j<body;j++)cpu.writeMemory(toAddr(0x439d3c+j*288),m);cpu.writeMemory(toAddr(0x4085b8),map);cpu.writeMemory(toAddr(0x4470b8),order);
                cpu.writeMemoryValue(toAddr(0x442780),4,5);cpu.writeMemoryValue(toAddr(0x442784),4,5);cpu.writeMemoryValue(toAddr(0x446f0a),2,30);cpu.writeMemoryValue(toAddr(0x446f0c),2,30);cpu.writeMemoryValue(toAddr(0x44273a),1,5);cpu.writeMemoryValue(toAddr(0x438776),2,1);cpu.writeMemoryValue(toAddr(0x4435c4),2,1);cpu.writeMemoryValue(toAddr(0x43874a),2,body);
                long sp=0x700000;cpu.writeMemory(toAddr(sp-4096),new byte[8192]);cpu.writeMemoryValue(toAddr(sp),4,0x900000);cpu.writeMemoryValue(toAddr(sp+4),2,body);cpu.writeMemoryValue(toAddr(sp+6),2,0);long frame=0x701000;cpu.writeMemory(toAddr(frame-1024),new byte[2048]);cpu.writeMemoryValue(toAddr(frame+8),2,1);cpu.writeMemoryValue(toAddr(frame+10),2,0);cpu.writeRegister("A6",frame);cpu.writeRegister("SP",sp);cpu.writeRegister("PC",0x1057da);boolean stopped=false;long seed=12,draws=0;SaneBoundary sane=new SaneBoundary(cpu,currentProgram);
                for(int step=0;step<500000;step++){
                    long pc=cpu.getExecutionAddress().getOffset(),stack=cpu.readRegister("SP").longValue();if(pc==0x1064c2){stopped=true;break;}
                    if(pc==0x400038){cpu.writeRegister("D0",Math.abs(word(cpu,stack+4)));ret(cpu);continue;}
                    if(pc==0x1046e0||pc==0x13e13c||pc==0x13a7c4||pc==0x13154c||pc==0x13a2c4||pc==0x12382a||pc==0x160fb2||pc==0x158454||pc==0x1205f8||pc==0x105572||pc==0x1382a8){ret(cpu);continue;}
                    if(pc==0x106252){cpu.writeRegister("PC",0x106332);continue;}
                    if(pc==0x153ca4){int bound=word(cpu,stack+4);seed=seed*16807%2147483647;draws++;short raw=(short)seed;if(raw==Short.MIN_VALUE)raw=0;cpu.writeRegister("D0",Math.abs((int)raw)*bound/32768+1);ret(cpu);continue;}
                    int opcode=word(cpu,pc)&0xffff;if(opcode==0xa9eb){sane.execute();continue;}
                    if(opcode==0xa939){cpu.writeRegister("SP",stack+6);cpu.writeRegister("PC",pc+2);continue;}if(opcode==0xaa1c||opcode==0xa93c){cpu.writeRegister("SP",stack+4);cpu.writeRegister("PC",pc+2);continue;}if(opcode==0xa937||opcode==0xa89b){cpu.writeRegister("PC",pc+2);continue;}if(opcode==0xa873){cpu.writeRegister("SP",stack+4);cpu.writeRegister("PC",pc+2);continue;}
                    if(opcode==0xaa1e){cpu.writeMemoryValue(toAddr(stack+2),4,0);cpu.writeRegister("SP",stack+2);cpu.writeRegister("PC",pc+2);continue;}
                    if(!cpu.step(monitor))throw new Exception("body "+body+" PC "+Long.toHexString(pc)+": "+cpu.getLastError());
                }
                if(!stopped)throw new Exception("Kill instruction budget");if(cases.length()>0)cases.append(',');
                cases.append("{\"body\":").append(body).append(",\"health\":").append(health).append(",\"warriors\":\"").append(hex.formatHex(warriors)).append("\",\"enemy\":\"").append(hex.formatHex(m)).append("\",\"after_warriors\":\"").append(hex.formatHex(cpu.readMemory(toAddr(0x448aa0),3138*6))).append("\",\"after_enemy\":\"").append(hex.formatHex(cpu.readMemory(toAddr(0x439d3c),288))).append("\",\"map\":\"").append(hex.formatHex(cpu.readMemory(toAddr(0x4085b8),16200))).append("\",\"order\":\"").append(hex.formatHex(cpu.readMemory(toAddr(0x4470b8),220))).append("\",\"seed\":").append(seed).append(",\"draws\":").append(draws).append("}");
            } finally {cpu.dispose();}
        }
        Files.writeString(root.resolve("evidence/fixtures/victory.json"),"{\"authority\":\"Original booty victory cleanup and rewards through 1064c2; selected enemy49 groups and six starters; presentation and profile loading intercepted, empty loot sorting skipped\",\"cases\":["+cases+"]}\n");println("21 kill cases exported");
    }
}
