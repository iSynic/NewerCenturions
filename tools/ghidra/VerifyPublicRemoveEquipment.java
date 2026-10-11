import java.nio.file.Files;
import java.nio.file.Path;
import com.google.gson.*;

/** Direct original Items Remove arguments for the finite firearm family. */
public class VerifyPublicRemoveEquipment extends VerifyEquipmentSpellCacheAlias {
    @Override public void run()throws Exception {
        root=Path.of(getScriptArgs()[0]);
        if(!ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyEquipmentSpellCacheAlias.java")).equals("ace119284790dbb1448730befe213300f91b99ecfe0e764d1a852514de8a1e07")||!ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyAmmunitionManagement.java")).equals("3702849cbc555b3ca18ad0e2bd2c41caae24f6d703ce65092fed8c78b7b991a1"))throw new Exception("Frozen parent probe changed");
        Path original=root.resolve("NewCenturions/TheNewCenturions"),scenario=original.resolve("Scenarios/The Thunder of Guns/0");
        items=Files.readAllBytes(original.resolve("Data Files/Data ID"));scenarioItems=Files.readAllBytes(scenario.resolve("Data NI"));races=Files.readAllBytes(original.resolve("Data Files/Data Race"));mos=Files.readAllBytes(original.resolve("Data Files/Data MOS"));monsters=Files.readAllBytes(scenario.resolve("Data MD"));properties=Files.readAllBytes(original.resolve("Data Files/Data Swamp BD"));attacks=Files.readAllBytes(original.resolve("Data Files/Data S"));areas=Files.readAllBytes(original.resolve("Data Files/Data AD"));
        party=new byte[18828];String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};for(int i=0;i<6;i++)System.arraycopy(Files.readAllBytes(original.resolve("Warriors/"+names[i])),0,party,i*3138,3138);
        warningStrings=Files.readAllBytes(root.resolve("analysis/extracted/resources/74/53545223_3.bin"));
        templates.addProperty("1","000100000000000c00000f0000010000000000000000");templates.addProperty("2","000200000000000c00000f0000010000000000000000");templates.addProperty("38","002600000000000300000f0000040023000000000000");templates.addProperty("165","00a500010118000100000f000003000000000a000000");
        JsonArray cases=new JsonArray();
        for(int id:new int[]{1,2,38,165})control(cases,id,false);
        control(cases,38,true);
        JsonObject identity=JsonParser.parseString(Files.readString(root.resolve("evidence/fixtures/ammunition-management.json"))).getAsJsonObject().getAsJsonObject("identity").deepCopy();identity.addProperty("ammunition_probe_sha256",identity.get("probe_sha256").getAsString());identity.addProperty("probe_sha256",ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyPublicRemoveEquipment.java")));identity.addProperty("equipment_cache_probe_sha256",ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyEquipmentSpellCacheAlias.java")));identity.addProperty("ammunition_fixture_sha256",ActionOracleHarness.sha256(root.resolve("evidence/fixtures/ammunition-management.json")));identity.addProperty("source_qualification_digest",ActionOracleHarness.sha256(root.resolve("evidence/equipment-special-effect-offset-qualification.md")));identity.addProperty("remove_source_digest",ActionOracleHarness.sha256(root.resolve("analysis/ghidra/shield-exports/removeitem.asm.tsv")));
        JsonObject out=new JsonObject();out.addProperty("authority","Controlled original starter party and finite authored inventory instances, full native initial wear/prep followed by direct original Items Remove arguments(member,slot,1,0). Combat AP0 active controls and inactive guard; complete before/after buffers and ordered full-party/work/cache snapshots. Bounded emulation, no original live menu or earned campaign claim.");out.add("identity",identity);out.add("controlled_instance_templates",templates);out.add("cases",cases);out.add("shared_rule_buffers",pooledBuffers);Files.writeString(root.resolve("analysis/completion/public-remove-equipment-draft.json"),out+"\n");println("Public Remove controls exported:"+cases.size());
    }
    void control(JsonArray cases,int id,boolean inactive)throws Exception {
        member=0;begin(12);resetTrace();aliasWrites=new JsonArray();aliasGates=new JsonArray();spellPhases=new JsonArray();liveSpellPhases.clear();pendingAliasWrite=null;capturing=true;ActionOracleHarness h=frame();RunResult prep=new RunResult(12);
        try {
            stockState(h,prep);h.write(0x443788,attacks);long base=0x448aa0;
            h.value(base+0x3a,2,5);h.value(base+0x40,2,8);h.value(base+0x3c,2,8);
            int slot=0;if(id!=1){slot=append(h,instance(id,id==165?1:capacity(id)-1,ammo(id)));selectOwned(h,slot);if(invoke(h,prep,0x164a0e,member,slot,1)!=1)throw new Exception("Initial native wear failed");}
            int installedSlot=slot;invoke(h,prep,0x12006e,member);slot=-1;for(int i=0;i<h.word(base+0x1a);i++)if(h.word(base+0x132+i*22)==id&&h.read(base+0x132+i*22+9,1)[0]==1){slot=i;break;}if(slot<0)throw new Exception("Native inventory compaction lost equipped record");selectOwned(h,slot);invoke(h,prep,0x130afe,id);invoke(h,prep,0x14c6c2,member,0);
            if(prep.draws!=0||h.read(base+0x132+slot*22+9,1)[0]!=1)throw new Exception("Native equipped setup disagreement id="+id+" primary="+h.word(base+0xe4)+" secondary="+h.word(base+0xe6)+" equipped="+h.read(base+0x132+slot*22+9,1)[0]);
            h.value(base+0x40,2,0);h.value(0x438776,2,1);h.value(0x4470b8,2,inactive?1:0);h.value(0x447194,2,0);
            JsonObject c=new JsonObject();c.addProperty("variant",inactive?"inactive_guard":"active_ap_zero");c.addProperty("member",member);c.addProperty("id",id);c.addProperty("slot",slot);c.addProperty("initial_native_wear_slot",installedSlot);c.addProperty("combat",1);c.addProperty("seed",12);c.add("native_setup_phases",phases);c.add("native_setup_mutations",operationEvents);c.add("native_setup_sounds",JsonParser.parseString("["+prep.sounds+"]"));
            resetTrace();aliasWrites=new JsonArray();aliasGates=new JsonArray();capturing=true;RunResult r=new RunResult(12);c.add("before",wholeSnap(h,r));int result=invoke(h,r,0x14ef76,member,slot,1,0);c.addProperty("native_helper_return_word",result);c.add("after",wholeSnap(h,r));c.add("ordered_native_phases",phases);c.add("ordered_mutation_events",operationEvents);c.add("ordered_alias_gates",aliasGates);c.add("ordered_alias_writes",aliasWrites);c.add("warnings",warnings);c.add("warning_phases",warningPhases);c.add("native_calls",nativeCalls);c.add("boundary_events",medicalEvents);c.add("rng",JsonParser.parseString("["+r.drawRecords+"]"));c.add("sounds",JsonParser.parseString("["+r.sounds+"]"));
            if(result!=(inactive?0:1)||!livePhases.isEmpty()||pendingMutation!=null||diagnostic||r.draws!=0||aliasWrites.size()!=0)throw new Exception("Incomplete native public Remove control");cases.add(c);println("Public Remove ID"+id+" "+c.get("variant").getAsString()+" returned"+result);
        }finally{h.cpu.dispose();capturing=false;}
    }
    @Override JsonObject smallSnap(ActionOracleHarness h,RunResult r)throws Exception {
        JsonObject j=super.smallSnap(h,r);buffer(j,"party",h,0x448aa0,18828);buffer(j,"remove_warrior_work",h,0x447e5e,3138);buffer(j,"selected_warrior_work",h,0x44721c,3138);return j;
    }
}
