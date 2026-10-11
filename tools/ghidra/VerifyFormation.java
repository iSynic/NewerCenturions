import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.ByteBuffer;
import java.util.HexFormat;
import java.util.regex.Pattern;

public class VerifyFormation extends GhidraScript {
    int word(EmulatorHelper cpu,long a){return ByteBuffer.wrap(cpu.readMemory(toAddr(a),2)).getShort();}
    long pointer(EmulatorHelper cpu,long a){return Integer.toUnsignedLong(ByteBuffer.wrap(cpu.readMemory(toAddr(a),4)).getInt());}
    void ret(EmulatorHelper cpu){long sp=cpu.readRegister("SP").longValue();cpu.writeRegister("PC",pointer(cpu,sp));cpu.writeRegister("SP",sp+4);}
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]);HexFormat hex=HexFormat.of();
        var matcher=Pattern.compile("\\\"tiles\\\"\\s*:\\s*\\\"([0-9a-f]+)\\\"").matcher(Files.readString(root.resolve("evidence/fixtures/combat-map.json")));
        if(!matcher.find())throw new Exception("Map fixture missing");byte[] map=hex.parseHex(matcher.group(1));
        byte[] properties=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data Swamp BD"));
        byte[] items=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data ID"));
        byte[] initial=new byte[6*3138];String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};
        for(int i=0;i<6;i++)System.arraycopy(Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Warriors/"+names[i])),0,initial,i*3138,3138);
        int start=getScriptArgs().length>1?14:17; String suffix=getScriptArgs().length>1?"-opening":"";
        StringBuilder cases=new StringBuilder();
        for(int direction:new int[]{1,45,90,135,180,225,270,315,360}) {
            EmulatorHelper cpu=new EmulatorHelper(currentProgram);
            try {
                for(String reg:new String[]{"D0","D1","D2","D3","D4","D5","D6","D7","A0","A1","A2","A3","A4","A5","A6","XF","NF","ZF","VF","CF"})cpu.writeRegister(reg,0);
                cpu.writeMemory(toAddr(0x4085b8),map);cpu.writeMemory(toAddr(0x448aa0),initial);
                cpu.writeMemory(toAddr(0x43167c),java.util.Arrays.copyOfRange(properties,64,12064));
                cpu.writeMemory(toAddr(0x435664),new byte[]{12,14,12,14,10,12,10,10,14,16,0,12});
                int[] bases={0x4537fe,0x46225e,0x45ad2e,0x46978e};
                for(int band=0;band<4;band++)cpu.writeMemory(toAddr(bases[band]),java.util.Arrays.copyOfRange(items,band*30000,(band+1)*30000));
                cpu.writeMemoryValue(toAddr(0x442780),4,-34);cpu.writeMemoryValue(toAddr(0x442784),4,-10);
                cpu.writeMemoryValue(toAddr(0x439ce8),2,start);cpu.writeMemoryValue(toAddr(0x439cea),2,start==14?14:13);cpu.writeMemoryValue(toAddr(0x44273a),1,5);
                long frame=0x701000;cpu.writeRegister("PC",0x11d8de);cpu.writeRegister("SP",0x700000);cpu.writeRegister("A6",frame);
                cpu.writeMemoryValue(toAddr(frame-8),2,5);cpu.writeMemoryValue(toAddr(frame-6),2,5);cpu.writeMemoryValue(toAddr(frame-4),2,85);cpu.writeMemoryValue(toAddr(frame-2),2,85);
                boolean vertical=direction<45||direction>315||(direction>135&&direction<225);
                cpu.writeMemoryValue(toAddr(frame-0x127),1,vertical?1:0);cpu.writeMemoryValue(toAddr(frame-0x121),1,direction>90&&direction<270?1:-1);cpu.writeRegister("D4",direction<180?-1:1);
                SaneBoundary sane=new SaneBoundary(cpu,currentProgram);boolean stopped=false;
                for(int step=0;step<200000;step++) {
                    long pc=cpu.getExecutionAddress().getOffset();
                    if(pc==0x11dd2e){stopped=true;break;}
                    if(pc==0x400038){long sp=cpu.readRegister("SP").longValue();cpu.writeRegister("D0",Math.abs(word(cpu,sp+4)));ret(cpu);continue;}
                    int opcode=word(cpu,pc)&0xffff;
                    if(opcode==0xa9eb){sane.execute();continue;}
                    if(opcode==0xa8ad){long sp=cpu.readRegister("SP").longValue(),r=pointer(cpu,sp);int y=word(cpu,sp+4),x=word(cpu,sp+6);boolean inside=y>=word(cpu,r)&&x>=word(cpu,r+2)&&y<word(cpu,r+4)&&x<word(cpu,r+6);cpu.writeMemoryValue(toAddr(sp+8),1,inside?1:0);cpu.writeRegister("SP",sp+8);cpu.writeRegister("PC",pc+2);continue;}
                    if(!cpu.step(monitor))throw new Exception("PC "+Long.toHexString(pc)+": "+cpu.getLastError());
                }
                if(!stopped)throw new Exception("Formation instruction budget");
                if(cases.length()>0)cases.append(',');cases.append("{\"direction\":").append(direction).append(",\"warriors\":\"").append(hex.formatHex(cpu.readMemory(toAddr(0x448aa0),initial.length))).append("\",\"tiles\":\"").append(hex.formatHex(cpu.readMemory(toAddr(0x4085b8),16200))).append("\",\"positions\":\"").append(hex.formatHex(cpu.readMemory(toAddr(0x44709a),24))).append("\",\"under\":\"").append(hex.formatHex(cpu.readMemory(toAddr(0x437a6e),12))).append("\"}");
            }finally{cpu.dispose();}
        }
        Files.writeString(root.resolve("evidence/fixtures/formation"+suffix+".json"),"{\"authority\":\"combatsetup squad instruction slice, integer abs and PtInRect boundaries, bounded SANE movement model\",\"input\":\""+hex.formatHex(initial)+"\",\"items\":\""+hex.formatHex(items)+"\",\"map\":\""+hex.formatHex(map)+"\",\"cases\":["+cases+"]}\n");
        println("Nine squad formation cases exported");
    }
}
