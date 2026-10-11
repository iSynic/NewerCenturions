import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.ByteBuffer;
import java.util.HexFormat;

public class VerifyCombatMovement extends GhidraScript {
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]);
        String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};
        byte[] raceMoves={12,14,12,14,10,12,10,10,14,16,0,12};
        StringBuilder cases=new StringBuilder();
        for(String name:names)for(int variant=0;variant<4;variant++) {
            EmulatorHelper cpu=new EmulatorHelper(currentProgram);
            try {
                byte[] input=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Warriors/"+name));
                ByteBuffer bytes=ByteBuffer.wrap(input);
                if(variant==1)bytes.putShort(0x62,(short)3);
                if(variant==2)bytes.putShort(0x6a,(short)1).putShort(0x8c,(short)1);
                if(variant==3)bytes.putShort(0x9d6,(short)12000).putShort(0x30,(short)-10);
                for(String reg:new String[]{"D0","D1","D2","D3","D4","D5","D6","D7","A0","A1","A2","A3","A4","A5","A6","XF","NF","ZF","VF","CF"})cpu.writeRegister(reg,0);
                cpu.writeMemory(toAddr(0x448aa0),input);cpu.writeMemory(toAddr(0x435664),raceMoves);
                cpu.writeRegister("PC",0x136994);cpu.writeRegister("SP",0x700000);
                cpu.writeMemoryValue(toAddr(0x700000),4,0x7ffff0);cpu.writeMemoryValue(toAddr(0x700004),2,0);
                SaneBoundary sane=new SaneBoundary(cpu,currentProgram);boolean returned=false;
                for(int step=0;step<30000;step++) {
                    long pc=cpu.getExecutionAddress().getOffset();
                    if(pc==0x7ffff0){returned=true;break;}
                    if(ByteBuffer.wrap(cpu.readMemory(toAddr(pc),2)).getShort()==(short)0xa9eb)sane.execute();
                    else if(!cpu.step(monitor))throw new Exception("PC "+Long.toHexString(pc)+": "+cpu.getLastError());
                }
                if(!returned)throw new Exception("Movement instruction budget");
                if(cases.length()>0)cases.append(',');
                cases.append("{\"input\":\"").append(HexFormat.of().formatHex(input)).append("\",\"output\":\"").append(HexFormat.of().formatHex(cpu.readMemory(toAddr(0x448aa0),3138))).append("\",\"race_movement\":").append(raceMoves[bytes.getShort(0x32)-1]).append('}');
            }finally{cpu.dispose();}
        }
        Files.writeString(root.resolve("evidence/fixtures/combat-movement.json"),"{\"authority\":\"Centurions 1.3.2 movecalc instructions with bounded finite SANE model; not original SANE execution\",\"cases\":["+cases+"]}\n");
        println("Twenty-four movement instruction cases exported");
    }
}
