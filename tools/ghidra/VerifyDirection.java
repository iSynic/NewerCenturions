import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.ByteBuffer;

public class VerifyDirection extends GhidraScript {
    int word(EmulatorHelper cpu,long a){return ByteBuffer.wrap(cpu.readMemory(toAddr(a),2)).getShort();}
    long pointer(EmulatorHelper cpu,long a){return Integer.toUnsignedLong(ByteBuffer.wrap(cpu.readMemory(toAddr(a),4)).getInt());}
    void ret(EmulatorHelper cpu){long sp=cpu.readRegister("SP").longValue();cpu.writeRegister("PC",pointer(cpu,sp));cpu.writeRegister("SP",sp+4);}
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]);StringBuilder cases=new StringBuilder();
        for(int direction=1;direction<=360;direction++) {
            EmulatorHelper cpu=new EmulatorHelper(currentProgram);
            try {
                for(String reg:new String[]{"D0","D1","D2","D3","D4","D5","D6","D7","A0","A1","A2","A3","A4","A5","A6","XF","NF","ZF","VF","CF"})cpu.writeRegister(reg,0);
                long frame=0x701000;cpu.writeRegister("PC",0x11d5ba);cpu.writeRegister("SP",0x700000);cpu.writeRegister("A6",frame);
                cpu.writeMemoryValue(toAddr(0x439cf6),1,0);
                SaneBoundary sane=new SaneBoundary(cpu,currentProgram);boolean stopped=false;int draws=0;
                for(int step=0;step<5000;step++) {
                    long pc=cpu.getExecutionAddress().getOffset(),sp=cpu.readRegister("SP").longValue();
                    if(pc==0x11d8de){stopped=true;break;}
                    if(pc==0x153ca4){int bound=word(cpu,sp+4);if(bound!=(draws==0?360:0))throw new Exception("Direction random boundary");cpu.writeRegister("D0",draws++==0?direction:1);ret(cpu);continue;}
                    if(pc==0x400190||pc==0x400198){sane.extended(pointer(cpu,sp+4),pc==0x400190?Math.cos(sane.extended(sp+8)):Math.sin(sane.extended(sp+8)));ret(cpu);continue;}
                    if((word(cpu,pc)&0xffff)==0xa9eb){sane.execute();continue;}
                    if(!cpu.step(monitor))throw new Exception("PC "+Long.toHexString(pc)+": "+cpu.getLastError());
                }
                if(!stopped||draws!=2)throw new Exception("Direction instruction budget");
                if(cases.length()>0)cases.append(',');cases.append("{\"direction\":").append(direction).append(",\"x\":").append(cpu.readRegister("D5").shortValue()).append(",\"y\":").append(cpu.readRegister("D6").shortValue()).append(",\"vertical\":").append(word(cpu,frame-0x127)>>8!=0).append(",\"ud\":").append(cpu.readRegister("D4").shortValue()).append(",\"lr\":").append((byte)(word(cpu,frame-0x121)>>8)).append('}');
            }finally{cpu.dispose();}
        }
        Files.writeString(root.resolve("evidence/fixtures/direction.json"),"{\"authority\":\"combatsetup direction instruction slice, supplied Rand outcomes, bounded SANE and trigonometric models with zero authored distance\",\"cases\":["+cases+"]}\n");
        println("360 direction cases exported");
    }
}
