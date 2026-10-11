import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import com.google.gson.*;

/** Original earned firearm targeting with complete range and sight admission. */
public class VerifyOwnedPlayerFirearmTarget extends VerifyOwnedPlayerFirearm {
    JsonArray admissionEvents=new JsonArray();
    @Override void observe(ActionOracleHarness h,RunResult r,long pc)throws Exception {
        super.observe(h,r,pc);
        if(pc==0x11235e||pc==0x10ae90||pc==0x11b04c||pc==0x11b08c||pc==0x11b0da||pc==0x11b0f0||pc==0x11b3e2){
            JsonObject e=new JsonObject();e.addProperty("address",Long.toHexString(pc));e.addProperty("d0",h.cpu.readRegister("D0").longValue());e.addProperty("range",h.word(0x4435bc));e.addProperty("distance",h.word(0x4434b6));e.addProperty("actor",h.word(0x4435d2));e.addProperty("target_x",h.word(0x44719e));e.addProperty("target_y",h.word(0x4471a0));e.addProperty("draws",r.draws);e.addProperty("body_local",h.word(h.cpu.readRegister("A6").longValue()-0x16));
            if(pc==0x11235e||pc==0x10ae90){JsonArray a=new JsonArray();for(int i=0;i<4;i++)a.add(h.word(h.sp()+4+i*2));e.add("arguments",a);e.addProperty("return",Long.toHexString(h.pointer(h.sp())));}
            admissionEvents.add(e);
        }
    }
    JsonObject targetSnap(ActionOracleHarness h,RunResult r){JsonObject j=ownedSnap(h,r);j.addProperty("map",hex(h.read(0x4085b8,16200)));j.addProperty("range",h.word(0x4435bc));j.addProperty("distance",h.word(0x4434b6));j.addProperty("target_x",h.word(0x44719e));j.addProperty("target_y",h.word(0x4471a0));j.addProperty("shift_x",h.pointer(0x442780));j.addProperty("shift_y",h.pointer(0x442784));j.add("admission_events",admissionEvents.deepCopy());return j;}
    void targetCase(JsonArray out,long seed,int member)throws Exception {
        ActionOracleHarness h=frame();RunResult r=new RunResult(seed);try{
            initialize(h,r,2);admissionEvents=new JsonArray();JsonObject j=new JsonObject();j.addProperty("seed_before",seed);j.addProperty("member",member);j.add("before",targetSnap(h,r));JsonArray stages=new JsonArray();takeEquip(h,r,2,stages);takeEquip(h,r,3,stages);j.add("equipment",stages);
            selectMember(h,member,firearmSlot(h,member));combatControls(h,member,3,5);
            byte[] map=terrain(),target=h.read(0x439d3c,288);place(h,map,target,10,20,15);h.write(0x439d3c,target);h.value(0x446f0a,2,20);h.value(0x446f0c,2,15);
            for(int i=0;i<6;i++){int x=20+i-member;h.value(0x44709a+i*4,2,x);h.value(0x44709c+i*4,2,10);ByteBuffer.wrap(map).putShort(((x+5)*90+15)*2,(short)i);}
            h.write(0x4085b8,map);
            h.value(0x44719e,2,20);h.value(0x4471a0,2,15);h.value(0x4435cc,2,1);h.value(0x4435c0,2,0);h.value(0x4435be,2,1);h.value(0x43565e,1,1);j.add("combat_before",targetSnap(h,r));j.addProperty("prepare_return",call(h,r,0x14c6c2,member,0));j.add("prepared",targetSnap(h,r));
            h.cpu.writeRegister("A6",0x700000);h.cpu.writeRegister("SP",0x6ffe00);h.jump(0x119238);execute(h,r,0x11925e);j.addProperty("prepared_range",h.word(0x4435bc));h.cpu.writeRegister("SP",0x6ffe00);h.jump(0x11b032);secondaryStop=0x11b5ac;execute(h,r,0x11b0f0);secondaryStop=0;
            if(h.pc()!=0x11b0f0)throw new Exception("Native target rejected range");
            j.add("admitted",targetSnap(h,r));execute(h,r,0x11b488);j.add("targeted",targetSnap(h,r));h.jump(0x11af0a);execute(h,r,0x11af72);j.add("fired",targetSnap(h,r));j.add("casts",JsonParser.parseString("["+calls+"]"));j.add("resolutions",JsonParser.parseString("["+resolutions+"]"));h.jump(0x116b6a);h.cpu.writeRegister("A6",0x700000);h.cpu.writeRegister("SP",0x6ffe00);advance(h,r,0x127090);j.add("advanced",targetSnap(h,r));j.add("after",targetSnap(h,r));out.add(j);println("Owned valid target seed"+seed+" member"+member);
        }finally{h.cpu.dispose();}
    }
    public void run()throws Exception {
        Path root=Path.of(getScriptArgs()[0]),original=root.resolve("NewCenturions/TheNewCenturions"),scenario=original.resolve("Scenarios/The Thunder of Guns/0");items=Files.readAllBytes(original.resolve("Data Files/Data ID"));scenarioItems=Files.readAllBytes(scenario.resolve("Data NI"));races=Files.readAllBytes(original.resolve("Data Files/Data Race"));mos=Files.readAllBytes(original.resolve("Data Files/Data MOS"));monsters=Files.readAllBytes(scenario.resolve("Data MD"));properties=Files.readAllBytes(original.resolve("Data Files/Data Swamp BD"));attacks=Files.readAllBytes(original.resolve("Data Files/Data S"));areas=Files.readAllBytes(original.resolve("Data Files/Data AD"));
        String input="analysis/completion/archive/laboratory-resources-fixed-route/laboratory-12.json";foundation=JsonParser.parseString(Files.readString(root.resolve(input))).getAsJsonObject().getAsJsonObject("session");party=warriors(foundation);JsonObject f=new JsonObject();f.addProperty("input",input);f.addProperty("digest",ActionOracleHarness.sha256(root.resolve(input)));f.add("session",foundation);JsonObject identity=JsonParser.parseString(Files.readString(root.resolve("evidence/fixtures/owned-player-firearm.json"))).getAsJsonObject().getAsJsonObject("identity").deepCopy();identity.addProperty("owned_probe_sha256",identity.get("probe_sha256").getAsString());identity.addProperty("probe_sha256",ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyOwnedPlayerFirearmTarget.java")));identity.addProperty("owned_fixture_sha256",ActionOracleHarness.sha256(root.resolve("evidence/fixtures/owned-player-firearm.json")));
        JsonArray cases=new JsonArray();for(long seed:new long[]{12,1})for(int member:new int[]{2,3})targetCase(cases,seed,member);JsonObject out=new JsonObject();out.addProperty("authority","Original earned two-carrier ordinary2/201 composition, complete native range initialization and target checkrange/cansee/map lookup before accepted target/ammunition debit and full confirmation/cast plus getup. Controlled actor20,10 target20,15 original MD13HP10000/faction1/flat terrain/AP3MOV5/order; complete earned party/crate/ammunition and serialized work projection restored; presentation and SANE boundaries inherited. No enemy action, live game or campaign battle claim.");out.add("identity",identity);out.add("foundation",f);out.add("fire_cases",cases);Files.writeString(root.resolve("evidence/fixtures/owned-player-firearm-target.json"),out+"\n");println("Owned firearm valid targets exported");
    }
}
