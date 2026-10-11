import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;

public class VerifyItemCondition extends VerifyStoryInventory {
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]);items=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data ID"));scenarioItems=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data NI"));races=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data Race"));party=new byte[3138*6];
        String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};for(int i=0;i<6;i++)System.arraycopy(Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Warriors/"+names[i])),0,party,i*3138,3138);
        StringBuilder cases=new StringBuilder();for(boolean present:new boolean[]{false,true})for(int mode:new int[]{-1,0,1,2,3,4})probe(present,mode,cases);
        Files.writeString(root.resolve("evidence/fixtures/item-condition.json"),"{\"authority\":\"Original newland21 block143ce4 executes checkforitem112650 with supplied warriors and synthetic quantity0 item833 in warrior5 when present. Found modes0/1/2 select script/menu/typed continuation; missing mode0 selects script, mode3 sets action index99, other modes continue. loaddoor2 is recorded only. No copied script, encounter UI or live route claim.\",\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyItemCondition.java"))+"\",\"inventory_probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyStoryInventory.java"))+"\",\"harness_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/ActionOracleHarness.java"))+"\",\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\",\"ghidra\":\"12.1.2\",\"rng_draws\":0,\"cases\":["+cases+"]}\n");println("Original item condition cases exported");
    }
    void probe(boolean present,int mode,StringBuilder cases) throws Exception {
        byte[] before=present?add(party,4,833,0,false):party.clone();byte[] extra=ByteBuffer.allocate(10).putShort((short)833).putShort((short)(present?mode:0)).putShort((short)(present?0:mode)).putShort((short)53).putShort((short)97).array();
        ActionOracleHarness h=frame();
        try {
            common(h,before);h.write(0x44330a,extra);h.value(0x6fffea,2,266);h.value(0x6fffee,2,3);h.jump(0x143ce4);boolean done=false;int target=0;
            for(int step=0;step<20000;step++) {
                long pc=h.pc();if(pc==0x14673c||pc==0x14032e||pc==0x142b3e||pc==0x142d96){done=true;break;}
                if(pc==0x123c60){target=h.word(h.sp()+4);h.ret();continue;}
                if(!h.cpu.step(monitor))throw new Exception("Item condition PC "+Long.toHexString(pc)+": "+h.cpu.getLastError());
            }
            if(!done)throw new Exception("Item condition budget");String continuation=h.pc()==0x14673c?"continue":h.pc()==0x14032e?"jump":h.pc()==0x142b3e?"encounter":"typed";
            if(h.pc()==0x142b3e||h.pc()==0x142d96)target=h.word(0x6fffea);if(cases.length()>0)cases.append(',');
            cases.append("{\"present\":").append(present).append(",\"mode\":").append(mode).append(",\"extra\":\"").append(hex(extra)).append("\",\"before\":\"").append(hex(before)).append("\",\"after\":\"").append(hex(h.read(0x448aa0,party.length))).append("\",\"continuation\":\"").append(continuation).append("\",\"target\":").append(target).append(",\"local_action_index\":").append(h.word(0x6fffee)).append('}');
        }finally{h.cpu.dispose();}
    }
}
