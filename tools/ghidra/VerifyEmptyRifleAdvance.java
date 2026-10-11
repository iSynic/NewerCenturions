import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import com.google.gson.*;

/** Original selected ammunition fitting followed by automatic initiative. */
public class VerifyEmptyRifleAdvance extends VerifyEmptyRifleAmmunition {
    JsonObject advanceSnap(ActionOracleHarness h,RunResult r){
        JsonObject j=snap(h,r);j.addProperty("order",hex(h.read(0x4470b8,220)));j.addProperty("cursor",h.word(0x447194));j.addProperty("monster_index",h.word(0x447196));j.addProperty("round",h.read(0x44255c,1)[0]&255);j.addProperty("death_count",h.word(0x44374c));j.addProperty("enemy_phase",h.word(0x438778));j.addProperty("party_positions",hex(h.read(0x44709a,24)));j.addProperty("monster_positions",hex(h.read(0x446f0a,4)));j.addProperty("memory",hex(h.read(0x43788a,480)));j.addProperty("selected",hex(h.read(0x442580,24)));j.addProperty("powers",hex(h.read(0x437a7a,24)));j.addProperty("unavailable",h.read(0x442739,1)[0]&255);j.addProperty("last_amount",h.word(0x4434b8));return j;
    }
    void advance(ActionOracleHarness h,RunResult r,long stop)throws Exception {
        SaneBoundary sane=new SaneBoundary(h.cpu,currentProgram);
        for(int step=0;step<1500000;step++){
            long pc=h.pc(),sp=h.sp();observe(h,r,pc);if(pc==stop)return;
            if(pc==0x400038){h.cpu.writeRegister("D0",(short)Math.abs(h.word(sp+4)));h.ret();continue;}
            if(pc==0x153ca4){rngSites.add(Long.toHexString(h.pointer(sp)));r.rng(h);continue;}
            if(pc==0x133a18||pc==0x16474a){r.sound(h);continue;}
            if(pc==0x158454){ResultAudioBoundary.execute(h,r,monitor);h.ret();continue;}
            if(pc==0x14de26){h.jump(0x14dede);continue;}
            if(pc==0x12946c){h.cpu.writeRegister("D0",1);h.ret();continue;}
            if(pc==0x133d46||pc==0x161b80||pc==0x13a2c4||pc==0x1205f8||pc==0x105572||pc==0x1382a8||pc==0x15f66c||pc==0x1600fc||pc==0x13a498||pc==0x12382a||pc==0x13154c||pc==0x123d1c||pc==0x160a8e||pc==0x160cca||pc==0x160baa||pc==0x160fb2||pc==0x11f1a8||pc==0x16043e||pc==0x10e6b6){h.ret();continue;}
            int op=h.word(pc)&65535;if(op==0xa8a7){h.pop(12);h.jump(pc+2);continue;}if(op==0xa8a8){h.pop(8);h.jump(pc+2);continue;}if(op==0xa873||op==0xa893||op==0xa884){h.pop(4);h.jump(pc+2);continue;}if(op==0xaa1e){h.value(sp+2,4,0);h.pop(2);h.jump(pc+2);continue;}if(op==0xa9eb){sane.execute();continue;}
            if(!h.cpu.step(monitor))throw new Exception("Ammunition initiative at "+Long.toHexString(pc)+": "+h.cpu.getLastError());
        }throw new Exception("Ammunition initiative instruction budget");
    }
    public void run()throws Exception {
        Path root=Path.of(getScriptArgs()[0]),original=root.resolve("NewCenturions/TheNewCenturions"),scenario=original.resolve("Scenarios/The Thunder of Guns/0");
        items=Files.readAllBytes(original.resolve("Data Files/Data ID"));scenarioItems=Files.readAllBytes(scenario.resolve("Data NI"));races=Files.readAllBytes(original.resolve("Data Files/Data Race"));mos=Files.readAllBytes(original.resolve("Data Files/Data MOS"));monsters=Files.readAllBytes(scenario.resolve("Data MD"));properties=Files.readAllBytes(original.resolve("Data Files/Data Swamp BD"));attacks=Files.readAllBytes(original.resolve("Data Files/Data S"));areas=Files.readAllBytes(original.resolve("Data Files/Data AD"));party=new byte[18828];String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};for(int i=0;i<6;i++)System.arraycopy(Files.readAllBytes(original.resolve("Warriors/"+names[i])),0,party,i*3138,3138);
        JsonObject prior=JsonParser.parseString(Files.readString(root.resolve("evidence/fixtures/empty-rifle-ammunition.json"))).getAsJsonObject();inputs[0]=prior.getAsJsonArray("foundations").get(0).getAsJsonObject().getAsJsonObject("session");JsonObject identity=prior.getAsJsonObject("identity").deepCopy();identity.addProperty("ammunition_probe_sha256",identity.get("probe_sha256").getAsString());identity.addProperty("probe_sha256",ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyEmptyRifleAdvance.java")));identity.addProperty("ammunition_fixture_sha256",ActionOracleHarness.sha256(root.resolve("evidence/fixtures/empty-rifle-ammunition.json")));
        JsonArray cases=new JsonArray();for(long seed:new long[]{12,1})for(int reserve:new int[]{12,2})probe(cases,seed,reserve);
        JsonObject out=new JsonObject();out.addProperty("authority","Four actualemptyrifle38 foundation cases after original pickup/emptywear/compact, manual203selection in complete1300ec with stock12or2 and AP2. Original Combatchoice117be2 prologue and accepted117cd4/117cf6..117d5c caller retainsnativeframe, complete epilogue11ce76..11ce7c returns to original outerAPcompare116b6a..116b86 and completegetup125a3a..127090. Sixhealthy actualwarriors, originalMD13body10 controlledHP10000, controlledinitiative body0slot1/enemy10slot2, nofields and packet19sentinel8. Complete buffers/context/order/scalars and orderedRNG/audio captured. No Finish or enemy action executes. GUIselection/menu/stringdisplay inputs and inherited presentation/SANE boundaries explicit; bounded originalinstructions, no live-game or campaignsurvival claim.");out.add("identity",identity);out.add("foundation",prior.getAsJsonArray("foundations").get(0).deepCopy());out.add("cases",cases);Files.writeString(root.resolve("evidence/fixtures/empty-rifle-advance.json"),out+"\n");println("Empty rifle initiative exported:4");
    }
    void probe(JsonArray output,long seed,int reserve)throws Exception {
        ActionOracleHarness h=frame();RunResult r=new RunResult(seed);try{byte[] p=warriors(inputs[0]),c=crate(inputs[0]);setup(h,p,c,0,0,r);orderedCalls=new JsonArray();guiSelections=new JsonArray();procedureReturns=new JsonArray();selectAmmo=true;h.write(0x442562,new byte[20]);h.write(0x4435c6,new byte[6]);h.value(0x4435cc,2,0);h.value(0x437a6c,1,0);JsonArray stock=inputs[0].getAsJsonArray("ammunition");for(int i=0;i<200;i++)h.value(0x42dbe4+i*4,4,stock.get(i).getAsLong());h.value(0x42dbe4+12,4,reserve);
            int cr=-1;for(int i=0;i<1000;i++)if(ByteBuffer.wrap(c).getShort(i*22)==38){cr=i;break;}h.write(0x41fc74,c);if(cr<0||pickup(h,r,0,cr)==0)throw new Exception("Empty rifle pickup failed");h.cpu.writeRegister("A6",0x700000);h.value(0x70000a,2,1);h.jump(0x10800e);execute(h,r,0x10808c);int slot=locate(h,0,38);selected(h,0,slot);if(call(h,r,0x164a0e,0,slot,1)==0)throw new Exception("Empty rifle wear failed");call(h,r,0x12006e,0);slot=locate(h,0,38);selected(h,0,slot);
            h.value(0x438776,2,1);h.value(0x448aa0+0x40,2,2);h.value(0x448aa0+0x3c,2,5);byte[] order=new byte[220];for(int i=0;i<110;i++)ByteBuffer.wrap(order).putShort(i*2,(short)-1);ByteBuffer.wrap(order).putShort(2,(short)0).putShort(4,(short)10);h.write(0x4470b8,order);h.value(0x44255c,1,1);h.value(0x447194,2,1);h.value(0x447196,2,0);h.value(0x44374c,2,0);h.value(0x43800c,2,0);h.value(0x43958e,2,0);h.write(0x435664,new byte[]{12,14,12,14,10,12,10,10,14,16,0,12});h.value(0x438784,2,0);h.value(0x438766,2,0);h.value(0x435662,1,0);h.value(0x442739,1,0);h.value(0x44273a,1,6);h.value(0x438778,2,0);h.value(0x435884,1,1);h.value(0x44374e+19,1,8);h.value(0x44719c,2,slot);h.value(0x4471aa,2,0);h.value(0x446f0a,2,20);h.value(0x446f0c,2,20);
            JsonObject j=new JsonObject();j.addProperty("source",0);j.addProperty("seed_before",seed);j.addProperty("reserve203",reserve);j.addProperty("member",0);j.addProperty("inventory_slot",slot);j.add("before",advanceSnap(h,r));h.enter(0x117be2,0x6ffe00,0x116b6a);execute(h,r,0x117c46);h.jump(0x117cd4);execute(h,r,0x117cde);h.jump(0x117cf6);execute(h,r,0x117d5c);j.add("manual",advanceSnap(h,r));execute(h,r,0x11ce76);advance(h,r,0x127090);j.add("after",advanceSnap(h,r));j.add("rng",JsonParser.parseString("["+r.drawRecords+"]"));j.add("sounds",JsonParser.parseString("["+r.sounds+"]"));output.add(j);println("Empty rifle initiative seed"+seed+" reserve"+reserve);
        }finally{h.cpu.dispose();}
    }
}
