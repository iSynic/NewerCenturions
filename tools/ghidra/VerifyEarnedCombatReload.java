import java.nio.file.Files;
import java.nio.file.Path;
import com.google.gson.*;

/** Original occupied-cell attempt with earned empty Mercury rifle. */
public class VerifyEarnedCombatReload extends VerifyEarnedLoadedShot {
    int initialNpcIndex;
    JsonArray reloadPhases=new JsonArray(),reloadEvents=new JsonArray();
    java.util.ArrayList<JsonObject> activeReloadPhases=new java.util.ArrayList<>();
    @Override void initializeActual(ActionOracleHarness h,JsonObject s)throws Exception {
        super.initializeActual(h,s);h.value(0x447196,2,initialNpcIndex);
    }
    @Override JsonObject completeSnap(ActionOracleHarness h,RunResult r)throws Exception {
        JsonObject j=super.completeSnap(h,r);j.addProperty("current_npc_index",h.word(0x447196));j.addProperty("direction_x",(int)h.read(0x442736,1)[0]);j.addProperty("direction_y",(int)h.read(0x442737,1)[0]);j.addProperty("key_word",h.word(0x443316));j.addProperty("inventory_scratch",h.word(0x4434aa));j.addProperty("selected_control",h.pointer(0x439404));j.addProperty("movement_control",h.pointer(0x439454));j.addProperty("npc_sprite_scratch",h.word(0x4434b6));j.addProperty("item_class",h.word(0x453656));j.addProperty("native_unavailable_byte",h.read(0x442739,1)[0]&255);j.addProperty("reaction_count",h.word(0x4434c2));j.addProperty("reaction_bodies",hex(h.read(0x4434c4,24)));j.addProperty("neighbor_tables",hex(h.read(0x442934,660)));return j;
    }
    @Override void observe(ActionOracleHarness h,RunResult r,long pc)throws Exception {
        super.observe(h,r,pc);
        for(int i=activeReloadPhases.size()-1;i>=0;i--){JsonObject e=activeReloadPhases.get(i);if(pc==e.get("return_address_value").getAsLong()&&h.sp()==e.get("entry_sp_value").getAsLong()+4){e.addProperty("d0_return_word",(short)h.cpu.readRegister("D0").intValue());e.add("after",completeSnap(h,r));activeReloadPhases.remove(i);}}
        if((pc==0x111984||pc==0x12946c||pc==0x110e96||pc==0x101eb6||pc==0x14c6c2||pc==0x14edd0||pc==0x14e5e4||pc==0x117be2||pc==0x11f1a8)&&activeReloadPhases.stream().noneMatch(e->e.get("address").getAsString().equals(Long.toHexString(pc))&&e.get("entry_sp_value").getAsLong()==h.sp())){JsonObject e=new JsonObject();e.addProperty("address",Long.toHexString(pc));e.addProperty("entry_sp_value",h.sp());e.addProperty("return_address_value",h.pointer(h.sp()));JsonArray args=new JsonArray();for(int i=0;i<(pc==0x101eb6?3:pc==0x14e5e4?4:pc==0x14c6c2?2:pc==0x117be2?0:1);i++)args.add(h.word(h.sp()+4+i*2));e.add("arguments",args);e.add("before",completeSnap(h,r));reloadPhases.add(e);activeReloadPhases.add(e);}
        if(pc==0x11b8ee||pc==0x11b902||pc==0x11b9ac||pc==0x11bc7a||pc==0x11bfc0||pc==0x1114da||pc==0x11150c||pc==0x111588||pc==0x111544||pc==0x102002||pc==0x102014||pc==0x11c192||pc==0x11c202||pc==0x11c26c||pc==0x14e6a4||pc==0x14e95e||pc==0x11c286||pc==0x116b6a||pc==0x117338){JsonObject e=new JsonObject();e.addProperty("address",Long.toHexString(pc));e.addProperty("draws",r.draws);e.addProperty("sound_count",soundCount(r));e.addProperty("d0",(short)h.cpu.readRegister("D0").intValue());e.addProperty("d3",(short)h.cpu.readRegister("D3").intValue());e.addProperty("target_local",h.word(h.cpu.readRegister("A6").longValue()-0x18));e.addProperty("loaded",h.word(0x448aa0+activeBody*3138+0x134));e.addProperty("required",h.word(0x45377e));e.addProperty("reserve",h.pointer(0x42dbe4+80*4));e.addProperty("item_class",h.word(0x453656));e.addProperty("current_npc_index",h.word(0x447196));e.add("snapshot",completeSnap(h,r));reloadEvents.add(e);}
    }
    @Override void updateRuleWork(ActionOracleHarness h,RunResult r)throws Exception {
        long returnPc=h.pointer(h.sp());int body=h.word(h.sp()+4);updating=true;long prior=secondaryStop;try{
            execute(h,r,0x11f1c8);JsonObject e=new JsonObject();e.addProperty("boundary","combatupdate2 native selection, full party class/slot gate and second refresh or NPC descriptor/weapon cache; physical name Pascal retained; paired authored names and drawing acknowledged");e.addProperty("body",body);e.addProperty("return",Long.toHexString(returnPc));
            if(body>=0&&body<6){h.jump(0x11f5ce);secondaryStop=0x11f804;execute(h,r,0x11f5e2);secondaryStop=prior;if(h.pc()==0x11f5e2){h.jump(0x11f61a);execute(h,r,0x11f804);e.addProperty("active_refresh",true);}else e.addProperty("active_refresh",false);h.jump(0x11faa8);execute(h,r,0x11fc4c);e.addProperty("mode_after",h.read(0x448aa0+body*3138+0x9dd,1)[0]&255);}
            else if(body>=10&&body<110){h.jump(0x11f734);execute(h,r,0x11f7d6);h.jump(0x11fd42);execute(h,r,0x11fdd4);}
            else throw new Exception("Unsupported selected UI body");
            e.addProperty("item_work_after",hex(h.read(0x45363c,150)));e.addProperty("inventory_scratch",h.word(0x4434aa));medicalEvents.add(e);h.jump(0x120058);execute(h,r,returnPc);
        }finally{secondaryStop=prior;updating=false;}
    }
    void presentationControls(ActionOracleHarness h)throws Exception {
        h.write(0x820000,new byte[4096]);h.write(0x830000,new byte[4096]);h.value(0x830000,4,0x830100);
        long[] pointers={0x4393f4,0x4393f8,0x43942c,0x439450,0x439454,0x439458,0x43945c,0x439484,0x439488,0x43948c,0x439490};for(int i=0;i<pointers.length;i++){long handle=0x820000+32*i;h.value(pointers[i],4,handle);h.value(handle,4,0x830200+32*i);}
        h.value(0x439404,4,0);h.value(0x443318,2,activeBody);
    }
    void normalizeView(ActionOracleHarness h,RunResult r)throws Exception {
        h.write(0x6dc000,new byte[8192]);h.cpu.writeRegister("SP",0x6dd000);h.jump(0x1268ac);execute(h,r,0x12690e);if(h.sp()!=0x6dd002)throw new Exception("Original view prefix stack cleanup");
    }
    void direction(ActionOracleHarness h,RunResult r)throws Exception {
        h.value(0x443316,2,0x5533);h.enter(0x111984,0x6ffe00,0x6fffc0);h.value(h.sp()+4,2,0);execute(h,r,0x6fffc0);h.pop(2);if(h.read(0x442736,1)[0]!=1||h.read(0x442737,1)[0]!=1||h.pointer(0x439404)!=h.pointer(0x439454))throw new Exception("Original diagonal direction/selector");
    }
    void reloadCase(JsonArray out,JsonObject f,int index,boolean normalized)throws Exception {
        JsonObject s=f.getAsJsonObject("session");long seed=s.getAsJsonObject("random").get("seed").getAsLong();begin(seed);poisonEvents=new JsonArray();aidEvents=new JsonArray();warnings=new JsonArray();medicalEvents=new JsonArray();shotEvents=new JsonArray();collisionTests=new JsonArray();trackingCounter=false;preKill=null;phaseReceipts=new JsonArray();activePhases.clear();reloadPhases=new JsonArray();reloadEvents=new JsonArray();activeReloadPhases.clear();itemUse=false;secondaryStop=0;priorKilledCount=f.get("source_reconstructed_killed_count").getAsInt();initialNpcIndex=f.get("source_reconstructed_current_npc_index").getAsInt();ActionOracleHarness h=frame();RunResult r=new RunResult(seed);r.draws=s.getAsJsonObject("random").get("draws").getAsInt();try{
            initializeActual(h,s);presentationControls(h);JsonObject c=new JsonObject();c.addProperty("foundation",index);c.addProperty("kind",normalized?"normalized_diagonal_attempt":"raw_offscreen_scroll");c.addProperty("actor",activeBody);c.add("before",completeSnap(h,r));if(normalized){normalizeView(h,r);c.add("display_normalized",completeSnap(h,r));}direction(h,r);c.add("before_command",completeSnap(h,r));h.cpu.writeRegister("SP",0x6ffe00);h.cpu.writeRegister("A6",0x6fff00);h.jump(0x116b2c);execute(h,r,0x116b6a);c.add("command_return",completeSnap(h,r));execute(h,r,0x117338);if(diagnostic)throw new Exception("Unsupported original reload continuation "+diagnosticSpell);c.add("after",completeSnap(h,r));saveTrace(c,r);if(!activeReloadPhases.isEmpty())throw new Exception("Unreturned native reload phase");c.add("native_phases",reloadPhases);c.add("native_events",reloadEvents);out.add(c);println("Earned combat reload "+c.get("kind").getAsString()+" foundation"+index);
        }finally{h.cpu.dispose();itemUse=false;secondaryStop=0;}
    }
    JsonObject foundation(Path root,int seed)throws Exception {
        Path input=root.resolve("analysis/completion/earned-adjacent-reload-input-"+seed+".json"),receiptPath=root.resolve("analysis/completion/earned-adjacent-reload-input-"+seed+".receipt.json");JsonObject s=JsonParser.parseString(Files.readString(input)).getAsJsonObject().getAsJsonObject("session"),receipt=JsonParser.parseString(Files.readString(receiptPath)).getAsJsonObject();if(!receipt.get("complete_state_and_every_ordered_event_replay").getAsBoolean()||!receipt.get("full_input_save_reload").getAsBoolean()||receipt.get("field_grants_or_random_trials").getAsBoolean()||receipt.getAsJsonArray("commands").size()!=(seed==12?148:168))throw new Exception("Unproved reload replay");int killed=0,lastNpc=-1;for(JsonElement group:receipt.getAsJsonArray("ordered_events"))for(JsonElement event:group.getAsJsonArray()){if(!event.isJsonObject())continue;JsonObject e=event.getAsJsonObject();if(e.has("CombatStarted")){killed=0;lastNpc=-1;}if(e.has("TurnStarted")){int body=e.getAsJsonObject("TurnStarted").get("body").getAsInt();if(body>=10)lastNpc=body;}if(e.has("AttackResolved")){JsonObject a=e.getAsJsonObject("AttackResolved");if(a.get("defeated").getAsBoolean()&&a.get("target").getAsInt()>=10)killed++;}}
        if(killed!=(seed==12?1:4)||lastNpc!=(seed==12?10:11))throw new Exception("Unproved retained native counters");JsonObject f=new JsonObject();f.addProperty("input",root.relativize(input).toString().replace('\\','/'));f.addProperty("digest",ActionOracleHarness.sha256(input));f.addProperty("receipt",root.relativize(receiptPath).toString().replace('\\','/'));f.addProperty("receipt_digest",ActionOracleHarness.sha256(receiptPath));f.addProperty("source_reconstructed_killed_count",killed);f.addProperty("source_reconstructed_current_npc_index",lastNpc-10);f.addProperty("source_reconstructed_inactivity_counter",0);f.add("session",s);return f;
    }
    @Override public void run()throws Exception {
        Path root=Path.of(getScriptArgs()[0]),original=root.resolve("NewCenturions/TheNewCenturions"),scenario=original.resolve("Scenarios/The Thunder of Guns/0");items=Files.readAllBytes(original.resolve("Data Files/Data ID"));scenarioItems=Files.readAllBytes(scenario.resolve("Data NI"));races=Files.readAllBytes(original.resolve("Data Files/Data Race"));mos=Files.readAllBytes(original.resolve("Data Files/Data MOS"));monsters=Files.readAllBytes(scenario.resolve("Data MD"));properties=Files.readAllBytes(original.resolve("Data Files/Data Swamp BD"));attacks=Files.readAllBytes(original.resolve("Data Files/Data S"));areas=Files.readAllBytes(original.resolve("Data Files/Data AD"));JsonObject identity=JsonParser.parseString(Files.readString(root.resolve("evidence/fixtures/earned-loaded-shot.json"))).getAsJsonObject().getAsJsonObject("identity").deepCopy();identity.addProperty("earned_loaded_shot_probe_sha256",identity.get("probe_sha256").getAsString());identity.addProperty("earned_loaded_shot_fixture_sha256",ActionOracleHarness.sha256(root.resolve("evidence/fixtures/earned-loaded-shot.json")));identity.addProperty("probe_sha256",ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyEarnedCombatReload.java")));JsonArray foundations=new JsonArray(),cases=new JsonArray();JsonObject f12=foundation(root,12),f1=foundation(root,1);foundations.add(f12);foundations.add(f1);reloadCase(cases,f12,0,true);reloadCase(cases,f1,1,true);reloadCase(cases,f12,0,false);JsonObject out=new JsonObject();out.addProperty("authority","Two actual earned empty165 diagonal occupied-cell attempts with declared original display normalization, full original keyboard/inwindow/neighbor reaction/map/refit/doreload/outer caller, and one separately qualified raw offscreen scroll control. No manual Reload selector, rule condition/cache class/reaction-flag grant or live original-game claim.");out.add("identity",identity);out.add("foundations",foundations);out.add("cases",cases);Files.writeString(root.resolve("evidence/fixtures/earned-combat-reload.json"),out+"\n");println("Earned combat reload exported3");
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
            if(op==0xa975){h.value(sp,4,0x830000);h.jump(pc+2);continue;}
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
