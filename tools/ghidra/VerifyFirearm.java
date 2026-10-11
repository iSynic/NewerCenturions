import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.ByteBuffer;
import java.util.HexFormat;

public class VerifyFirearm extends GhidraScript {
    int word(EmulatorHelper c,long a){return ByteBuffer.wrap(c.readMemory(toAddr(a),2)).getShort();}
    long pointer(EmulatorHelper c,long a){return Integer.toUnsignedLong(ByteBuffer.wrap(c.readMemory(toAddr(a),4)).getInt());}
    void ret(EmulatorHelper c){long s=c.readRegister("SP").longValue();c.writeRegister("PC",pointer(c,s));c.writeRegister("SP",s+4);}
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]);HexFormat hex=HexFormat.of();StringBuilder cases=new StringBuilder();
        byte[] items=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data ID"));
        byte[] attacks=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data S"));
        for(String name:new String[]{"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"}) {
            EmulatorHelper c=new EmulatorHelper(currentProgram);
            try {
                for(String r:new String[]{"D0","D1","D2","D3","D4","D5","D6","D7","A0","A1","A2","A3","A4","A5","A6","XF","NF","ZF","VF","CF"})c.writeRegister(r,0);
                byte[] input=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Warriors/"+name));input[0x9df]=1;input[0x9dd]=1;
                c.writeMemory(toAddr(0x448aa0),input);c.writeMemory(toAddr(0x4537fe),java.util.Arrays.copyOfRange(items,0,30000));c.writeMemory(toAddr(0x46225e),java.util.Arrays.copyOfRange(items,30000,60000));c.writeMemory(toAddr(0x443788),attacks);
                c.writeMemoryValue(toAddr(0x4435d2),2,0);
                long sp=0x700000;c.writeMemory(toAddr(sp-8192),new byte[16384]);c.writeMemoryValue(toAddr(sp),4,0x900000);c.writeMemoryValue(toAddr(sp+4),2,0);c.writeMemoryValue(toAddr(sp+6),2,0);
                c.writeRegister("SP",sp);c.writeRegister("PC",0x14c6c2);boolean stopped=false;SaneBoundary sane=new SaneBoundary(c,currentProgram);
                for(int step=0;step<100000;step++){
                    long pc=c.getExecutionAddress().getOffset(),s=c.readRegister("SP").longValue();
                    if(pc==0x900000){stopped=true;break;}
                    if(pc==0x400038){c.writeRegister("D0",Math.abs(word(c,s+4)));ret(c);continue;}
                    if(pc==0x13154c){ret(c);continue;}
                    if((word(c,pc)&0xffff)==0xa9eb){sane.execute();continue;}
                    if(!c.step(monitor))throw new Exception(name+" PC "+Long.toHexString(pc)+": "+c.getLastError());
                }
                if(!stopped)throw new Exception("Firearm instruction budget");
                String packet=hex.formatHex(c.readMemory(toAddr(0x44374e),58));println(name+" packet "+packet);
                if(cases.length()>0)cases.append(',');cases.append("{\"name\":\"").append(name).append("\",\"input\":\"").append(hex.formatHex(input)).append("\",\"packet\":\"").append(packet).append("\",\"weapon\":\"").append(hex.formatHex(c.readMemory(toAddr(0x453768),150))).append("\",\"ammo\":\"").append(hex.formatHex(c.readMemory(toAddr(0x4536d2),150))).append("\"}");
            }finally{c.dispose();}
        }
        Files.writeString(root.resolve("evidence/fixtures/firearm-preparation.json"),"{\"authority\":\"Complete original prepattack, loadweapon, loadammo and loadspell2; starter secondary weapon1 and ammo200; locationlog intercepted; SANE modeled\",\"cases\":["+cases+"]}\n");
    }
}
