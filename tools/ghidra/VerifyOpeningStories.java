import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.ByteBuffer;
import java.util.HexFormat;
import java.util.Arrays;

public class VerifyOpeningStories extends GhidraScript {
    int word(EmulatorHelper c,long a){return ByteBuffer.wrap(c.readMemory(toAddr(a),2)).getShort();}
    long pointer(EmulatorHelper c,long a){return Integer.toUnsignedLong(ByteBuffer.wrap(c.readMemory(toAddr(a),4)).getInt());}
    void ret(EmulatorHelper c){long s=c.readRegister("SP").longValue();c.writeRegister("PC",pointer(c,s));c.writeRegister("SP",s+4);}
    void loadScript(EmulatorHelper c,byte[] scripts,int id){c.writeMemory(toAddr(0x479eb2),Arrays.copyOfRange(scripts,id*42+8,id*42+42));}
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]);Path scenario=root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0");
        byte[] scripts=Files.readAllBytes(scenario.resolve("Data ED3")),extra=Files.readAllBytes(scenario.resolve("Data EDCD")),menus=Files.readAllBytes(scenario.resolve("Data ED")),monsters=Files.readAllBytes(scenario.resolve("Data MD"));
        byte[] regions=Arrays.copyOf(Files.readAllBytes(scenario.resolve("Data RD")),644);HexFormat hex=HexFormat.of();StringBuilder cases=new StringBuilder();
        for(int script:new int[]{3,4,6})for(int variant=0;variant<(script==6?1:3);variant++) {
            EmulatorHelper c=new EmulatorHelper(currentProgram);
            try {
                for(String r:new String[]{"D0","D1","D2","D3","D4","D5","D6","D7","A0","A1","A2","A3","A4","A5","A6","XF","NF","ZF","VF","CF"})c.writeRegister(r,0);
                byte[] enemy=new byte[9*288],positions=new byte[36],map=new byte[16200],order=new byte[220];ByteBuffer em=ByteBuffer.wrap(enemy),ps=ByteBuffer.wrap(positions),mp=ByteBuffer.wrap(map),or=ByteBuffer.wrap(order);
                for(int i=0;i<8100;i++)mp.putShort(i*2,(short)1155);for(int i=0;i<110;i++)or.putShort(i*2,(short)-1);
                int[][] xy={{20,20},{24,20},{20,24},{30,20},{35,20},{24,21},{21,24},{30,21},{35,21}};
                for(int i=0;i<9;i++) {int cls=i==0?(script==4?16:17):i<5?22:23;System.arraycopy(monsters,cls*288,enemy,i*288,288);em.putShort(i*288+0x110,(short)(i==0?0:10)).putShort(i*288+0x10c,(short)1).putShort(i*288+0x11e,(short)1155);enemy[i*288+0xb4]=2;enemy[i*288+0xc9]=1;int x=xy[i][0]+(i>0&&variant==1?10:0),y=xy[i][1]+(i>0&&variant==2?20:0);ps.putShort(i*4,(short)x).putShort(i*4+2,(short)y);mp.putShort(((x+5)*90+y+5)*2,(short)(i+10));or.putShort((i+1)*2,(short)(i+10));}
                c.writeMemory(toAddr(0x439d3c),enemy);c.writeMemory(toAddr(0x446f0a),positions);c.writeMemory(toAddr(0x4085b8),map);c.writeMemory(toAddr(0x4470b8),order);c.writeMemoryValue(toAddr(0x43874a),2,9);c.writeMemoryValue(toAddr(0x408574),2,10);c.writeMemoryValue(toAddr(0x442780),4,5);c.writeMemoryValue(toAddr(0x442784),4,5);c.writeMemoryValue(toAddr(0x438776),2,script==6?0:1);
                c.writeMemory(toAddr(0x443086),regions);byte[] savedRegion=regions.clone();
                c.writeMemoryValue(toAddr(0x442764),4,43);c.writeMemoryValue(toAddr(0x442768),4,84);c.writeMemoryValue(toAddr(0x43959c),1,1);
                long sp=0x700000;c.writeMemory(toAddr(sp-4096),new byte[8192]);c.writeMemoryValue(toAddr(sp),4,0x900000);c.writeMemoryValue(toAddr(sp+4),4,43);c.writeMemoryValue(toAddr(sp+8),4,84);c.writeMemoryValue(toAddr(sp+12),2,1);c.writeMemoryValue(toAddr(sp+14),2,script);c.writeMemoryValue(toAddr(sp+16),2,0);c.writeRegister("SP",sp);c.writeRegister("PC",0x1400e4);
                SaneBoundary sane=new SaneBoundary(c,currentProgram);boolean stopped=false;int fileCursor=0;StringBuilder texts=new StringBuilder(),sounds=new StringBuilder();long seed=12,draws=0;
                for(int step=0;step<1000000;step++) {
                    long pc=c.getExecutionAddress().getOffset(),s=c.readRegister("SP").longValue();if(pc==0x900000){stopped=true;break;}
                    if(pc==0x123c60){loadScript(c,scripts,word(c,s+4));ret(c);continue;}
                    if(pc==0x1383e6){int id=word(c,s+4);c.writeMemory(toAddr(0x44330a),Arrays.copyOfRange(extra,id*10,id*10+10));ret(c);continue;}
                    if(pc==0x14d21a){c.writeRegister("D0",1);if(sounds.length()>0)sounds.append(',');sounds.append(20005);ret(c);continue;}
                    if(pc==0x121596){c.writeRegister("D0",0);ret(c);continue;}
                    if(pc==0x151e24){savedRegion=c.readMemory(toAddr(0x443086),644);ret(c);continue;}
                    if(pc==0x13107a){c.writeMemory(toAddr(0x443086),savedRegion);ret(c);continue;}
                    if(pc==0x15dc34){if(texts.length()>0)texts.append(',');texts.append(word(c,s+6));ret(c);continue;}
                    if(pc==0x133a18){if(sounds.length()>0)sounds.append(',');sounds.append(word(c,s+4));ret(c);continue;}
                    if(pc==0x4001a0){sane.extended(pointer(c,s+4),Math.sqrt(sane.extended(s+8)));ret(c);continue;}
                    if(pc==0x400038){c.writeRegister("D0",Math.abs(word(c,s+4)));ret(c);continue;}
                    if(pc==0x400070){c.writeRegister("A0",0x600000);ret(c);continue;}
                    if(pc==0x400078){fileCursor=(int)pointer(c,s+8);c.writeRegister("D0",0);ret(c);continue;}
                    if(pc==0x400048){int n=(int)pointer(c,s+8);c.writeMemory(toAddr(pointer(c,s+4)),Arrays.copyOfRange(menus,fileCursor,fileCursor+n));fileCursor+=n;c.writeRegister("D0",1);ret(c);continue;}
                    if(pc==0x400068){c.writeRegister("D0",0);ret(c);continue;}
                    if(pc==0x153ca4){int bound=word(c,s+4);seed=seed*16807%2147483647;draws++;short raw=(short)seed;if(raw==Short.MIN_VALUE)raw=0;c.writeRegister("D0",Math.abs((int)raw)*bound/32768+1);ret(c);continue;}
                    if(pc==0x20017c||pc==0x10f186||pc==0x133c56||pc==0x136d86||pc==0x13a2c4||pc==0x12382a||pc==0x160fb2||pc==0x158454||pc==0x1205f8||pc==0x105572||pc==0x1382a8||pc==0x1500d8||pc==0x152dc4){ret(c);continue;}
                    int op=word(c,pc)&65535;if(op==0xa032){c.writeRegister("PC",pc+2);continue;}if(op==0xa9eb){sane.execute();continue;}if(op==0xa873){c.writeRegister("SP",s+4);c.writeRegister("PC",pc+2);continue;}if(op==0xaa1e){c.writeMemoryValue(toAddr(s+2),4,0);c.writeRegister("SP",s+2);c.writeRegister("PC",pc+2);continue;}
                    if(!c.step(monitor))throw new Exception("script "+script+" PC "+Long.toHexString(pc)+": "+c.getLastError());
                }
                if(!stopped)throw new Exception("Story instruction budget");if(cases.length()>0)cases.append(',');
                cases.append("{\"script\":").append(script).append(",\"variant\":").append(variant).append(",\"enemies\":\"").append(hex.formatHex(enemy)).append("\",\"positions\":\"").append(hex.formatHex(positions)).append("\",\"map_before\":\"").append(hex.formatHex(map)).append("\",\"order_before\":\"").append(hex.formatHex(order)).append("\",\"after_enemies\":\"").append(hex.formatHex(c.readMemory(toAddr(0x439d3c),enemy.length))).append("\",\"map\":\"").append(hex.formatHex(c.readMemory(toAddr(0x4085b8),map.length))).append("\",\"order\":\"").append(hex.formatHex(c.readMemory(toAddr(0x4470b8),order.length))).append("\",\"regions\":\"").append(hex.formatHex(c.readMemory(toAddr(0x443086),644))).append("\",\"x\":").append(pointer(c,0x442764)).append(",\"y\":").append(pointer(c,0x442768)).append(",\"texts\":[").append(texts).append("],\"sounds\":[").append(sounds).append("],\"seed\":").append(seed).append(",\"draws\":").append(draws).append('}');
            }finally{c.dispose();}
        }
        Files.writeString(root.resolve("evidence/fixtures/opening-stories.json"),"{\"authority\":\"Original newland for death scripts3/4 and camp script6 through Yes choice, region change, teleport, and menu cancel. Authored file reads modeled; question returns one and menu returns zero; textbox/sound recorded and presentation intercepted; getclosest/destroymonster/bodyfield execute with bounded SANE model. No live whole-story claim.\",\"cases\":["+cases+"]}\n");println("Opening story cases exported");
    }
}
