import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;

/** Consecutive native rewards retain records but consume the enemy count once. */
public class VerifyNestedVictory extends VerifyStoryInventory {
    byte[] monsters,battle,properties;
    StringBuilder events;
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]),original=root.resolve("NewCenturions/TheNewCenturions"),scenario=original.resolve("Scenarios/The Thunder of Guns/0");
        items=Files.readAllBytes(original.resolve("Data Files/Data ID"));scenarioItems=Files.readAllBytes(scenario.resolve("Data NI"));races=Files.readAllBytes(original.resolve("Data Files/Data Race"));monsters=Files.readAllBytes(scenario.resolve("Data MD"));battle=Arrays.copyOfRange(Files.readAllBytes(scenario.resolve("Data BD")),15*1372,16*1372);properties=Files.readAllBytes(original.resolve("Data Files/Data Swamp BD"));party=new byte[6*3138];String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};for(int i=0;i<6;i++)System.arraycopy(Files.readAllBytes(original.resolve("Warriors/"+names[i])),0,party,i*3138,3138);
        StringBuilder cases=new StringBuilder();for(long seed:new long[]{12,1})probe(seed,cases);
        String identity="\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyNestedVictory.java"))+"\",\"inventory_probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyStoryInventory.java"))+"\",\"harness_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/ActionOracleHarness.java"))+"\",\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\",\"battles_sha256\":\""+ActionOracleHarness.sha256(scenario.resolve("Data BD"))+"\",\"monsters_sha256\":\""+ActionOracleHarness.sha256(scenario.resolve("Data MD"))+"\",\"items_sha256\":\""+ActionOracleHarness.sha256(original.resolve("Data Files/Data ID"))+"\",\"scenario_items_sha256\":\""+ActionOracleHarness.sha256(scenario.resolve("Data NI"))+"\",\"races_sha256\":\""+ActionOracleHarness.sha256(original.resolve("Data Files/Data Race"))+"\",\"properties_sha256\":\""+ActionOracleHarness.sha256(original.resolve("Data Files/Data Swamp BD"))+"\",\"sane_boundary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/SaneBoundary.java"))+"\"";
        Files.writeString(root.resolve("evidence/fixtures/nested-victory.json"),"{\"authority\":\"Original BD15 enemy generation11e0fe..11eb86 including exact MD67, placement and SANE with controlled flat1155 map/origin5/base40/bounds5..85. Only generated HP110 is forced0 for victory input. Two successive original booty(0,0)105748 calls execute all native initialization, bodycount, currency/drop/randomitem/reward arithmetic and closing gameplay. At first window allocation1064c2 presentation is acknowledged as Close and control enters native close gameplay107e52; no inventory pickup, GUI viewport or live battle claim. Native closing getexp/levelup execute if triggered. Enemy count/records, crate, party, seed, currency, share and ordered service calls captured after each return.\",\"identity\":{"+identity+"},\"battle\":\""+hex(battle)+"\",\"definition\":\""+hex(Arrays.copyOfRange(monsters,67*288,68*288))+"\",\"cases\":["+cases+"]}\n");println("Two consecutive original nested victory cases exported");
    }
    byte[] terrain(){byte[] map=new byte[16200];for(int i=0;i<8100;i++)ByteBuffer.wrap(map).putShort(i*2,(short)1155);return map;}
    void event(String event){if(events.length()>0)events.append(',');events.append(event);}
    void generate(ActionOracleHarness h,RunResult result) throws Exception {
        h.write(0x43979c,battle);h.write(0x43167c,Arrays.copyOfRange(properties,64,12064));h.value(0x43874a,2,0);h.value(0x4435c4,2,0);h.value(0x43877c,2,0);h.value(0x439582,2,0);h.cpu.writeRegister("D5",40);h.cpu.writeRegister("D6",40);h.value(0x6ffff8,2,5);h.value(0x6ffffa,2,5);h.value(0x6ffffc,2,85);h.value(0x6ffffe,2,85);h.jump(0x11e0fe);SaneBoundary sane=new SaneBoundary(h.cpu,currentProgram);long offset=0;
        for(int step=0;step<1500000;step++){
            long pc=h.pc(),sp=h.sp();if(pc==0x11eb86)return;
            if(pc==0x136cfa||pc==0x400068){h.ret();continue;}if(pc==0x400070){h.cpu.writeRegister("A0",0x600000);h.ret();continue;}
            if(pc==0x400078){offset=h.pointer(sp+8);h.cpu.writeRegister("D0",0);h.ret();continue;}
            if(pc==0x400048){int size=(int)h.pointer(sp+8);if(size!=288||offset+size>monsters.length)throw new Exception("Monster file boundary");h.write(h.pointer(sp+4),Arrays.copyOfRange(monsters,(int)offset,(int)offset+size));offset+=size;h.cpu.writeRegister("D0",1);h.ret();continue;}
            if(pc==0x400038){h.cpu.writeRegister("D0",Math.abs(h.word(sp+4)));h.ret();continue;}if(pc==0x153ca4){result.rng(h);continue;}
            int op=h.word(pc)&65535;if(op==0xa9eb){sane.execute();continue;}
            if(op==0xa8ad){long rect=h.pointer(sp);int y=h.word(sp+4),x=h.word(sp+6);h.value(sp+8,1,y>=h.word(rect)&&x>=h.word(rect+2)&&y<h.word(rect+4)&&x<h.word(rect+6)?1:0);h.pop(8);h.jump(pc+2);continue;}
            if(!h.cpu.step(monitor))throw new Exception("Nested enemy generation at "+Long.toHexString(pc)+": "+h.cpu.getLastError());
        }
        throw new Exception("Nested enemy generation budget");
    }
    int reward(ActionOracleHarness h,RunResult result,int invocation) throws Exception {
        h.enter(0x105748,0x6ffe00,0x900000);h.value(0x6ffe04,2,0);h.value(0x6ffe06,2,0);SaneBoundary sane=new SaneBoundary(h.cpu,currentProgram);int share=0;
        event("[\"booty\","+invocation+","+h.word(0x43874a)+"]");
        for(int step=0;step<1500000;step++){
            long pc=h.pc(),sp=h.sp();if(pc==0x900000)return share;
            if(pc==0x1064c2){share=(int)h.pointer(h.cpu.readRegister("A6").longValue()-0x96);event("[\"close\","+invocation+","+h.word(0x43874a)+"]");h.jump(0x107e52);continue;}
            if(pc==0x130afe){loadItem(h);continue;}if(pc==0x153ca4){result.rng(h);continue;}if(pc==0x133a18||pc==0x16474a){event("[\"sound\","+h.word(sp+4)+"]");result.sound(h);continue;}if(pc==0x400038){h.cpu.writeRegister("D0",Math.abs(h.word(sp+4)));h.ret();continue;}
            if(pc==0x1047f8){event("[\"bodycount\","+h.word(0x43874a)+"]");}
            if(pc==0x1046e0||pc==0x13e13c||pc==0x13a7c4||pc==0x13154c||pc==0x13a2c4||pc==0x12382a||pc==0x160fb2||pc==0x158454||pc==0x1205f8||pc==0x105572||pc==0x1382a8){h.ret();continue;}
            int op=h.word(pc)&65535;if(op==0xa9eb){sane.execute();continue;}
            if(op==0xa8a7){long rect=h.pointer(sp+8);h.write(rect,h.read(sp+4,4));h.write(rect+4,h.read(sp,4));h.pop(12);h.jump(pc+2);continue;}
            if(op==0xa939){h.pop(6);h.jump(pc+2);continue;}if(op==0xaa1c||op==0xa93c||op==0xa873||op==0xa914){h.pop(4);h.jump(pc+2);continue;}if(op==0xa937){h.jump(pc+2);continue;}if(op==0xa89b){h.pop(4);h.jump(pc+2);continue;}
            if(op==0xaa1e){h.value(sp+2,4,0);h.pop(2);h.jump(pc+2);continue;}
            if(!h.cpu.step(monitor))throw new Exception("Nested reward"+invocation+" at "+Long.toHexString(pc)+": "+h.cpu.getLastError());
        }
        throw new Exception("Nested reward budget");
    }
    void probe(long seed,StringBuilder cases) throws Exception {
        ActionOracleHarness h=frame();RunResult result=new RunResult(seed);events=new StringBuilder();
        try {
            byte[] warriors=party.clone();for(int i=0;i<6;i++){warriors[i*3138+0x9df]=1;ByteBuffer.wrap(warriors).putShort(i*3138+0x20,(short)(i+1)).putShort(i*3138+0x18,(short)-1);}
            common(h,warriors);h.write(0x439d3c,new byte[100*288]);h.write(0x440edc,new byte[20*288]);h.write(0x4085b8,terrain());h.value(0x442780,4,5);h.value(0x442784,4,5);h.write(0x4537fe,Arrays.copyOfRange(items,0,30000));h.write(0x46978e,Arrays.copyOfRange(items,90000,120000));h.value(0x43957e,2,0);h.value(0x443392,4,0);h.value(0x438776,2,1);
            for(int i=0;i<6;i++){h.value(0x44709a+i*4,2,20+i);h.value(0x44709c+i*4,2,20);h.value(0x437a6e+i*2,2,1155);}h.write(0x4470b8,new byte[220]);for(int i=0;i<110;i++)h.value(0x4470b8+i*2,2,-1);for(int i=0;i<6;i++)h.value(0x4470b8+(i+1)*2,2,i);
            generate(h,result);int count=h.word(0x43874a);byte[] generated=h.read(0x439d3c,count*288);for(int i=0;i<count;i++)h.value(0x439d3c+i*288+0x110,2,0);h.value(0x44374c,2,count);byte[] before=h.read(0x439d3c,100*288);long afterSetup=result.seed;int setupDraws=result.draws;StringBuilder calls=new StringBuilder();
            for(int n=1;n<=2;n++){long rngBefore=result.seed;int drawsBefore=result.draws;byte[] partyBefore=h.read(0x448aa0,party.length);int share=reward(h,result,n);if(n>1)calls.append(',');calls.append("{\"invocation\":").append(n).append(",\"seed_before\":").append(rngBefore).append(",\"seed_after\":").append(result.seed).append(",\"rng_draws\":").append(result.draws-drawsBefore).append(",\"share\":").append(share).append(",\"enemy_count_after\":").append(h.word(0x43874a)).append(",\"records_after\":\"").append(hex(h.read(0x439d3c,100*288))).append("\",\"party_before\":\"").append(hex(partyBefore)).append("\",\"party_after\":\"").append(hex(h.read(0x448aa0,party.length))).append("\",\"loot_after\":\"").append(hex(h.read(0x41fc74,22000))).append("\",\"currency\":").append(h.pointer(0x442788)).append('}');}
            if(cases.length()>0)cases.append(',');cases.append("{\"seed\":").append(seed).append(",\"setup_seed_after\":").append(afterSetup).append(",\"setup_draws\":").append(setupDraws).append(",\"count\":").append(count).append(",\"generated\":\"").append(hex(generated)).append("\",\"records_before\":\"").append(hex(before)).append("\",\"calls\":[").append(calls).append("],\"events\":[").append(events).append("]}");
        }finally{h.cpu.dispose();}
    }
}
