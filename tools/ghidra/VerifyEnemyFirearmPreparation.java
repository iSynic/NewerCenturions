import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.ByteBuffer;
import java.util.HexFormat;

public class VerifyEnemyFirearmPreparation extends GhidraScript {
    int word(EmulatorHelper c,long a){return ByteBuffer.wrap(c.readMemory(toAddr(a),2)).getShort();}
    long pointer(EmulatorHelper c,long a){return Integer.toUnsignedLong(ByteBuffer.wrap(c.readMemory(toAddr(a),4)).getInt());}
    void ret(EmulatorHelper c){long s=c.readRegister("SP").longValue();c.writeRegister("PC",pointer(c,s));c.writeRegister("SP",s+4);}
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]);HexFormat hex=HexFormat.of();StringBuilder cases=new StringBuilder();
        byte[] items=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data ID"));
        byte[] attacks=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data S"));
        for(int body:new int[]{10,11,12,13,14,15,16,17,18})for(int weapon:new int[]{680,681}) {
            EmulatorHelper c=new EmulatorHelper(currentProgram);
            try {
                for(String r:new String[]{"D0","D1","D2","D3","D4","D5","D6","D7","A0","A1","A2","A3","A4","A5","A6","XF","NF","ZF","VF","CF"})c.writeRegister(r,0);
                c.writeMemory(toAddr(0x448aa0),new byte[3138*6]);
                c.writeMemory(toAddr(0x4537fe),java.util.Arrays.copyOfRange(items,0,30000));
                c.writeMemory(toAddr(0x46225e),java.util.Arrays.copyOfRange(items,30000,60000));
                c.writeMemory(toAddr(0x46978e),java.util.Arrays.copyOfRange(items,90000,120000));
                c.writeMemory(toAddr(0x443788),attacks);
                c.writeMemoryValue(toAddr(0x447196),2,body-10);c.writeMemoryValue(toAddr(0x447194),2,1);c.writeMemoryValue(toAddr(0x4470ba),2,body);
                c.writeMemoryValue(toAddr(0x439d3c+(body-10)*288+0x26),2,weapon);
                c.writeMemoryValue(toAddr(0x439d3c+(body-10)*288+0x4c),2,0);
                long sp=0x700000,frame=0x701000;c.writeMemory(toAddr(sp-8192),new byte[16384]);c.writeMemoryValue(toAddr(sp),4,0x900000);
                c.writeRegister("A6",frame);c.writeRegister("SP",sp);c.writeRegister("PC",0x1146ee);boolean stopped=false;SaneBoundary sane=new SaneBoundary(c,currentProgram);
                for(int step=0;step<100000;step++){
                    long pc=c.getExecutionAddress().getOffset();
                    if(pc==0x114b78){stopped=true;break;}
                    if(pc==0x400038){c.writeRegister("D0",Math.abs(word(c,c.readRegister("SP").longValue()+4)));ret(c);continue;}
                    if(pc==0x13154c){ret(c);continue;}
                    if((word(c,pc)&0xffff)==0xa9eb){sane.execute();continue;}
                    if(!c.step(monitor))throw new Exception("body "+body+" PC "+Long.toHexString(pc)+": "+c.getLastError());
                }
                if(!stopped)throw new Exception("Enemy preparation instruction budget");
                String packet=hex.formatHex(c.readMemory(toAddr(0x44374e),58));println(body+" weapon "+weapon+" packet "+packet);
                if(cases.length()>0)cases.append(',');cases.append("{\"body\":").append(body).append(",\"weapon_id\":").append(weapon).append(",\"packet\":\"").append(packet).append("\",\"weapon\":\"").append(hex.formatHex(c.readMemory(toAddr(0x453768),150))).append("\",\"ammo\":\"").append(hex.formatHex(c.readMemory(toAddr(0x4536d2),150))).append("\"}");
            }finally{c.dispose();}
        }
        Files.writeString(root.resolve("evidence/fixtures/enemy-firearm-preparation.json"),"{\"authority\":\"Original combat ranged preparation from 1146ee through 114b78, including loadweapon, loadammo, prepattack and loadspell2; weapons 680 and 681, bodies 10 through 18, D3 zero; item tables populated; presentation intercepted and SANE modeled\",\"cases\":["+cases+"]}\n");
    }
}
