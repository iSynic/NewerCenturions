import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;

/** Original randomitem initialization and enhancement branches. */
public class VerifyRandomItems extends VerifyStoryInventory {
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]),original=root.resolve("NewCenturions/TheNewCenturions");
        items=Files.readAllBytes(original.resolve("Data Files/Data ID"));
        scenarioItems=Files.readAllBytes(original.resolve("Scenarios/The Thunder of Guns/0/Data NI"));
        StringBuilder cases=new StringBuilder();
        int[] ids={178,37,192,100,165,179,180,181,683,177,440,470,420,400,630,430,460,600,602,1,634,409,627,637,802,828,804,200,616,700,38,9,643,425,238,-178,-37,-192};
        for(long seed:new long[]{12,1})for(int id:ids)for(int enhancement:new int[]{-1,0,1,3})probe(seed,id,enhancement,cases);
        StringBuilder identity=new StringBuilder();
        String[][] inputs={{"probe","tools/ghidra/VerifyRandomItems.java"},{"parent_probe","tools/ghidra/VerifyStoryInventory.java"},{"harness","tools/ghidra/ActionOracleHarness.java"},{"mapped_binary","analysis/ghidra/startup/code1-mapped.bin"},{"items","NewCenturions/TheNewCenturions/Data Files/Data ID"},{"scenario_items","NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data NI"}};
        for(String[] input:inputs){if(identity.length()>0)identity.append(',');identity.append('"').append(input[0]).append("_sha256\":\"").append(ActionOracleHarness.sha256(root.resolve(input[1]))).append('"');}
        Files.writeString(root.resolve("evidence/fixtures/random-items.json"),"{\"authority\":\"Complete original randomitem12d558 initializes supplied fresh22B crate records. Thirty-eight signed IDs cover mandatory178/37/192, additional finalweapon100/165/179/180/181/683, meaningful definition kinds1..28 and each enhancement attribute branch; exact original base/scenario definitions supplied by loaditem boundary. Seeds12/1 and signed enhancement-1/0/1/3, difficulty0, exact22B results and ordered RNG receipts. No pickup, equipment, UI or live original battle claim.\",\"identity\":{"+identity+"},\"cases\":["+cases+"]}\n");
        println("304 original item initialization cases exported");
    }
    void probe(long seed,int id,int enhancement,StringBuilder cases)throws Exception {
        ActionOracleHarness h=frame();RunResult result=new RunResult(seed);
        byte[] before=new byte[22];ByteBuffer.wrap(before).putShort(0,(short)id);
        byte[] definition=id<0?definition(-id):definition(id);
        try {
            h.write(0x41fc74,new byte[22000]);h.write(0x41fc74,before);h.value(0x442576,2,0);
            h.write(0x46225e,Arrays.copyOfRange(items,30000,60000));
            h.enter(0x12d558,0x6ffe00,0x900000);h.value(0x6ffe04,2,0);h.value(0x6ffe06,2,enhancement);
            boolean done=false;
            for(int step=0;step<500000;step++) {
                long pc=h.pc();if(pc==0x900000){done=true;break;}
                if(pc==0x130afe){loadItem(h);continue;}
                if(pc==0x153ca4){result.rng(h);continue;}
                if(!h.cpu.step(monitor))throw new Exception("randomitem ID"+id+" enhancement"+enhancement+" PC"+Long.toHexString(pc)+": "+h.cpu.getLastError());
            }
            if(!done)throw new Exception("randomitem instruction budget");
            if(cases.length()>0)cases.append(',');
            cases.append("{\"id\":").append(id).append(",\"enhancement\":").append(enhancement).append(",\"difficulty\":0,\"definition\":\"").append(hex(definition)).append("\",\"before\":\"").append(hex(before)).append("\",\"after\":\"").append(hex(h.read(0x41fc74,22))).append("\",\"seed_before\":").append(seed).append(",\"seed_after\":").append(result.seed).append(",\"rng_draws\":").append(result.draws).append(",\"draws\":[").append(result.drawRecords).append("]}");
        }finally{h.cpu.dispose();}
    }
    byte[] definition(int id) {
        byte[] table=id>=800?scenarioItems:items;int slot=id>=800?id-800:id;
        return Arrays.copyOfRange(table,slot*150,(slot+1)*150);
    }
}
