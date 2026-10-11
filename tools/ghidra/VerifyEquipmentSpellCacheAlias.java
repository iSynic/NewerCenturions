import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import com.google.gson.*;

/** Original equipment cache gates and subsequent spell packet read. */
public class VerifyEquipmentSpellCacheAlias extends VerifyAmmunitionManagement {
    JsonArray aliasWrites=new JsonArray(),spellPhases=new JsonArray(),aliasGates=new JsonArray();
    java.util.ArrayList<JsonObject> liveSpellPhases=new java.util.ArrayList<>();
    JsonObject pendingAliasWrite;

    @Override public void run()throws Exception {
        root=Path.of(getScriptArgs()[0]);
        String contract="8e9ba6d062293f4a869f3fa24828c6a5ba2ba6ffe0cfeee1903d4232a5740d13";
        if(!ActionOracleHarness.sha256(root.resolve("evidence/equipment-spell-cache-alias.md")).equals(contract))throw new Exception("Frozen equipment cache contract changed");
        String qualification="e136d4dce168e3ac021be311fd65262070fb787c1ecba9015e8e5f85e32d9701";
        if(!ActionOracleHarness.sha256(root.resolve("evidence/equipment-special-effect-offset-qualification.md")).equals(qualification)||!ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyAmmunitionManagement.java")).equals("3702849cbc555b3ca18ad0e2bd2c41caae24f6d703ce65092fed8c78b7b991a1")||!ActionOracleHarness.sha256(root.resolve("evidence/fixtures/ammunition-management.json")).equals("4f2b282b7e6735dd3428d39bdaaabb160f10e02442ad81052032a604742bf9bb"))throw new Exception("Frozen equipment input dependency changed");
        Path original=root.resolve("NewCenturions/TheNewCenturions"),scenario=original.resolve("Scenarios/The Thunder of Guns/0");
        items=Files.readAllBytes(original.resolve("Data Files/Data ID"));scenarioItems=Files.readAllBytes(scenario.resolve("Data NI"));races=Files.readAllBytes(original.resolve("Data Files/Data Race"));mos=Files.readAllBytes(original.resolve("Data Files/Data MOS"));monsters=Files.readAllBytes(scenario.resolve("Data MD"));properties=Files.readAllBytes(original.resolve("Data Files/Data Swamp BD"));attacks=Files.readAllBytes(original.resolve("Data Files/Data S"));areas=Files.readAllBytes(original.resolve("Data Files/Data AD"));
        if(attacks.length!=11368||java.nio.ByteBuffer.wrap(attacks).getShort(3496)!=266)throw new Exception("Original authored cache input changed");
        party=new byte[18828];String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};for(int i=0;i<6;i++)System.arraycopy(Files.readAllBytes(original.resolve("Warriors/"+names[i])),0,party,i*3138,3138);
        warningStrings=Files.readAllBytes(root.resolve("analysis/extracted/resources/74/53545223_3.bin"));
        templates.addProperty("38","002600000000000300000f0000040023000000000000");
        JsonArray cases=new JsonArray();
        for(boolean replacing:new boolean[]{false,true}){
            member=0;begin(12);resetTrace();aliasWrites=new JsonArray();spellPhases=new JsonArray();aliasGates=new JsonArray();liveSpellPhases.clear();pendingAliasWrite=null;capturing=true;
            ActionOracleHarness h=frame();RunResult preparation=new RunResult(12);
            try {
                stockState(h,preparation);
                // Establish the declared complete cache input after inherited target scratch setup.
                h.write(0x443788,attacks);long base=0x448aa0;
                h.value(base+0x3a,2,5);h.value(base+0x40,2,8);h.value(base+0x3c,2,8);
                if(h.word(base+0xe4)!=1||h.word(base+0x132)!=1||h.read(base+0x13b,1)[0]!=1)throw new Exception("Original equipped item1 setup");
                int slot=append(h,instance(38,3,203));selectOwned(h,slot);
                invoke(h,preparation,0x130afe,1);invoke(h,preparation,0x14c6c2,0,0);
                if(preparation.draws!=0||h.word(0x444530)!=266||aliasWrites.size()!=0)throw new Exception("Cache control setup already mutated alias");
                JsonObject c=new JsonObject();c.addProperty("variant",replacing?"wear_retry_old1_to38":"no_removal_control");c.addProperty("member",0);c.addProperty("slot",slot);c.addProperty("combat",0);c.addProperty("spell",2205);c.addProperty("source_packet_offset",3480);c.addProperty("alias_cache_offset",3496);
                c.add("native_setup_phases",phases);c.add("native_setup_mutations",operationEvents);c.add("native_setup_sounds",JsonParser.parseString("["+preparation.sounds+"]"));
                resetTrace();aliasWrites=new JsonArray();spellPhases=new JsonArray();aliasGates=new JsonArray();liveSpellPhases.clear();pendingAliasWrite=null;capturing=true;RunResult r=new RunResult(12);
                c.add("before",wholeSnap(h,r));int result=0;
                if(replacing){result=invoke(h,r,0x164a0e,0,slot,1);if(result!=1)throw new Exception("Source wear retry admission failed");}
                c.addProperty("equipment_return_word",result);c.add("after_equipment",wholeSnap(h,r));
                int spellResult=invoke(h,r,0x13145c,2205);if(spellResult!=0)throw new Exception("Source spell load return");
                byte[] expected=Arrays.copyOfRange(attacks,3480,3538);
                if(!Arrays.equals(h.read(0x44374e,58),expected)||h.word(0x444530)!=266||!Arrays.equals(h.read(0x443788,11368),attacks)){byte[] cache=h.read(0x443788,11368);StringBuilder differences=new StringBuilder();for(int x=0;x<cache.length;x++)if(cache[x]!=attacks[x]&&differences.length()<300)differences.append(x).append(':').append(cache[x]&255).append('/').append(attacks[x]&255).append(',');throw new Exception("Native complete authored cache/packet preservation disagreement "+c.get("variant")+" alias="+h.word(0x444530)+" cache="+differences+" packet="+hex(h.read(0x44374e,58)));}
                if(aliasWrites.size()!=0||aliasGates.size()!=(replacing?4:0)||!livePhases.isEmpty()||!liveSpellPhases.isEmpty()||pendingAliasWrite!=null||diagnostic)throw new Exception("Incomplete equipment cache trace");
                c.add("after_spell_load",wholeSnap(h,r));c.add("ordered_native_phases",phases);c.add("spell_load_phases",spellPhases);c.add("ordered_alias_writes",aliasWrites);c.add("ordered_alias_gates",aliasGates);c.add("ordered_mutation_events",operationEvents);c.add("warnings",warnings);c.add("native_calls",nativeCalls);c.add("boundary_events",medicalEvents);c.add("rng",JsonParser.parseString("["+r.drawRecords+"]"));c.add("sounds",JsonParser.parseString("["+r.sounds+"]"));cases.add(c);
                println("Equipment cache control "+c.get("variant").getAsString()+" WORD"+h.word(0x444530));
            }finally{h.cpu.dispose();capturing=false;}
        }
        JsonObject identity=JsonParser.parseString(Files.readString(root.resolve("evidence/fixtures/ammunition-management.json"))).getAsJsonObject().getAsJsonObject("identity").deepCopy();identity.addProperty("ammunition_probe_sha256",identity.get("probe_sha256").getAsString());identity.addProperty("ammunition_fixture_sha256",ActionOracleHarness.sha256(root.resolve("evidence/fixtures/ammunition-management.json")));identity.addProperty("probe_sha256",ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyEquipmentSpellCacheAlias.java")));identity.addProperty("source_contract_digest",contract);identity.addProperty("source_qualification_digest",qualification);
        JsonObject out=new JsonObject();out.addProperty("authority","Controlled complete original starter party with declared full authored11368-byte DataS cache input; native full wear/remove/retry versus no-removal preserves the full cache because original secondary special indices WORD46/48 are zero, followed by full original loadspell2(2205). No startup-tail ownership or live menu proof.");out.add("identity",identity);out.add("cases",cases);out.add("shared_rule_buffers",pooledBuffers);Files.writeString(root.resolve("analysis/completion/equipment-spell-cache-alias-draft.json"),out+"\n");println("Equipment spell cache alias controls exported:"+cases.size());
    }
    @Override JsonObject smallSnap(ActionOracleHarness h,RunResult r)throws Exception {
        JsonObject j=super.smallSnap(h,r);buffer(j,"spell_cache",h,0x443788,11368);j.addProperty("spell_alias_word",h.word(0x444530));j.addProperty("packet",hex(h.read(0x44374e,58)));j.addProperty("identity",hex(h.read(0x4435c6,6)));return j;
    }
    @Override void observe(ActionOracleHarness h,RunResult r,long pc)throws Exception {
        if(!capturing)return;
        if(pendingAliasWrite!=null){pendingAliasWrite.addProperty("next_address",Long.toHexString(pc));pendingAliasWrite.add("after_instruction",smallSnap(h,r));pendingAliasWrite=null;}
        for(int i=liveSpellPhases.size()-1;i>=0;i--){JsonObject p=liveSpellPhases.get(i);if(pc==p.get("return_address_value").getAsLong()&&h.sp()==p.get("entry_sp_value").getAsLong()+4){p.addProperty("d0_return_word",(short)h.cpu.readRegister("D0").intValue());p.add("after",smallSnap(h,r));liveSpellPhases.remove(i);}}
        if(pc==0x13145c){JsonObject p=new JsonObject();p.addProperty("address","13145c");p.addProperty("entry_sp_value",h.sp());p.addProperty("return_address_value",h.pointer(h.sp()));p.addProperty("argument",h.word(h.sp()+4));p.add("before",smallSnap(h,r));spellPhases.add(p);liveSpellPhases.add(p);}
        if(pc==0x14f2d8||pc==0x14f366||pc==0x165654||pc==0x1656f2){JsonObject e=new JsonObject();e.addProperty("address",Long.toHexString(pc));e.addProperty("index46_word",h.word(0x45366a));e.addProperty("index48_word",h.word(0x45366c));e.addProperty("preceding_magnitude44_word",h.word(0x453668));e.addProperty("magnitude50_word",h.word(0x45366e));e.add("snapshot",smallSnap(h,r));aliasGates.add(e);}
        if(pc==0x14f362||pc==0x1656ee){int index=(short)h.cpu.readRegister(pc==0x14f362?"D0":"D1").intValue();long a0=h.cpu.readRegister("A0").longValue(),address=pc==0x14f362?a0+2L*index:a0;if(address!=0x444530)throw new Exception("Unexpected native equipment cache effective address");JsonObject e=new JsonObject();e.addProperty("address",Long.toHexString(pc));e.addProperty("effective_address",Long.toHexString(address));e.addProperty("index_word",index);e.addProperty("a0",Long.toHexString(a0));e.addProperty("width",2);e.addProperty("source_d0_word",(short)h.cpu.readRegister("D0").intValue());e.add("before_instruction",smallSnap(h,r));aliasWrites.add(e);pendingAliasWrite=e;}
        super.observe(h,r,pc);
    }
}
