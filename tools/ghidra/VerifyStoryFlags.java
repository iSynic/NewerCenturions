import ghidra.app.script.GhidraScript;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.HexFormat;

public class VerifyStoryFlags extends GhidraScript {
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]);StringBuilder sets=new StringBuilder(),conditions=new StringBuilder(),scores=new StringBuilder();
        for(int parameter:new int[]{0,1,-1,17,-17,63,-63}) {
            ActionOracleHarness h=new ActionOracleHarness(currentProgram);
            try {
                byte[] before=new byte[64];for(int i=0;i<64;i++)before[i]=(byte)(i%3);
                h.write(0x44258c,before);h.write(0x6ff000,new byte[8192]);h.cpu.writeRegister("A6",0x700000);h.cpu.writeRegister("SP",0x700500);h.value(0x6fffea,2,parameter);h.jump(0x1454de);
                bounded(h,0x14673c,0);
                if(sets.length()>0)sets.append(',');sets.append("{\"parameter\":").append(parameter).append(",\"before\":\"").append(HexFormat.of().formatHex(before)).append("\",\"after\":\"").append(HexFormat.of().formatHex(h.read(0x44258c,64))).append("\"}");
            } finally {h.cpu.dispose();}
        }
        for(int flag:new int[]{0,1,255})for(int mode:new int[]{-1,0,1,2,3}) {
            ActionOracleHarness h=new ActionOracleHarness(currentProgram);
            try {
                h.write(0x6ff000,new byte[8192]);h.cpu.writeRegister("SP",0x700500);h.value(0x44258c+17,1,flag);h.value(0x44330a,2,17);h.value(0x44330c,2,mode);h.jump(0x145490);
                bounded(h,0x144fe4,0x14673c);
                if(conditions.length()>0)conditions.append(',');conditions.append("{\"index\":17,\"flag\":").append(flag).append(",\"mode\":").append(mode).append(",\"matched\":").append(h.pc()==0x144fe4).append("}");
            } finally {h.cpu.dispose();}
        }
        for(int[] input:new int[][]{{0,5},{32767,1},{-32768,-1},{42,-17},{-10,3}}) {
            ActionOracleHarness h=new ActionOracleHarness(currentProgram);
            try {
                h.write(0x6ff000,new byte[8192]);h.cpu.writeRegister("A6",0x700000);h.cpu.writeRegister("SP",0x700500);h.value(0x435502,2,input[0]);h.value(0x6fffea,2,input[1]);h.jump(0x14191c);bounded(h,0x14673c,0);
                if(scores.length()>0)scores.append(',');scores.append("{\"before\":").append(input[0]).append(",\"parameter\":").append(input[1]).append(",\"after\":").append(h.word(0x435502)).append("}");
            } finally {h.cpu.dispose();}
        }
        Files.writeString(root.resolve("evidence/fixtures/story-flags.json"),"{\"authority\":\"Original newland opcode47 instructionblock1454de writes/clears byteflag; opcode46 block145490 tests0/nonzero/unconditional2 to originalbranch144fe4 without executing jump; opcode109 block14191c wraps signedwordscore. C abs_short service intercepted with verified absolute word model; no RNG calls within executed blocks. No whole-script/live claim.\",\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyStoryFlags.java"))+"\",\"harness_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/ActionOracleHarness.java"))+"\",\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\",\"ghidra\":\"12.1.2\",\"rng_draws\":0,\"set_cases\":["+sets+"],\"condition_cases\":["+conditions+"],\"score_cases\":["+scores+"]}\n");println("Story flag cases exported");
    }
    void bounded(ActionOracleHarness h,long stop1,long stop2)throws Exception {
        for(int steps=0;steps<500;steps++) {
            if(h.pc()==stop1||(stop2!=0&&h.pc()==stop2))return;
            if(h.pc()==0x400038){h.cpu.writeRegister("D0",Math.abs(h.word(h.sp()+4)));h.ret();continue;}
            if(!h.cpu.step(monitor))throw new Exception("PC "+Long.toHexString(h.pc())+": "+h.cpu.getLastError());
        }
        throw new Exception("Story instruction budget");
    }
}
