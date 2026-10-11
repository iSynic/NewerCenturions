import com.google.gson.*;
import java.nio.file.Files;
import java.nio.file.Path;

/** Exact NPC caller with the declared effects-only music preference control. */
public class VerifyQualifiedNpcPoisonCaller extends VerifyCurrentNpcPoisonCaller {
    JsonArray musicPhases=new JsonArray();
    boolean musicPreferencePrepared;
    @Override public void run()throws Exception {
        root=Path.of(getScriptArgs()[0]);draftDirectory=root.resolve("analysis/completion/qualified-npc-poison-caller");Files.createDirectories(draftDirectory);
        Path candidate=root.resolve("evidence/fixtures/current-npc-poison-caller.json"),boundary=root.resolve("evidence/fixtures/submitmusic-boundary.json");
        if(!ActionOracleHarness.sha256(candidate).equals("25b67ca6625d56445cfb7616b881bf752053480bc4982a9fa297a3dd46785cc1")||!ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyCurrentNpcPoisonCaller.java")).equals("8035c4716a426e115d35197243d4456e29694b09db9e98fdb3c971fb78df9f0b")||!ActionOracleHarness.sha256(boundary).equals("e8dcbe1fcd3f5ede2689a5958d34d8d7184f14fbde5bf751a1ac677b1c7322e4")||!ActionOracleHarness.sha256(root.resolve("tools/ghidra/SubmitMusicBoundary.java")).equals("dc6fbf19a3afb5f3252fd4dc2f2a110cc85a5c34b5029bc35f54cb3062323aa6"))throw new Exception("Frozen candidate/music boundary changed");
        JsonObject prior=JsonParser.parseString(Files.readString(candidate)).getAsJsonObject(),foundation=prior.getAsJsonObject("foundation").deepCopy();JsonObject session=foundation.getAsJsonObject("session");
        Path input=root.resolve(foundation.get("input").getAsString());if(!ActionOracleHarness.sha256(input).equals(foundation.get("input_sha256").getAsString())||!JsonParser.parseString(Files.readString(input)).getAsJsonObject().getAsJsonObject("session").equals(session))throw new Exception("Exact saved caller state changed");
        Path original=root.resolve("NewCenturions/TheNewCenturions"),scenario=original.resolve("Scenarios/The Thunder of Guns/0");items=Files.readAllBytes(original.resolve("Data Files/Data ID"));scenarioItems=Files.readAllBytes(scenario.resolve("Data NI"));races=Files.readAllBytes(original.resolve("Data Files/Data Race"));mos=Files.readAllBytes(original.resolve("Data Files/Data MOS"));monsters=Files.readAllBytes(scenario.resolve("Data MD"));properties=Files.readAllBytes(original.resolve("Data Files/Data Biologic BD"));attacks=Files.readAllBytes(original.resolve("Data Files/Data S"));areas=Files.readAllBytes(original.resolve("Data Files/Data AD"));originalRegions=Files.readAllBytes(scenario.resolve("Data RD"));warningStrings=Files.readAllBytes(root.resolve("analysis/extracted/resources/74/53545223_3.bin"));
        foundation.addProperty("controlled_music_disabled_word",1);foundation.addProperty("music_control_qualification","Original WORD439594=1 is an explicit effects-only reconstruction CONTROL outside the saved Session. It is not inferred from saved native audio/channel state. Native submitmusic executes command4 then3 with declared OSErr0 acknowledgements; enabled/idle Rand13 is separately proven and excluded by this declared preference.");
        JsonArray cases=new JsonArray();nativeCaller(cases,session);cases.get(0).getAsJsonObject().add("native_music_phases",musicPhases);
        JsonObject identity=prior.getAsJsonObject("identity").deepCopy();identity.addProperty("omitted_music_candidate_probe_sha256",identity.get("probe_sha256").getAsString());identity.addProperty("probe_sha256",ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyQualifiedNpcPoisonCaller.java")));identity.addProperty("omitted_music_candidate_fixture_sha256",ActionOracleHarness.sha256(candidate));identity.addProperty("music_boundary_sha256",ActionOracleHarness.sha256(root.resolve("tools/ghidra/SubmitMusicBoundary.java")));identity.addProperty("music_boundary_probe_sha256",ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifySubmitMusicBoundary.java")));identity.addProperty("music_boundary_fixture_sha256",ActionOracleHarness.sha256(boundary));
        JsonObject out=new JsonObject();out.addProperty("authority","Exact retained NPC21 caller1145c8->115776 and actual Getup115782->11578a, with explicit original MUSICDISABLED WORD439594=1 effects-only reconstruction CONTROL outside saved Session. Complete native submitmusic133d46 executes both reached calls and ordered Sound Manager command4 then3 under declared OSErr0; no omitted music Rand branch or inferred saved channel status. All80 previous gameplay keys retained plus explicit music_context; qualified service captures native full before/after, ABI/RNG/audio. Old omitted-music candidate preserved unchanged and not promoted as unrestricted caller evidence. No NPC round slice/full live UI/campaign claim.");out.add("identity",identity);out.add("foundation",foundation);out.add("cases",cases);out.add("shared_rule_buffers",pooledBuffers);Files.writeString(draftDirectory.resolve("qualified-npc-poison-caller.json"),out+"\n");println("Qualified current NPC Poison caller exported:1");
    }
    @Override void presentationControls(ActionOracleHarness h)throws Exception {
        super.presentationControls(h);if(h.word(0x439594)!=(musicPreferencePrepared?1:0))throw new Exception("Original startup/declared music preference changed");h.value(0x439594,2,1);musicPreferencePrepared=true;
    }
    @Override JsonObject smallSnap(ActionOracleHarness h,RunResult r)throws Exception {
        JsonObject out=super.smallSnap(h,r);JsonObject context=new JsonObject();context.addProperty("disabled_word",h.word(0x439594));context.addProperty("error_word",h.word(0x4386ac));context.addProperty("track_mask13",hex(h.read(0x44270e,13)));context.addProperty("channel",h.pointer(0x4436b6));context.addProperty("controller",h.pointer(0x439570));context.addProperty("format_word",h.word(0x439576));context.addProperty("music_resource",h.pointer(0x4394dc));out.add("music_context",context);return out;
    }
    @Override void observe(ActionOracleHarness h,RunResult r,long pc)throws Exception {
        super.observe(h,r,pc);if(pc==0x133d46)secondaryStop=pc;
    }
    @Override void execute(ActionOracleHarness h,RunResult r,long stop)throws Exception {
        long previousStop=secondaryStop;
        try {
            for(int count=0;count<20;count++) {
                secondaryStop=previousStop;super.execute(h,r,stop);
                if(h.pc()==stop||(previousStop!=0&&h.pc()==previousStop)||diagnostic)return;
                if(h.pc()!=0x133d46)throw new Exception("Unexpected qualified music interception stop "+Long.toHexString(h.pc()));
                if(h.word(0x439594)!=1)throw new Exception("Caller effects-only music control lost");
                JsonObject phase=new JsonObject();phase.addProperty("address","133d46");phase.add("before",wholeSnap(h,r));JsonArray functions=new JsonArray();SubmitMusicBoundary.execute(h,monitor,0,functions,()->r.rng(h));phase.add("native_submission",functions);phase.add("after",wholeSnap(h,r));musicPhases.add(phase);
            }
            throw new Exception("Qualified music interception budget");
        } finally {secondaryStop=previousStop;}
    }
}
