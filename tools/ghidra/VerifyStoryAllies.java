import ghidra.app.script.GhidraScript;
import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.HexFormat;

public class VerifyStoryAllies extends GhidraScript {
    static String hex(byte[] bytes){return HexFormat.of().formatHex(bytes);}
    byte[] monsters;
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]);monsters=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data MD"));StringBuilder cases=new StringBuilder();
        for(long seed:new long[]{12,1})for(int[] input:new int[][]{{0,0,1},{0,0,2},{1,0,1},{19,0,1},{19,0,2},{20,0,1},{0,1,1},{0,-1,1}})probe(seed,input[0],input[1],input[2],cases);
        Files.writeString(root.resolve("evidence/fixtures/story-allies.json"),"{\"authority\":\"Original newland89 block141eb6..142142 and complete updatenpcmenu1046e0 execute. Exact MD65 supplied through original file seek/read boundaries. Each new ally rolls Rand8 once because MD65 levelword0 is1; its nonnegative weaponword4e avoids combatsetup selection. Difficulty arithmetic preserves byte/word widths. Full menu20records includes stale activity bytes; updatenpcmenu clears active word10c and byte105 after normal addition or capacity20 warning108. Toolbox menu/string and warning presentation modeled. No combat participation or live NPC join claim.\",\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyStoryAllies.java"))+"\",\"harness_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/ActionOracleHarness.java"))+"\",\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\",\"monsters_sha256\":\""+ActionOracleHarness.sha256(root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data MD"))+"\",\"ghidra\":\"12.1.2\",\"definition\":\""+hex(Arrays.copyOfRange(monsters,65*288,66*288))+"\",\"cases\":["+cases+"]}\n");println("Sixteen original ally join cases exported");
    }
    void probe(long initial,int count,int difficulty,int calls,StringBuilder cases) throws Exception {
        byte[] before=new byte[20*288],definition=Arrays.copyOfRange(monsters,65*288,66*288);
        for(int i=0;i<count;i++){System.arraycopy(definition,0,before,i*288,288);before[i*288+0x105]=3;ByteBuffer.wrap(before).putShort(i*288+0x10c,(short)(i+1)).putShort(i*288+0x110,(short)(20+i)).putShort(i*288+0x112,(short)(20+i));}
        ActionOracleHarness h=new ActionOracleHarness(currentProgram);long seed=initial;int draws=0;StringBuilder warnings=new StringBuilder(),drawRecords=new StringBuilder();int cursor=0;
        try {
            h.write(0x440edc,before);h.value(0x44257c,2,count);h.value(0x442576,2,difficulty);
            for(int call=0;call<calls;call++) {
                h.write(0x6fd000,new byte[16384]);h.cpu.writeRegister("A6",0x700000);h.cpu.writeRegister("SP",0x6ffe00);h.value(0x6fffea,2,65);h.jump(0x141eb6);boolean done=false;
                for(int step=0;step<30000;step++) {
                    long pc=h.pc(),sp=h.sp();if(pc==0x14673c){done=true;break;}
                    if(pc==0x136cfa||pc==0x400090){h.ret();continue;}
                    if(pc==0x400070){h.cpu.writeRegister("A0",0x600100);h.ret();continue;}
                    if(pc==0x400078){cursor=(int)h.pointer(sp+8);if(h.word(sp+12)!=0)throw new Exception("Ally nonabsolute seek");h.ret();continue;}
                    if(pc==0x400048){int size=Math.toIntExact(h.pointer(sp+8)*h.pointer(sp+12));if(size!=288||cursor<0||cursor+size>monsters.length)throw new Exception("Ally file boundary");h.write(h.pointer(sp+4),Arrays.copyOfRange(monsters,cursor,cursor+size));cursor+=size;h.cpu.writeRegister("D0",1);h.ret();continue;}
                    if(pc==0x400068){h.ret();continue;}
                    if(pc==0x16474a){if(warnings.length()>0)warnings.append(',');warnings.append(h.word(sp+4));h.ret();continue;}
                    if(pc==0x153ca4){int bound=h.word(sp+4);seed=seed*16807%2147483647;draws++;short raw=(short)seed;if(raw==Short.MIN_VALUE)raw=0;int value=Math.abs((int)raw)*bound/32768+1;h.cpu.writeRegister("D0",value);if(drawRecords.length()>0)drawRecords.append(',');drawRecords.append("{\"bound\":").append(bound).append(",\"result\":").append(value).append('}');h.ret();continue;}
                    if(pc==0x11d406)throw new Exception("Unexpected ally65 weapon-selection branch");
                    int op=h.word(pc)&65535;if(op==0xa939||op==0xa93a){h.pop(6);h.jump(pc+2);continue;}if(op==0xa947){h.pop(10);h.jump(pc+2);continue;}if(op==0xa940){h.pop(8);h.jump(pc+2);continue;}
                    if(!h.cpu.step(monitor))throw new Exception("Ally at "+Long.toHexString(pc)+": "+h.cpu.getLastError());
                }
                if(!done)throw new Exception("Ally instruction budget");
            }
            if(cases.length()>0)cases.append(',');cases.append("{\"id\":65,\"count_before\":").append(count).append(",\"count_after\":").append(h.word(0x44257c)).append(",\"difficulty\":").append(difficulty).append(",\"calls\":").append(calls).append(",\"seed_before\":").append(initial).append(",\"seed_after\":").append(seed).append(",\"rng_draws\":").append(draws).append(",\"draws\":[").append(drawRecords).append("],\"warnings\":[").append(warnings).append("],\"before\":\"").append(hex(before)).append("\",\"after\":\"").append(hex(h.read(0x440edc,before.length))).append("\"}");
        }finally{h.cpu.dispose();}
    }
}
