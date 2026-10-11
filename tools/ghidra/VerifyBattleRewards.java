import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.security.MessageDigest;

/** Native combat reward preparation and ordinary Close with no pending level gain. */
public class VerifyBattleRewards extends VerifyNestedVictory {
    Path root; byte[] battleTable; StringBuilder cases=new StringBuilder();
    String changes(byte[] a,byte[] b){StringBuilder s=new StringBuilder("[");for(int i=0;i<a.length;i++)if(a[i]!=b[i]){if(s.length()>1)s.append(',');s.append('[').append(i).append(',').append(b[i]&255).append(']');}return s.append(']').toString();}
    String digest(byte[] b)throws Exception{return hex(MessageDigest.getInstance("SHA-256").digest(b));}
    void set(byte[] b,int offset,int value){ByteBuffer.wrap(b).putShort(offset,(short)value);}
    void probe(String name,long initial,byte[] input,int difficulty,int npc,int currency,int variant,boolean authored)throws Exception {
        ActionOracleHarness h=frame();RunResult result=new RunResult(initial);events=new StringBuilder();
        try {
            byte[] before=party.clone();for(int i=0;i<6;i++){int o=i*3138;before[o+0x9df]=(byte)(variant==1&&i%2==0?0:1);before[o+0x9de]=1;set(before,o+0x5e,3);set(before,o+0x60,-4);set(before,o+0x62,-2);set(before,o+0x64,5);set(before,o+0x90,variant==2&&i<2?-1:3);set(before,o+0x50,variant==3?0:20);ByteBuffer.wrap(before).putInt(o+0x9ce,1000000).putInt(o+0x9d2,variant==4?-300000:16777217);}
            common(h,before);h.write(0x439d3c,new byte[100*288]);h.write(0x440edc,new byte[20*288]);h.write(0x4085b8,terrain());h.value(0x442780,4,5);h.value(0x442784,4,5);h.write(0x4537fe,Arrays.copyOfRange(items,0,30000));h.write(0x46978e,Arrays.copyOfRange(items,90000,120000));h.value(0x43957e,2,0);h.value(0x438776,2,1);
            for(int i=0;i<6;i++){h.value(0x44709a+i*4,2,20+i);h.value(0x44709c+i*4,2,20);h.value(0x437a6e+i*2,2,1155);}h.write(0x4470b8,new byte[220]);for(int i=0;i<110;i++)h.value(0x4470b8+i*2,2,-1);for(int i=0;i<6;i++)h.value(0x4470b8+(i+1)*2,2,i);
            int count;
            if(authored){println("Generating "+name);battle=input;generate(h,result);count=h.word(0x43874a);for(int i=0;i<count;i++)h.value(0x439d3c+i*288+0x110,2,0);}
            else {count=input.length/288;h.write(0x439d3c,input);h.value(0x43874a,2,count);}
            byte[] enemies=h.read(0x439d3c,count*288);h.value(0x442576,2,difficulty);h.value(0x44257e,2,npc);h.value(0x442788,4,currency);h.value(0x44374c,2,count);long rngBefore=result.seed;int drawsBefore=result.draws;
            h.enter(0x105748,0x6ffe00,0x900000);h.value(0x6ffe04,2,0);h.value(0x6ffe06,2,0);SaneBoundary sane=new SaneBoundary(h.cpu,currentProgram);boolean prepared=false,closed=false;int share=0,total=0;byte[] prep=null,crate=null,records=null;
            for(int step=0;step<1500000;step++) {
                long pc=h.pc(),sp=h.sp();if(pc==0x900000){closed=true;break;}
                if(pc==0x1064c2){prepared=true;share=(int)h.pointer(h.cpu.readRegister("A6").longValue()-0x96);total=(int)h.pointer(0x443392);prep=h.read(0x448aa0,party.length);crate=h.read(0x41fc74,22000);records=h.read(0x439d3c,count*288);h.jump(0x107e52);continue;}
                if(pc==0x1380ac||pc==0x12ed42)throw new Exception("Pending level gain escaped reward scope");
                if(pc==0x130afe){loadItem(h);continue;}if(pc==0x153ca4){result.rng(h);continue;}if(pc==0x133a18||pc==0x16474a){result.sound(h);continue;}if(pc==0x400038){h.cpu.writeRegister("D0",(short)Math.abs(h.word(sp+4)));h.ret();continue;}
                if(pc==0x1046e0||pc==0x13e13c||pc==0x13a7c4||pc==0x13154c||pc==0x13a2c4||pc==0x12382a||pc==0x160fb2||pc==0x158454||pc==0x1205f8||pc==0x105572||pc==0x1382a8){h.ret();continue;}
                int op=h.word(pc)&65535;if(op==0xa9eb){sane.execute();continue;}
                if(op==0xa8a7){long rect=h.pointer(sp+8);h.write(rect,h.read(sp+4,4));h.write(rect+4,h.read(sp,4));h.pop(12);h.jump(pc+2);continue;}
                if(op==0xa939){h.pop(6);h.jump(pc+2);continue;}if(op==0xaa1c||op==0xa93c||op==0xa873||op==0xa914){h.pop(4);h.jump(pc+2);continue;}if(op==0xa937){h.jump(pc+2);continue;}if(op==0xa89b){h.pop(4);h.jump(pc+2);continue;}if(op==0xaa1e){h.value(sp+2,4,0);h.pop(2);h.jump(pc+2);continue;}
                if(!h.cpu.step(monitor))throw new Exception(name+" reward at "+Long.toHexString(pc)+": "+h.cpu.getLastError());
            }
            if(!prepared||!closed)throw new Exception("Reward budget");if(cases.length()>0)cases.append(',');byte[] after=h.read(0x448aa0,party.length);
            cases.append("{\"name\":\"").append(name).append("\",\"difficulty\":").append(difficulty).append(",\"npc_denominator\":").append(npc).append(",\"currency_before\":").append(currency).append(",\"currency_after\":").append((int)h.pointer(0x442788)).append(",\"share\":").append(share).append(",\"total\":").append(total).append(",\"count\":").append(count).append(",\"enemy_count_after\":").append(h.word(0x43874a)).append(",\"npc_after\":").append(h.word(0x44257e)).append(",\"xp_after_close\":").append((int)h.pointer(0x443392)).append(",\"before_changes\":").append(changes(party,before)).append(",\"prep_changes\":").append(changes(before,prep)).append(",\"close_changes\":").append(changes(prep,after)).append(",\"prep_sha256\":\"").append(digest(prep)).append("\",\"after_sha256\":\"").append(digest(after)).append("\",\"enemies\":\"").append(hex(enemies)).append("\",\"enemy_changes\":").append(changes(enemies,records)).append(",\"crate_changes\":").append(changes(new byte[22000],crate)).append(",\"crate_sha256\":\"").append(digest(crate)).append("\",\"seed_before\":").append(rngBefore).append(",\"seed_after\":").append(result.seed).append(",\"rng_draws\":").append(result.draws-drawsBefore).append("}");
        } finally {h.cpu.dispose();}
    }
    public void run()throws Exception {
        root=Path.of(getScriptArgs()[0]);Path original=root.resolve("NewCenturions/TheNewCenturions"),scenario=original.resolve("Scenarios/The Thunder of Guns/0");items=Files.readAllBytes(original.resolve("Data Files/Data ID"));scenarioItems=Files.readAllBytes(scenario.resolve("Data NI"));races=Files.readAllBytes(original.resolve("Data Files/Data Race"));monsters=Files.readAllBytes(scenario.resolve("Data MD"));battleTable=Files.readAllBytes(scenario.resolve("Data BD"));properties=Files.readAllBytes(original.resolve("Data Files/Data Swamp BD"));party=new byte[6*3138];String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};for(int i=0;i<6;i++)System.arraycopy(Files.readAllBytes(original.resolve("Warriors/"+names[i])),0,party,i*3138,3138);
        for(long seed:new long[]{12,1}){
            for(int level=0;level<=21;level++){byte[] m=new byte[288];set(m,0,level);set(m,0x112,level%3==0?1500:50);set(m,0x54,level%2==0?1500:-45);set(m,0x10e,level%2);m[0x7f]=(byte)(level%3==1?1:0);set(m,0x3a,13);probe("level-"+level,seed,m,0,0,70,0,false);}
            for(int variant=0;variant<5;variant++){byte[] m=new byte[288];set(m,0,-1);set(m,0x112,-32768);set(m,0x54,-32768);set(m,0x10e,1);m[0x7f]=1;set(m,0x3a,-1);probe("signed-"+variant,seed,m,0,variant==1?2:variant==3?0:-1,Integer.MAX_VALUE,variant,false);}
            for(int difficulty:new int[]{0,2,-1,32767}){byte[] m=new byte[288];set(m,0,14);set(m,0x112,150);set(m,0x10e,150);set(m,0x54,1500);set(m,0x26,804);set(m,0x28,1);set(m,0x3a,100);probe("drops-"+difficulty,seed,m,difficulty,0,10,0,false);}
            for(int id:new int[]{87,107,117})probe("battle-"+id,seed,Arrays.copyOfRange(battleTable,id*1372,(id+1)*1372),0,0,37,0,true);
        }
        Files.writeString(root.resolve("evidence/fixtures/battle-rewards.json"),"{\"authority\":\"Original booty105748 initialization, native bodycount, per-enemy cash/XP/drop arithmetic and actual randomitem/SANE; capture at1064c2, acknowledge presentation Close and resume107e52 to return. Numeric cases use explicit input records; authored battles87/107/117 use original generation11e0fe..11eb86 on controlled terrain, with HP110 forced0 only after generation. Rewards preserve participation until Close, banked party currency is distinct from warrior wallet, full six warrior records and count enemy records plus complete1000-item crate compared. Pending level gains are excluded by positive remainingXP and require a separate player allocation boundary. SANE is the previously modeled finite arithmetic service, not native ROM. Drawing/profile output/sound rendering intercepted; no live combat or whole campaign claim.\",\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyBattleRewards.java"))+"\",\"parent_probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyNestedVictory.java"))+"\",\"inventory_probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyStoryInventory.java"))+"\",\"harness_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/ActionOracleHarness.java"))+"\",\"sane_boundary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/SaneBoundary.java"))+"\",\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\",\"warriors\":\""+hex(party)+"\",\"cases\":["+cases+"]}\n");println("Original battle rewards exported: "+cases.length());
    }
}
