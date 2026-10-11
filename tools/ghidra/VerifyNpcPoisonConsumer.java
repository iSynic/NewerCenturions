import com.google.gson.*;
import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;

/** NPC condition consumer from the complete qualified retained caller result. */
public class VerifyNpcPoisonConsumer extends VerifyQualifiedNpcPoisonCaller {
    boolean consuming;
    JsonObject expectedEarned,earnedCaller;
    JsonArray roundEvents=new JsonArray(),resultAudio=new JsonArray();
    JsonObject roundPending;
    long roundLastPc=-1,roundLastSp=-1;
    final String[] labels={"earned_expiry","zero","positive2","positive6_uncapped","positive6_full","positive6_cap","negative5","negative128_uncapped","negative128_full","positive_word_overflow","negative_word_overflow","nonpositive_hp","cb1","cb2","cb_negative","ce_positive","ce_negative","cb2_ce2","neutral_lethal","neutral_overkill","neutral_ce_lethal"};
    final int[][] controls={{1,5,12,0,0},{0,5,12,0,0},{2,5,12,0,0},{6,5,12,0,0},{6,12,12,0,0},{6,11,12,0,0},{-5,5,12,0,0},{-128,100,300,0,0},{-128,200,200,0,0},{6,32766,32767,0,0},{-128,32700,32767,0,0},{6,-1,12,0,0},{6,5,12,1,0},{6,5,12,2,0},{6,5,12,-1,0},{6,5,12,0,2},{6,5,12,0,-2},{6,5,12,2,2},{6,4,4,0,0},{-15,5,5,0,0},{0,5,12,0,-10}};
    @Override public void run()throws Exception {
        root=Path.of(getScriptArgs()[0]);draftDirectory=root.resolve("analysis/completion/npc-poison-consumer");Files.createDirectories(draftDirectory);
        Path qualified=root.resolve("evidence/fixtures/qualified-npc-poison-caller.json");
        if(!ActionOracleHarness.sha256(qualified).equals("ef3793d279e37d76341229df00b2905004e2f21a58f09bf4d85bc41ab90ff912")||!ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyQualifiedNpcPoisonCaller.java")).equals("835a97171afac460e43f3b22e094b7acd5d2af314c0d1ceda333455e9d4548c5"))throw new Exception("Frozen qualified caller changed");
        JsonObject prior=JsonParser.parseString(Files.readString(qualified)).getAsJsonObject();expectedEarned=prior.getAsJsonArray("cases").get(0).getAsJsonObject().getAsJsonObject("after").deepCopy();JsonObject session=prior.getAsJsonObject("foundation").getAsJsonObject("session");
        Path original=root.resolve("NewCenturions/TheNewCenturions"),scenario=original.resolve("Scenarios/The Thunder of Guns/0");items=Files.readAllBytes(original.resolve("Data Files/Data ID"));scenarioItems=Files.readAllBytes(scenario.resolve("Data NI"));races=Files.readAllBytes(original.resolve("Data Files/Data Race"));mos=Files.readAllBytes(original.resolve("Data Files/Data MOS"));monsters=Files.readAllBytes(scenario.resolve("Data MD"));properties=Files.readAllBytes(original.resolve("Data Files/Data Biologic BD"));attacks=Files.readAllBytes(original.resolve("Data Files/Data S"));areas=Files.readAllBytes(original.resolve("Data Files/Data AD"));originalRegions=Files.readAllBytes(scenario.resolve("Data RD"));warningStrings=Files.readAllBytes(root.resolve("analysis/extracted/resources/74/53545223_3.bin"));
        JsonArray cases=new JsonArray();for(int i=0;i<controls.length;i++)earnAndConsume(cases,session,i);
        JsonObject identity=prior.getAsJsonObject("identity").deepCopy();identity.addProperty("qualified_caller_probe_sha256",identity.get("probe_sha256").getAsString());identity.addProperty("probe_sha256",ActionOracleHarness.sha256(root.resolve("analysis/completion/npc-poison-consumer/VerifyNpcPoisonConsumer.java")));identity.addProperty("qualified_caller_fixture_sha256",ActionOracleHarness.sha256(qualified));identity.addProperty("round_source_digest",ActionOracleHarness.sha256(root.resolve("evidence/npc-poison-round.md")));
        JsonObject out=new JsonObject();out.addProperty("authority","21 bounded native NPC condition slices125f44->126330 before feedback. Each fresh case executes the actual qualified NPC21 caller/Getup to earn the exact81-key after-state; only NPC15 declared signedB5/HP/max/CB/CE and final3neutral faction/script controls differ. All100 slots, complete party/map/geometry/order/three retained active fields/references/queues/menu/work preserved. Native full collision/resolver/killbody and native result prefix/audio/epilogue retained under declared presentation/service boundaries. No forced round wrap, complete round/Getup feedback suffix, live original GUI or campaign claim.");out.add("identity",identity);out.add("foundation",prior.getAsJsonObject("foundation"));out.add("earned_native_caller",earnedCaller);out.add("cases",cases);out.add("shared_rule_buffers",pooledBuffers);Files.writeString(draftDirectory.resolve("npc-poison-consumer.json"),out+"\n");println("NPC Poison consumer exported:"+cases.size());
    }
    void earnAndConsume(JsonArray output,JsonObject session,int index)throws Exception {
        consuming=false;musicPreferencePrepared=false;musicPhases=new JsonArray();long seed=session.getAsJsonObject("random").get("seed").getAsLong();member=session.get("selected_warrior").getAsInt();party=joined(session.getAsJsonArray("warriors"),3138);begin(seed);resetTrace();callerPhases=new JsonArray();pendingCaller.clear();callerEvents=new JsonArray();externalBoundaries=new JsonArray();pendingEvent=null;lastPc=-1;lastSp=-1;ActionOracleHarness h=frame();RunResult preparation=new RunResult(seed);
        try{
            if(!sha(h.read(0x44ecb0,18828)).equals("14d7ec3817141b883de67692c2493ea12bf4f380548f7ef8e8787a2adcdeba0e"))throw new Exception("Original startup backup changed");
            stockState(h,preparation);initializeActual(h,session);
            JsonObject raw=session.getAsJsonObject("ammunition_work");byte[] itemRaw=bytes(raw.getAsJsonArray("item")),weaponRaw=bytes(raw.getAsJsonArray("weapon")),ammoRaw=bytes(raw.getAsJsonArray("ammo"));if(itemRaw.length!=150||weaponRaw.length!=150||ammoRaw.length!=150||ByteBuffer.wrap(itemRaw).getShort(32)!=session.get("loaded_item_field32").getAsInt())throw new Exception("Missing actual full work context");h.write(0x45363c,itemRaw);h.write(0x453768,weaponRaw);h.write(0x4536d2,ammoRaw);
            JsonObject combat=session.getAsJsonObject("combat"),exploration=session.getAsJsonObject("exploration");byte[] region=bytes(exploration.getAsJsonArray("region_state")),headers=bytes(exploration.getAsJsonArray("event_headers"));if(region.length!=644||headers.length!=4200||region[0x208]!=9||exploration.get("map").getAsInt()!=2||session.get("bandage_ready").getAsInt()!=1)throw new Exception("Actual caller terrain/Aid input");h.write(0x443086,region);h.write(0x478e42,headers);h.value(0x438786,2,session.get("bandage_ready").getAsInt());establishTerrainProperties(h);
            h.write(0x437a94,new byte[1400]);if(combat.getAsJsonObject("death_queue").get("count").getAsInt()!=0||!session.get("combat_story").isJsonNull())throw new Exception("Unqualified active death/story queue input");h.value(0x437a90,4,0);h.value(0x43800c,2,0);h.value(0x4471ec,2,0);presentationControls(h);h.cpu.writeRegister("A6",0x700000);h.cpu.writeRegister("SP",0x6ffe00);
            if(activeBody!=21||h.word(0x439d3c+11*288+0x110)!=51||h.word(0x439d3c+11*288+0x112)!=51||h.word(0x447194)!=6||preparation.draws!=0||preparation.sounds.length()!=0||!Arrays.equals(h.read(0x443788,11368),attacks))throw new Exception("Exact caller foundation projection");
            resetTrace();capturing=true;RunResult r=new RunResult(seed);r.draws=session.getAsJsonObject("random").get("draws").getAsInt();JsonObject c=new JsonObject();c.addProperty("actor",21);c.addProperty("entry","1145c8");c.add("terrain_writer",terrainWriter);c.add("before",wholeSnap(h,r));h.jump(0x1145c8);execute(h,r,0x115776);if(diagnostic)throw new Exception("Unqualified decision diagnostic "+diagnosticSpell);c.add("after_decision",wholeSnap(h,r));execute(h,r,0x11578a);if(diagnostic||!pendingCaller.isEmpty()||pendingEvent!=null||!livePhases.isEmpty())throw new Exception("Unqualified/incomplete full caller continuation");c.add("after",wholeSnap(h,r));c.add("native_phases",callerPhases);c.add("instruction_events",callerEvents);c.add("external_boundaries",externalBoundaries);c.add("inherited_work_phases",phases);c.add("native_calls",nativeCalls);c.add("bit_calls",bitCalls);c.add("boundary_events",medicalEvents);c.add("rng",JsonParser.parseString("["+r.drawRecords+"]"));c.add("sounds",JsonParser.parseString("["+r.sounds+"]"));JsonObject earned=wholeSnap(h,r);if(!earned.equals(expectedEarned))throw new Exception("Earned qualified81 projection mismatch case"+index);if(index==0){c.add("native_music_phases",musicPhases);earnedCaller=c;}consume(h,r,output,index,earned);
        }finally{h.cpu.dispose();capturing=false;}
    }
    void consume(ActionOracleHarness h,RunResult previous,JsonArray output,int index,JsonObject earned)throws Exception {
        RunResult r=new RunResult(previous.seed);r.draws=previous.draws;resetTrace();callerPhases=new JsonArray();callerEvents=new JsonArray();externalBoundaries=new JsonArray();pendingCaller.clear();pendingEvent=null;lastPc=lastSp=-1;roundEvents=new JsonArray();resultAudio=new JsonArray();roundPending=null;roundLastPc=roundLastSp=-1;consuming=true;capturing=true;
        int[] v=controls[index];long a=0x439d3c+5*288;JsonObject c=new JsonObject();c.addProperty("index",index);c.addProperty("label",labels[index]);c.add("earned_after",earned);JsonObject declared=new JsonObject();declared.addProperty("body",15);declared.addProperty("b5_signed",v[0]);declared.addProperty("hp_word110",v[1]);declared.addProperty("max_word112",v[2]);declared.addProperty("cb_signed",v[3]);declared.addProperty("ce_signed",v[4]);declared.addProperty("neutral_death",index>=18);c.add("controls",declared);
        if(index!=0){h.value(a+0xb5,1,v[0]);h.value(a+0x110,2,v[1]);h.value(a+0x112,2,v[2]);h.value(a+0xcb,1,v[3]);h.value(a+0xce,1,v[4]);if(index>=18){h.value(a+0x0e,2,0);h.value(a+0x60,2,0);}}
        c.add("before",wholeSnap(h,r));h.jump(0x125f44);execute(h,r,0x126330);if(diagnostic||roundPending!=null||pendingEvent!=null||!pendingCaller.isEmpty()||!livePhases.isEmpty())throw new Exception("Incomplete condition slice"+index);if((short)h.cpu.readRegister("D3").intValue()!=100)throw new Exception("NPC loop did not visit100 slots");c.add("after",wholeSnap(h,r));c.add("native_phases",callerPhases);c.add("instruction_events",roundEvents);c.add("result_audio",resultAudio);c.add("external_boundaries",externalBoundaries);c.add("inherited_work_phases",phases);c.add("native_calls",nativeCalls);c.add("bit_calls",bitCalls);c.add("boundary_events",medicalEvents);c.add("rng",JsonParser.parseString("["+r.drawRecords+"]"));c.add("sounds",JsonParser.parseString("["+r.sounds+"]"));output.add(c);println("NPC Poison consumer"+index+":"+labels[index]);consuming=false;
    }
    @Override JsonObject wholeSnap(ActionOracleHarness h,RunResult r)throws Exception {
        JsonObject j=super.wholeSnap(h,r);if(consuming){JsonObject f=new JsonObject();f.addProperty("healing",h.word(0x438766));f.addProperty("poison",h.word(0x438784));f.addProperty("feedback",h.word(0x43958e));f.addProperty("reaction_count",h.word(0x4434c2));f.addProperty("poison_amount",h.word(0x442bc8));j.add("round_result_flags",f);}return j;
    }
    @Override void observe(ActionOracleHarness h,RunResult r,long pc)throws Exception {
        if(consuming&&capturing&&(pc!=roundLastPc||h.sp()!=roundLastSp)){
            roundLastPc=pc;roundLastSp=h.sp();
            if(roundPending!=null){roundPending.addProperty("next_address",Long.toHexString(pc));roundPending.add("after_instruction",wholeSnap(h,r));roundPending=null;}
            int n=pc==0x12e4e4?2:pc==0x158454?3:-1;
            if(n>=0&&pendingCaller.stream().noneMatch(p->p.get("address").getAsString().equals(Long.toHexString(pc))&&p.get("entry_sp_value").getAsLong()==h.sp())){JsonObject p=new JsonObject();p.addProperty("address",Long.toHexString(pc));p.addProperty("entry_sp_value",h.sp());p.addProperty("return_address_value",h.pointer(h.sp()));JsonArray args=new JsonArray();for(int i=0;i<n;i++)args.add(h.word(h.sp()+4+2*i));p.add("arguments",args);p.add("before",wholeSnap(h,r));callerPhases.add(p);pendingCaller.add(p);}
            boolean selected=false;for(long at:new long[]{0x125f74,0x125f8e,0x12601c,0x12606c,0x12608e,0x126094,0x1260b4,0x126130,0x1261e4,0x1261e8,0x1261fe,0x126262,0x126288,0x1262d6,0x1262e4,0x12630c,0x126322})if(pc==at)selected=true;
            if(selected&&(pc!=0x125f8e&&pc!=0x126322||h.read(0x439d3c+288L*(short)h.cpu.readRegister("D3").intValue()+0x105,1)[0]!=0)&&(pc!=0x1262e4||h.read(0x435884+(short)h.cpu.readRegister("D4").intValue(),1)[0]!=0)){JsonObject e=new JsonObject();e.addProperty("address",Long.toHexString(pc));e.addProperty("npc_index",(short)h.cpu.readRegister("D3").intValue());e.addProperty("d4_word",(short)h.cpu.readRegister("D4").intValue());e.addProperty("d0_word",(short)h.cpu.readRegister("D0").intValue());e.add("before_instruction",wholeSnap(h,r));roundEvents.add(e);roundPending=e;}
        }
        super.observe(h,r,pc);
    }
    void execute(ActionOracleHarness h,RunResult result,long stop) throws Exception {
        if(!consuming){super.execute(h,result,stop);return;}
        SaneBoundary sane=new SaneBoundary(h.cpu,currentProgram);boolean done=false;
        for(int step=0;step<1500000;step++) {
            long pc=h.pc(),sp=h.sp();
            if(pc==0x1363e4&&h.word(sp+4)==0){diagnostic=true;diagnosticSpell=(h.word(0x4435c6)+1)*1000+(h.word(0x4435c8)+1)*100+h.word(0x4435ca)+1;done=true;break;}
            observe(h,result,pc);if(pc==0x10ed92){h.jump(0x10f168);continue;}if(pc==0x10e844){h.jump(0x10ec34);continue;}if(pc==stop||(secondaryStop!=0&&pc==secondaryStop)){done=true;break;}
            if(pc==0x130114){h.jump(0x13015e);continue;}
            if(pc==0x130192||pc==0x13023a||pc==0x130296){h.jump(0x1302d6);continue;}
            if(pc==0x13033a){h.jump(0x130434);continue;}
            if(pc==0x13043e){h.jump(0x1305ca);continue;}
            if(pc==0x13071c){popup(h,result);continue;}
            if(pc==0x130a8a){h.jump(0x130aaa);continue;}
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
            if(pc==0x16474a&&!warningRunning){nativeWarning(h,result);continue;}
            if(pc==0x14de26){h.jump(0x14dede);continue;}
            if(pc==0x161b80||pc==0x15f66c||pc==0x13a498||pc==0x160a8e||pc==0x160cca||pc==0x160baa){h.ret();continue;}
            if(pc==0x4001a0){sane.extended(h.pointer(sp+4),Math.sqrt(sane.extended(sp+8)));h.ret();continue;}
            if(pc==0x400038){h.cpu.writeRegister("D0",Math.abs(h.word(sp+4)));h.ret();continue;}
            if(pc==0x153ca4){result.rng(h);continue;}
            if(pc==0x133a18){result.sound(h);continue;}
            if(pc==0x123d1c){int sound=h.word(sp+14);if(sound!=0){if(result.sounds.length()>0)result.sounds.append(',');result.sounds.append(sound);}h.ret();continue;}
            if(pc==0x15b2f6||pc==0x15a1f6){h.ret();continue;}
            if(pc==0x159192){displaySpell(h,result);continue;}
            if(pc==0x158454){long caller=h.pointer(sp);JsonObject p=new JsonObject();p.addProperty("body",h.word(sp+4));p.addProperty("code",h.word(sp+6));p.addProperty("amount",h.word(0x4434b8));p.addProperty("native_entry","158454");p.addProperty("native_return_address",caller);p.add("before",wholeSnap(h,result));if(!h.cpu.step(monitor))throw new Exception("Native result LINK");execute(h,result,caller);if(h.sp()!=sp+4)throw new Exception("Native result full RTS ABI");p.add("after",wholeSnap(h,result));resultAudio.add(p);continue;}
            if(pc==0x133d46){JsonArray functions=new JsonArray();SubmitMusicBoundary.execute(h,monitor,0,functions,()->result.rng(h));continue;}
            if(pc==0x1600fc||pc==0x120c14||pc==0x105572||pc==0x133d46||pc==0x13154c||pc==0x11edc4||pc==0x14a02c||pc==0x1205f8||pc==0x16043e||pc==0x1382a8||pc==0x11f136||pc==0x1537ee||pc==0x160fb2||pc==0x10ff54||pc==0x12f040||pc==0x15edfa){h.ret();continue;}
            if(pc==0x12382a&&h.word(0x439596)!=0)throw new Exception("Unsupported tutorial-enabled UI preference");
            int op=h.word(pc)&65535;
            if(op==0xa874){h.value(h.pointer(sp),4,0x830000);h.pop(4);h.jump(pc+2);continue;}
            if(op==0xaa19||op==0xaa1a){h.write(h.pointer(sp),new byte[6]);h.pop(4);h.jump(pc+2);continue;}
            if(op==0xaa14||op==0xaa15||op==0xaa0b||op==0xa915||op==0xa981){h.pop(4);h.jump(pc+2);continue;}
            if(op==0xa98b){h.pop(16);h.jump(pc+2);continue;}
            if(op==0xa97c){h.pop(10);h.value(h.sp(),4,0x830500);h.jump(pc+2);continue;}
            if(op==0xa91b){h.pop(10);h.jump(pc+2);continue;}
            if(op==0xa860){if(!warningRunning||h.pointer(sp)!=0||h.pointer(sp+4)!=0||h.pointer(sp+8)!=0x44720a||h.word(sp+12)!=-1)throw new Exception("Uncontracted warning WaitNextEvent");byte[] event=new byte[16];java.nio.ByteBuffer.wrap(event).putShort((short)3).putInt(0x3120);h.write(h.pointer(sp+8),event);JsonObject e=new JsonObject();e.addProperty("address",Long.toHexString(pc));e.addProperty("boundary","WaitNextEvent ordinary keydown dismissal");e.addProperty("event_record",hex(event));e.addProperty("sp_before",sp);h.pop(14);h.value(h.sp(),1,1);e.addProperty("sp_after",h.sp());nativeCalls.add(e);h.jump(pc+2);continue;}
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
            if(op==0xa852||op==0xa853){h.jump(pc+2);continue;}
            if(op==0xa9b4){h.jump(pc+2);continue;}
            if(op==0xa974){h.cpu.writeRegister("SP",sp-2);h.value(sp-2,2,0);h.jump(pc+2);continue;}
            if(!h.cpu.step(monitor))throw new Exception("Ally action at "+Long.toHexString(pc)+": "+h.cpu.getLastError());
        }
        if(!done)throw new Exception("Ally action instruction budget at "+Long.toHexString(h.pc()));
    }

}
