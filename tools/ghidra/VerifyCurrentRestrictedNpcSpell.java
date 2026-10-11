import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import com.google.gson.*;

/** Original restricted spell turns at the corrected campaign boundary. */
public class VerifyCurrentRestrictedNpcSpell extends VerifyEarnedCombatReload {
    JsonArray backupWrites=new JsonArray();
    boolean writerControl;
    @Override public void run()throws Exception {
        if(getScriptArgs().length!=2)throw new Exception("A declared coherent frontier directory is required");
        Path root=Path.of(getScriptArgs()[0]),frontiers=root.resolve(getScriptArgs()[1]);
        if(frontiers.toString().contains("corrected-firearm-range-boundary-audit-verified"))throw new Exception("Historical incoherent frontier cannot be used");
        Path source=root.resolve("evidence/current-restricted-npc-memory.md");
        if(!ActionOracleHarness.sha256(source).equals("0a1e966e681b0b0bf1ff833636f54012a151dd30388dc8611cc833be793585ba"))throw new Exception("Frozen backup source contract changed");
        Path controlInput=root.resolve("analysis/completion/current-restricted-backup-controls.json"),controlSource=root.resolve("analysis/completion/current-restricted-backup-controls.probe.java");
        JsonObject controls=JsonParser.parseString(Files.readString(controlInput)).getAsJsonObject();
        if(!ActionOracleHarness.sha256(controlInput).equals("4c1dcc5e1dd0b831b38a9a54a130beebb7f5e70906c15632290b2d423b05d538")||!ActionOracleHarness.sha256(controlSource).equals(controls.getAsJsonObject("identity").get("probe_digest").getAsString()))throw new Exception("Separate writer control receipt changed");
        Path original=root.resolve("NewCenturions/TheNewCenturions"),scenario=original.resolve("Scenarios/The Thunder of Guns/0");
        items=Files.readAllBytes(original.resolve("Data Files/Data ID"));scenarioItems=Files.readAllBytes(scenario.resolve("Data NI"));races=Files.readAllBytes(original.resolve("Data Files/Data Race"));mos=Files.readAllBytes(original.resolve("Data Files/Data MOS"));monsters=Files.readAllBytes(scenario.resolve("Data MD"));properties=Files.readAllBytes(original.resolve("Data Files/Data Swamp BD"));attacks=Files.readAllBytes(original.resolve("Data Files/Data S"));areas=Files.readAllBytes(original.resolve("Data Files/Data AD"));
        JsonArray foundations=new JsonArray(),cases=new JsonArray();
        for(int seed:new int[]{12,1})foundations.add(currentFoundation(root,frontiers,seed));
        for(int index=0;index<foundations.size();index++)currentCase(cases,foundations.get(index).getAsJsonObject(),index);
        JsonObject identity=JsonParser.parseString(Files.readString(root.resolve("evidence/fixtures/earned-combat-reload.json"))).getAsJsonObject().getAsJsonObject("identity").deepCopy();identity.addProperty("earned_reload_probe_sha256",identity.get("probe_sha256").getAsString());identity.addProperty("probe_sha256",ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyCurrentRestrictedNpcSpell.java")));identity.addProperty("earned_reload_fixture_sha256",ActionOracleHarness.sha256(root.resolve("evidence/fixtures/earned-combat-reload.json")));identity.addProperty("source_contract_digest",ActionOracleHarness.sha256(source));identity.addProperty("writer_controls_digest",ActionOracleHarness.sha256(controlInput));identity.addProperty("writer_controls_source_digest",ActionOracleHarness.sha256(controlSource));
        JsonObject out=new JsonObject();out.addProperty("authority","Two complete original corrected-route NPC decisions1145c8..115776 and original115782 Getup through115788 cleanup/11578a, on newly declared coherent complete replay/save/resume foundations. Original startup Warrior Editor backup retained under source-proved ordinary override0; no synthetic twelve-record zeroing or forced race alias. Real NPC weapon187 work refresh, exact selected spells, targets, count rolls, all prior seed1 effects, complete resolver and initiative suffix, ordered RNG/audio and full rule buffers captured. Separate immutable original backup writer-prefix controls are embedded as controls, not earned actor decisions. Presentation, event, SANE and historical Apple BitTst boundaries remain qualified; no live original-game or campaign completion claim.");out.add("identity",identity);out.add("foundations",foundations);out.add("cases",cases);out.add("writer_controls",controls);JsonObject sharedBuffers=new JsonObject();shareBuffers(out,sharedBuffers);out.add("shared_rule_buffers",sharedBuffers);Files.writeString(root.resolve("evidence/fixtures/current-restricted-npc-spell.json"),out+"\n");println("Current restricted NPC decisions with Getup exported:2");
    }

    void shareBuffers(JsonElement node,JsonObject shared)throws Exception {
        if(node.isJsonArray()){for(JsonElement child:node.getAsJsonArray())shareBuffers(child,shared);return;}
        if(!node.isJsonObject())return;JsonObject object=node.getAsJsonObject();
        for(String key:new String[]{"first_item_cache","item_cache_1","item_cache_2","item_cache_3","scenario_item_cache","backup_warriors","warrior_record_envelope","backup","envelope"})if(object.has(key)&&object.get(key).isJsonPrimitive()){
            String value=object.get(key).getAsString();byte[] data=new byte[value.length()/2];for(int i=0;i<data.length;i++)data[i]=(byte)Integer.parseInt(value.substring(2*i,2*i+2),16);String digest=sha(data);shared.addProperty(digest,value);JsonObject reference=new JsonObject();reference.addProperty("shared_buffer_sha256",digest);reference.addProperty("byte_length",data.length);object.add(key,reference);
        }
        for(java.util.Map.Entry<String,JsonElement> entry:object.entrySet())shareBuffers(entry.getValue(),shared);
    }

    JsonObject currentFoundation(Path root,Path frontiers,int seed)throws Exception {
        Path input=frontiers.resolve("seed"+seed+"-before-runenemy.session.json"),receiptInput=frontiers.resolve("seed"+seed+"-before-runenemy.receipt.json");
        JsonObject session=JsonParser.parseString(Files.readString(input)).getAsJsonObject(),receipt=JsonParser.parseString(Files.readString(receiptInput)).getAsJsonObject();
        if(session.getAsJsonObject("health_context").get("effect").getAsInt()!=session.getAsJsonObject("combat").getAsJsonArray("attack_packet").get(18).getAsInt())throw new Exception("Declared frontier contains contradictory Health alias");
        for(String gate:new String[]{"full_final_session_and_events_equal","full_save_load_equal","midpoint_saved_resumed_suffix_complete_session_and_events_equal","whole_session_rng_unchanged"})if(!receipt.get(gate).getAsBoolean())throw new Exception("Incomplete coherent replay receipt: "+gate);
        if(!ActionOracleHarness.sha256(input).equals(receipt.get("session_sha256").getAsString()))throw new Exception("Declared frontier digest does not match replay receipt");
        int killed=0,combatStart=-1;JsonArray events=receipt.getAsJsonArray("ordered_events"),deaths=new JsonArray();
        for(int i=0;i<events.size();i++)for(JsonElement element:events.get(i).getAsJsonArray())if(element.isJsonObject()){
            JsonObject event=element.getAsJsonObject();if(event.has("CombatStarted")){killed=0;combatStart=i;deaths=new JsonArray();}
            for(String kind:new String[]{"AttackResolved","SpellResolved"})if(event.has(kind)){
                JsonObject action=event.getAsJsonObject(kind);int target=action.get("target").getAsInt();
                if(action.get("defeated").getAsBoolean()&&target>=10){
                    byte[] record=bytes(session.getAsJsonObject("combat").getAsJsonObject("enemies").getAsJsonArray("records").get(target-10).getAsJsonObject().getAsJsonArray("bytes"));
                    int faction=(short)(((record[14]&255)<<8)|(record[15]&255)),status=record[0xb0]&255;boolean counted=faction!=0&&status!=255;
                    JsonObject death=new JsonObject();death.addProperty("command_index",i);death.addProperty("kind",kind);death.addProperty("target",target);death.addProperty("retained_faction_word_0e",faction);death.addProperty("retained_status_byte_b0",status);death.addProperty("native_counter_increment",counted);deaths.add(death);if(counted)killed=(killed+1)&65535;
                }
            }
        }
        if(combatStart<0||killed!=(seed==12?0:8)||deaths.size()!=(seed==12?0:9))throw new Exception("Unexpected actual current battle death prefix");
        JsonObject f=new JsonObject();f.addProperty("seed",seed);f.addProperty("input",root.relativize(input).toString().replace('\\','/'));f.addProperty("digest",ActionOracleHarness.sha256(input));f.addProperty("receipt",root.relativize(receiptInput).toString().replace('\\','/'));f.addProperty("receipt_digest",ActionOracleHarness.sha256(receiptInput));f.addProperty("source_reconstructed_killed_count",killed);f.add("current_battle_defeats",deaths);f.addProperty("killed_count_initializer_qualification","Original NPC killbody12e5c2..12e5e2 counts body>=10 only when retained WORD0e!=0 and BYTEb0!=ff; those fields are preserved by the original kill path. Current accepted event prefix supplies nine seed1 defeats, excludes friendly body10 faction0, and counts eight. Original combat reset plus no unsupported writer history are required; not a persisted public counter.");f.addProperty("source_reconstructed_inactivity_counter",0);f.addProperty("combined_command_count",receipt.getAsJsonArray("commands").size());f.addProperty("current_battle_start_command_index",combatStart);f.add("session",session);return f;
    }

    void currentCase(JsonArray cases,JsonObject foundation,int index)throws Exception {
        JsonObject session=foundation.getAsJsonObject("session");long seed=session.getAsJsonObject("random").get("seed").getAsLong();
        begin(seed);poisonEvents=new JsonArray();aidEvents=new JsonArray();warnings=new JsonArray();medicalEvents=new JsonArray();shotEvents=new JsonArray();collisionTests=new JsonArray();phaseReceipts=new JsonArray();activePhases.clear();reloadPhases=new JsonArray();reloadEvents=new JsonArray();activeReloadPhases.clear();currentEvents=new JsonArray();currentPhases=new JsonArray();activeCurrentPhases.clear();trackingCounter=false;writerControl=false;itemUse=false;secondaryStop=0;reconstructedKilled=foundation.get("source_reconstructed_killed_count").getAsInt();
        ActionOracleHarness h=frame();RunResult r=new RunResult(seed);r.draws=session.getAsJsonObject("random").get("draws").getAsInt();try{
            initializeActual(h,session);JsonObject c=new JsonObject();c.addProperty("foundation",index);c.addProperty("actor",activeBody);c.add("before",completeSnap(h,r));
            if(!sha(h.read(0x44ecb0,18828)).equals("14d7ec3817141b883de67692c2493ea12bf4f380548f7ef8e8787a2adcdeba0e")||h.word(0x4471ec)!=0)throw new Exception("Original startup backup/ordinary override not preserved");
            h.jump(0x1145c8);execute(h,r,0x115776);if(diagnostic)throw new Exception("Unsupported original actor continuation "+diagnosticSpell);c.add("after_decision",completeSnap(h,r));
            execute(h,r,0x11578a);if(diagnostic)throw new Exception("Unsupported original Getup continuation "+diagnosticSpell);c.add("after",completeSnap(h,r));
            if(!sha(h.read(0x44ecb0,18828)).equals("14d7ec3817141b883de67692c2493ea12bf4f380548f7ef8e8787a2adcdeba0e")||h.word(0x4471ec)!=0||!activeCurrentPhases.isEmpty())throw new Exception("Current backup preservation/native phase return failed");
            saveTrace(c,r);c.add("current_native_phases",currentPhases);c.add("current_native_events",currentEvents);cases.add(c);println("Current restricted NPC actor"+activeBody+" complete native Getup");
        }finally{h.cpu.dispose();}
    }
    JsonArray currentEvents=new JsonArray();
    JsonArray currentPhases=new JsonArray();
    java.util.ArrayList<JsonObject> activeCurrentPhases=new java.util.ArrayList<>();
    int reconstructedKilled;

    JsonObject backupSnap(ActionOracleHarness h,RunResult r)throws Exception {
        JsonObject j=new JsonObject();j.addProperty("party",hex(h.read(0x448aa0,18828)));j.addProperty("backup",hex(h.read(0x44ecb0,18828)));j.addProperty("backup_digest",sha(h.read(0x44ecb0,18828)));j.addProperty("envelope",hex(h.read(0x448aa0,14*3138)));j.addProperty("override_word",h.word(0x4471ec));j.addProperty("registration_word",h.word(0x4046ea));j.addProperty("last_party_index",(int)h.read(0x44273a,1)[0]);j.addProperty("loop_index",h.word(0x4434aa));j.addProperty("native_stack",Long.toHexString(h.sp()));j.addProperty("caller_frame",Long.toHexString(h.cpu.readRegister("A6").longValue()));j.addProperty("backup11_alias_word",h.word(0x4511a8));j.addProperty("backup12_alias_word",h.word(0x451dea));j.addProperty("seed",r.seed);j.addProperty("draws",r.draws);return j;
    }

    void backupControl(JsonArray controls,int override)throws Exception {
        begin(12);poisonEvents=new JsonArray();aidEvents=new JsonArray();warnings=new JsonArray();medicalEvents=new JsonArray();shotEvents=new JsonArray();collisionTests=new JsonArray();phaseReceipts=new JsonArray();activePhases.clear();reloadPhases=new JsonArray();reloadEvents=new JsonArray();activeReloadPhases.clear();currentEvents=new JsonArray();currentPhases=new JsonArray();activeCurrentPhases.clear();backupWrites=new JsonArray();writerControl=true;trackingCounter=false;itemUse=false;secondaryStop=0;
        ActionOracleHarness h=frame();RunResult r=new RunResult(12);try{
            h.write(0x448aa0,party);h.value(0x44273a,1,5);h.value(0x4471ec,2,override);h.value(0x4046ea,2,0);
            JsonObject control=new JsonObject();control.addProperty("kind",override==0?"ordinary_override_zero":"editor_override_one");control.addProperty("qualified_control",true);control.add("before",backupSnap(h,r));
            long stop=override==0?0x1622c8:0x1622ae;h.enter(0x162238,0x6ffe00,0x900000);h.value(h.sp()+4,2,0);execute(h,r,stop);control.addProperty("native_stop",Long.toHexString(h.pc()));control.add("after",backupSnap(h,r));control.add("ordered_backup_writes",backupWrites);control.add("rng",JsonParser.parseString("["+r.drawRecords+"]"));control.add("sounds",JsonParser.parseString("["+r.sounds+"]"));
            byte[] expected=new byte[18828];if(override==1)System.arraycopy(party,0,expected,0,5*3138);
            if(!Arrays.equals(expected,h.read(0x44ecb0,18828))||!Arrays.equals(party,h.read(0x448aa0,18828))||backupWrites.size()!=(override==0?0:3925)||h.sp()!=0x6ffde0||h.cpu.readRegister("A6").longValue()!=0x6ffdfc||r.draws!=0||r.sounds.length()!=0)throw new Exception("Original backup writer control disagrees");
            controls.add(control);println("Current native backup entry override"+override);
        }finally{h.cpu.dispose();writerControl=false;}
    }

    void state(ActionOracleHarness h,byte[] records,byte[] map) {
        common(h,party);h.write(0x439d3c,records);h.write(0x4085b8,map);h.value(0x43874a,2,2);
        h.write(0x4537fe,Arrays.copyOfRange(items,0,30000));h.write(0x46225e,Arrays.copyOfRange(items,30000,60000));h.write(0x45ad2e,Arrays.copyOfRange(items,60000,90000));h.write(0x46978e,Arrays.copyOfRange(items,90000,120000));h.write(0x443788,attacks);
        h.write(0x470cbe,Arrays.copyOfRange(scenarioItems,0,30000));
        h.write(0x43167c,Arrays.copyOfRange(properties,64,12064));h.value(0x442780,4,5);h.value(0x442784,4,5);
        h.value(0x438776,2,1);h.write(0x438210,areas);
        h.value(0x447196,2,0);h.value(0x447194,2,1);h.value(0x4470ba,2,10);h.value(0x4435c4,2,6);h.value(0x44374c,2,0);
        h.value(0x402f43,1,2);h.value(0x43550c,2,155);h.value(0x439568,2,0);h.write(0x442bd6,new byte[1200]);
        h.value(0x43565f,1,0);h.value(0x447d96,2,0);h.value(0x4435cc,2,0);
        for(int i=0;i<6;i++){h.value(0x44709a+i*4,2,40+i);h.value(0x44709c+i*4,2,40);}
    }
    @Override void initializeActual(ActionOracleHarness h,JsonObject s)throws Exception {
        JsonObject c=s.getAsJsonObject("combat");party=joined(s.getAsJsonArray("warriors"),3138);byte[] records=joined(c.getAsJsonObject("enemies").getAsJsonArray("records"),288);recordLength=records.length;state(h,records,words(c.getAsJsonObject("map").getAsJsonArray("tiles")));bodyCount=records.length/288;
        h.write(0x448aa0,party);h.write(0x44709a,positions(c.getAsJsonObject("squad").getAsJsonArray("positions")));h.write(0x437a6e,words(c.getAsJsonObject("squad").getAsJsonArray("under_tiles")));h.write(0x446f0a,positions(c.getAsJsonObject("enemies").getAsJsonArray("positions")));h.write(0x4470b8,words(c.getAsJsonArray("order")));h.value(0x447194,2,c.get("cursor").getAsInt());h.value(0x447196,2,11);h.value(0x43874a,2,bodyCount);h.value(0x4435c4,2,bodyCount);h.value(0x438778,2,1);h.value(0x44255c,1,c.get("round").getAsInt());h.value(0x44273a,1,5);h.value(0x44273f,1,s.get("selected_warrior").getAsInt());h.value(0x4435d2,2,s.get("selected_warrior").getAsInt());
        JsonArray shift=c.getAsJsonObject("map").getAsJsonArray("shift");h.value(0x442780,4,shift.get(0).getAsInt()+5);h.value(0x442784,4,shift.get(1).getAsInt()+5);h.write(0x442bd6,bytes(c.getAsJsonArray("spell_fields")));h.write(0x435884,bytes(c.getAsJsonArray("field_used")));h.write(0x44374e,bytes(c.getAsJsonArray("attack_packet")));h.write(0x4435c6,words(c.getAsJsonArray("attack_identity")));h.value(0x4435cc,2,s.get("attack_count").getAsInt());h.value(0x45365c,2,s.get("loaded_item_field32").getAsInt());
        h.write(0x453768,new byte[150]);h.write(0x4536d2,new byte[150]);h.write(0x45363c,new byte[150]);JsonObject p=c.getAsJsonObject("weapon_work");h.value(0x453776,2,p.get("sound").getAsInt());h.value(0x4537f0,2,p.get("weapon_armor_filter").getAsInt());h.value(0x45375a,2,p.get("ammunition_armor_filter").getAsInt());h.write(0x43788a,bytes(s.getAsJsonArray("psi_memory")));h.write(0x442562,bytes(s.getAsJsonArray("party_buffs")));ByteBuffer ammo=ByteBuffer.allocate(800);for(JsonElement e:s.getAsJsonArray("ammunition"))ammo.putInt(e.getAsInt());h.write(0x42dbe4,ammo.array());h.write(0x442580,new byte[24]);h.write(0x437a7a,new byte[24]);h.write(0x442798,new byte[24]);h.write(0x4434e0,new byte[220]);h.write(0x443722,new byte[42]);h.value(0x442739,1,c.get("unavailable").getAsInt());h.value(0x435662,1,0);h.value(0x438760,2,0);h.value(0x438784,2,0);h.value(0x438766,2,0);h.value(0x443580,1,0);h.value(0x4358c2,1,0);h.value(0x438780,2,s.get("enemy_spell_disabled").getAsInt());h.write(0x437a94,new byte[1400]);h.value(0x43800c,2,0);h.value(0x437a90,4,0);h.value(0x438582,2,0);h.value(0x438584,1,0);h.value(0x438585,1,0);h.value(0x447198,2,0);h.value(0x44719a,2,0);h.value(0x4436b4,2,127);h.write(0x43878c,new byte[4]);h.write(0x4436fa,new byte[16]);h.write(0x4434ba,new byte[8]);h.write(0x40856c,new byte[4]);h.value(0x439582,2,0);h.write(0x479ed8,new byte[256]);h.cpu.writeRegister("A6",0x700000);h.cpu.writeRegister("SP",0x6ffe00);
        restoreRepresentedState(h,s);
        h.value(0x408570,2,5);h.value(0x408572,2,5);
        h.value(0x43942c,4,0x820000);h.value(0x820000,4,0x820100);h.write(0x820100,new byte[32]);
    }
    void restoreRepresentedState(ActionOracleHarness h,JsonObject s)throws Exception {
        JsonObject c=s.getAsJsonObject("combat"),health=s.getAsJsonObject("health_context"),e=s.getAsJsonObject("exploration");
        activeBody=c.getAsJsonArray("order").get(c.get("cursor").getAsInt()).getAsInt();
        if(activeBody!=16&&activeBody!=20)throw new Exception("Unexpected current actor");
        h.value(0x447196,2,activeBody-10);
        h.write(0x440edc,bytes(s.getAsJsonArray("allies")));h.value(0x44257c,2,s.get("ally_count").getAsInt());
        h.write(0x44258c,bytes(s.getAsJsonArray("flags")));h.value(0x442578,2,e.get("fatigue").getAsInt());
        h.value(0x442774,4,e.get("map").getAsInt());h.value(0x4427c8,2,e.get("minute").getAsInt());
        h.value(0x4427ca,2,e.get("hour").getAsInt());h.value(0x4427d4,2,e.get("day").getAsInt());
        h.write(0x439cec,words(c.getAsJsonObject("round_scripts").getAsJsonArray("slots")));
        h.value(0x44255c,1,c.getAsJsonObject("round_scripts").get("counter").getAsInt());
        h.write(0x4285f4,inventories(s.getAsJsonArray("crate_items")));h.write(0x41fc74,inventories(s.getAsJsonArray("booty_items")));
        JsonObject q=c.getAsJsonObject("death_queue");ByteBuffer entries=ByteBuffer.allocate(1400);
        for(JsonElement element:q.getAsJsonArray("entries")){JsonObject n=element.getAsJsonObject();entries.putShort(n.get("monster").getAsShort()).putShort(n.get("faction").getAsShort()).putShort(n.get("script").getAsShort()).putInt(n.get("x").getAsInt()).putInt(n.get("y").getAsInt());}
        h.write(0x437a94,entries.array());h.value(0x43800c,2,q.get("count").getAsInt());h.value(0x437a90,4,q.get("pointer").getAsInt());
        if(health.get("effect").getAsInt()!=c.getAsJsonArray("attack_packet").get(18).getAsInt())throw new Exception("Current Health effect and packet alias disagree");
        h.value(0x435662,1,health.get("marks_affected").getAsBoolean()?1:0);h.write(0x442798,words(health.getAsJsonArray("affected")));
        h.value(0x442750,1,health.get("marks_affected").getAsBoolean()?1:0);h.value(0x408574,2,health.get("last_body").getAsInt());
        h.value(0x442788,4,s.get("party_currency").getAsInt());h.value(0x45365c,2,s.get("loaded_item_field32").getAsInt());
        h.value(0x442749,1,s.get("psi_disabled").getAsInt());h.write(0x442742,bytes(s.getAsJsonArray("party_mask")));
        h.write(0x442580,new byte[12]);h.write(0x437a7a,new byte[12]);JsonArray choices=s.getAsJsonArray("psi_menu_choices");
        for(int i=0;i<choices.size();i++)if(!choices.get(i).isJsonNull()){JsonObject n=choices.get(i).getAsJsonObject();h.value(0x442580+2*i,1,n.get("row").getAsInt());h.value(0x442581+2*i,1,n.get("column").getAsInt());h.value(0x437a7a+2*i,2,n.get("power").getAsInt());}
        h.value(0x44374c,2,reconstructedKilled);h.value(0x4427a6,2,0);
    }

    @Override JsonObject completeSnap(ActionOracleHarness h,RunResult r)throws Exception {
        JsonObject j=super.completeSnap(h,r);j.remove("shot_events");j.remove("collision_tests");
        j.addProperty("backup_warriors",hex(h.read(0x44ecb0,18828)));
        j.addProperty("backup_digest",sha(h.read(0x44ecb0,18828)));
        j.addProperty("warrior_record_envelope",hex(h.read(0x448aa0,14*3138)));
        j.addProperty("backup11_alias_word",h.word(0x4511a8));j.addProperty("backup12_alias_word",h.word(0x451dea));
        j.addProperty("item_cache_1",hex(h.read(0x46225e,30000)));j.addProperty("item_cache_2",hex(h.read(0x45ad2e,30000)));j.addProperty("item_cache_3",hex(h.read(0x46978e,30000)));j.addProperty("scenario_item_cache",hex(h.read(0x470cbe,30000)));
        j.addProperty("warrior_editor_override",h.word(0x4471ec));j.addProperty("warrior_editor_registered",h.word(0x4046ea));
        return j;
    }

    @Override void observe(ActionOracleHarness h,RunResult r,long pc)throws Exception {
        super.observe(h,r,pc);
        if(writerControl&&(pc==0x162290||pc==0x162296)){
            long source=h.cpu.readRegister("A0").longValue(),destination=h.cpu.readRegister("A1").longValue();int width=pc==0x162290?4:2;
            JsonObject write=new JsonObject();write.addProperty("address",Long.toHexString(pc));write.addProperty("source",Long.toHexString(source));write.addProperty("destination",Long.toHexString(destination));write.addProperty("width",width);write.addProperty("bytes",hex(h.read(source,width)));write.addProperty("loop_index",h.word(0x4434aa));backupWrites.add(write);
        }
        for(int i=activeCurrentPhases.size()-1;i>=0;i--){JsonObject phase=activeCurrentPhases.get(i);if(pc==phase.get("return_address_value").getAsLong()&&h.sp()==phase.get("entry_sp_value").getAsLong()+4){phase.addProperty("d0_return_word",(short)h.cpu.readRegister("D0").intValue());phase.add("after",completeSnap(h,r));activeCurrentPhases.remove(i);}}
        if((pc==0x11f1a8||pc==0x130afe||pc==0x13145c||pc==0x1314fc||pc==0x159192||pc==0x10d4fe||pc==0x151022||pc==0x15075a||pc==0x125a3a)&&activeCurrentPhases.stream().noneMatch(phase->phase.get("address").getAsString().equals(Long.toHexString(pc))&&phase.get("entry_sp_value").getAsLong()==h.sp())){
            JsonObject phase=new JsonObject();phase.addProperty("address",Long.toHexString(pc));phase.addProperty("entry_sp_value",h.sp());phase.addProperty("return_address_value",h.pointer(h.sp()));
            JsonArray args=new JsonArray();int count=pc==0x15075a?5:pc==0x1314fc?3:pc==0x11f1a8||pc==0x130afe||pc==0x13145c||pc==0x10d4fe||pc==0x125a3a?1:0;for(int i=0;i<count;i++)args.add(h.word(h.sp()+4+i*2));phase.add("arguments",args);phase.add("before",completeSnap(h,r));currentPhases.add(phase);activeCurrentPhases.add(phase);
        }
        if(pc==0x1145c8||pc==0x1150d6||pc==0x115776||pc==0x115788||pc==0x11578a||pc==0x11fd42||pc==0x11fd6e||pc==0x150130||pc==0x15014c||pc==0x150156||pc==0x1505b2||pc==0x1505d0||pc==0x1505fe){
            JsonObject event=new JsonObject();event.addProperty("address",Long.toHexString(pc));event.addProperty("d0",h.cpu.readRegister("D0").intValue());event.addProperty("body",(short)h.cpu.readRegister("D4").intValue());event.addProperty("draws",r.draws);event.addProperty("sound_count",soundCount(r));
            if(pc==0x15014c||pc==0x150156){int body=(short)h.cpu.readRegister("D4").intValue();long alias=0x448aa0+body*3138L+0x32;event.addProperty("alias_address",Long.toHexString(alias));event.addProperty("alias_word",h.word(alias));}
            event.add("snapshot",completeSnap(h,r));currentEvents.add(event);
        }
    }
}
