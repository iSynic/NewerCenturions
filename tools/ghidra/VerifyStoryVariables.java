import ghidra.app.script.GhidraScript;
import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.HexFormat;

public class VerifyStoryVariables extends GhidraScript {
    StringBuilder update = new StringBuilder(), predicate = new StringBuilder();
    public void run() throws Exception {
        Path root = Path.of(getScriptArgs()[0]);
        for (int[] input : new int[][]{
            {21,0,1,0,0,266},{21,126,1,0,0,266},{21,127,1,0,0,266},
            {21,-127,-1,0,0,266},{21,-128,0,0,0,266},{21,100,32767,0,0,266},
            {21,-100,-32768,0,0,266},{0,0,5,0,0,266},{99,0,5,0,0,266},{100,7,5,0,0,266},
            {21,0,1,1,1,266},{21,0,1,2,1,266},{21,0,1,3,1,266},
            {21,0,1,4,1,266},{21,0,1,1,2,266},{21,0,-1,1,-1,266}
        }) probe(76,input);
        for(int value:new int[]{-128,-1,0,1,127}) for(int threshold:new int[]{-1,0,1})
            for(int mode:new int[]{0,1,2,3}) probe(77,new int[]{21,value,threshold,mode,136,139});
        probe(77,new int[]{21,0,1,0,0,139});
        probe(77,new int[]{21,1,1,0,136,0});
        Files.writeString(root.resolve("evidence/fixtures/story-variables.json"),
            "{\"authority\":\"Original newland opcode76 block14433c executes byte load, signed-word addition, original pin13f65e and byte store, then continuation selection. Opcode77 block1443f8 executes signed-byte/signed-word comparison and branch selection. loaddoor2 is a recorded boundary; encounters and copied scripts are not executed. The supplied extra records and synthetic 128byte flag table isolate arithmetic and continuation. No RNG is called.\",\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyStoryVariables.java"))+"\",\"harness_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/ActionOracleHarness.java"))+"\",\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\",\"ghidra\":\"12.1.2\",\"rng_draws\":0,\"update_cases\":["+update+"],\"predicate_cases\":["+predicate+"]}\n");
        println("Story variable cases exported");
    }
    void probe(int opcode,int[] input) throws Exception {
        ActionOracleHarness h=new ActionOracleHarness(currentProgram);
        try {
            byte[] before=new byte[128];for(int i=0;i<before.length;i++)before[i]=(byte)(i%5-2);
            before[input[0]]=(byte)input[1];h.write(0x44258c,before);
            byte[] extra=ByteBuffer.allocate(10).putShort((short)input[0]).putShort((short)input[2]).putShort((short)input[3]).putShort((short)input[4]).putShort((short)input[5]).array();
            h.write(0x44330a,extra);h.write(0x6fd000,new byte[16384]);h.cpu.writeRegister("A6",0x700000);h.cpu.writeRegister("SP",0x6ffe00);h.value(0x6fffea,2,266);h.value(0x6fffee,2,3);h.jump(opcode==76?0x14433c:0x1443f8);
            int target=0;boolean done=false;
            for(int step=0;step<500;step++) {
                long pc=h.pc();if(pc==0x14673c||pc==0x14032e||pc==0x142b3e||pc==0x142d96){done=true;break;}
                if(pc==0x123c60){target=h.word(h.sp()+4);h.ret();continue;}
                if(!h.cpu.step(monitor))throw new Exception("Variable at "+Long.toHexString(pc)+": "+h.cpu.getLastError());
            }
            if(!done)throw new Exception("Variable instruction budget");
            String continuation=h.pc()==0x14673c?"continue":h.pc()==0x14032e?"jump":h.pc()==0x142b3e?"encounter":"typed";
            if(h.pc()==0x142b3e||h.pc()==0x142d96)target=h.word(0x6fffea);
            StringBuilder out=opcode==76?update:predicate;if(out.length()>0)out.append(',');
            out.append("{\"extra\":\"").append(HexFormat.of().formatHex(extra)).append("\",\"before\":\"").append(HexFormat.of().formatHex(before)).append("\",\"after\":\"").append(HexFormat.of().formatHex(h.read(0x44258c,128))).append("\",\"comparison\":").append(opcode==77?h.word(0x4471ea):0).append(",\"continuation\":\"").append(continuation).append("\",\"target\":").append(target).append(",\"local_action_index\":").append(h.word(0x6fffee)).append('}');
        }finally{h.cpu.dispose();}
    }
}
