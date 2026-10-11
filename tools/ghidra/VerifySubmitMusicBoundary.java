import com.google.gson.*;
import java.nio.file.Files;
import java.nio.file.Path;

/** Four bounded native submission paths, including idle music RNG. */
public class VerifySubmitMusicBoundary extends VerifyStoryInventory {
    Path root;byte[] startup;JsonObject buffers=new JsonObject();
    @Override public void run() throws Exception {
        root=Path.of(getScriptArgs()[0]);startup=Files.readAllBytes(root.resolve("analysis/ghidra/startup/a5-mapped.bin"));
        if(!ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin")).equals("c7a67a96f4d758f078d9d6943cb98a69e60f0e45a22a00ab95bc7c64c8168fd9"))throw new Exception("Mapped executable identity");
        JsonArray cases=new JsonArray();runCase(cases,"disabled",1,0,-1);runCase(cases,"busy",0,1,-1);runCase(cases,"idle_first_allowed",0,0,2);runCase(cases,"idle_rejection",0,0,6);
        JsonObject identity=new JsonObject();identity.addProperty("probe_sha256",ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifySubmitMusicBoundary.java")));identity.addProperty("harness_sha256",ActionOracleHarness.sha256(root.resolve("tools/ghidra/ActionOracleHarness.java")));identity.addProperty("parent_probe_sha256",ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyStoryInventory.java")));identity.addProperty("music_boundary_sha256",ActionOracleHarness.sha256(root.resolve("tools/ghidra/SubmitMusicBoundary.java")));identity.addProperty("mapped_binary_sha256",ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin")));identity.addProperty("startup_digest",ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/a5-mapped.bin")));identity.addProperty("submitmusic_source_digest",ActionOracleHarness.sha256(root.resolve("analysis/ghidra/mapped-exports/submitmusic.asm.tsv")));
        JsonObject result=new JsonObject();result.addProperty("authority","Four complete native submitmusic133d46 RTS controls: disabled WORD439594=1, enabled SoundManager busy1, idle only track index2 or6. Busy status24/error0 and disabled SndDoImmediate error0 are explicit hardware contracts. Idle playback controller439570 remains original startup zero; no resource/heap/playback, error dialog or live audio recovery claimed. Full A5 region, registers and ordered service ABI/RNG captured. No other oracle changed.");result.add("identity",identity);result.add("cases",cases);result.add("shared_a5_buffers",buffers);Files.writeString(root.resolve("evidence/fixtures/submitmusic-boundary.json"),result+"\n");println("Native submitmusic service exported:4");
    }
    void runCase(JsonArray output,String name,int disabled,int busy,int allowed)throws Exception {
        ActionOracleHarness h=frame();RunResult r=new RunResult(12);JsonArray services=new JsonArray();
        try {
            if(!java.util.Arrays.equals(h.read(0x3fe210,startup.length),startup))throw new Exception("Original startup A5 bytes");
            if(h.word(0x439594)!=0 || h.pointer(0x439570)!=0 || h.pointer(0x4436b6)!=0)throw new Exception("Original music startup context");
            h.value(0x439594,2,disabled);byte[] mask=new byte[13];if(allowed>=0)mask[allowed]=1;h.write(0x44270e,mask);
            h.enter(0x133d46,0x6ffe00,0x900000);h.value(0x6ffe04,2,0);JsonObject c=new JsonObject();c.addProperty("case",name);c.addProperty("controlled_disabled_word",disabled);c.addProperty("hardware_busy",busy);c.addProperty("allowed_track_index",allowed);c.add("before",snapshot(h,r));SubmitMusicBoundary.execute(h,monitor,busy,services,()->r.rng(h));c.add("after",snapshot(h,r));c.add("native_music",services);c.add("rng",JsonParser.parseString("["+r.drawRecords+"]"));c.add("sounds",JsonParser.parseString("["+r.sounds+"]"));output.add(c);
        } finally {h.cpu.dispose();}
    }
    JsonObject snapshot(ActionOracleHarness h,RunResult r)throws Exception {
        JsonObject s=new JsonObject();byte[] a5=h.read(0x3fe210,startup.length);String key=java.util.HexFormat.of().formatHex(java.security.MessageDigest.getInstance("SHA-256").digest(a5));if(!buffers.has(key))buffers.addProperty(key,hex(a5));s.addProperty("a5_sha256",key);s.addProperty("a5_length",a5.length);s.addProperty("disabled_word",h.word(0x439594));s.addProperty("error_word",h.word(0x4386ac));s.addProperty("track_mask13",hex(h.read(0x44270e,13)));s.addProperty("channel",h.pointer(0x4436b6));s.addProperty("controller",h.pointer(0x439570));s.addProperty("format_word",h.word(0x439576));s.addProperty("music_resource",h.pointer(0x4394dc));s.addProperty("menu",h.pointer(0x4386a0));s.addProperty("seed",r.seed);s.addProperty("draws",r.draws);s.addProperty("sound_count",r.sounds.length());JsonObject registers=new JsonObject();for(String n:new String[]{"D0","D1","D2","D3","D4","D5","D6","D7","A0","A1","A2","A3","A4","A5","A6","SP","XF","NF","ZF","VF","CF"})registers.addProperty(n,h.cpu.readRegister(n).longValue());s.add("registers",registers);return s;
    }
}
