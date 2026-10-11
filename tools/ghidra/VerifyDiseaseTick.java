import ghidra.app.script.GhidraScript;
import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.HexFormat;

public class VerifyDiseaseTick extends GhidraScript {
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]);HexFormat hex=HexFormat.of();StringBuilder cases=new StringBuilder();
        byte[] original=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Warriors/Cindred"));
        for(int duration:new int[]{-32768,-1,0,1,2,29,32767})for(int health:new int[]{-10,0,1,20,100,32767})for(int condition:new int[]{-1,0,1}) {
            ActionOracleHarness h=new ActionOracleHarness(currentProgram);
            try {
                byte[] input=original.clone();ByteBuffer.wrap(input).putShort(0x96,(short)duration).putShort(0x50,(short)health).putShort(0x90,(short)condition);
                h.write(0x448aa0,input);h.value(0x438776,2,1);h.cpu.writeRegister("D3",0);h.cpu.writeRegister("SP",0x700000);h.jump(0x14e014);boolean done=false,killed=false;
                for(int step=0;step<1000;step++) {
                    long pc=h.pc();if(pc==0x14e10a){done=true;break;}
                    if(pc==0x400038){h.cpu.writeRegister("D0",(short)Math.abs(h.word(h.sp()+4)));h.ret();continue;}
                    if(pc==0x12e4e4){killed=true;h.ret();continue;}
                    if(pc==0x123d1c||pc==0x1600fc||pc==0x158454){h.ret();continue;}
                    if(!h.cpu.step(monitor))throw new Exception("Disease tick "+Long.toHexString(pc)+": "+h.cpu.getLastError());
                }
                if(!done)throw new Exception("Disease tick instruction budget");
                if(cases.length()>0)cases.append(',');cases.append("{\"input\":\"").append(hex.formatHex(input)).append("\",\"after\":\"").append(hex.formatHex(h.read(0x448aa0,3138))).append("\",\"killed\":").append(killed).append('}');
            }finally{h.cpu.dispose();}
        }
        Files.writeString(root.resolve("evidence/fixtures/disease-tick.json"),"{\"authority\":\"Original reduce disease block14e014..14e10a, including signed guards, abs_short and wrapping HP subtraction. Killbody intercepted and recorded before its effects; presentation intercepted. Does not validate kill cleanup or later duration reduction.\",\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyDiseaseTick.java"))+"\",\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\",\"cases\":["+cases+"]}\n");println("126 original disease tick cases exported");
    }
}
