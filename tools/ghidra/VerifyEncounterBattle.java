import ghidra.app.script.GhidraScript;
import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.HexFormat;

public class VerifyEncounterBattle extends GhidraScript {
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]);StringBuilder cases=new StringBuilder();
        for(int[] input:new int[][]{{8,0,0,0,0},{11,13,663,138,0},{13,11,-663,-138,5},{-8,0,0,0,6},{0,0,1,1,-1},{8,8,0,0,0}})for(long initial:new long[]{12,1}) {
            ActionOracleHarness h=new ActionOracleHarness(currentProgram);
            try {
                byte[] extra=ByteBuffer.allocate(10).putShort((short)input[0]).putShort((short)input[1]).putShort((short)input[2]).putShort((short)input[3]).putShort((short)input[4]).array();
                h.write(0x44330a,extra);h.write(0x6fc000,new byte[24576]);h.cpu.writeRegister("A6",0x700000);h.cpu.writeRegister("SP",0x6ffe00);h.jump(0x14287a);
                long seed=initial;int draws=0;StringBuilder events=new StringBuilder();boolean done=false;
                for(int step=0;step<1000;step++) {
                    long pc=h.pc();
                    if(pc==0x1428e0){done=true;break;}
                    if(pc==0x133a18){if(events.length()>0)events.append(',');events.append("[\"sound\",").append(h.word(h.sp()+4)).append(']');h.ret();continue;}
                    if(pc==0x15dc34){if(events.length()>0)events.append(',');events.append("[\"message\",").append(h.word(h.sp()+6)).append(']');h.ret();continue;}
                    if(pc==0x153ca4){int bound=h.word(h.sp()+4);seed=seed*16807%2147483647;draws++;short raw=(short)seed;if(raw==Short.MIN_VALUE)raw=0;h.cpu.writeRegister("D0",Math.abs((int)raw)*bound/32768+1);h.ret();continue;}
                    if(!h.cpu.step(monitor))throw new Exception("ED selection at "+Long.toHexString(pc)+": "+h.cpu.getLastError());
                }
                if(!done)throw new Exception("ED selection budget");
                if(cases.length()>0)cases.append(',');cases.append("{\"extra\":\"").append(HexFormat.of().formatHex(extra)).append("\",\"seed_before\":").append(initial).append(",\"seed_after\":").append(seed).append(",\"draws\":").append(draws).append(",\"battle\":").append(h.word(0x43874e)).append(",\"events\":[").append(events).append("]}");
            }finally{h.cpu.dispose();}
        }
        Files.writeString(root.resolve("evidence/fixtures/encounter-battle.json"),"{\"authority\":\"Original newland14287a..1428e0 instructions after loadextracode, including native randrange13849e. Sound and textbox recorded at call boundaries; Rand modeled. Synthetic extra records include reversed bounds, negative IDs and equal bounds. Combat body and victory continuation not executed.\",\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyEncounterBattle.java"))+"\",\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\",\"cases\":["+cases+"]}\n");
        println("Twelve original encounter battle cases exported");
    }
}
