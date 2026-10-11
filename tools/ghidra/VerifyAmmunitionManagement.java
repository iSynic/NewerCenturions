import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import com.google.gson.*;

/** Original firearm replacement, selected ammunition and both refit operations. */
public class VerifyAmmunitionManagement extends VerifyEarnedCombatReloadBounds {
    Path root;
    int member,acceptedAmmo;
    boolean capturing;
    JsonArray phases=new JsonArray(),operationEvents=new JsonArray(),popupInputs=new JsonArray(),fitSoundReads=new JsonArray();
    java.util.ArrayList<JsonObject> livePhases=new java.util.ArrayList<>();
    JsonObject templates=new JsonObject();
    JsonObject pendingMutation;
    JsonObject pooledBuffers=new JsonObject();

    @Override public void run()throws Exception {
        root=Path.of(getScriptArgs()[0]);Path original=root.resolve("NewCenturions/TheNewCenturions"),scenario=original.resolve("Scenarios/The Thunder of Guns/0");
        if(!ActionOracleHarness.sha256(root.resolve("evidence/ammunition-management-and-replacement.md")).equals("b77d24ac5180fb9e92c1d52d6eb8aa2f92caf1b3dc1b917dfd8f4eec48efd75c"))throw new Exception("Frozen ammunition source contract changed");
        items=Files.readAllBytes(original.resolve("Data Files/Data ID"));scenarioItems=Files.readAllBytes(scenario.resolve("Data NI"));races=Files.readAllBytes(original.resolve("Data Files/Data Race"));mos=Files.readAllBytes(original.resolve("Data Files/Data MOS"));monsters=Files.readAllBytes(scenario.resolve("Data MD"));properties=Files.readAllBytes(original.resolve("Data Files/Data Swamp BD"));attacks=Files.readAllBytes(original.resolve("Data Files/Data S"));areas=Files.readAllBytes(original.resolve("Data Files/Data AD"));
        party=new byte[18828];String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};for(int i=0;i<6;i++)System.arraycopy(Files.readAllBytes(original.resolve("Warriors/"+names[i])),0,party,i*3138,3138);
        Path strings=root.resolve("analysis/extracted/resources/74/53545223_3.bin");if(!ActionOracleHarness.sha256(strings).equals("b11e47595c18787e109b64b05483f22f408fe07200a2ba977513ed2f826e2a41"))throw new Exception("Warning strings changed");warningStrings=Files.readAllBytes(strings);
        templates.addProperty("1","000100000000000c00000f0000010000000000000000");templates.addProperty("2","000200000000000c00000f0000010000000000000000");templates.addProperty("38","002600000000000300000f0000040023000000000000");templates.addProperty("165","00a500010118000100000f000003000000000a000000");
        JsonArray cases=new JsonArray();
        for(int combat:new int[]{0,1})for(int[] pair:new int[][]{{1,2},{2,38},{38,165},{165,38}})operation(cases,"equip",pair[0],pair[1],0,0,0,12,combat,"ordinary");
        operation(cases,"equip",165,38,0,0,0,1,1,"preserved_enhancement18");operation(cases,"equip",1,38,0,0,0,12,0,"canuse_denied");operation(cases,"equip",1,38,0,0,0,12,0,"level_denied");
        for(int id:new int[]{1,2,38,165})operation(cases,"fit",id,0,0,0,capacity(id)+2,12,1,"ordinary");
        operation(cases,"fit",1,0,0,0,20,1,0,"ordinary");operation(cases,"fit",38,0,0,0,2,12,1,"partial");operation(cases,"fit",38,0,0,0,3,1,1,"equal");operation(cases,"fit",38,0,0,0,0,12,1,"cancel_no_candidate");
        operation(cases,"fit",1,0,4,200,20,12,1,"same_ammo_reselect");operation(cases,"fit",1,0,12,200,20,1,1,"full_same_ammo_reselect");operation(cases,"fit",2,0,12,201,20,12,1,"full_same_ammo_reselect");operation(cases,"fit",38,0,1,203,65538,12,1,"high_long_stock");operation(cases,"fit",38,0,0,0,4,12,1,"adjacent_loot_quality_writer");
        for(int id:new int[]{1,2,38,165})operation(cases,"manual_refit",id,0,id==165?0:1,ammo(id),capacity(id)+1,12,0,"ordinary");
        operation(cases,"manual_refit",38,0,0,203,2,12,0,"first_partial");operation(cases,"manual_refit",38,0,0,203,3,1,0,"first_equal");operation(cases,"manual_refit",38,0,3,203,4,12,0,"full_clip");
        for(int stock:new int[]{2,3,4,0})operation(cases,"manual_refit",38,0,0,0,stock,12,0,"second_pass");
        for(int stock:new int[]{65538,65537,65536,-1})operation(cases,"manual_refit",38,0,1,203,stock,12,0,"signed_low_word_stock");
        for(int id:new int[]{1,2,38,165})for(int extra:new int[]{0,1})operation(cases,"automatic_refit",id,0,0,ammo(id),capacity(id)+extra,12,1,extra==0?"equal":"sufficient");
        operation(cases,"automatic_refit",38,0,0,203,2,12,1,"partial");operation(cases,"automatic_refit",38,0,0,203,0,12,1,"exhausted");operation(cases,"automatic_refit",38,0,0,203,4,12,1,"warm_other_class");
        operation(cases,"move_reload",38,0,0,203,3,12,1,"occupied_cell_original_mos4");operation(cases,"move_reload",165,0,0,280,1,1,1,"occupied_cell_original_mos4");
        operation(cases,"move_reload",38,0,0,203,2,12,1,"partial_original_mos4");operation(cases,"move_reload",38,0,0,203,0,12,1,"exhausted_original_mos4");
        JsonObject identity=JsonParser.parseString(Files.readString(root.resolve("evidence/fixtures/earned-combat-reload-bounds.json"))).getAsJsonObject().getAsJsonObject("identity").deepCopy();identity.addProperty("reload_bounds_probe_sha256",identity.get("probe_sha256").getAsString());identity.addProperty("reload_bounds_fixture_sha256",ActionOracleHarness.sha256(root.resolve("evidence/fixtures/earned-combat-reload-bounds.json")));identity.addProperty("probe_sha256",ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyAmmunitionManagement.java")));identity.addProperty("source_contract_digest",ActionOracleHarness.sha256(root.resolve("evidence/ammunition-management-and-replacement.md")));
        JsonObject out=new JsonObject();out.addProperty("authority","Controlled original authored firearm definitions/instances on complete original starter party, full native wear/remove/retry, selected Fit helper and flags000/111 refit with native cache establishment and original warning49/25 dismissal. Whole before/after buffers and compact ordered phases captured. Not an earned route, original live input or campaign completion.");out.add("identity",identity);out.add("controlled_instance_templates",templates);out.add("cases",cases);out.add("shared_rule_buffers",pooledBuffers);Files.writeString(root.resolve("analysis/completion/ammunition-management-draft.json"),out+"\n");println("Ammunition management controlled native draft exported:"+cases.size());
    }
    int capacity(int id){return ByteBuffer.wrap(items).getShort(id*150+24);}
    int ammo(int id){return id==1?200:id==2?201:id==38?203:280;}
    byte[] unhex(String text){byte[] b=new byte[text.length()/2];for(int i=0;i<b.length;i++)b[i]=(byte)Integer.parseInt(text.substring(2*i,2*i+2),16);return b;}
    byte[] instance(int id,int loaded,int ammunition){byte[] b=unhex(templates.get(Integer.toString(id)).getAsString());ByteBuffer.wrap(b).putShort(2,(short)loaded).putShort(4,(short)ammunition);return b;}
    void stockState(ActionOracleHarness h,RunResult r)throws Exception {
        common(h,party);h.write(0x439d3c,new byte[28800]);h.write(0x4085b8,terrain());h.write(0x4470b8,new byte[220]);h.write(0x446f0a,new byte[400]);h.write(0x44709a,new byte[24]);
        h.write(0x4537fe,Arrays.copyOfRange(items,0,30000));h.write(0x46225e,Arrays.copyOfRange(items,30000,60000));h.write(0x45ad2e,Arrays.copyOfRange(items,60000,90000));h.write(0x46978e,Arrays.copyOfRange(items,90000,120000));h.write(0x470cbe,Arrays.copyOfRange(scenarioItems,0,30000));h.write(0x443788,attacks);h.write(0x438210,areas);h.write(0x43167c,Arrays.copyOfRange(properties,64,12064));
        h.write(0x4285f4,new byte[22000]);h.write(0x41fc48,new byte[22]);h.write(0x45363c,new byte[150]);h.write(0x453768,new byte[150]);h.write(0x4536d2,new byte[150]);h.write(0x44374e,new byte[58]);h.write(0x4435c6,new byte[6]);h.write(0x442562,new byte[20]);h.write(0x44258c,new byte[128]);h.write(0x43788a,new byte[480]);h.write(0x440edc,new byte[5760]);h.write(0x442bd6,new byte[1200]);h.write(0x435884,new byte[128]);h.write(0x4434e0,new byte[220]);h.write(0x443722,new byte[660]);h.write(0x442798,new byte[24]);h.write(0x42dbe4,new byte[800]);h.write(0x442742,new byte[6]);
        h.value(0x438776,2,0);h.value(0x438760,2,0);h.value(0x438764,2,0);h.value(0x43874a,2,0);h.value(0x4435c4,2,0);h.value(0x44374c,2,0);h.value(0x4435c0,2,0);h.value(0x4435cc,2,0);h.value(0x447194,2,0);h.value(0x44255c,1,1);h.value(0x4427a6,2,0);h.value(0x4471ec,2,0);h.value(0x44273f,1,member);h.value(0x4435d2,2,member);h.value(0x443318,2,member);h.value(0x437a6c,1,0);h.value(0x438780,2,0);h.value(0x442750,1,0);h.value(0x435662,1,0);h.value(0x4358c2,1,0);h.value(0x443580,1,0);h.value(0x439582,2,0);h.value(0x408570,2,5);h.value(0x408572,2,5);h.value(0x442780,4,5);h.value(0x442784,4,5);h.value(0x408574,2,0);h.write(0x479ed8,new byte[256]);h.write(0x439cec,new byte[10]);
        activeBody=member;bodyCount=0;recordLength=0;presentationControls(h);h.value(0x4470b8,2,member);
        h.cpu.writeRegister("A6",0x700000);h.cpu.writeRegister("SP",0x6ffe00);h.jump(0x1057f8);execute(h,r,0x10582a);
    }
    int invoke(ActionOracleHarness h,RunResult r,long entry,int... args)throws Exception {
        h.enter(entry,0x6ffe00,0x900000);for(int i=0;i<args.length;i++)h.value(h.sp()+4+2*i,2,args[i]);execute(h,r,0x900000);if(h.sp()!=0x6ffe04)throw new Exception("Native helper return stack");return (short)h.cpu.readRegister("D0").intValue();
    }
    byte[] positionsControl(int x,int y){return ByteBuffer.allocate(4).putShort((short)x).putShort((short)y).array();}
    int append(ActionOracleHarness h,byte[] record){long base=0x448aa0+member*3138;int slot=h.word(base+0x1a);if(slot>=99)throw new IllegalStateException("Controlled inventory capacity");h.write(base+0x132+slot*22,record);h.value(base+0x1a,2,slot+1);return slot;}
    void selectOwned(ActionOracleHarness h,int slot){h.write(0x41fc48,h.read(0x448aa0+member*3138+0x132+slot*22,22));h.value(0x44719c,2,slot);}
    void resetTrace(){phases=new JsonArray();livePhases.clear();operationEvents=new JsonArray();popupInputs=new JsonArray();fitSoundReads=new JsonArray();warnings=new JsonArray();warningPhases=new JsonArray();medicalEvents=new JsonArray();nativeCalls=new JsonArray();bitCalls=new JsonArray();aidEvents=new JsonArray();poisonEvents=new JsonArray();shotEvents=new JsonArray();collisionTests=new JsonArray();phaseReceipts=new JsonArray();activePhases.clear();reloadPhases=new JsonArray();activeReloadPhases.clear();warningRunning=false;trackingCounter=false;pendingMutation=null;capturing=false;}
    void operation(JsonArray output,String kind,int oldId,int newId,int loaded,int ammunition,int stock,long seed,int combat,String variant)throws Exception {
        member=kind.equals("move_reload")?4:variant.equals("canuse_denied")?1:0;if(kind.equals("equip")){loaded=oldId==165?1:capacity(oldId)-1;ammunition=ammo(oldId);}begin(seed);resetTrace();capturing=true;ActionOracleHarness h=frame();RunResult preparation=new RunResult(seed);
        try{
            stockState(h,preparation);long base=0x448aa0+member*3138;h.value(base+0x3a,2,5);h.value(base+0x40,2,8);h.value(base+0x3c,2,8);
            if(oldId!=1){byte[] oldRecord=instance(oldId,loaded,ammunition);if(oldId==165&&!variant.equals("preserved_enhancement18"))oldRecord[18]=0;int slot=append(h,oldRecord);selectOwned(h,slot);if(invoke(h,preparation,0x164a0e,member,slot,1)!=1)throw new Exception("Controlled original old weapon admission failed");invoke(h,preparation,0x12006e,member);}else{h.value(base+0x134,2,loaded);h.value(base+0x136,2,ammunition);invoke(h,preparation,0x12006e,member);}
            int selectedSlot=0;if(kind.equals("equip")){selectedSlot=append(h,instance(newId,capacity(newId),ammo(newId)));if(variant.equals("level_denied"))h.value(base+0x3a,2,3);}
            invoke(h,preparation,0x130afe,oldId);invoke(h,preparation,0x14c6c2,member,0);if(variant.equals("warm_other_class"))invoke(h,preparation,0x130afe,700);
            if(variant.equals("adjacent_loot_quality_writer")){h.value(0x4046e7,1,1);h.cpu.writeRegister("SP",0x6ffe00);h.jump(0x12ca28);execute(h,preparation,0x12ca32);}
            h.value(0x42dbe4+(ammo(oldId)-200)*4,4,stock);h.value(0x438776,2,combat);h.value(0x438760,2,0);h.value(0x4470b8,2,member);h.value(0x447194,2,0);selectOwned(h,selectedSlot);acceptedAmmo=stock>0?ammo(oldId):-1;
            if(kind.equals("move_reload")){
                byte[] map=terrain(),npc=Arrays.copyOfRange(monsters,13*288,14*288);place(h,map,npc,10,20,20);h.write(0x439d3c,npc);h.value(0x43874a,2,1);h.value(0x4435c4,2,1);bodyCount=1;recordLength=288;h.write(0x446f0a,positionsControl(20,20));
                for(int i=0;i<6;i++){int x=i==member?19:40+i,y=i==member?19:40;h.value(0x44709a+4*i,2,x);h.value(0x44709c+4*i,2,y);ByteBuffer.wrap(map).putShort(((x+5)*90+y+5)*2,(short)i);h.value(0x437a6e+2*i,2,1155);}h.write(0x4085b8,map);h.value(0x447196,2,0);h.value(0x438760,2,0);invoke(h,preparation,0x10e6b6,19,19);
            }
            if(preparation.draws!=0||preparation.seed!=seed)throw new Exception("Native setup consumed unexpected rule RNG");
            JsonObject c=new JsonObject();c.addProperty("kind",kind);c.addProperty("variant",variant);c.addProperty("member",member);c.addProperty("combat",combat);c.addProperty("old_id",oldId);c.addProperty("new_id",newId);c.addProperty("controlled_loaded",loaded);c.addProperty("controlled_ammo",ammunition);c.addProperty("controlled_stock_long",stock);c.addProperty("seed",seed);c.addProperty("slot",selectedSlot);c.add("native_setup",JsonParser.parseString("["+preparation.drawRecords+"]"));c.add("native_setup_sounds",JsonParser.parseString("["+preparation.sounds+"]"));c.add("native_setup_phases",phases);c.add("native_setup_mutations",operationEvents);
            RunResult r=new RunResult(seed);resetTrace();capturing=true;c.add("before",wholeSnap(h,r));int result;
            if(kind.equals("equip"))result=invoke(h,r,0x164a0e,member,selectedSlot,1);else if(kind.equals("fit"))result=invoke(h,r,0x1300ec,member,0,1,0);else if(kind.equals("manual_refit"))result=invoke(h,r,0x14e5e4,member,0,0,0);else if(kind.equals("automatic_refit"))result=invoke(h,r,0x14e5e4,member,1,1,1);else{
                direction(h,r);c.add("before_command",wholeSnap(h,r));h.cpu.writeRegister("SP",0x6ffe00);h.cpu.writeRegister("A6",0x6fff00);h.jump(0x116b2c);execute(h,r,0x116b6a);result=(short)h.cpu.readRegister("D0").intValue();c.add("command_return",wholeSnap(h,r));execute(h,r,0x117338);
            }
            c.addProperty("native_helper_return_word",result);c.add("after",wholeSnap(h,r));c.add("ordered_native_phases",phases);c.add("ordered_mutation_events",operationEvents);c.add("popup_inputs",popupInputs);c.add("fit_sound_table_reads",fitSoundReads);c.add("warnings",warnings);c.add("warning_phases",warningPhases);c.add("native_calls",nativeCalls);c.add("boundary_events",medicalEvents);c.add("collision_tests",collisionTests);c.add("rng",JsonParser.parseString("["+r.drawRecords+"]"));c.add("sounds",JsonParser.parseString("["+r.sounds+"]"));if(!livePhases.isEmpty()||diagnostic)throw new Exception("Incomplete controlled native operation");output.add(c);println("Ammo control "+kind+" ID"+oldId+" "+variant);
        }finally{h.cpu.dispose();capturing=false;}
    }
    void buffer(JsonObject out,String key,ActionOracleHarness h,long address,int length)throws Exception{byte[] bytes=h.read(address,length);String digest=sha(bytes);pooledBuffers.addProperty(digest,hex(bytes));JsonObject ref=new JsonObject();ref.addProperty("shared_buffer_sha256",digest);ref.addProperty("byte_length",length);out.add(key,ref);}
    JsonObject smallSnap(ActionOracleHarness h,RunResult r)throws Exception {
        JsonObject j=new JsonObject();buffer(j,"member_record",h,0x448aa0+member*3138,3138);buffer(j,"reserves",h,0x42dbe4,804);j.addProperty("item_work",hex(h.read(0x45363c,150)));j.addProperty("weapon_work",hex(h.read(0x453768,150)));j.addProperty("ammo_work",hex(h.read(0x4536d2,150)));j.addProperty("selected_record",hex(h.read(0x41fc48,22)));j.addProperty("markers",hex(h.read(0x442798,24)));j.addProperty("auto_mask",hex(h.read(0x442742,6)));j.addProperty("selection_control",h.pointer(0x439404));j.addProperty("mode",h.word(0x438760));j.addProperty("involuntary_movement",h.word(0x438764));j.addProperty("changed_ammo",h.read(0x437a6c,1)[0]&255);j.addProperty("seed",r.seed);j.addProperty("draws",r.draws);j.addProperty("sound_count",soundCount(r));j.addProperty("native_stack",Long.toHexString(h.sp()));j.addProperty("caller_frame",Long.toHexString(h.cpu.readRegister("A6").longValue()));return j;
    }
    JsonObject wholeSnap(ActionOracleHarness h,RunResult r)throws Exception {
        JsonObject j=smallSnap(h,r);String[] names={"party","npcs","map","order","party_positions","npc_positions","party_buffs","fields","field_used","flags","memory","references","crate","loot","loot_quality","selected_warrior_work","remove_warrior_work","backup_warriors","item_cache0","item_cache1","item_cache2","item_cache3","scenario_cache"};long[] addresses={0x448aa0,0x439d3c,0x4085b8,0x4470b8,0x44709a,0x446f0a,0x442562,0x442bd6,0x435884,0x44258c,0x43788a,0x440edc,0x4285f4,0x41fc74,0x42df04,0x44721c,0x447e5e,0x44ecb0,0x4537fe,0x46225e,0x45ad2e,0x46978e,0x470cbe};int[] lengths={18828,28800,16200,220,24,400,20,1200,128,128,480,5760,22000,22000,1000,3138,3138,18828,30000,30000,30000,30000,30000};for(int i=0;i<names.length;i++)buffer(j,names[i],h,addresses[i],lengths[i]);j.addProperty("packet",hex(h.read(0x44374e,58)));j.addProperty("identity",hex(h.read(0x4435c6,6)));j.addProperty("count",h.word(0x4435cc));j.addProperty("mask",hex(h.read(0x4434e0,220)));j.addProperty("targets",hex(h.read(0x443722,660)));j.addProperty("cursor",h.word(0x447194));j.addProperty("round_script_counter",(int)h.read(0x44255c,1)[0]);j.addProperty("inactivity_counter",h.word(0x4427a6));j.addProperty("killed_count",h.word(0x44374c));j.addProperty("override_word",h.word(0x4471ec));j.addProperty("health_effect",h.read(0x443760,1)[0]&255);j.addProperty("heal_marks_affected",h.read(0x442750,1)[0]&255);j.addProperty("health_last_body",h.word(0x408574));j.addProperty("shift_x",(int)h.pointer(0x442780));j.addProperty("shift_y",(int)h.pointer(0x442784));return j;
    }
    @Override JsonObject completeSnap(ActionOracleHarness h,RunResult r)throws Exception{return smallSnap(h,r);}
    @Override void observe(ActionOracleHarness h,RunResult r,long pc)throws Exception {
        if(!capturing)return;
        if(pendingMutation!=null){pendingMutation.addProperty("next_instruction_address",Long.toHexString(pc));pendingMutation.add("after_instruction",smallSnap(h,r));pendingMutation=null;}
        if(pc==0x130ab2){long frame=h.cpu.readRegister("A6").longValue();int result=(short)h.cpu.readRegister("D6").intValue(),index=(short)(result-2);long address=frame-0x264+2L*index;JsonObject read=new JsonObject();read.addProperty("address",Long.toHexString(pc));read.addProperty("frame",frame);read.addProperty("popup_result_word",result);read.addProperty("table_index_word",index);read.addProperty("effective_address",address);read.addProperty("effective_word",h.word(address));read.addProperty("table_prefix",hex(h.read(frame-0x268,12)));read.add("snapshot",smallSnap(h,r));fitSoundReads.add(read);}
        for(int i=livePhases.size()-1;i>=0;i--){JsonObject p=livePhases.get(i);if(pc==p.get("return_address_value").getAsLong()&&h.sp()==p.get("entry_sp_value").getAsLong()+4){p.addProperty("d0_return_word",(short)h.cpu.readRegister("D0").intValue());p.add("after",smallSnap(h,r));livePhases.remove(i);}}
        int argc=pc==0x164a0e||pc==0x157586?3:pc==0x14ef76||pc==0x1300ec||pc==0x14e5e4?4:pc==0x14c6c2||pc==0x10e6b6?2:pc==0x130afe||pc==0x130e54||pc==0x130d38||pc==0x139192||pc==0x136994||pc==0x12006e||pc==0x14edd0||pc==0x111984||pc==0x12946c||pc==0x110e96||pc==0x11f1a8?1:-1;
        if(argc>=0&&livePhases.stream().noneMatch(p->p.get("address").getAsString().equals(Long.toHexString(pc))&&p.get("entry_sp_value").getAsLong()==h.sp())){JsonObject p=new JsonObject();p.addProperty("address",Long.toHexString(pc));p.addProperty("entry_sp_value",h.sp());p.addProperty("return_address_value",h.pointer(h.sp()));JsonArray args=new JsonArray();for(int i=0;i<argc;i++)args.add(h.word(h.sp()+4+2*i));p.add("arguments",args);p.add("before",smallSnap(h,r));phases.add(p);livePhases.add(p);}
        long[] landmarks={0x11b8ee,0x11b902,0x11b9ac,0x11bc7a,0x11bfc0,0x1114da,0x11150c,0x111588,0x111544,0x14ee22,0x14ee76,0x116b6a,0x117338,0x164ca6,0x164cb8,0x164eb8,0x1653a2,0x16548a,0x1657fc,0x14f16e,0x14f4ee,0x14f550,0x14f660,0x14f950,0x14e7bc,0x14e8a4,0x14e95e,0x14eb3e,0x14eb5c,0x14ec54,0x14ec76,0x14ed42,0x13082c,0x13087e,0x1308bc,0x1308d8,0x1308f8,0x130918,0x130940,0x1309ac,0x1309f2,0x130a08,0x130a1a,0x130a32,0x130ab2,0x130ac0};
        for(long address:landmarks)if(pc==address){JsonObject e=new JsonObject();e.addProperty("address",Long.toHexString(pc));e.addProperty("d0",h.cpu.readRegister("D0").intValue());e.addProperty("d3",h.cpu.readRegister("D3").intValue());e.addProperty("d4",h.cpu.readRegister("D4").intValue());e.addProperty("d5",h.cpu.readRegister("D5").intValue());e.addProperty("d6",h.cpu.readRegister("D6").intValue());e.add("before_instruction",smallSnap(h,r));operationEvents.add(e);pendingMutation=e;}
    }
    void popup(ActionOracleHarness h,RunResult r)throws Exception {
        long frame=h.cpu.readRegister("A6").longValue();int count=h.word(frame-0x266),result=0;JsonArray candidates=new JsonArray();for(int i=0;i<count;i++){int value=h.word(frame-0x264+2*i);candidates.add(value);if(value==acceptedAmmo-200)result=i+2;}if(acceptedAmmo>=0&&result==0)throw new Exception("Accepted ammo absent from native candidate table");JsonObject j=new JsonObject();j.add("candidate_indices",candidates);j.addProperty("accepted_ammo",acceptedAmmo);j.addProperty("popup_result",result);j.addProperty("adjacent_loot_quality_long",(int)h.pointer(0x42df04));j.add("before",smallSnap(h,r));popupInputs.add(j);h.cpu.writeRegister("D6",result);h.jump(0x13075c);
    }
    @Override void stringBoundary(ActionOracleHarness h){long sp=h.sp();if(warningRunning&&h.word(sp+6)==3){int wanted=h.word(sp+4),offset=2;for(int i=1;i<wanted;i++)offset+=1+(warningStrings[offset]&255);int length=warningStrings[offset]&255;byte[] text=Arrays.copyOfRange(warningStrings,offset,offset+length+1);h.write(h.pointer(sp+8),text);JsonObject e=new JsonObject();e.addProperty("resource",3);e.addProperty("index",wanted);e.addProperty("pascal_text",hex(text));e.addProperty("sp_before",sp);h.ret();h.pop(8);e.addProperty("sp_after",h.sp());medicalEvents.add(e);}else super.stringBoundary(h);}
    @Override void nativeWarning(ActionOracleHarness h,RunResult r)throws Exception {
        long sp=h.sp(),ret=h.pointer(sp),d3=h.cpu.readRegister("D3").longValue(),a2=h.cpu.readRegister("A2").longValue();int number=h.word(sp+4);if(number!=10&&number!=25&&number!=29&&number!=49)throw new Exception("Uncontracted ammunition warning"+number);warnings.add(number);JsonObject p=new JsonObject();p.addProperty("argument",number);p.addProperty("entry_sp",sp);p.addProperty("return_address",ret);p.add("before",smallSnap(h,r));warningRunning=true;try{if(!h.cpu.step(monitor))throw new Exception("Warning native LINK");execute(h,r,ret);}finally{warningRunning=false;}if(h.sp()!=sp+4||h.cpu.readRegister("D3").longValue()!=d3||h.cpu.readRegister("A2").longValue()!=a2)throw new Exception("Warning native preserved ABI");p.addProperty("d0_return_word",(short)h.cpu.readRegister("D0").intValue());p.add("after",smallSnap(h,r));warningPhases.add(p);
    }
    void execute(ActionOracleHarness h,RunResult result,long stop) throws Exception {
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
            if(pc==0x158454){long caller=h.pointer(sp);for(int i=0;h.pc()!=0x15847a;i++){if(i>12||!h.cpu.step(monitor))throw new Exception("Native result prefix failed");}long savedStack=h.sp();h.cpu.writeRegister("SP",sp);showresultsSound(h,result);h.cpu.writeRegister("SP",savedStack);JsonObject e=new JsonObject();e.addProperty("address","158474");e.addProperty("aid_gate",h.word(0x438786));e.addProperty("return",Long.toHexString(caller));medicalEvents.add(e);h.jump(0x15916c);execute(h,result,0x159178);h.jump(0x15917a);execute(h,result,caller);continue;}
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
            if(op==0xa9b4){h.jump(pc+2);continue;}
            if(op==0xa974){h.cpu.writeRegister("SP",sp-2);h.value(sp-2,2,0);h.jump(pc+2);continue;}
            if(!h.cpu.step(monitor))throw new Exception("Ally action at "+Long.toHexString(pc)+": "+h.cpu.getLastError());
        }
        if(!done)throw new Exception("Ally action instruction budget at "+Long.toHexString(h.pc()));
    }

}
