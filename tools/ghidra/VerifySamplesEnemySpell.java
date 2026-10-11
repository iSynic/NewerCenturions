import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import com.google.gson.*;

/** Original Samples enemy turn and race-restricted damage resolution. */
public class VerifySamplesEnemySpell extends VerifyPlayerSupportPsi {
    JsonArray nativeCalls;
    int recordLength;
    int monsterRaceControl=-1;
    byte[] bytes(JsonArray a){byte[] b=new byte[a.size()];for(int i=0;i<b.length;i++)b[i]=(byte)a.get(i).getAsInt();return b;}
    byte[] words(JsonArray a){ByteBuffer b=ByteBuffer.allocate(a.size()*2);for(JsonElement e:a)b.putShort((short)e.getAsInt());return b.array();}
    byte[] positions(JsonArray a){ByteBuffer b=ByteBuffer.allocate(a.size()*4);for(JsonElement e:a)for(JsonElement n:e.getAsJsonArray())b.putShort((short)n.getAsInt());return b.array();}
    byte[] joined(JsonArray a,int width){byte[] b=new byte[a.size()*width];for(int i=0;i<a.size();i++)System.arraycopy(bytes(a.get(i).getAsJsonObject().getAsJsonArray("bytes")),0,b,i*width,width);return b;}
    @Override void observe(ActionOracleHarness h,RunResult r,long pc)throws Exception {
        super.observe(h,r,pc);resistanceReturn(h,r,pc);
        if(pc==0x159192&&(h.read(0x443761,1)[0]&255)==9)throw new Exception("Unproved physical selection159192");
        if(pc==0x10d4fe||pc==0x151022||pc==0x15075a||pc==0x153ca4||pc==0x133a18||pc==0x130d38||pc==0x130e54||pc==0x150114){JsonObject e=new JsonObject();e.addProperty("address",Long.toHexString(pc));e.addProperty("return",Long.toHexString(h.pointer(h.sp())));e.addProperty("spell_id",(h.word(0x4435c6)+1)*1000+(h.word(0x4435c8)+1)*100+h.word(0x4435ca)+1);e.addProperty("draws",r.draws);JsonArray a=new JsonArray();int n=pc==0x15075a?5:pc==0x10d4fe?1:pc==0x151022?0:1;for(int i=0;i<n;i++)a.add(h.word(h.sp()+4+i*2));e.add("arguments",a);nativeCalls.add(e);}
    }
    void execute(ActionOracleHarness h,RunResult result,long stop) throws Exception {
        SaneBoundary sane=new SaneBoundary(h.cpu,currentProgram);boolean done=false;
        for(int step=0;step<1500000;step++) {
            long pc=h.pc(),sp=h.sp();
            if(pc==0x1363e4&&h.word(sp+4)==0){diagnostic=true;diagnosticSpell=(h.word(0x4435c6)+1)*1000+(h.word(0x4435c8)+1)*100+h.word(0x4435ca)+1;done=true;break;}
            observe(h,result,pc);if(pc==stop){done=true;break;}
            if(pc==0x4001a0){sane.extended(h.pointer(sp+4),Math.sqrt(sane.extended(sp+8)));h.ret();continue;}
            if(pc==0x400038){h.cpu.writeRegister("D0",Math.abs(h.word(sp+4)));h.ret();continue;}
            if(pc==0x153ca4){result.rng(h);continue;}
            if(pc==0x133a18||pc==0x16474a){result.sound(h);continue;}
            if(pc==0x123d1c){int sound=h.word(sp+14);if(sound!=0){if(result.sounds.length()>0)result.sounds.append(',');result.sounds.append(sound);}h.ret();continue;}
            if(pc==0x15b2f6||pc==0x15a1f6){h.ret();continue;}
            if(pc==0x159192){displaySpell(h,result);continue;}
            if(pc==0x158454){showresultsSound(h,result);h.ret();continue;}
            if(pc==0x1600fc||pc==0x120c14||pc==0x105572||pc==0x10e6b6||pc==0x133d46||pc==0x13154c||pc==0x11f1a8||pc==0x11edc4||pc==0x14a02c||pc==0x13a2c4||pc==0x1205f8||pc==0x16043e||pc==0x1382a8||pc==0x11f136||pc==0x1537ee||pc==0x12382a||pc==0x160fb2||pc==0x10ff54||pc==0x12f040||pc==0x15edfa){h.ret();continue;}
            int op=h.word(pc)&65535;
            if(op==0xa85d){int bit=(int)h.pointer(sp);long address=h.pointer(sp+4);int value=h.read(address+(bit>>3),1)[0]&255;h.value(sp+8,1,(value&(0x80>>>(bit&7)))!=0?1:0);h.pop(8);h.jump(pc+2);continue;}
            if(op==0xa9eb){
                // Apple Numerics Manual FSQRTX uses one extended destination address.
                if((h.word(sp)&65535)==0x12){long destination=h.pointer(sp+2);sane.extended(destination,Math.sqrt(sane.extended(destination)));h.pop(6);h.jump(pc+2);}else sane.execute();
                continue;
            }
            if(op==0xa873||op==0xaa1c||op==0xa862||op==0xa863||op==0xa89b||op==0xa893||op==0xa891||op==0xa884){h.pop(4);h.jump(pc+2);continue;}
            if(op==0xa89c){h.pop(2);h.jump(pc+2);continue;}
            if(op==0xa8b7){h.pop(4);h.jump(pc+2);continue;}
            if(op==0xa8ec){h.pop(22);h.jump(pc+2);continue;}
            if(op==0xa8a9){int vertical=h.word(sp),horizontal=h.word(sp+2);long rect=h.pointer(sp+4);h.value(rect,2,h.word(rect)+vertical);h.value(rect+2,2,h.word(rect+2)+horizontal);h.value(rect+4,2,h.word(rect+4)-vertical);h.value(rect+6,2,h.word(rect+6)-horizontal);h.pop(8);h.jump(pc+2);continue;}
            if(op==0xa8aa){long output=h.pointer(sp),b=h.pointer(sp+4),a=h.pointer(sp+8);int top=Math.max(h.word(a),h.word(b)),left=Math.max(h.word(a+2),h.word(b+2)),bottom=Math.min(h.word(a+4),h.word(b+4)),right=Math.min(h.word(a+6),h.word(b+6));boolean empty=top>=bottom||left>=right;h.value(output,2,empty?0:top);h.value(output+2,2,empty?0:left);h.value(output+4,2,empty?0:bottom);h.value(output+6,2,empty?0:right);h.value(sp+12,1,empty?0:1);h.pop(12);h.jump(pc+2);continue;}
            if(op==0xa8a7){long rectangle=h.pointer(sp+8);h.write(rectangle,h.read(sp+4,4));h.write(rectangle+4,h.read(sp,4));h.pop(12);h.jump(pc+2);continue;}
            if(op==0xaa1e){h.value(sp+2,4,0);h.pop(2);h.jump(pc+2);continue;}
            if(op==0xa8ad){long rect=h.pointer(sp);int x=h.word(sp+4),y=h.word(sp+6);h.value(sp+8,1,x>=h.word(rect)&&x<h.word(rect+4)&&y>=h.word(rect+2)&&y<h.word(rect+6)?1:0);h.pop(8);h.jump(pc+2);continue;}
            if(op==0xa9b4){h.jump(pc+2);continue;}
            if(op==0xa974){h.cpu.writeRegister("SP",sp-2);h.value(sp-2,2,0);h.jump(pc+2);continue;}
            if(!h.cpu.step(monitor))throw new Exception("Ally action at "+Long.toHexString(pc)+": "+h.cpu.getLastError());
        }
        if(!done)throw new Exception("Ally action instruction budget");
    }

    void displaySpell(ActionOracleHarness h,RunResult r)throws Exception {
        long returned=h.pointer(h.sp());for(int step=0;step<500;step++){long pc=h.pc(),sp=h.sp();if(pc==returned)return;if(pc==0x15939a||pc==0x1593a0){JsonObject e=new JsonObject();e.addProperty("address",Long.toHexString(pc));e.addProperty("selected_weapon",h.word(0x44719a));e.addProperty("draws",r.draws);nativeCalls.add(e);}
            if(pc==0x400178||pc==0x1382d0){h.ret();continue;}
            int op=h.word(pc)&65535;if(op==0xa88a||op==0xa889){h.pop(2);h.jump(pc+2);continue;}
            if(op==0xa873||op==0xa863||op==0xa862||op==0xa893||op==0xa884||op==0xaa25||op==0xa8a3){h.pop(4);h.jump(pc+2);continue;}
            if(op==0xaa1e){h.value(sp+2,4,0);h.pop(2);h.jump(pc+2);continue;}
            if(op==0xa8a8||op==0xaa1f){h.pop(8);h.jump(pc+2);continue;}
            if(!h.cpu.step(monitor))throw new Exception("Spell display "+Long.toHexString(pc)+": "+h.cpu.getLastError());
        }throw new Exception("Spell display instruction budget");
    }
    void resistanceReturn(ActionOracleHarness h,RunResult r,long pc){if(pc==0x151870){JsonObject e=new JsonObject();e.addProperty("address","151870");e.addProperty("body",(short)h.cpu.readRegister("D3").intValue());e.addProperty("result",(short)h.cpu.readRegister("D0").intValue());e.addProperty("draws",r.draws);nativeCalls.add(e);}}
    JsonObject completeSnap(ActionOracleHarness h,RunResult r)throws Exception {
        JsonObject j=snap(h,r);j.addProperty("records",hex(h.read(0x439d3c,recordLength)));j.addProperty("monster_positions",hex(h.read(0x446f0a,recordLength/72)));j.addProperty("order",hex(h.read(0x4470b8,220)));j.addProperty("weapon_work",hex(h.read(0x453768,150)));j.addProperty("ammo_work",hex(h.read(0x4536d2,150)));j.addProperty("item_work",hex(h.read(0x45363c,150)));j.addProperty("item_context",h.word(0x45365c));j.addProperty("cursor",h.word(0x447194));j.addProperty("monster_index",h.word(0x447196));j.addProperty("active_member",h.word(0x4435d2));j.addProperty("round",h.read(0x44255c,1)[0]&255);j.addProperty("death_count",h.word(0x44374c));j.addProperty("enemy_phase",h.word(0x438778));j.addProperty("nearest",h.word(0x4436b4));j.addProperty("range",h.word(0x4435bc));j.addProperty("target_count",h.word(0x4435c0));j.addProperty("health_context",h.read(0x435662,1)[0]&255);j.addProperty("spell_semaphore",h.word(0x438760));j.addProperty("ammunition",hex(h.read(0x42dbe4,800)));j.addProperty("death_queue",hex(h.read(0x437a94,1400)));j.addProperty("death_queue_count",h.word(0x43800c));j.addProperty("selected_weapon",h.word(0x44719a));j.addProperty("monster_race_word",h.word(0x448aa0+10*3138+0x32));return j;
    }
    void initializeActual(ActionOracleHarness h,JsonObject s)throws Exception {
        JsonObject c=s.getAsJsonObject("combat");party=joined(s.getAsJsonArray("warriors"),3138);byte[] records=joined(c.getAsJsonObject("enemies").getAsJsonArray("records"),288);recordLength=records.length;state(h,records,words(c.getAsJsonObject("map").getAsJsonArray("tiles")));bodyCount=records.length/288;
        h.write(0x448aa0,party);h.write(0x44709a,positions(c.getAsJsonObject("squad").getAsJsonArray("positions")));h.write(0x437a6e,words(c.getAsJsonObject("squad").getAsJsonArray("under_tiles")));h.write(0x446f0a,positions(c.getAsJsonObject("enemies").getAsJsonArray("positions")));h.write(0x4470b8,words(c.getAsJsonArray("order")));h.value(0x447194,2,c.get("cursor").getAsInt());h.value(0x447196,2,11);h.value(0x43874a,2,bodyCount);h.value(0x4435c4,2,bodyCount);h.value(0x438778,2,1);h.value(0x44255c,1,c.get("round").getAsInt());h.value(0x44273a,1,5);h.value(0x44273f,1,s.get("selected_warrior").getAsInt());h.value(0x4435d2,2,s.get("selected_warrior").getAsInt());
        JsonArray shift=c.getAsJsonObject("map").getAsJsonArray("shift");h.value(0x442780,4,shift.get(0).getAsInt()+5);h.value(0x442784,4,shift.get(1).getAsInt()+5);h.write(0x442bd6,bytes(c.getAsJsonArray("spell_fields")));h.write(0x435884,bytes(c.getAsJsonArray("field_used")));h.write(0x44374e,bytes(c.getAsJsonArray("attack_packet")));h.write(0x4435c6,words(c.getAsJsonArray("attack_identity")));h.value(0x4435cc,2,s.get("attack_count").getAsInt());h.value(0x45365c,2,s.get("loaded_item_field32").getAsInt());
        h.write(0x453768,new byte[150]);h.write(0x4536d2,new byte[150]);h.write(0x45363c,new byte[150]);JsonObject p=c.getAsJsonObject("weapon_work");h.value(0x453776,2,p.get("sound").getAsInt());h.value(0x4537f0,2,p.get("weapon_armor_filter").getAsInt());h.value(0x45375a,2,p.get("ammunition_armor_filter").getAsInt());h.write(0x43788a,bytes(s.getAsJsonArray("psi_memory")));h.write(0x442562,bytes(s.getAsJsonArray("party_buffs")));ByteBuffer ammo=ByteBuffer.allocate(800);for(JsonElement e:s.getAsJsonArray("ammunition"))ammo.putInt(e.getAsInt());h.write(0x42dbe4,ammo.array());h.write(0x442580,new byte[24]);h.write(0x437a7a,new byte[24]);h.write(0x442798,new byte[24]);h.write(0x4434e0,new byte[220]);h.write(0x443722,new byte[42]);h.value(0x442739,1,c.get("unavailable").getAsInt());h.value(0x435662,1,0);h.value(0x438760,2,0);h.value(0x438784,2,0);h.value(0x438766,2,0);h.value(0x443580,1,0);h.value(0x4358c2,1,0);h.value(0x438780,2,s.get("enemy_spell_disabled").getAsInt());h.write(0x437a94,new byte[1400]);h.value(0x43800c,2,0);h.value(0x437a90,4,0);h.value(0x438582,2,0);h.value(0x438584,1,0);h.value(0x438585,1,0);h.value(0x447198,2,0);h.value(0x44719a,2,0);h.value(0x4436b4,2,127);h.write(0x43878c,new byte[4]);h.write(0x4436fa,new byte[16]);h.write(0x4434ba,new byte[8]);h.write(0x40856c,new byte[4]);h.value(0x439582,2,0);h.write(0x479ed8,new byte[256]);h.cpu.writeRegister("A6",0x700000);h.cpu.writeRegister("SP",0x6ffe00);
    }
    void probe(JsonArray out,JsonObject s,long seed,boolean turn)throws Exception {
        reset(seed);nativeCalls=new JsonArray();ActionOracleHarness h=frame();RunResult r=new RunResult(seed);try{initializeActual(h,s);r.draws=s.getAsJsonObject("random").get("draws").getAsInt();JsonObject j=new JsonObject();j.addProperty("complete_turn",turn);j.addProperty("seed_before",seed);j.addProperty("monster_race_control",monsterRaceControl);if(!turn){select(h,2207,1);if(monsterRaceControl>=0){h.value(0x4434e0+20,2,1);h.value(0x448aa0+10*3138+0x32,2,monsterRaceControl);}else{for(int i=0;i<6;i++)h.value(0x4434e0+i*2,2,1);}h.value(0x438760,2,1);}if(turn)h.value(0x44719a,2,7);j.add("before",completeSnap(h,r));if(turn){if(h.sp()>h.cpu.readRegister("A6").longValue()-0x3c)throw new Exception("Invalid combat caller stack frame");h.jump(0x1145c8);execute(h,r,0x115776);}else{h.enter(0x151022,0x6ffe00,0x900000);execute(h,r,0x900000);}if(diagnostic)throw new Exception("Unproved diagnostic "+diagnosticSpell);j.add("after",completeSnap(h,r));j.add("casts",JsonParser.parseString("["+calls+"]"));j.add("resolutions",JsonParser.parseString("["+resolutions+"]"));j.add("native_calls",nativeCalls);j.add("rng",JsonParser.parseString("["+r.drawRecords+"]"));j.add("sounds",JsonParser.parseString("["+r.sounds+"]"));out.add(j);println("Samples enemy spell turn="+turn+" seed="+seed);}finally{h.cpu.dispose();}
    }
    public void run()throws Exception {
        Path root=Path.of(getScriptArgs()[0]),original=root.resolve("NewCenturions/TheNewCenturions"),scenario=original.resolve("Scenarios/The Thunder of Guns/0");items=Files.readAllBytes(original.resolve("Data Files/Data ID"));scenarioItems=Files.readAllBytes(scenario.resolve("Data NI"));races=Files.readAllBytes(original.resolve("Data Files/Data Race"));monsters=Files.readAllBytes(scenario.resolve("Data MD"));properties=Files.readAllBytes(original.resolve("Data Files/Data Swamp BD"));attacks=Files.readAllBytes(original.resolve("Data Files/Data S"));areas=Files.readAllBytes(original.resolve("Data Files/Data AD"));
        String input="analysis/completion/samples-failure-1.json";JsonObject s=JsonParser.parseString(Files.readString(root.resolve(input))).getAsJsonObject().getAsJsonObject("session"),identity=JsonParser.parseString(Files.readString(root.resolve("evidence/fixtures/player-support-psi.json"))).getAsJsonObject().getAsJsonObject("identity").deepCopy();identity.addProperty("support_probe_sha256",identity.get("probe_sha256").getAsString());identity.addProperty("probe_sha256",ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifySamplesEnemySpell.java")));identity.addProperty("support_fixture_sha256",ActionOracleHarness.sha256(root.resolve("evidence/fixtures/player-support-psi.json")));JsonObject foundation=new JsonObject();foundation.addProperty("input",input);foundation.addProperty("digest",ActionOracleHarness.sha256(root.resolve(input)));foundation.add("session",s);JsonArray cases=new JsonArray();probe(cases,s,s.getAsJsonObject("random").get("seed").getAsLong(),true);probe(cases,s,12,false);probe(cases,s,1,false);monsterRaceControl=0;probe(cases,s,12,false);monsterRaceControl=3;probe(cases,s,12,false);
        JsonObject out=new JsonObject();out.addProperty("authority","Actual Samples frozen host beforefailedcommand573 projected to original 14-body combat state. Complete original body21 actor decision1145c8..115776 plus two complete original2207 resolvespell151022 cases with explicit six-party mask/count1 and independent seeds12/1. Full rule buffers/packets/casts/resolutions/nativecalls/orderedRNG/audio captured. Completecaller initiallycontrolsunsavedselectedweapon7andexecutesoriginalclear44719a beforefirstresolve; thisisnotahistoricalselectedweaponclaim. Only2207positiveW50party exclusion recovered. Two additionalNPC10mask cases explicitly control the native out-of-party raceword0/3 to prove unboundedpartyaddress read, notmonster-race semantics; Complete nonphysical showspellinfo159192 executes its native rule writes and return with presentation services acknowledged; physical159192 selection branch hardfails. Unserialized work bytes, ephemeral target/mask/selection and presentation/health feedback globals explicitly neutral; three retained work words projected. Native map origin uses hostshift+5, not world shift alone. Inherited documented presentation/resultaudio/SANE boundaries only. No getup suffix, other actor, catalog expansion or liveoriginalcampaign claim.");out.add("identity",identity);out.add("foundation",foundation);out.add("cases",cases);Files.writeString(root.resolve("evidence/fixtures/samples-enemy-spell.json"),out+"\n");println("Samples enemy spell cases exported:5");
    }
}
