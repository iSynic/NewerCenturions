import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import com.google.gson.*;

/** Original learned Crystalize cast, field creation and automatic initiative. */
public class VerifyCrystalizePsi extends VerifySupportPsiAdvance {
    JsonObject advanceSnap(ActionOracleHarness h,RunResult r)throws Exception {
        JsonObject j=snap(h,r);j.addProperty("order",hex(h.read(0x4470b8,220)));j.addProperty("cursor",h.word(0x447194));j.addProperty("monster_index",h.word(0x447196));j.addProperty("active_member",h.word(0x4435d2));j.addProperty("round",h.read(0x44255c,1)[0]&255);j.addProperty("death_count",h.word(0x44374c));j.addProperty("enemy_phase",h.word(0x438778));j.addProperty("item_context",h.word(0x45365c));j.addProperty("weapon_work",hex(h.read(0x453768,150)));j.addProperty("ammo_work",hex(h.read(0x4536d2,150)));j.add("rng_records",JsonParser.parseString("["+r.drawRecords+"]"));j.add("sounds",JsonParser.parseString("["+r.sounds+"]"));return j;
    }
    JsonArray fieldEvents=new JsonArray(),sites=new JsonArray(),soundCalls=new JsonArray();
    @Override void observe(ActionOracleHarness h,RunResult r,long pc)throws Exception {
        super.observe(h,r,pc);if(pc==0x133a18){JsonObject a=new JsonObject();a.addProperty("return_address",Long.toHexString(h.pointer(h.sp())));a.addProperty("argument",h.word(h.sp()+4));soundCalls.add(a);}if(pc==0x153ca4)sites.add(Long.toHexString(h.pointer(h.sp())));if(pc==0x15cba6||pc==0x15cd9c){JsonObject e=new JsonObject();e.addProperty("address",Long.toHexString(pc));e.addProperty("fields",hex(h.read(0x442bd6,1200)));e.addProperty("rng_draws",r.draws);fieldEvents.add(e);}
    }
    void advance(ActionOracleHarness h,RunResult r,long stop)throws Exception {
        SaneBoundary sane=new SaneBoundary(h.cpu,currentProgram);
        for(int step=0;step<1500000;step++){
            long pc=h.pc(),sp=h.sp();observe(h,r,pc);if(pc==stop)return;
            if(pc==0x400038){h.cpu.writeRegister("D0",(short)Math.abs(h.word(sp+4)));h.ret();continue;}
            if(pc==0x153ca4){r.rng(h);continue;}
            if(pc==0x133a18||pc==0x16474a){r.sound(h);continue;}
            if(pc==0x158454){ResultAudioBoundary.execute(h,r,monitor);h.ret();continue;}
            if(pc==0x14de26){h.jump(0x14dede);continue;}
            if(pc==0x12946c){h.cpu.writeRegister("D0",1);h.ret();continue;}
            if(pc==0x133d46||pc==0x161b80||pc==0x13a2c4||pc==0x1205f8||pc==0x105572||pc==0x1382a8||pc==0x15f66c||pc==0x1600fc||pc==0x13a498||pc==0x12382a||pc==0x13154c||pc==0x123d1c||pc==0x160a8e||pc==0x160cca||pc==0x160baa||pc==0x160fb2||pc==0x11f1a8||pc==0x16043e||pc==0x10e6b6){h.ret();continue;}
            int op=h.word(pc)&65535;if(op==0xa8a7){h.pop(12);h.jump(pc+2);continue;}if(op==0xa8a8){h.pop(8);h.jump(pc+2);continue;}if(op==0xa873||op==0xa893||op==0xa884){h.pop(4);h.jump(pc+2);continue;}if(op==0xaa1e){h.value(sp+2,4,0);h.pop(2);h.jump(pc+2);continue;}if(op==0xa9eb){sane.execute();continue;}
            if(!h.cpu.step(monitor))throw new Exception("Crystalize initiative at "+Long.toHexString(pc)+": "+h.cpu.getLastError());
        }throw new Exception("Crystalize initiative instruction budget");
    }
    public void run()throws Exception {
        Path root=Path.of(getScriptArgs()[0]),original=root.resolve("NewCenturions/TheNewCenturions"),scenario=original.resolve("Scenarios/The Thunder of Guns/0");
        items=Files.readAllBytes(original.resolve("Data Files/Data ID"));scenarioItems=Files.readAllBytes(scenario.resolve("Data NI"));races=Files.readAllBytes(original.resolve("Data Files/Data Race"));monsters=Files.readAllBytes(scenario.resolve("Data MD"));properties=Files.readAllBytes(original.resolve("Data Files/Data Swamp BD"));attacks=Files.readAllBytes(original.resolve("Data Files/Data S"));areas=Files.readAllBytes(original.resolve("Data Files/Data AD"));party=new byte[18828];String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};for(int i=0;i<6;i++)System.arraycopy(Files.readAllBytes(original.resolve("Warriors/"+names[i])),0,party,i*3138,3138);
        JsonObject prior=JsonParser.parseString(Files.readString(root.resolve("evidence/fixtures/player-support-psi.json"))).getAsJsonObject();JsonObject identity=prior.getAsJsonObject("identity").deepCopy();identity.addProperty("support_probe_sha256",identity.get("probe_sha256").getAsString());identity.addProperty("probe_sha256",ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyCrystalizePsi.java")));identity.addProperty("support_fixture_sha256",ActionOracleHarness.sha256(root.resolve("evidence/fixtures/player-support-psi.json")));identity.addProperty("advance_probe_sha256",ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifySupportPsiAdvance.java")));
        JsonArray cases=new JsonArray();for(long seed:new long[]{12,1})for(int power:new int[]{1,2})probe(cases,seed,power);
        JsonObject out=new JsonObject();out.addProperty("authority","Four original learned2101CrystalizeMkI playercaller power1/2seeds12/1 with complete selection charge10d2b4, pointtarget11b164, fullcast10d4fe/spelltargetsincludingfieldcreation15cba6/resolvespell151022, original confirmationtail11af72..11afd0 and automaticAPcompare116b6a..116b86/getup125a3a..127090. OriginallearnedCoriand1 AP3/MOV5/MP100controls, sixhealthywarriors, originalMD13body10HP10000controls, target20,20/caster17,20 openflatSwampterrain. Explicitinitiativecaster1slot1/enemy10slot2 and initiallyzero60fields/128usage. No enemy action executes. Complete records/fields/usage/work/context/map/positions/spellmemory/party/order/scalars and ordered RNG/audio including full fieldcreation milestones captured. Presentation/SANEboundaries inherited, accepted selection explicit; no livegame or campaignvictory claim.");out.add("identity",identity);out.add("cases",cases);Files.writeString(root.resolve("evidence/fixtures/crystalize-psi.json"),out+"\n");println("Crystalize psi exported:4");
    }
    void probe(JsonArray output,long seed,int power)throws Exception {
        reset(seed);fieldEvents=new JsonArray();sites=new JsonArray();soundCalls=new JsonArray();ActionOracleHarness h=frame();RunResult r=new RunResult(seed);
        try{byte[] w=controlledParty(),m=monster(13,10000),map=terrain();ByteBuffer.wrap(w).putShort(3138+0x58,(short)100).putShort(3138+0xac6,(short)10).putShort(3138+0xac8,(short)0);ByteBuffer.wrap(m).putShort(0xe,(short)1);initialize(h,w,m,map,true,1);map=terrain();for(int i=0;i<6;i++){int x=i==1?17:10,y=i==1?20:10+i;h.value(0x44709a+i*4,2,x);h.value(0x44709c+i*4,2,y);ByteBuffer.wrap(map).putShort(((x+5)*90+y+5)*2,(short)i);}h.value(0x446f0a,2,20);h.value(0x446f0c,2,20);ByteBuffer.wrap(map).putShort((25*90+25)*2,(short)10);h.write(0x4085b8,map);
            byte[] order=new byte[220];for(int i=0;i<110;i++)ByteBuffer.wrap(order).putShort(i*2,(short)-1);ByteBuffer.wrap(order).putShort(2,(short)1).putShort(4,(short)10);h.write(0x4470b8,order);h.value(0x44255c,1,1);h.value(0x447194,2,1);h.value(0x447196,2,0);h.value(0x44374c,2,0);h.value(0x43800c,2,0);h.value(0x43958e,2,0);h.write(0x435664,new byte[]{12,14,12,14,10,12,10,10,14,16,0,12});h.value(0x438784,2,0);h.value(0x438766,2,0);h.value(0x45365c,2,0);h.value(0x435662,1,0);select(h,2101,power);
            JsonObject j=new JsonObject();j.addProperty("spell_id",2101);j.addProperty("seed_before",seed);j.addProperty("caster",1);j.addProperty("target",10);j.addProperty("power",power);j.add("before",advanceSnap(h,r));h.cpu.writeRegister("SP",0x6ffe00);h.jump(0x10d2b4);execute(h,r,0x10d3c2);j.add("charged",advanceSnap(h,r));
            h.value(0x44719e,2,20);h.value(0x4471a0,2,20);h.jump(0x11b164);execute(h,r,0x11b1be);j.add("targeted",advanceSnap(h,r));h.cpu.writeRegister("SP",0x6ffe00);h.jump(0x11af0a);execute(h,r,0x11af72);j.add("cast",advanceSnap(h,r));execute(h,r,0x11afd0);h.jump(0x116b6a);advance(h,r,0x127090);j.add("after",advanceSnap(h,r));j.add("casts",JsonParser.parseString("["+calls+"]"));j.add("resolutions",JsonParser.parseString("["+resolutions+"]"));j.add("field_events",fieldEvents);j.add("sound_calls",soundCalls);j.add("rng_sites",sites);j.add("rng",JsonParser.parseString("["+r.drawRecords+"]"));j.add("sounds",JsonParser.parseString("["+r.sounds+"]"));output.add(j);println("Crystalize seed"+seed+" power"+power);
        }finally{h.cpu.dispose();}
    }
}
