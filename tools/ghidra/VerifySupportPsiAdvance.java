import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import com.google.gson.*;

/** Original learned combat support casts followed by initiative selection. */
public class VerifySupportPsiAdvance extends VerifyPlayerSupportPsi {
    JsonObject advanceSnap(ActionOracleHarness h,RunResult r)throws Exception {
        JsonObject j=snap(h,r);j.addProperty("order",hex(h.read(0x4470b8,220)));j.addProperty("cursor",h.word(0x447194));j.addProperty("monster_index",h.word(0x447196));j.addProperty("active_member",h.word(0x4435d2));j.addProperty("round",h.read(0x44255c,1)[0]&255);j.addProperty("death_count",h.word(0x44374c));j.addProperty("enemy_phase",h.word(0x438778));j.addProperty("item_context",h.word(0x45365c));j.addProperty("weapon_work",hex(h.read(0x453768,150)));j.addProperty("ammo_work",hex(h.read(0x4536d2,150)));return j;
    }
    void advance(ActionOracleHarness h,RunResult r,long stop)throws Exception {
        SaneBoundary sane=new SaneBoundary(h.cpu,currentProgram);
        for(int step=0;step<1500000;step++){
            long pc=h.pc(),sp=h.sp();if(pc==stop)return;
            if(pc==0x400038){h.cpu.writeRegister("D0",(short)Math.abs(h.word(sp+4)));h.ret();continue;}
            if(pc==0x153ca4){r.rng(h);continue;}
            if(pc==0x133a18||pc==0x16474a){r.sound(h);continue;}
            if(pc==0x158454){ResultAudioBoundary.execute(h,r,monitor);h.ret();continue;}
            if(pc==0x14de26){h.jump(0x14dede);continue;}
            if(pc==0x12946c){h.cpu.writeRegister("D0",1);h.ret();continue;}
            if(pc==0x133d46||pc==0x161b80||pc==0x13a2c4||pc==0x1205f8||pc==0x105572||pc==0x1382a8||pc==0x15f66c||pc==0x1600fc||pc==0x13a498||pc==0x12382a||pc==0x13154c||pc==0x123d1c||pc==0x160a8e||pc==0x160cca||pc==0x160baa||pc==0x160fb2||pc==0x11f1a8||pc==0x16043e||pc==0x10e6b6){h.ret();continue;}
            int op=h.word(pc)&65535;if(op==0xa8a7){h.pop(12);h.jump(pc+2);continue;}if(op==0xa8a8){h.pop(8);h.jump(pc+2);continue;}if(op==0xa873||op==0xa893||op==0xa884){h.pop(4);h.jump(pc+2);continue;}if(op==0xaa1e){h.value(sp+2,4,0);h.pop(2);h.jump(pc+2);continue;}if(op==0xa9eb){sane.execute();continue;}
            if(!h.cpu.step(monitor))throw new Exception("Support initiative at "+Long.toHexString(pc)+": "+h.cpu.getLastError());
        }throw new Exception("Support initiative instruction budget");
    }
    public void run()throws Exception {
        Path root=Path.of(getScriptArgs()[0]),original=root.resolve("NewCenturions/TheNewCenturions"),scenario=original.resolve("Scenarios/The Thunder of Guns/0");
        items=Files.readAllBytes(original.resolve("Data Files/Data ID"));scenarioItems=Files.readAllBytes(scenario.resolve("Data NI"));races=Files.readAllBytes(original.resolve("Data Files/Data Race"));monsters=Files.readAllBytes(scenario.resolve("Data MD"));properties=Files.readAllBytes(original.resolve("Data Files/Data Swamp BD"));attacks=Files.readAllBytes(original.resolve("Data Files/Data S"));areas=Files.readAllBytes(original.resolve("Data Files/Data AD"));party=new byte[18828];String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};for(int i=0;i<6;i++)System.arraycopy(Files.readAllBytes(original.resolve("Warriors/"+names[i])),0,party,i*3138,3138);
        JsonObject prior=JsonParser.parseString(Files.readString(root.resolve("evidence/fixtures/player-support-psi.json"))).getAsJsonObject();JsonObject identity=prior.getAsJsonObject("identity").deepCopy();identity.addProperty("support_probe_sha256",identity.get("probe_sha256").getAsString());identity.addProperty("probe_sha256",ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifySupportPsiAdvance.java")));identity.addProperty("support_fixture_sha256",ActionOracleHarness.sha256(root.resolve("evidence/fixtures/player-support-psi.json")));
        JsonArray cases=new JsonArray();for(int id:new int[]{1101,1305})for(long seed:new long[]{12,1})probe(cases,id,seed);
        JsonObject out=new JsonObject();out.addProperty("authority","Four original learned combat support casts with complete selection charge10d2b4/body-or-point targeting/fullcast10d4fe/resolvespell151022 followed by original confirmation tail11af72..11afd0 and automatic APcompare116b6a..116b86/getup125a3a..127090 selecting original MD13 enemy10. Explicit initiative caster5 atslot1/enemy10 atslot2, sixwarriors/oneoriginalenemy/terrain and nofields; normalAP3 becomes1 before automatic advancement. No enemy action executes. Complete records, map, fields, spellmemory, partybuffers, order/scalars and ordered RNG/audio captured. Presentation and SANE boundaries inherited; no live-game or campaignvictory claim.");out.add("identity",identity);out.add("cases",cases);Files.writeString(root.resolve("evidence/fixtures/support-psi-advance.json"),out+"\n");println("Support psi initiative exported:4");
    }
    void probe(JsonArray output,int id,long seed)throws Exception {
        reset(seed);ActionOracleHarness h=frame();RunResult r=new RunResult(seed);
        try{byte[] w=controlledParty(),m=monster(13,10000),map=terrain();if(id==1101)ByteBuffer.wrap(w).putShort(0x50,(short)10);initialize(h,w,m,map,true,5);
            byte[] order=new byte[220];for(int i=0;i<110;i++)ByteBuffer.wrap(order).putShort(i*2,(short)-1);ByteBuffer.wrap(order).putShort(2,(short)5).putShort(4,(short)10);h.write(0x4470b8,order);h.value(0x44255c,1,1);h.value(0x447194,2,1);h.value(0x447196,2,0);h.value(0x44374c,2,0);h.value(0x43800c,2,0);h.value(0x43958e,2,0);h.write(0x435664,new byte[]{12,14,12,14,10,12,10,10,14,16,0,12});h.value(0x438784,2,0);h.value(0x438766,2,0);h.value(0x45365c,2,0);h.value(0x435662,1,0);select(h,id,1);
            if(id==1101){h.value(0x44709a+20,2,19);h.value(0x44709c+20,2,20);ByteBuffer.wrap(map).putShort((25*90+30)*2,(short)1155).putShort((24*90+25)*2,(short)5);h.write(0x4085b8,map);}
            JsonObject j=new JsonObject();j.addProperty("spell_id",id);j.addProperty("seed_before",seed);j.addProperty("caster",5);j.addProperty("target",0);j.addProperty("power",1);j.add("before",advanceSnap(h,r));h.cpu.writeRegister("SP",0x6ffe00);h.jump(0x10d2b4);execute(h,r,0x10d3c2);j.add("charged",advanceSnap(h,r));
            if(id==1101){h.value(0x6ffd00-0x16,2,0);h.jump(0x11b30a);execute(h,r,0x11b332);h.jump(0x11b33c);execute(h,r,0x11b342);}else{h.value(0x44719e,2,20);h.value(0x4471a0,2,20);h.jump(0x11b164);execute(h,r,0x11b1be);}j.add("targeted",advanceSnap(h,r));h.cpu.writeRegister("SP",0x6ffe00);h.jump(0x11af0a);execute(h,r,0x11af72);j.add("cast",advanceSnap(h,r));
            execute(h,r,0x11afd0);h.jump(0x116b6a);advance(h,r,0x127090);j.add("after",advanceSnap(h,r));j.add("casts",JsonParser.parseString("["+calls+"]"));j.add("resolutions",JsonParser.parseString("["+resolutions+"]"));j.add("healing_calls",healingCalls);j.add("rng",JsonParser.parseString("["+r.drawRecords+"]"));j.add("sounds",JsonParser.parseString("["+r.sounds+"]"));output.add(j);
        }finally{h.cpu.dispose();}
    }
}
