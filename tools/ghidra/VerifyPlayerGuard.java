import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import com.google.gson.*;

/** Original accepted player Guard command and initiative advancement. */
public class VerifyPlayerGuard extends VerifySupportPsiAdvance {
    JsonArray ordered;
    JsonObject guardSnap(ActionOracleHarness h,RunResult r)throws Exception {
        JsonObject j=advanceSnap(h,r);j.addProperty("command",h.word(0x443316));j.addProperty("guard_word",h.word(0x448aa0+h.word(0x4435d2)*3138+0x16));j.addProperty("buttons",hex(h.read(0x44292c,8)));j.add("ordered_calls",ordered.deepCopy());return j;
    }
    void command(ActionOracleHarness h,RunResult r)throws Exception {
        h.jump(0x115b72);
        for(int step=0;step<20000;step++){
            long pc=h.pc(),sp=h.sp();
            if(pc==0x133a18||pc==0x153720||pc==0x125a3a){JsonObject e=new JsonObject();e.addProperty("address",Long.toHexString(pc));e.addProperty("argument",h.word(sp+4));e.addProperty("return",Long.toHexString(h.pointer(sp)));ordered.add(e);if(pc==0x125a3a)return;if(pc==0x133a18)r.sound(h);else h.ret();continue;}
            if(!h.cpu.step(monitor))throw new Exception("Guard command PC "+Long.toHexString(pc)+": "+h.cpu.getLastError());
        }throw new Exception("Guard command instruction budget");
    }
    public void run()throws Exception {
        Path root=Path.of(getScriptArgs()[0]),original=root.resolve("NewCenturions/TheNewCenturions"),scenario=original.resolve("Scenarios/The Thunder of Guns/0");
        items=Files.readAllBytes(original.resolve("Data Files/Data ID"));scenarioItems=Files.readAllBytes(scenario.resolve("Data NI"));races=Files.readAllBytes(original.resolve("Data Files/Data Race"));monsters=Files.readAllBytes(scenario.resolve("Data MD"));properties=Files.readAllBytes(original.resolve("Data Files/Data Swamp BD"));attacks=Files.readAllBytes(original.resolve("Data Files/Data S"));areas=Files.readAllBytes(original.resolve("Data Files/Data AD"));party=new byte[18828];String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};for(int i=0;i<6;i++)System.arraycopy(Files.readAllBytes(original.resolve("Warriors/"+names[i])),0,party,i*3138,3138);
        JsonObject prior=JsonParser.parseString(Files.readString(root.resolve("evidence/fixtures/support-psi-advance.json"))).getAsJsonObject();JsonObject identity=prior.getAsJsonObject("identity").deepCopy();identity.addProperty("advance_probe_sha256",identity.get("probe_sha256").getAsString());identity.addProperty("probe_sha256",ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyPlayerGuard.java")));identity.addProperty("advance_fixture_sha256",ActionOracleHarness.sha256(root.resolve("evidence/fixtures/support-psi-advance.json")));
        JsonArray cases=new JsonArray();for(int member:new int[]{0,5})for(int ap:new int[]{2,0})probe(cases,member,ap);
        JsonObject out=new JsonObject();out.addProperty("authority","Four original accepted Guard key0x567 commanddispatch115b72 through sound10121/downbutton1/warriorW16=1 and actualBSRgetup0 returning1161de; completeoriginalgetup125a3a..127090 selectingoriginalMD13enemy10. Controlledactive0/5 AP2/0 MOV5, originalsixhealthywarriorcontrols inheritedSupportAdvance, flatSwamp/noactivefields/orderactiveat1 enemy10at2, packet1305/count1 and used0sentinel. GuardbranchhasenemyphaserejectionbutnoAPorMOVtest; AP0casesexplicitacceptedcommandcutcontrolsdoNOTclaimordinaryAP0playerinputreachability becausecombatautoadvanceAP<2isoutsideentry. NoFinish/syntheticgetupcall/enemyaction/livecampaignclaim. Fullrecords/map/fields/memory/buffs/packet/identity/order/work/itemcontext and orderedcommandcalls/RNG/audio captured. Downbuttonpresentation acknowledged; otherSANE/presentationboundaries inherited.");out.add("identity",identity);out.add("cases",cases);Files.writeString(root.resolve("evidence/fixtures/player-guard.json"),out+"\n");println("Player Guard exported:4");
    }
    void probe(JsonArray output,int member,int ap)throws Exception {
        long seed=12;reset(seed);ordered=new JsonArray();ActionOracleHarness h=frame();RunResult r=new RunResult(seed);
        try{byte[] w=controlledParty();for(int i=0;i<6;i++)ByteBuffer.wrap(w).putShort(i*3138+0x16,(short)0);ByteBuffer.wrap(w).putShort(member*3138+0x40,(short)ap).putShort(member*3138+0x3c,(short)5);initialize(h,w,monster(13,10000),terrain(),true,member);
            byte[] order=new byte[220];for(int i=0;i<110;i++)ByteBuffer.wrap(order).putShort(i*2,(short)-1);ByteBuffer.wrap(order).putShort(2,(short)member).putShort(4,(short)10);h.write(0x4470b8,order);h.value(0x44255c,1,1);h.value(0x447194,2,1);h.value(0x447196,2,0);h.value(0x44374c,2,0);h.value(0x43800c,2,0);h.value(0x43958e,2,0);h.write(0x435664,new byte[]{12,14,12,14,10,12,10,10,14,16,0,12});h.value(0x438784,2,0);h.value(0x438766,2,0);h.value(0x45365c,2,0);h.value(0x435662,1,0);select(h,1305,1);h.value(0x435884,1,1);h.value(0x443316,2,0x567);h.value(0x40856c,2,0);h.value(0x40856e,2,0);h.value(0x439582,2,0);h.write(0x44292c,new byte[8]);
            JsonObject j=new JsonObject();j.addProperty("member",member);j.addProperty("action_points",ap);j.addProperty("movement",5);j.addProperty("seed_before",seed);j.add("before",guardSnap(h,r));command(h,r);j.add("guarded",guardSnap(h,r));advance(h,r,0x127090);j.add("after",guardSnap(h,r));j.add("rng",JsonParser.parseString("["+r.drawRecords+"]"));j.add("sounds",JsonParser.parseString("["+r.sounds+"]"));output.add(j);println("Guard member"+member+" AP"+ap);
        }finally{h.cpu.dispose();}
    }
}
