import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.ByteBuffer;
import java.util.HexFormat;

public class VerifyCombatMap extends GhidraScript {
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]);
        byte[] land=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data LD"));
        byte[] properties=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data Swamp BD"));
        StringBuilder cases=new StringBuilder();
        for(int[] position:new int[][]{{2,10},{45,45},{88,87},{10,2}})for(long initial:new long[]{1,2147483646}) {
            EmulatorHelper cpu=new EmulatorHelper(currentProgram);
            try {
                for(String reg:new String[]{"D0","D1","D2","D3","D4","D5","D6","D7","A0","A1","A2","A3","A4","A5","A6","XF","NF","ZF","VF","CF"})cpu.writeRegister(reg,0);
                cpu.writeMemory(toAddr(0x4085b8),java.util.Arrays.copyOfRange(land,0,16200));
                cpu.writeMemory(toAddr(0x414390),java.util.Arrays.copyOfRange(land,32400,64800));
                cpu.writeMemory(toAddr(0x43167c),java.util.Arrays.copyOfRange(properties,64,12064));
                cpu.writeMemoryValue(toAddr(0x410446),2,0);
                cpu.writeMemoryValue(toAddr(0x402f43),1,2);
                cpu.writeMemoryValue(toAddr(0x43550c),2,ByteBuffer.wrap(properties).getShort(0));
                cpu.writeMemoryValue(toAddr(0x435534),2,ByteBuffer.wrap(properties).getShort(2));
                cpu.writeMemoryValue(toAddr(0x442778),4,0);cpu.writeMemoryValue(toAddr(0x44277c),4,0);
                cpu.writeMemoryValue(toAddr(0x442764),4,position[0]);cpu.writeMemoryValue(toAddr(0x442768),4,position[1]);
                cpu.writeRegister("PC",0x11ce8e);cpu.writeRegister("SP",0x700000);cpu.writeMemoryValue(toAddr(0x700000),4,0x7ffff0);
                long seed=initial,draws=0;boolean returned=false;
                for(int step=0;step<4000000;step++) {
                    long pc=cpu.getExecutionAddress().getOffset();
                    if(pc==0x7ffff0){returned=true;break;}
                    if(pc==0x153ca4) {
                        long sp=cpu.readRegister("SP").longValue();int bound=ByteBuffer.wrap(cpu.readMemory(toAddr(sp+4),2)).getShort();
                        seed=seed*16807%2147483647;draws++;short raw=(short)seed;if(raw==Short.MIN_VALUE)raw=0;
                        cpu.writeRegister("D0",Math.abs((int)raw)*bound/32768+1);
                        cpu.writeRegister("PC",Integer.toUnsignedLong(ByteBuffer.wrap(cpu.readMemory(toAddr(sp),4)).getInt()));cpu.writeRegister("SP",sp+4);
                    }else if(!cpu.step(monitor))throw new Exception("PC "+Long.toHexString(pc)+": "+cpu.getLastError());
                }
                if(!returned)throw new Exception("Map instruction budget");
                if(cases.length()>0)cases.append(',');
                cases.append("{\"x\":").append(position[0]).append(",\"y\":").append(position[1]).append(",\"seed\":").append(initial).append(",\"after_seed\":").append(seed).append(",\"draws\":").append(draws).append(",\"edge_after\":").append(ByteBuffer.wrap(cpu.readMemory(toAddr(0x410446),2)).getShort()).append(",\"shift_x\":").append(ByteBuffer.wrap(cpu.readMemory(toAddr(0x4427a8),2)).getShort()).append(",\"shift_y\":").append(ByteBuffer.wrap(cpu.readMemory(toAddr(0x4427aa),2)).getShort()).append(",\"tiles\":\"").append(HexFormat.of().formatHex(cpu.readMemory(toAddr(0x4085b8),16200))).append("\"}");
            }finally{cpu.dispose();}
        }
        Files.writeString(root.resolve("evidence/fixtures/combat-map.json"),"{\"authority\":\"Centurions 1.3.2 combatmap instructions; Rand intercepted; explicit preceding word zero\",\"land\":\""+HexFormat.of().formatHex(land)+"\",\"properties\":\""+HexFormat.of().formatHex(properties)+"\",\"cases\":["+cases+"]}\n");
        println("Six tactical map instruction cases exported");
    }
}
