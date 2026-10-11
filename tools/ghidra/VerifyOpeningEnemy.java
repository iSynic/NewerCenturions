import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.ByteBuffer;
import java.util.HexFormat;

public class VerifyOpeningEnemy extends GhidraScript {
    int word(EmulatorHelper c,long a){return ByteBuffer.wrap(c.readMemory(toAddr(a),2)).getShort();}
    long pointer(EmulatorHelper c,long a){return Integer.toUnsignedLong(ByteBuffer.wrap(c.readMemory(toAddr(a),4)).getInt());}
    void ret(EmulatorHelper c){long s=c.readRegister("SP").longValue();c.writeRegister("PC",pointer(c,s));c.writeRegister("SP",s+4);}
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]),snapshot=root.resolve("analysis/oracle/opening");HexFormat hex=HexFormat.of();
        String[] files={"warriors","enemies","map","squad","positions","order"};long[] bases={0x448aa0,0x439d3c,0x4085b8,0x44709a,0x446f0a,0x4470b8};
        byte[] properties=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data Swamp BD"));
        EmulatorHelper c=new EmulatorHelper(currentProgram);
        try {
            for(String r:new String[]{"D0","D1","D2","D3","D4","D5","D6","D7","A0","A1","A2","A3","A4","A5","A6","XF","NF","ZF","VF","CF"})c.writeRegister(r,0);
            c.writeMemory(toAddr(0x439d3c),new byte[28800]);c.writeMemory(toAddr(0x446f0a),new byte[400]);
            StringBuilder fields=new StringBuilder();
            for(int i=0;i<files.length;i++){byte[] data=Files.readAllBytes(snapshot.resolve(files[i]+".bin"));c.writeMemory(toAddr(bases[i]),data);fields.append(",\"").append(files[i]).append("\":\"").append(hex.formatHex(data)).append('"');}
            c.writeMemory(toAddr(0x43167c),java.util.Arrays.copyOfRange(properties,64,12064));
            c.writeMemoryValue(toAddr(0x447196),2,3);c.writeMemoryValue(toAddr(0x447194),2,1);c.writeMemoryValue(toAddr(0x4435c4),2,6);c.writeMemoryValue(toAddr(0x43874a),2,6);
            c.writeMemoryValue(toAddr(0x442780),4,-25);c.writeMemoryValue(toAddr(0x442784),4,-31);c.writeMemoryValue(toAddr(0x44273a),1,5);c.writeMemoryValue(toAddr(0x402f43),1,2);c.writeMemoryValue(toAddr(0x43550c),2,155);c.writeMemoryValue(toAddr(0x439568),2,0);c.writeMemory(toAddr(0x442bd6),new byte[1200]);
            long frame=0x701000,sp=0x700000;c.writeMemory(toAddr(frame-1024),new byte[2048]);c.writeMemory(toAddr(sp-1024),new byte[2048]);c.writeRegister("A6",frame);c.writeMemoryValue(toAddr(sp),4,0x900000);c.writeRegister("SP",sp);c.writeRegister("PC",0x1145c8);
            long seed=685250556,draws=0;boolean stopped=false;int selected=-1;SaneBoundary sane=new SaneBoundary(c,currentProgram);
            for(int step=0;step<1500000;step++){
                long pc=c.getExecutionAddress().getOffset(),s=c.readRegister("SP").longValue();
                if(pc==0x115408){int target=word(c,0x439d3c+3*288+0x10c);if(selected<0)selected=target;if(target>=16){stopped=true;break;}}
                if(pc==0x115732)throw new Exception("Expected unsafe target search did not occur");
                if(pc==0x400038){c.writeRegister("D0",Math.abs(word(c,s+4)));ret(c);continue;}
                if(pc==0x153ca4){int bound=word(c,s+4);seed=seed*16807%2147483647;draws++;short raw=(short)seed;if(raw==Short.MIN_VALUE)raw=0;c.writeRegister("D0",Math.abs((int)raw)*bound/32768+1);ret(c);continue;}
                if(pc==0x133a18||pc==0x120c14||pc==0x105572||pc==0x10e6b6||pc==0x133d46||pc==0x158454||pc==0x13154c||pc==0x11f1a8||pc==0x11edc4){ret(c);continue;}
                int op=word(c,pc)&65535;
                if(op==0xa873){c.writeRegister("SP",s+4);c.writeRegister("PC",pc+2);continue;}
                if(op==0xa9b4){c.writeRegister("PC",pc+2);continue;}
                if(op==0xa974){c.writeRegister("SP",s-2);c.writeMemoryValue(toAddr(s-2),2,0);c.writeRegister("PC",pc+2);continue;}
                if(op==0xa9eb){sane.execute();continue;}
                if(!c.step(monitor))throw new Exception("PC "+Long.toHexString(pc)+": "+c.getLastError());
            }
            if(!stopped)throw new Exception("Opening enemy prefix instruction budget");
            int target=word(c,0x439d3c+3*288+0x10c),scan=word(c,frame-4);
            Files.writeString(root.resolve("evidence/fixtures/opening-enemy-prefix.json"),"{\"authority\":\"Original combat decision branch on the composed Rust opening snapshot; stopped before reading target coordinates outside the active body arrays; presentation intercepted; SANE modeled\",\"initial_seed\":685250556,\"active_body\":13,\"cursor\":1,\"shift\":[-30,-36],\"enemy_count\":6,\"selected_target\":"+selected+",\"unsafe_target\":"+target+",\"scan\":"+scan+",\"seed\":"+seed+",\"draws\":"+draws+",\"properties\":\""+hex.formatHex(properties)+"\""+fields+"}\n");
            println("Unsafe target "+target+" scan "+scan+" after "+draws+" draws; original selected target "+selected);
        }finally{c.dispose();}
    }
}
