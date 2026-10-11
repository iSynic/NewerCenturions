import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.ByteBuffer;
import java.util.HexFormat;

public class VerifyRegional extends GhidraScript {
    int word(EmulatorHelper cpu,long a){return ByteBuffer.wrap(cpu.readMemory(toAddr(a),2)).getShort();}
    long pointer(EmulatorHelper cpu,long a){return Integer.toUnsignedLong(ByteBuffer.wrap(cpu.readMemory(toAddr(a),4)).getInt());}
    void ret(EmulatorHelper cpu){long sp=cpu.readRegister("SP").longValue();cpu.writeRegister("PC",pointer(cpu,sp));cpu.writeRegister("SP",sp+4);}
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]);
        byte[] regions=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data RD"));
        StringBuilder cases=new StringBuilder();int encounters=0;
        for(int initial=1;initial<=256;initial++) {
            EmulatorHelper cpu=new EmulatorHelper(currentProgram);
            try {
                for(String reg:new String[]{"D0","D1","D2","D3","D4","D5","D6","D7","A0","A1","A2","A3","A4","A5","A6","XF","NF","ZF","VF","CF"})cpu.writeRegister(reg,0);
                long frame=0x701000;
                cpu.writeMemory(toAddr(frame-512),new byte[1024]);cpu.writeMemory(toAddr(0x443086),java.util.Arrays.copyOf(regions,644));
                cpu.writeRegister("PC",0x15e906);cpu.writeRegister("SP",0x700000);cpu.writeRegister("A6",frame);
                cpu.writeMemoryValue(toAddr(frame-2),2,5);cpu.writeMemoryValue(toAddr(frame-4),2,3);cpu.writeMemoryValue(toAddr(frame+10),2,1);
                cpu.writeMemoryValue(toAddr(0x442570),2,0);cpu.writeMemoryValue(toAddr(0x402f42),1,1);
                long seed=initial,draws=0;int battle=0,advantage=0;boolean stopped=false;
                for(int step=0;step<20000;step++) {
                    long pc=cpu.getExecutionAddress().getOffset(),sp=cpu.readRegister("SP").longValue();
                    if(pc==0x15ec40||pc==0x15ec42){stopped=true;break;}
                    if(pc==0x114378){battle=word(cpu,0x43874e);advantage=word(cpu,sp+4);encounters++;stopped=true;break;}
                    if(pc==0x400038){cpu.writeRegister("D0",Math.abs(word(cpu,sp+4)));ret(cpu);continue;}
                    if(pc==0x153ca4){int bound=word(cpu,sp+4);seed=seed*16807%2147483647;draws++;short raw=(short)seed;if(raw==Short.MIN_VALUE)raw=0;cpu.writeRegister("D0",Math.abs((int)raw)*bound/32768+1);ret(cpu);continue;}
                    if(!cpu.step(monitor))throw new Exception("PC "+Long.toHexString(pc)+": "+cpu.getLastError());
                }
                if(!stopped)throw new Exception("Regional instruction budget");
                if(cases.length()>0)cases.append(',');cases.append("{\"seed\":").append(initial).append(",\"after_seed\":").append(seed).append(",\"draws\":").append(draws).append(",\"battle\":").append(battle).append(",\"advantage\":").append(advantage).append('}');
            } finally {cpu.dispose();}
        }
        Files.writeString(root.resolve("evidence/fixtures/regional.json"),"{\"authority\":\"timeclick regional dispatch instructions; Rand supplied by separately verified QuickDraw model; stop before combat\",\"regions\":\""+HexFormat.of().formatHex(regions)+"\",\"cases\":["+cases+"]}\n");
        println("256 regional cases exported; "+encounters+" encounters");
    }
}
