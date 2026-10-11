import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import com.google.gson.*;

/** Consecutive original learned-healing menus retain remembered power. */
public class VerifyLearnedHealMemory extends VerifyLearnedPortraitHeal {
    JsonArray menuEvents=new JsonArray();
    @Override void observe(ActionOracleHarness h,RunResult r,long pc)throws Exception {
        super.observe(h,r,pc);
        if(pc==0x10bc0a||pc==0x10bc3e||pc==0x10c5ba||pc==0x10c5dc||pc==0x10c5e2||pc==0x10c5e6||pc==0x10c644||pc==0x10c654||pc==0x10c662||pc==0x10bf56){JsonObject e=new JsonObject();e.addProperty("address",Long.toHexString(pc));e.addProperty("count",h.word(0x4435cc));e.addProperty("remembered_power",h.word(0x437a7a+activeBody*2));e.addProperty("input",h.word(0x4434b2));e.addProperty("d6",h.cpu.readRegister("D6").intValue());e.addProperty("draws",r.draws);e.addProperty("psi",h.word(0x448aa0+activeBody*3138+0x58));menuEvents.add(e);}
    }
    @Override JsonObject completeSnap(ActionOracleHarness h,RunResult r)throws Exception {JsonObject j=super.completeSnap(h,r);j.add("menu_events",menuEvents.deepCopy());return j;}

    @Override boolean prepare(ActionOracleHarness h,RunResult r,int power,String kind)throws Exception {
        combatFrame(h,r);secondaryStop=0x116b6a;execute(h,r,0x10b9bc);secondaryStop=0;if(h.pc()==0x116b6a)return false;
        execute(h,r,0x10b9c4);h.jump(0x10b9da);execute(h,r,0x10b9ee);h.pop(2);
        h.jump(0x10bba6);execute(h,r,0x10bbc2);h.jump(0x10bbc8);execute(h,r,0x10bc3e);
        h.value(0x4434b2,2,23);h.jump(0x10c464);execute(h,r,0x10bf56);
        int school=h.cpu.readRegister("D5").intValue();h.cpu.writeRegister("D3",0);h.cpu.writeRegister("D4",school);h.jump(0x10b700);secondaryStop=0x10b7d4;execute(h,r,0x10b728);secondaryStop=0;boolean learned=h.pc()==0x10b728;
        h.write(0x479ed8,new byte[256]);if(learned)h.write(0x479ed8,healName);
        h.value(0x4434b2,2,1);h.jump(0x10c846);execute(h,r,0x10c852);h.jump(0x10c880);secondaryStop=0x10cb60;execute(h,r,0x10c890);secondaryStop=0;if(!learned)return false;
        h.jump(0x10c918);execute(h,r,0x10c938);h.jump(0x10c9f0);execute(h,r,0x10ca32);h.jump(0x10ca78);execute(h,r,0x10cab0);
        h.value(0x4434b2,2,29+power);h.jump(0x10c5ba);secondaryStop=0x10bf56;execute(h,r,0x10c5e6);secondaryStop=0;
        if(h.pc()==0x10c5e6){secondaryStop=0x10c788;execute(h,r,0x10c622);secondaryStop=0;if(h.pc()==0x10c788){execute(h,r,0x10c792);return false;}execute(h,r,0x10c644);execute(h,r,0x10c698);}
        h.jump(0x10cb8c);secondaryStop=0x10c788;execute(h,r,0x10cc40);secondaryStop=0;if(h.pc()==0x10c788){execute(h,r,0x10c792);return false;}
        if(h.sp()!=h.cpu.readRegister("A6").longValue()-0x3c)throw new Exception("Unbalanced native castspell frame");h.jump(0x10d1d6);execute(h,r,0x119238);execute(h,r,0x1192d0);return true;
    }

    void portraitInputs(ActionOracleHarness h,int target)throws Exception {
        h.write(0x821000,new byte[512]);h.value(0x821000,4,0x821100);h.value(0x821108,2,0);h.value(0x82110a,2,330);h.value(0x82110c,2,300);h.value(0x82110e,2,380);h.value(0x439440,4,0x821000);h.value(0x4471ee,2,target*50+10);h.value(0x4471f0,2,340);
    }
    void chain(JsonArray out,JsonObject s,int power)throws Exception {
        long seed=s.getAsJsonObject("random").get("seed").getAsLong();begin(seed);poisonEvents=new JsonArray();aidEvents=new JsonArray();warnings=new JsonArray();medicalEvents=new JsonArray();learnedEvents=new JsonArray();menuEvents=new JsonArray();choice=1;itemUse=true;ActionOracleHarness h=frame();RunResult r=new RunResult(seed);r.draws=s.getAsJsonObject("random").get("draws").getAsInt();
        try {initializeActual(h,s);JsonObject c=new JsonObject();c.addProperty("second_power",power);c.add("before",completeSnap(h,r));if(!prepare(h,r,2,"first"))throw new Exception("First actual power2 not admitted");c.add("first_prepared",completeSnap(h,r));portraitInputs(h,1);h.value(0x44720e,2,0x61);h.jump(0x11a18e);execute(h,r,0x116b6a);execute(h,r,0x117338);c.add("after_cancel",completeSnap(h,r));if(!prepare(h,r,power,"second"))throw new Exception("Second actual learned selection not admitted");c.add("second_prepared",completeSnap(h,r));portraitInputs(h,1);h.jump(0x119db6);execute(h,r,0x116b6a);execute(h,r,0x117338);c.add("after",completeSnap(h,r));saveTrace(c,r);out.add(c);println("Learned healing memory power2 cancel then power"+power+" portrait");}finally {h.cpu.dispose();itemUse=false;secondaryStop=0;}
    }
    public void run()throws Exception {
        Path root=Path.of(getScriptArgs()[0]),original=root.resolve("NewCenturions/TheNewCenturions"),scenario=original.resolve("Scenarios/The Thunder of Guns/0");items=Files.readAllBytes(original.resolve("Data Files/Data ID"));scenarioItems=Files.readAllBytes(scenario.resolve("Data NI"));races=Files.readAllBytes(original.resolve("Data Files/Data Race"));monsters=Files.readAllBytes(scenario.resolve("Data MD"));properties=Files.readAllBytes(original.resolve("Data Files/Data Swamp BD"));attacks=Files.readAllBytes(original.resolve("Data Files/Data S"));areas=Files.readAllBytes(original.resolve("Data Files/Data AD"));mos=Files.readAllBytes(original.resolve("Data Files/Data MOS"));healName=Arrays.copyOfRange(Files.readAllBytes(original.resolve("Data Files/Cargo Pod.rsrc")),6520247,6520257);if(healName[0]!=9)throw new Exception("Changed Heal name resource");
        Path input=root.resolve("analysis/completion/learned-portrait-heal-input-12.json");JsonObject s=JsonParser.parseString(Files.readString(input)).getAsJsonObject().getAsJsonObject("session");JsonObject identity=JsonParser.parseString(Files.readString(root.resolve("evidence/fixtures/learned-portrait-heal.json"))).getAsJsonObject().getAsJsonObject("identity").deepCopy();identity.addProperty("learned_portrait_probe_sha256",identity.get("probe_sha256").getAsString());identity.addProperty("probe_sha256",ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyLearnedHealMemory.java")));identity.addProperty("learned_portrait_fixture_sha256",ActionOracleHarness.sha256(root.resolve("evidence/fixtures/learned-portrait-heal.json")));JsonArray cases=new JsonArray();for(int power:new int[]{1,2})chain(cases,s,power);JsonObject f=new JsonObject();f.addProperty("input",root.relativize(input).toString().replace('\\','/'));f.addProperty("digest",ActionOracleHarness.sha256(input));f.add("session",s);JsonObject out=new JsonObject();out.addProperty("authority","Two complete consecutive ordinary learnedHeal1101 menus onactualearnedseed12prefix before316: originalfirstpower2 preparation andconfirmedEscape Yes, originalsecondfreshmenu restoresnativepower2 andexplicitwidget30/31 executesnative10c5ba..e2 unchanged-power skip orchangedpoweraffordability/audio/writes, thenfullunconsciousportrait/Getup. Native rememberedtables/counters/Psimemory preservedbetweensteps; noinventory/HP/Psi/RNGgrant andnooldfrozenprobe/fixtureedit. Inherits originalboundedToolbox/presentation contracts andlegacyhealth_contextscalaralias qualification; explicit learned_marker/heal_marks_affected authoritative. No broader spellmenu/livecampaign claim.");out.add("identity",identity);out.add("foundation",f);out.add("cases",cases);Files.writeString(root.resolve("evidence/fixtures/learned-heal-memory.json"),out+"\n");println("Learned healing memory exported2cases");
    }
}
