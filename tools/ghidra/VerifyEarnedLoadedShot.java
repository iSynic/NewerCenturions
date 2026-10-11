import java.nio.file.Files;
import java.nio.file.Path;
import com.google.gson.*;

/** Original earned loaded secondary shots and complete turn continuation. */
public class VerifyEarnedLoadedShot extends VerifyEarnedCombatMedical {
    JsonArray shotEvents=new JsonArray(),collisionTests=new JsonArray();
    JsonObject preKill;
    JsonArray phaseReceipts=new JsonArray();java.util.ArrayList<JsonObject> activePhases=new java.util.ArrayList<>();
    int soundCount(RunResult r){return JsonParser.parseString("["+r.sounds+"]").getAsJsonArray().size();}
    void phaseObserve(ActionOracleHarness h,RunResult r,long pc)throws Exception {
        for(int i=activePhases.size()-1;i>=0;i--){JsonObject e=activePhases.get(i);if(pc==e.get("return_address_value").getAsLong()&&h.sp()==e.get("entry_sp_value").getAsLong()+4){e.addProperty("draws_after",r.draws);e.addProperty("sound_count_after",soundCount(r));e.addProperty("d0_return_word",(short)h.cpu.readRegister("D0").intValue());activePhases.remove(i);}}
        if(pc==0x151022||pc==0x15075a||pc==0x10d4fe||pc==0x125a3a||pc==0x1140ae){JsonObject e=new JsonObject();e.addProperty("address",Long.toHexString(pc));e.addProperty("entry_sp_value",h.sp());e.addProperty("return_address_value",h.pointer(h.sp()));e.addProperty("draws_before",r.draws);e.addProperty("sound_count_before",soundCount(r));e.addProperty("spell_id",(h.word(0x4435c6)+1)*1000+(h.word(0x4435c8)+1)*100+h.word(0x4435ca)+1);if(pc==0x15075a){JsonArray a=new JsonArray();for(int i=0;i<5;i++)a.add(h.word(h.sp()+4+2*i));e.add("arguments",a);}phaseReceipts.add(e);activePhases.add(e);}
    }
    @Override void saveTrace(JsonObject j,RunResult r){super.saveTrace(j,r);JsonArray nativeResolutions=new JsonArray();for(JsonElement element:phaseReceipts)if(element.getAsJsonObject().get("address").getAsString().equals("151022"))nativeResolutions.add(element);JsonArray resolutions=j.getAsJsonArray("resolutions");if(resolutions.size()!=nativeResolutions.size())throw new IllegalStateException("Resolver phase receipt count");for(int i=0;i<resolutions.size();i++){JsonObject receipt=nativeResolutions.get(i).getAsJsonObject(),resolution=resolutions.get(i).getAsJsonObject();resolution.add("sound_count_before",receipt.get("sound_count_before"));resolution.add("sound_count_after",receipt.get("sound_count_after"));}if(!activePhases.isEmpty())throw new IllegalStateException("Unreturned native phase");j.add("phase_receipts",phaseReceipts.deepCopy());}

    int priorKilledCount,counterPrior;long previousPc;boolean trackingCounter;
    @Override void initializeActual(ActionOracleHarness h,JsonObject s)throws Exception {
        super.initializeActual(h,s);h.write(0x442742,bytes(s.getAsJsonArray("party_mask")));h.value(0x408570,2,5);h.value(0x408572,2,5);
        h.value(0x44374c,2,priorKilledCount);
        h.value(0x44255c,1,s.getAsJsonObject("combat").getAsJsonObject("round_scripts").get("counter").getAsInt());h.value(0x4427a6,2,0);counterPrior=h.word(0x4427a6);trackingCounter=true;
    }
    @Override void observe(ActionOracleHarness h,RunResult r,long pc)throws Exception {
        phaseObserve(h,r,pc);
        if(trackingCounter&&counterPrior!=h.word(0x4427a6)){JsonObject e=new JsonObject();e.addProperty("boundary","native inactivity counter change");e.addProperty("previous_address",Long.toHexString(previousPc));e.addProperty("next_address",Long.toHexString(pc));e.addProperty("before",counterPrior);e.addProperty("after",h.word(0x4427a6));shotEvents.add(e);counterPrior=h.word(0x4427a6);}previousPc=pc;
        if(pc!=0x159192)super.observe(h,r,pc);
        if(pc==0x13a2c4||pc==0x10e6b6||pc==0x10ed92||pc==0x10e844||pc==0x159192||pc==0x14c6c2||pc==0x11b01c||pc==0x11b3e2||pc==0x11b488||pc==0x11ae74||pc==0x11af72||pc==0x12e4e4||pc==0x12eb32||pc==0x125a3a||pc==0x125d34||pc==0x125df6||pc==0x1140ae||pc==0x1270b6||pc==0x116b6a||pc==0x116b86||pc==0x117338){
            JsonObject e=new JsonObject();e.addProperty("address",Long.toHexString(pc));e.addProperty("sp",Long.toHexString(h.sp()));e.addProperty("frame",Long.toHexString(h.cpu.readRegister("A6").longValue()));e.addProperty("return",Long.toHexString(h.pointer(h.sp())));e.addProperty("draws",r.draws);e.addProperty("sound_count",soundCount(r));e.addProperty("actor",h.word(0x4435d2));e.addProperty("cursor",h.word(0x447194));e.addProperty("round",h.read(0x44255c,1)[0]&255);e.addProperty("target_hp",h.word(0x439d3c+0x110));e.addProperty("loaded_rounds",h.word(0x448aa0+4*3138+0x134));e.addProperty("killed_count",h.word(0x44374c));e.addProperty("health_effect",h.read(0x443760,1)[0]&255);e.addProperty("last_body",h.word(0x408574));e.addProperty("mode",h.word(0x438760));e.addProperty("shift_x",h.pointer(0x442780));e.addProperty("shift_y",h.pointer(0x442784));e.addProperty("party_positions",hex(h.read(0x44709a,24)));e.addProperty("all_monster_positions",hex(h.read(0x446f0a,400)));e.addProperty("firearm_mode",h.read(0x43565e,1)[0]&255);if(pc==0x13a2c4||pc==0x10e6b6||pc==0x12e4e4||pc==0x14c6c2||pc==0x1140ae){JsonArray a=new JsonArray();for(int i=0;i<2;i++)a.add(h.word(h.sp()+4+2*i));e.add("arguments",a);}shotEvents.add(e);if(pc==0x12e4e4)preKill=completeSnap(h,r);
        }
    }
    @Override JsonObject completeSnap(ActionOracleHarness h,RunResult r)throws Exception {
        JsonObject j=super.completeSnap(h,r);j.addProperty("round_script_counter",(int)h.read(0x44255c,1)[0]);j.addProperty("inactivity_counter",h.word(0x4427a6));j.addProperty("killed_count",h.word(0x44374c));j.addProperty("selection_base",h.word(0x4471e8));j.addProperty("learned_marker",h.read(0x435663,1)[0]&255);j.addProperty("last_party_index",h.read(0x44273a,1)[0]&255);j.addProperty("party_mask",hex(h.read(0x442742,6)));j.addProperty("shift_x",(int)h.pointer(0x442780));j.addProperty("shift_y",(int)h.pointer(0x442784));j.addProperty("view_x",h.word(0x408570));j.addProperty("view_y",h.word(0x408572));j.addProperty("display_profile",h.word(0x439582));j.addProperty("sound_count",soundCount(r));j.add("ordered_sounds",JsonParser.parseString("["+r.sounds+"]"));j.add("shot_events",shotEvents.deepCopy());j.add("collision_tests",collisionTests.deepCopy());return j;
    }
    @Override void showresultsSound(ActionOracleHarness h,RunResult r)throws Exception {
        if(h.word(h.sp()+6)!=-11){super.showresultsSound(h,r);return;}
        int body=h.word(h.sp()+4);if(body<10||body>=110)throw new Exception("Unproved party death audio boundary");
        String[] names={"D0","D1","D2","D3","D4","D5","D6","D7","A0","A1","A2","A3","A4","A5","A6","SP","PC","XF","NF","ZF","VF","CF"};java.math.BigInteger[] saved=new java.math.BigInteger[names.length];for(int i=0;i<names.length;i++)saved[i]=h.cpu.readRegister(names[i]);
        JsonObject e=new JsonObject();e.addProperty("boundary","native NPC result minus11 audio");e.addProperty("address","158ee0");e.addProperty("body",body);e.addProperty("draws_before",r.draws);e.addProperty("sound_count_before",soundCount(r));e.addProperty("descriptor_sound",h.word(0x439d3c+(body-10)*288+0x6a));
        h.write(0x6dc000,new byte[8192]);h.cpu.writeRegister("SP",0x6dd000);h.cpu.writeRegister("D3",body);h.jump(0x158ee0);boolean done=false;
        for(int i=0;i<20;i++){long pc=h.pc();if(pc==0x158efa){done=true;break;}if(pc==0x133a18){r.sound(h);continue;}if(!h.cpu.step(monitor))throw new Exception("NPC result audio at "+Long.toHexString(pc)+": "+h.cpu.getLastError());}
        if(!done||h.sp()!=0x6dd000)throw new Exception("NPC result audio instruction/stack boundary");e.addProperty("draws_after",r.draws);e.addProperty("sound_count_after",soundCount(r));medicalEvents.add(e);for(int i=0;i<names.length;i++)h.cpu.writeRegister(names[i],saved[i]);
    }
    void shootInputs(ActionOracleHarness h,RunResult r)throws Exception {
        h.enter(0x117be2,0x6ffe00,0x116b6a);execute(h,r,0x117c46);h.jump(0x117c56);execute(h,r,0x117c6e);
        h.value(0x443318,2,activeBody);h.write(0x820000,new byte[512]);h.value(0x820000,4,0x820100);h.value(0x820020,4,0x820100);h.value(0x439404,4,0x820020);h.value(0x43945c,4,0x820020);
        h.jump(0x117e5e);execute(h,r,0x117eaa);h.jump(0x1186d2);execute(h,r,0x119238);execute(h,r,0x11936e);
        if(h.sp()!=h.cpu.readRegister("A6").longValue()-0x7e)throw new Exception("Shoot Combatchoice prepared frame");
    }
    void shotCase(JsonArray out,JsonObject s,int foundation)throws Exception {
        long seed=s.getAsJsonObject("random").get("seed").getAsLong();begin(seed);poisonEvents=new JsonArray();aidEvents=new JsonArray();warnings=new JsonArray();medicalEvents=new JsonArray();shotEvents=new JsonArray();collisionTests=new JsonArray();trackingCounter=false;preKill=null;phaseReceipts=new JsonArray();activePhases.clear();itemUse=false;secondaryStop=0;ActionOracleHarness h=frame();RunResult r=new RunResult(seed);r.draws=s.getAsJsonObject("random").get("draws").getAsInt();
        try{initializeActual(h,s);JsonObject c=new JsonObject();c.addProperty("foundation",foundation);c.addProperty("actor",activeBody);c.addProperty("target",10);c.add("before",completeSnap(h,r));shootInputs(h,r);c.add("prepared",completeSnap(h,r));h.value(0x44719e,2,h.word(0x446f0a));h.value(0x4471a0,2,h.word(0x446f0c));c.addProperty("target_x",h.word(0x44719e));c.addProperty("target_y",h.word(0x4471a0));h.jump(0x11b01c);execute(h,r,0x11b5b8);if(h.sp()!=h.cpu.readRegister("A6").longValue()-0x7e||h.word(0x4435c0)!=1)throw new Exception("Actual Shoot target frame/admission");c.add("targeted",completeSnap(h,r));h.jump(0x11ae6a);execute(h,r,0x116b6a);c.add("resolved",completeSnap(h,r));execute(h,r,0x117338);if(diagnostic)throw new Exception("Unproved native shot continuation "+diagnosticSpell);c.add("after",completeSnap(h,r));if(preKill!=null)c.add("pre_kill",preKill);saveTrace(c,r);out.add(c);println("Earned loaded shot foundation"+foundation);}finally{h.cpu.dispose();itemUse=false;secondaryStop=0;}
    }
    void reconstructInactivity(JsonObject f,JsonObject s,JsonObject receipt)throws Exception {
        JsonObject combat=s.getAsJsonObject("combat");if(combat.get("round").getAsInt()!=0||combat.getAsJsonObject("round_scripts").get("counter").getAsInt()!=1)throw new Exception("Unproved initial native inactivity counter");int turns=0;boolean started=false;
        for(JsonElement group:receipt.getAsJsonArray("ordered_events"))for(JsonElement event:group.getAsJsonArray())if(event.isJsonObject()){JsonObject e=event.getAsJsonObject();if(e.has("CombatStarted")){started=true;turns=0;}if(started&&e.has("TurnStarted")){if(e.getAsJsonObject("TurnStarted").get("round").getAsInt()!=0)throw new Exception("Unproved inactivity counter after round wrap");turns++;}}
        if(!started||turns==0)throw new Exception("Missing current battle no-wrap receipt");f.addProperty("source_reconstructed_inactivity_counter",0);f.addProperty("current_battle_round_zero_turn_receipts",turns);
    }
    JsonObject medicalCounterFoundation(Path root,String name,int expectedKilledCount)throws Exception {
        Path input=root.resolve("analysis/completion/"+name+".json"),receiptPath=root.resolve("analysis/completion/"+name+".receipt.json");JsonObject s=JsonParser.parseString(Files.readString(input)).getAsJsonObject().getAsJsonObject("session"),receipt=JsonParser.parseString(Files.readString(receiptPath)).getAsJsonObject();int killed=0;
        for(JsonElement group:receipt.getAsJsonArray("ordered_events"))for(JsonElement event:group.getAsJsonArray())if(event.isJsonObject()){JsonObject e=event.getAsJsonObject();if(e.has("CombatStarted"))killed=0;if(e.has("AttackResolved")){JsonObject a=e.getAsJsonObject("AttackResolved");if(a.get("defeated").getAsBoolean()&&a.get("target").getAsInt()>=10)killed++;}}
        if(killed!=expectedKilledCount)throw new Exception("Unproved supplemental medical prior killed count");JsonObject f=new JsonObject();f.addProperty("input",root.relativize(input).toString().replace('\\','/'));f.addProperty("digest",ActionOracleHarness.sha256(input));f.addProperty("receipt",root.relativize(receiptPath).toString().replace('\\','/'));f.addProperty("receipt_digest",ActionOracleHarness.sha256(receiptPath));f.addProperty("source_reconstructed_killed_count",killed);reconstructInactivity(f,s,receipt);f.add("session",s);return f;
    }
    void medicalCounterCase(JsonArray out,JsonObject foundation,int index,int target,String kind)throws Exception {
        shotEvents=new JsonArray();collisionTests=new JsonArray();trackingCounter=false;preKill=null;phaseReceipts=new JsonArray();activePhases.clear();priorKilledCount=foundation.get("source_reconstructed_killed_count").getAsInt();medicalCase(out,foundation.getAsJsonObject("session"),index,target,kind);
    }
    public void run()throws Exception {
        Path root=Path.of(getScriptArgs()[0]),original=root.resolve("NewCenturions/TheNewCenturions"),scenario=original.resolve("Scenarios/The Thunder of Guns/0");items=Files.readAllBytes(original.resolve("Data Files/Data ID"));scenarioItems=Files.readAllBytes(scenario.resolve("Data NI"));races=Files.readAllBytes(original.resolve("Data Files/Data Race"));mos=Files.readAllBytes(original.resolve("Data Files/Data MOS"));monsters=Files.readAllBytes(scenario.resolve("Data MD"));properties=Files.readAllBytes(original.resolve("Data Files/Data Swamp BD"));attacks=Files.readAllBytes(original.resolve("Data Files/Data S"));areas=Files.readAllBytes(original.resolve("Data Files/Data AD"));
        JsonObject identity=JsonParser.parseString(Files.readString(root.resolve("evidence/fixtures/earned-combat-medical.json"))).getAsJsonObject().getAsJsonObject("identity").deepCopy();identity.addProperty("earned_medical_probe_sha256",identity.get("probe_sha256").getAsString());identity.addProperty("probe_sha256",ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyEarnedLoadedShot.java")));identity.addProperty("earned_medical_fixture_sha256",ActionOracleHarness.sha256(root.resolve("evidence/fixtures/earned-combat-medical.json")));
        JsonArray foundations=new JsonArray(),cases=new JsonArray();for(int seed:new int[]{12,1}){Path input=root.resolve("analysis/completion/earned-loaded-shot-input-"+seed+".json");JsonObject s=JsonParser.parseString(Files.readString(input)).getAsJsonObject().getAsJsonObject("session"),f=new JsonObject();f.addProperty("input",root.relativize(input).toString().replace('\\','/'));f.addProperty("digest",ActionOracleHarness.sha256(input));f.add("session",s);Path receiptPath=root.resolve("analysis/completion/earned-loaded-shot-input-"+seed+".receipt.json");JsonObject receipt=JsonParser.parseString(Files.readString(receiptPath)).getAsJsonObject();priorKilledCount=0;for(JsonElement group:receipt.getAsJsonArray("ordered_events"))for(JsonElement event:group.getAsJsonArray()){if(!event.isJsonObject())continue;JsonObject e=event.getAsJsonObject();if(e.has("AttackResolved")&&e.getAsJsonObject("AttackResolved").get("defeated").getAsBoolean()){if(e.getAsJsonObject("AttackResolved").get("target").getAsInt()!=16)throw new Exception("Unexpected prior defeated body");priorKilledCount++;}}if(priorKilledCount!=1)throw new Exception("Unproved prior killed count");f.addProperty("receipt",root.relativize(receiptPath).toString().replace('\\','/'));f.addProperty("receipt_digest",ActionOracleHarness.sha256(receiptPath));f.addProperty("source_reconstructed_killed_count",priorKilledCount);reconstructInactivity(f,s,receipt);foundations.add(f);shotCase(cases,s,seed==12?0:1);}JsonArray medicalFoundations=new JsonArray(),medicalCases=new JsonArray();JsonObject medical12=medicalCounterFoundation(root,"earned-class28-medical-input-12",0),medical1=medicalCounterFoundation(root,"earned-class28-medical-input-1",1);medicalFoundations.add(medical12);medicalFoundations.add(medical1);medicalCounterCase(medicalCases,medical1,1,5,"actual");medicalCounterCase(medicalCases,medical1,1,-1,"cancel");medicalCounterCase(medicalCases,medical12,0,2,"range_reject");JsonObject out=new JsonObject();out.addProperty("authority","Two original complete earned loaded secondary165/ammunition280 shots on immutable actual full production-command Sessions, real enemy10 HP7/5 and original map/fields/initiative/RNG. Original selected Shoot admission/loading/prepattack4101, complete physical showspellinfo/getselectionbase/Pascal8 cleanup and caller saved D3/SP, complete real target helper11b01c through confirmed Execute/full cast/resist/damage/actual killbody and full outer AP/getup/round/reduce/collide continuation. Scheduled story remains embedded. Three separate new supplemental actual class28 medical normal/cancel/range cases restore represented signed-byte round-script counter1, independently reconstructed inactivity WORD0 and prior killed counters from their exact receipts; earlier frozen medical family projected public zero-based round0 into the native round-script BYTE and is not complete-state counter proof. Only declared original presentation/event/audio/SANE boundaries are acknowledged; no old family edits or live-game/campaign completion claim.");out.add("identity",identity);out.add("foundations",foundations);out.add("cases",cases);out.add("medical_counter_foundations",medicalFoundations);out.add("medical_counter_cases",medicalCases);Files.writeString(root.resolve("evidence/fixtures/earned-loaded-shot.json"),out+"\n");println("Earned loaded shot exported2shots3medicalcountercases");
    }
    void execute(ActionOracleHarness h,RunResult result,long stop) throws Exception {
        SaneBoundary sane=new SaneBoundary(h.cpu,currentProgram);boolean done=false;
        for(int step=0;step<1500000;step++) {
            long pc=h.pc(),sp=h.sp();
            if(pc==0x1363e4&&h.word(sp+4)==0){diagnostic=true;diagnosticSpell=(h.word(0x4435c6)+1)*1000+(h.word(0x4435c8)+1)*100+h.word(0x4435ca)+1;done=true;break;}
            observe(h,result,pc);if(pc==0x10ed92){h.jump(0x10f168);continue;}if(pc==0x10e844){h.jump(0x10ec34);continue;}if(pc==stop||(secondaryStop!=0&&pc==secondaryStop)){done=true;break;}
            if(pc==0x15fa20&&!characterInfo){characterRuleInfo(h,result);continue;}
            if(pc==0x14d21a&&!confirming){confirmedDialog(h,result);continue;}
            if(pc==0x11f1a8&&!updating){updateRuleWork(h,result);continue;}
            if(pc==0x13ba6c||pc==0x13bac6){profileFile=pc==0x13ba6c?0:1;profileCursor=0;h.value(0x4434a6,4,0x600000+profileFile);h.ret();continue;}
            if(pc==0x400078){profileCursor=(int)h.pointer(sp+8);h.cpu.writeRegister("D0",0);h.ret();continue;}
            if(pc==0x400048){int amount=(int)(h.pointer(sp+8)*h.pointer(sp+12));byte[] source=profileFile==0?races:mos;h.write(h.pointer(sp+4),java.util.Arrays.copyOfRange(source,profileCursor,profileCursor+amount));profileCursor+=amount;h.cpu.writeRegister("D0",h.pointer(sp+12));h.ret();continue;}
            if(pc==0x400068){h.cpu.writeRegister("D0",0);h.ret();continue;}
            if(pc==0x400090||pc==0x400098||pc==0x400168||pc==0x400180){h.cpu.writeRegister("D0",0);h.ret();continue;}
            if(pc==0x400178){stringBoundary(h);continue;}
            if(pc==0x1382d0){h.ret();continue;}
            if(pc==0x1620ea){JsonObject e=new JsonObject();e.addProperty("boundary","updatespell drawing omitted; native checkrange and cached distance retained");e.addProperty("draws",result.draws);e.addProperty("native_distance",(short)h.cpu.readRegister("D3").intValue());medicalEvents.add(e);h.jump(0x1621e8);continue;}
            if(pc==0x13e13c||pc==0x13b56c||pc==0x15f4b8){h.ret();continue;}
            if(pc==0x1251b8){JsonObject e=new JsonObject();e.addProperty("boundary","portrait event poll and geometry");e.addProperty("selected_input",choice);e.addProperty("include_dead",h.word(h.cpu.readRegister("A6").longValue()+12));aidEvents.add(e);if(choice<0){h.value(0x44720e,2,0x61);h.jump(0x125222);}else{h.value(0x4434b2,2,choice);h.jump(0x125300);}continue;}
            if(pc==0x14ce2a||pc==0x149fc6||pc==0x153720||pc==0x160d08||pc==0x13a510||pc==0x1377f4){h.ret();continue;}
            if(pc==0x16474a){warnings.add(h.word(sp+4));h.ret();continue;}
            if(pc==0x14de26){h.jump(0x14dede);continue;}
            if(pc==0x12946c){h.cpu.writeRegister("D0",1);h.ret();continue;}
            if(pc==0x161b80||pc==0x15f66c||pc==0x13a498||pc==0x160a8e||pc==0x160cca||pc==0x160baa){h.ret();continue;}
            if(pc==0x4001a0){sane.extended(h.pointer(sp+4),Math.sqrt(sane.extended(sp+8)));h.ret();continue;}
            if(pc==0x400038){h.cpu.writeRegister("D0",Math.abs(h.word(sp+4)));h.ret();continue;}
            if(pc==0x153ca4){result.rng(h);continue;}
            if(pc==0x133a18||pc==0x16474a){result.sound(h);continue;}
            if(pc==0x123d1c){int sound=h.word(sp+14);if(sound!=0){if(result.sounds.length()>0)result.sounds.append(',');result.sounds.append(sound);}h.ret();continue;}
            if(pc==0x15b2f6||pc==0x15a1f6){h.ret();continue;}
            if(pc==0x159192){displaySpell(h,result);continue;}
            if(pc==0x158454){long caller=h.pointer(sp);for(int i=0;h.pc()!=0x15847a;i++){if(i>12||!h.cpu.step(monitor))throw new Exception("Native result prefix failed");}long savedStack=h.sp();h.cpu.writeRegister("SP",sp);showresultsSound(h,result);h.cpu.writeRegister("SP",savedStack);JsonObject e=new JsonObject();e.addProperty("address","158474");e.addProperty("aid_gate",h.word(0x438786));e.addProperty("return",Long.toHexString(caller));medicalEvents.add(e);h.jump(0x15916c);execute(h,result,0x159178);h.jump(0x15917a);execute(h,result,caller);continue;}
            if(pc==0x1600fc||pc==0x120c14||pc==0x105572||pc==0x133d46||pc==0x13154c||pc==0x11edc4||pc==0x14a02c||pc==0x1205f8||pc==0x16043e||pc==0x1382a8||pc==0x11f136||pc==0x1537ee||pc==0x160fb2||pc==0x10ff54||pc==0x12f040||pc==0x15edfa){h.ret();continue;}
            if(pc==0x12382a&&h.word(0x439596)!=0)throw new Exception("Unsupported tutorial-enabled UI preference");
            int op=h.word(pc)&65535;
            if(op==0xa91f){h.value(h.pointer(sp),4,0);h.pop(4);h.jump(pc+2);continue;}
            if(op==0xa032){JsonObject event=new JsonObject();event.addProperty("address",Long.toHexString(pc));event.addProperty("boundary","FlushEvents");event.addProperty("d0",h.cpu.readRegister("D0").intValue());event.addProperty("draws",result.draws);nativeCalls.add(event);h.jump(pc+2);continue;}
            if(op==0xa85d){nativeBit(h,pc+2);continue;}
            if(op==0xa9eb){
                // Apple Numerics Manual FSQRTX uses one extended destination address.
                if((h.word(sp)&65535)==0x12){long destination=h.pointer(sp+2);sane.extended(destination,Math.sqrt(sane.extended(destination)));h.pop(6);h.jump(pc+2);}else sane.execute();
                continue;
            }
            if(op==0xaa1b){h.pop(2);h.value(h.sp(),4,0x810000);h.jump(pc+2);continue;}
            if(op==0xaa26){h.pop(4);h.jump(pc+2);continue;}
            if(op==0xa93c){h.pop(4);h.jump(pc+2);continue;}
            if(op==0xa935){h.pop(6);h.jump(pc+2);continue;}
            if(op==0xa937){h.jump(pc+2);continue;}
            if(op==0xa983||op==0xa914||op==0xa873||op==0xaa1c||op==0xa862||op==0xa863||op==0xa89b||op==0xa893||op==0xa891||op==0xa884){h.pop(4);h.jump(pc+2);continue;}
            if(op==0xa887||op==0xa888||op==0xa889||op==0xa88a){h.pop(2);h.jump(pc+2);continue;}
            if(op==0xaa25||op==0xa8a3){h.pop(4);h.jump(pc+2);continue;}
            if(op==0xaa1f||op==0xa8a8){h.pop(8);h.jump(pc+2);continue;}
            if(op==0xa89c){h.pop(2);h.jump(pc+2);continue;}
            if(op==0xa8b7){h.pop(4);h.jump(pc+2);continue;}
            if(op==0xa8ec){h.pop(22);h.jump(pc+2);continue;}
            if(op==0xa8a9){int vertical=h.word(sp),horizontal=h.word(sp+2);long rect=h.pointer(sp+4);h.value(rect,2,h.word(rect)+vertical);h.value(rect+2,2,h.word(rect+2)+horizontal);h.value(rect+4,2,h.word(rect+4)-vertical);h.value(rect+6,2,h.word(rect+6)-horizontal);h.pop(8);h.jump(pc+2);continue;}
            if(op==0xa8aa){long output=h.pointer(sp),b=h.pointer(sp+4),a=h.pointer(sp+8);int top=Math.max(h.word(a),h.word(b)),left=Math.max(h.word(a+2),h.word(b+2)),bottom=Math.min(h.word(a+4),h.word(b+4)),right=Math.min(h.word(a+6),h.word(b+6));boolean empty=top>=bottom||left>=right;h.value(output,2,empty?0:top);h.value(output+2,2,empty?0:left);h.value(output+4,2,empty?0:bottom);h.value(output+6,2,empty?0:right);h.value(sp+12,1,empty?0:1);h.pop(12);h.jump(pc+2);continue;}
            if(op==0xa8a7){long rectangle=h.pointer(sp+8);h.write(rectangle,h.read(sp+4,4));h.write(rectangle+4,h.read(sp,4));h.pop(12);h.jump(pc+2);continue;}
            if(op==0xaa1e){h.value(sp+2,4,0);h.pop(2);h.jump(pc+2);continue;}
            if(op==0xa8ad){long rect=h.pointer(sp);int vertical=h.word(sp+4),horizontal=h.word(sp+6);boolean inside=vertical>=h.word(rect)&&vertical<h.word(rect+4)&&horizontal>=h.word(rect+2)&&horizontal<h.word(rect+6);JsonObject e=new JsonObject();e.addProperty("address",Long.toHexString(pc));e.addProperty("vertical",vertical);e.addProperty("horizontal",horizontal);e.addProperty("rectangle",hex(h.read(rect,8)));e.addProperty("inside",inside);e.addProperty("draws",result.draws);collisionTests.add(e);h.value(sp+8,1,inside?1:0);h.pop(8);h.jump(pc+2);continue;}
            if(op==0xa9b4){h.jump(pc+2);continue;}
            if(op==0xa974){h.cpu.writeRegister("SP",sp-2);h.value(sp-2,2,0);h.jump(pc+2);continue;}
            if(!h.cpu.step(monitor))throw new Exception("Ally action at "+Long.toHexString(pc)+": "+h.cpu.getLastError());
        }
        if(!done)throw new Exception("Ally action instruction budget at "+Long.toHexString(h.pc()));
    }

}
