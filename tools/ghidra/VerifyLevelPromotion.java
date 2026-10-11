import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;

/** Native booty Close, profile promotion and unchanged-spell Done/Yes. */
public class VerifyLevelPromotion extends VerifyBattleRewards {
    byte[] profiles,attacks,treasures;boolean registered;long libraryDraws;StringBuilder calls;
    void call(String name,int a,int b){if(calls.length()>0)calls.append(',');calls.append("[\"").append(name).append("\",").append(a).append(',').append(b).append(']');}
    void loadProfile(ActionOracleHarness h){int race=h.word(h.sp()+4),profile=h.word(h.sp()+6);if(race!=0)h.write(0x41f85a,Arrays.copyOfRange(races,(race-1)*408,race*408));if(profile!=0)h.write(0x41f9f2,Arrays.copyOfRange(profiles,(profile-1)*576,profile*576));h.ret();}
    void execute(ActionOracleHarness h,RunResult result,boolean wholeBooty)throws Exception {
        SaneBoundary sane=new SaneBoundary(h.cpu,currentProgram);
        for(int step=0;step<1500000;step++){
            long pc=h.pc(),sp=h.sp();if(pc==0x900000)return;
            if(pc==0x1064c2){h.jump(0x107e52);continue;}
            if(pc==0x1380ac)call("getexp",h.word(sp+4),h.word(sp+6));
            if(pc==0x12ed42)call("levelup",h.word(sp+4),h.word(sp+6));
            if(pc==0x400080){libraryDraws++;call("library_rand",0,0);}
            if(pc==0x4d2){long a=h.cpu.readRegister("D0").longValue(),b=h.cpu.readRegister("D1").longValue();h.cpu.writeRegister("D0",(a*b)&0xffffffffL);h.ret();continue;}
            if(pc==0x13a7c4){loadProfile(h);continue;}
            if(pc==0x13a6d4||pc==0x1469f8){h.cpu.writeRegister("D0",registered?1:0);h.ret();continue;}
            if(pc==0x12ef50){h.jump(0x12f048);continue;}
            if(pc==0x15a9cc){call("spell_done",h.word(0x447256),h.word(0x447250));}
            if(pc==0x15aa2c){h.pop(8);h.cpu.writeRegister("A2",0x600000);h.jump(0x15ac7c);continue;}
            if(pc==0x15acf4){h.pop(4);h.value(0x4434b2,2,1);h.jump(0x15ad3e);continue;}
            if(pc==0x14cff6){call("question",h.word(sp+4),h.word(sp+6));h.cpu.writeRegister("D0",1);h.ret();continue;}
            if(pc==0x130afe){loadItem(h);continue;}if(pc==0x153ca4){result.rng(h);continue;}if(pc==0x133a18||pc==0x16474a){call("sound",h.word(sp+4),0);result.sound(h);continue;}
            if(pc==0x400038){h.cpu.writeRegister("D0",(short)Math.abs(h.word(sp+4)));h.ret();continue;}
            if(pc==0x1046e0||pc==0x13e13c||pc==0x13154c||pc==0x13a2c4||pc==0x12382a||pc==0x160fb2||pc==0x158454||pc==0x1205f8||pc==0x105572||pc==0x1382a8||pc==0x13b56c||pc==0x153d56||pc==0x14ce2a){h.ret();continue;}
            int op=h.word(pc)&65535;if(op==0xa9eb){sane.execute();continue;}
            if(op==0xa874){h.value(h.pointer(sp),4,0x600000);h.pop(4);h.jump(pc+2);continue;}
            if(op==0xaa19||op==0xaa1a){h.write(h.pointer(sp),new byte[6]);h.pop(4);h.jump(pc+2);continue;}
            if(op==0xa991){h.value(h.pointer(sp),2,1);h.pop(8);h.jump(pc+2);continue;}
            if(op==0xa8a7){long rect=h.pointer(sp+8);h.write(rect,h.read(sp+4,4));h.write(rect+4,h.read(sp,4));h.pop(12);h.jump(pc+2);continue;}
            if(op==0xa939){h.pop(6);h.jump(pc+2);continue;}if(op==0xaa1c||op==0xa93c||op==0xa873||op==0xa914||op==0xaa14||op==0xaa15||op==0xa983){h.pop(4);h.jump(pc+2);continue;}if(op==0xa937||op==0xa032){h.jump(pc+2);continue;}if(op==0xa89b){h.pop(4);h.jump(pc+2);continue;}if(op==0xaa1e){h.value(sp+2,4,0);h.pop(2);h.jump(pc+2);continue;}
            if(!h.cpu.step(monitor))throw new Exception("Promotion PC "+Long.toHexString(pc)+": "+h.cpu.getLastError());
        }throw new Exception("Promotion instruction budget");
    }
    void closeCase(int member,int level,int remaining,int health,boolean accepted,long librarySeed)throws Exception {
        ActionOracleHarness h=frame();RunResult result=new RunResult(12);registered=accepted;libraryDraws=0;calls=new StringBuilder();
        try{
            byte[] before=party.clone();for(int i=0;i<6;i++){ByteBuffer.wrap(before).putInt(i*3138+0x9ce,1000000);before[i*3138+0x9df]=(byte)(i%2);}
            set(before,member*3138+0x3a,level);set(before,member*3138+0x50,health);ByteBuffer.wrap(before).putInt(member*3138+0x9ce,remaining);
            common(h,before);h.write(0x439d3c,new byte[28800]);h.write(0x440edc,new byte[5760]);h.write(0x443788,attacks);h.value(0x442788,4,937);h.value(0x3ffe60,4,librarySeed);h.cpu.writeRegister("A5",0x400000);h.value(0x43957e,2,0);h.value(0x439596,1,0);h.value(0x442560,2,0);
            h.enter(0x105748,0x6ffe00,0x900000);h.value(0x6ffe04,2,0);h.value(0x6ffe06,2,0);execute(h,result,true);byte[] after=h.read(0x448aa0,party.length);
            if(cases.length()>0)cases.append(',');cases.append("{\"kind\":\"close\",\"member\":").append(member).append(",\"level\":").append(level).append(",\"remaining\":").append(remaining).append(",\"health\":").append(health).append(",\"registered\":").append(accepted).append(",\"before_changes\":").append(changes(party,before)).append(",\"after_changes\":").append(changes(before,after)).append(",\"after_sha256\":\"").append(digest(after)).append("\",\"library_seed_before\":").append(librarySeed).append(",\"library_seed_after\":").append(h.pointer(0x3ffe60)).append(",\"library_draws\":").append(libraryDraws).append(",\"seed_after\":").append(result.seed).append(",\"rng_draws\":").append(result.draws).append(",\"currency_after\":").append(h.pointer(0x442788)).append(",\"calls\":[").append(calls).append("]}");
        }finally{h.cpu.dispose();}
    }
    void chain(long initial)throws Exception {
        ActionOracleHarness h=frame();RunResult result=new RunResult(initial);registered=true;libraryDraws=0;calls=new StringBuilder();
        try{
            byte[] before=party.clone();for(int i=0;i<6;i++)before[i*3138+0x9df]=1;
            common(h,before);h.write(0x439d3c,new byte[28800]);h.write(0x440edc,new byte[5760]);h.write(0x443788,attacks);h.value(0x442788,4,937);h.value(0x3ffe60,4,1);h.cpu.writeRegister("A5",0x400000);h.value(0x43957e,2,0);h.value(0x439596,1,0);h.value(0x442560,2,0);StringBuilder stages=new StringBuilder();
            for(int stage=0;stage<4;stage++){
                byte[] table=stage==1?Arrays.copyOfRange(treasures,8*88,9*88):new byte[88];if(stage!=1)ByteBuffer.wrap(table).putInt(80,stage==2?8000:stage==3?2000:8000);
                h.write(0x44331c,table);if(stage!=1)for(int i=0;i<6;i++)h.value(0x448aa0+i*3138+0x9df,1,1);
                h.enter(0x105748,0x6ffe00,0x900000);h.value(0x6ffe04,2,stage==1?1:12);h.value(0x6ffe06,2,0);execute(h,result,true);
                if(stages.length()>0)stages.append(',');stages.append("{\"stage\":").append(stage).append(",\"table\":\"").append(hex(table)).append("\",\"warriors\":\"").append(hex(h.read(0x448aa0,party.length))).append("\",\"crate\":\"").append(hex(h.read(0x41fc74,22000))).append("\",\"library_seed\":").append(h.pointer(0x3ffe60)).append(",\"library_draws\":").append(libraryDraws).append(",\"seed\":").append(result.seed).append(",\"draws\":").append(result.draws).append(",\"currency\":").append(h.pointer(0x442788)).append('}');
            }
            if(cases.length()>0)cases.append(',');cases.append("{\"kind\":\"chain\",\"seed_before\":").append(initial).append(",\"before_changes\":").append(changes(party,before)).append(",\"stages\":[").append(stages).append("],\"calls\":[").append(calls).append("]}");
        }finally{h.cpu.dispose();}
    }
    public void run()throws Exception{
        root=Path.of(getScriptArgs()[0]);Path original=root.resolve("NewCenturions/TheNewCenturions"),scenario=original.resolve("Scenarios/The Thunder of Guns/0");items=Files.readAllBytes(original.resolve("Data Files/Data ID"));scenarioItems=Files.readAllBytes(scenario.resolve("Data NI"));races=Files.readAllBytes(original.resolve("Data Files/Data Race"));profiles=Files.readAllBytes(original.resolve("Data Files/Data MOS"));attacks=Files.readAllBytes(original.resolve("Data Files/Data S"));treasures=Files.readAllBytes(scenario.resolve("Data TD"));party=new byte[6*3138];String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};for(int i=0;i<6;i++)System.arraycopy(Files.readAllBytes(original.resolve("Warriors/"+names[i])),0,party,i*3138,3138);
        for(boolean accepted:new boolean[]{false,true})for(int member=0;member<6;member++)for(int level:new int[]{0,3,4,5,6,7,9})closeCase(member,level,-1,20,accepted,1);
        for(int member=0;member<6;member++)for(int health:new int[]{0,-3,-10})closeCase(member,3,-790,health,true,0xffffffffL);
        for(int remaining:new int[]{0,1,-100000000})closeCase(1,3,remaining,20,true,123);
        chain(12);chain(1);
        Files.writeString(root.resolve("evidence/fixtures/level-promotion.json"),"{\"authority\":\"Original full ordinary booty105748 preparation and Close107e52..return, actual getexp1380ac, levelup12ed42 and updatespec161d86, with exact original Race/MOS/S records. Levelup information drawing12ef50..12f048 is skipped; it has no allocation loop. Native scanselect initialization and unchanged-spell normalization15a9cc..15aa2c/15ac7c..15acf4, native Done/Yes15ad3e..15ad86 and suffix15b1cc..return execute; UI is controlled to Done and question17 Yes. Actual separate C rand thunk400080/CODE3+2b50 executes with unsigned multiply service4d2 intercepted; QuickDraw service and finite SANE modeled. Registration services use explicit accepted boolean. Native firstt/info rendering/UI drawing services intercepted. Full original warrior records compared. No user stat allocation, alternate spell choices, registration serial or live battle claim.\",\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyLevelPromotion.java"))+"\",\"parent_probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyBattleRewards.java"))+"\",\"harness_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/ActionOracleHarness.java"))+"\",\"sane_boundary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/SaneBoundary.java"))+"\",\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\",\"code3_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/extracted/resources/106/434f4445_3.bin"))+"\",\"warriors\":\""+hex(party)+"\",\"cases\":["+cases+"]}\n");println("Native level promotion cases exported");
    }
}
