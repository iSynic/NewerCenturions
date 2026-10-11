import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.ByteBuffer;
import java.util.HexFormat;

public class VerifyAmmoDrop extends GhidraScript {
    int word(EmulatorHelper c,long a){return ByteBuffer.wrap(c.readMemory(toAddr(a),2)).getShort();}
    long pointer(EmulatorHelper c,long a){return Integer.toUnsignedLong(ByteBuffer.wrap(c.readMemory(toAddr(a),4)).getInt());}
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]);HexFormat hex=HexFormat.of();StringBuilder cases=new StringBuilder();
        byte[] table=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data ID"));
        byte[] races=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data Race"));
        for(String name:new String[]{"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"}) {
            EmulatorHelper c=new EmulatorHelper(currentProgram);
            try {
                for(String r:new String[]{"D0","D1","D2","D3","D4","D5","D6","D7","A0","A1","A2","A3","A4","A5","A6","XF","NF","ZF","VF","CF"})c.writeRegister(r,0);
                byte[] input=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Warriors/"+name));
                System.arraycopy(input,0x132+22,input,0x132+44,44);
                java.util.Arrays.fill(input,0x132+22,0x132+44,(byte)0);
                ByteBuffer.wrap(input).putShort(0x1a,(short)4).putShort(0x132+22,(short)200).putShort(0x134+22,(short)32);input[0x13c+22]=26;
                c.writeMemory(toAddr(0x448aa0),input);c.writeMemory(toAddr(0x46225e),java.util.Arrays.copyOfRange(table,30000,60000));c.writeMemory(toAddr(0x41fc32),new byte[22]);
                for(int i=0;i<10;i++)c.writeMemoryValue(toAddr(0x435664+i),1,races[i*408+197]);
                long sp=0x700000;c.writeMemory(toAddr(sp-8192),new byte[16384]);c.writeMemoryValue(toAddr(sp),4,0x900000);
                c.writeMemoryValue(toAddr(sp+4),2,0);c.writeMemoryValue(toAddr(sp+6),2,200);c.writeMemoryValue(toAddr(sp+8),2,1);c.writeMemoryValue(toAddr(sp+10),2,0);
                c.writeRegister("SP",sp);c.writeRegister("PC",0x121152);SaneBoundary sane=new SaneBoundary(c,currentProgram);boolean stopped=false;
                for(int step=0;step<100000;step++){
                    long pc=c.getExecutionAddress().getOffset();if(pc==0x900000){stopped=true;break;}
                    if((word(c,pc)&0xffff)==0xa9eb){sane.execute();continue;}
                    if(!c.step(monitor))throw new Exception(name+" PC "+Long.toHexString(pc)+": "+c.getLastError());
                }
                if(!stopped)throw new Exception("Drop instruction budget");
                if(cases.length()>0)cases.append(',');cases.append("{\"name\":\"").append(name).append("\",\"input\":\"").append(hex.formatHex(input)).append("\",\"after\":\"").append(hex.formatHex(c.readMemory(toAddr(0x448aa0),3138))).append("\"}");
            }finally{c.dispose();}
        }
        Files.writeString(root.resolve("evidence/fixtures/ammunition-drop.json"),"{\"authority\":\"Complete original dropitem and movecalc for an unequipped zero-weight starter ammunition pack in slot 1; SANE boundary modeled; empty inventory template zeroed\",\"cases\":["+cases+"]}\n");
    }
}
