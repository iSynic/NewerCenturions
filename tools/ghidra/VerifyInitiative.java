import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.ByteBuffer;
import java.util.Arrays;

public class VerifyInitiative extends GhidraScript {
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]);
        int[] warriors={12,10,9,8,8,9},enemies={8,7,6,5,9,7,9,5,8};
        StringBuilder cases=new StringBuilder();
        for(long initial:new long[]{1,2,2147483646})for(int mode:new int[]{-1,0,1}) {
            EmulatorHelper cpu=new EmulatorHelper(currentProgram);
            try {
                for(String reg:new String[]{"D0","D1","D2","D3","D4","D5","D6","D7","A0","A1","A2","A3","A4","A5","A6","XF","NF","ZF","VF","CF"})cpu.writeRegister(reg,0);
                cpu.writeRegister("PC",0x124d9e);cpu.writeRegister("SP",0x700000);
                cpu.writeMemoryValue(toAddr(0x700000),4,0x7ffff0);cpu.writeMemoryValue(toAddr(0x700004),2,mode);
                cpu.writeMemoryValue(toAddr(0x442739),1,0);cpu.writeMemoryValue(toAddr(0x44273a),1,5);cpu.writeMemoryValue(toAddr(0x43874a),2,9);
                for(int index=0;index<6;index++) {
                    cpu.writeMemoryValue(toAddr(0x448aa0+index*3138+0x50),2,30);
                    cpu.writeMemoryValue(toAddr(0x448aa0+index*3138+0x90),2,0);
                    cpu.writeMemoryValue(toAddr(0x448aa0+index*3138+0x9e8),2,warriors[index]);
                }
                for(int index=0;index<9;index++)cpu.writeMemoryValue(toAddr(0x439d3c+index*288+4),2,enemies[index]);
                long seed=initial,draws=0;boolean returned=false;
                for(int step=0;step<100000;step++) {
                    long pc=cpu.getExecutionAddress().getOffset();
                    if(pc==0x7ffff0){returned=true;break;}
                    if(pc==0x153ca4) {
                        long sp=cpu.readRegister("SP").longValue();int bound=ByteBuffer.wrap(cpu.readMemory(toAddr(sp+4),2)).getShort();
                        seed=seed*16807%2147483647;draws++;short raw=(short)seed;if(raw==Short.MIN_VALUE)raw=0;
                        cpu.writeRegister("D0",(Math.abs((int)raw)*bound/32768)+1);
                        cpu.writeRegister("PC",Integer.toUnsignedLong(ByteBuffer.wrap(cpu.readMemory(toAddr(sp),4)).getInt()));cpu.writeRegister("SP",sp+4);
                    }else if(!cpu.step(monitor))throw new Exception(cpu.getLastError());
                }
                if(!returned)throw new Exception("Initiative instruction budget");
                int[] order=new int[110];for(int index=0;index<110;index++)order[index]=ByteBuffer.wrap(cpu.readMemory(toAddr(0x4470b8+index*2),2)).getShort();
                if(cases.length()>0)cases.append(',');cases.append("{\"seed\":").append(initial).append(",\"mode\":").append(mode).append(",\"after_seed\":").append(seed).append(",\"draws\":").append(draws).append(",\"order\":").append(Arrays.toString(order)).append('}');
            }finally{cpu.dispose();}
        }
        Files.writeString(root.resolve("evidence/fixtures/initiative.json"),"{\"authority\":\"Centurions 1.3.2 getbattleorder instruction execution; verified Rand boundary intercepted\",\"warrior_speeds\":"+Arrays.toString(warriors)+",\"enemy_speeds\":"+Arrays.toString(enemies)+",\"cases\":["+cases+"]}\n");
        println("Nine initiative instruction cases exported");
    }
}
