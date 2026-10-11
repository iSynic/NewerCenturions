import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;

/** Original reward availability recount and persistence through Close. */
public class VerifyBootyAvailability extends VerifyBattleRewards {
    void availabilityProbe(String name,long initial,byte[] input,int difficulty,int npc,int currency,int variant,boolean authored)throws Exception {
        ActionOracleHarness h=frame();RunResult result=new RunResult(initial);events=new StringBuilder();
        try {
            byte[] before=party.clone();for(int i=0;i<6;i++){
                int o=i*3138;before[o+0x9df]=0;before[o+0x9de]=0;set(before,o+0x5e,3);set(before,o+0x60,-4);set(before,o+0x62,-2);set(before,o+0x64,5);
                int hp=20,condition=0;
                if(variant==1&&i<2)condition=-1;if(variant==2)condition=1;if(variant==3)hp=0;
                if(variant==4){if(i<3)hp=0;if(i>=2&&i<5)condition=-1;}
                if(variant==5&&i==3)condition=-32768;if(variant==6)condition=32767;if(variant==7)hp=i==0?-32768:1;
                set(before,o+0x90,condition);set(before,o+0x50,hp);ByteBuffer.wrap(before).putInt(o+0x9ce,1000000).putInt(o+0x9d2,0);
            }
            common(h,before);h.write(0x439d3c,new byte[100*288]);h.write(0x440edc,new byte[20*288]);h.write(0x4085b8,terrain());h.value(0x442780,4,5);h.value(0x442784,4,5);h.write(0x4537fe,Arrays.copyOfRange(items,0,30000));h.write(0x46978e,Arrays.copyOfRange(items,90000,120000));h.value(0x43957e,2,0);h.value(0x438776,2,1);
            for(int i=0;i<6;i++){h.value(0x44709a+i*4,2,20+i);h.value(0x44709c+i*4,2,20);h.value(0x437a6e+i*2,2,1155);}h.write(0x4470b8,new byte[220]);for(int i=0;i<110;i++)h.value(0x4470b8+i*2,2,-1);for(int i=0;i<6;i++)h.value(0x4470b8+(i+1)*2,2,i);
            int count;
            if(authored){println("Generating "+name);battle=input;generate(h,result);count=h.word(0x43874a);for(int i=0;i<count;i++)h.value(0x439d3c+i*288+0x110,2,0);}
            else {count=input.length/288;h.write(0x439d3c,input);h.value(0x43874a,2,count);}
            byte[] enemies=h.read(0x439d3c,count*288);h.value(0x442576,2,difficulty);h.value(0x44257e,2,npc);h.value(0x442788,4,currency);h.value(0x44374c,2,count);long rngBefore=result.seed;int drawsBefore=result.draws;
            h.value(0x442739,1,new int[]{127,0,-1,-128,7,1,100,5}[variant]);h.enter(0x105748,0x6ffe00,0x900000);h.value(0x6ffe04,2,0);h.value(0x6ffe06,2,0);SaneBoundary sane=new SaneBoundary(h.cpu,currentProgram);boolean prepared=false,closed=false;int share=0,total=0,prepUnavailable=0;byte[] prep=null,crate=null,records=null;
            for(int step=0;step<1500000;step++) {
                long pc=h.pc(),sp=h.sp();if(pc==0x900000){closed=true;break;}
                if(pc==0x1064c2){prepared=true;prepUnavailable=(int)(h.read(0x442739,1)[0]);share=(int)h.pointer(h.cpu.readRegister("A6").longValue()-0x96);total=(int)h.pointer(0x443392);prep=h.read(0x448aa0,party.length);crate=h.read(0x41fc74,22000);records=h.read(0x439d3c,count*288);h.jump(0x107e52);continue;}
                if(pc==0x1380ac||pc==0x12ed42)throw new Exception("Pending level gain escaped reward scope");
                if(pc==0x130afe){loadItem(h);continue;}if(pc==0x153ca4){result.rng(h);continue;}if(pc==0x133a18||pc==0x16474a){result.sound(h);continue;}if(pc==0x400038){h.cpu.writeRegister("D0",(short)Math.abs(h.word(sp+4)));h.ret();continue;}
                if(pc==0x1046e0||pc==0x13e13c||pc==0x13a7c4||pc==0x13154c||pc==0x13a2c4||pc==0x12382a||pc==0x160fb2||pc==0x158454||pc==0x1205f8||pc==0x105572||pc==0x1382a8){h.ret();continue;}
                int op=h.word(pc)&65535;if(op==0xa9eb){sane.execute();continue;}
                if(op==0xa8a7){long rect=h.pointer(sp+8);h.write(rect,h.read(sp+4,4));h.write(rect+4,h.read(sp,4));h.pop(12);h.jump(pc+2);continue;}
                if(op==0xa939){h.pop(6);h.jump(pc+2);continue;}if(op==0xaa1c||op==0xa93c||op==0xa873||op==0xa914){h.pop(4);h.jump(pc+2);continue;}if(op==0xa937){h.jump(pc+2);continue;}if(op==0xa89b){h.pop(4);h.jump(pc+2);continue;}if(op==0xaa1e){h.value(sp+2,4,0);h.pop(2);h.jump(pc+2);continue;}
                if(!h.cpu.step(monitor))throw new Exception(name+" reward at "+Long.toHexString(pc)+": "+h.cpu.getLastError());
            }
            if(!prepared||!closed)throw new Exception("Reward budget");if(cases.length()>0)cases.append(',');byte[] after=h.read(0x448aa0,party.length);int finalUnavailable=(int)h.read(0x442739,1)[0];
            cases.append("{\"unavailable_before\":").append(new int[]{127,0,-1,-128,7,1,100,5}[variant]).append(",\"unavailable_prepared\":").append(prepUnavailable).append(",\"unavailable_after\":").append(finalUnavailable).append(",\"variant\":").append(variant).append(",\"before\":\"").append(hex(before)).append("\",\"prepared\":\"").append(hex(prep)).append("\",\"after\":\"").append(hex(after)).append("\",\"sounds\":[").append(result.sounds).append("],\"draws\":[").append(result.drawRecords).append("],\"name\":\"").append(name).append("\",\"difficulty\":").append(difficulty).append(",\"npc_denominator\":").append(npc).append(",\"currency_before\":").append(currency).append(",\"currency_after\":").append((int)h.pointer(0x442788)).append(",\"share\":").append(share).append(",\"total\":").append(total).append(",\"count\":").append(count).append(",\"enemy_count_after\":").append(h.word(0x43874a)).append(",\"npc_after\":").append(h.word(0x44257e)).append(",\"xp_after_close\":").append((int)h.pointer(0x443392)).append(",\"before_changes\":").append(changes(party,before)).append(",\"prep_changes\":").append(changes(before,prep)).append(",\"close_changes\":").append(changes(prep,after)).append(",\"prep_sha256\":\"").append(digest(prep)).append("\",\"after_sha256\":\"").append(digest(after)).append("\",\"enemies\":\"").append(hex(enemies)).append("\",\"enemy_changes\":").append(changes(enemies,records)).append(",\"crate_changes\":").append(changes(new byte[22000],crate)).append(",\"crate_sha256\":\"").append(digest(crate)).append("\",\"seed_before\":").append(rngBefore).append(",\"seed_after\":").append(result.seed).append(",\"rng_draws\":").append(result.draws-drawsBefore).append("}");
        } finally {h.cpu.dispose();}
    }
    public void run() throws Exception {
        root=Path.of(getScriptArgs()[0]);Path original=root.resolve("NewCenturions/TheNewCenturions"),scenario=original.resolve("Scenarios/The Thunder of Guns/0");
        items=Files.readAllBytes(original.resolve("Data Files/Data ID"));scenarioItems=Files.readAllBytes(scenario.resolve("Data NI"));races=Files.readAllBytes(original.resolve("Data Files/Data Race"));monsters=Files.readAllBytes(scenario.resolve("Data MD"));properties=Files.readAllBytes(original.resolve("Data Files/Data Swamp BD"));party=new byte[6*3138];String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};for(int i=0;i<6;i++)System.arraycopy(Files.readAllBytes(original.resolve("Warriors/"+names[i])),0,party,i*3138,3138);
        for(long seed:new long[]{12,1,2,3,4,5,6,7})for(int variant=0;variant<8;variant++)availabilityProbe("availability-"+variant,seed,new byte[288],0,0,37,variant,false);
        StringBuilder identity=new StringBuilder();String[][] inputs={{"probe","tools/ghidra/VerifyBootyAvailability.java"},{"reward_probe","tools/ghidra/VerifyBattleRewards.java"},{"parent_probe","tools/ghidra/VerifyNestedVictory.java"},{"inventory_probe","tools/ghidra/VerifyStoryInventory.java"},{"harness","tools/ghidra/ActionOracleHarness.java"},{"sane_boundary","tools/ghidra/SaneBoundary.java"},{"mapped_binary","analysis/ghidra/startup/code1-mapped.bin"},{"items","NewCenturions/TheNewCenturions/Data Files/Data ID"},{"scenario_items","NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data NI"},{"races","NewCenturions/TheNewCenturions/Data Files/Data Race"},{"properties","NewCenturions/TheNewCenturions/Data Files/Data Swamp BD"}};
        for(String[] input:inputs){if(identity.length()>0)identity.append(',');identity.append('"').append(input[0]).append("_sha256\":\"").append(ActionOracleHarness.sha256(root.resolve(input[1]))).append('"');}
        for(int i=0;i<6;i++)identity.append(",\"warrior").append(i).append("_sha256\":\"").append(ActionOracleHarness.sha256(original.resolve("Warriors/"+names[i]))).append('"');
        Files.writeString(root.resolve("evidence/fixtures/booty-availability.json"),"{\"authority\":\"Original complete booty reward preparation105748..1064c2 and native Close107e52 through return. Six original warriors receive explicit HP and condition90 variants, incoming signed byte counters127/0/-1/-128/7/1/100/5, eight seeds, one controlled zero enemy, no pending level gain. Full party before/prepared/after, crate and enemy receipts, signed unavailable counter before/prepared/after and ordered RNG/audio captured. Original recount resets then counts HP50<1 or condition90<0 before clearing positive condition90. Remaining presentation and bounded SANE services explicitly modeled. No live victory observation.\",\"identity\":{"+identity+"},\"warriors\":\""+hex(party)+"\",\"cases\":["+cases+"]}\n");
        println("64 original booty availability cases exported");
    }
}
