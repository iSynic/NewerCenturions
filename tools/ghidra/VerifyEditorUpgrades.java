import com.google.gson.GsonBuilder;
import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;

public class VerifyEditorUpgrades extends VerifyStoryInventory {
    void execute(ActionOracleHarness h, long entry, int selector, boolean upgrade) throws Exception {
        h.enter(entry, 0x6ffe00, 0x900000);
        h.value(0x6ffe04, 2, 0);
        if (upgrade) { h.value(0x6ffe06, 2, selector); h.value(0x6ffe08, 2, 1); }
        for (int step = 0; step < 100000; step++) {
            if (h.pc() == 0x900000) return;
            if (!h.cpu.step(monitor)) throw new Exception("Editor at " + Long.toHexString(h.pc()) + ": " + h.cpu.getLastError());
        }
        throw new Exception("Editor instruction budget");
    }
    @Override public void run() throws Exception {
        Path root = Path.of(getScriptArgs()[0]);
        byte[] original = Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Warriors/Cindred"));
        JsonObject fixture = new JsonObject();
        fixture.addProperty("authority", "Complete original applyupgrade1634f8 and calcstats1636ec instructions, no intercepted children or services. Original Cindred and explicit signed-WORD boundary controls. Caller eligibility/counters are separately source-composed.");
        fixture.addProperty("probe_sha256", ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyEditorUpgrades.java")));
        fixture.addProperty("binary_sha256", ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin")));
        JsonArray cases = new JsonArray();
        for (int index = 0; index < 35; index++) {
            int selector = index <= 32 ? index : index == 33 ? 0 : 7;
            byte[] before = original.clone();
            if (index > 32) {
                ByteBuffer.wrap(before).putShort(0x9d8, (short)32767);
                if (selector == 0) ByteBuffer.wrap(before).putShort(0x9e2, (short)32767);
            }
            ActionOracleHarness h = frame();
            try {
                h.write(0x448aa0, before);
                execute(h, 0x1634f8, selector, true);
                byte[] applied = h.read(0x448aa0, 3138);
                execute(h, 0x1636ec, selector, false);
                JsonObject observation = new JsonObject();
                observation.addProperty("case", "selector" + selector + (index > 32 ? "_word_boundary" : ""));
                observation.addProperty("selector", selector);
                observation.addProperty("before", hex(before));
                observation.addProperty("applied", hex(applied));
                observation.addProperty("after", hex(h.read(0x448aa0, 3138)));
                cases.add(observation);
            } finally { h.cpu.dispose(); }
        }
        fixture.add("cases", cases);
        Files.writeString(root.resolve("evidence/fixtures/editor-upgrades.json"), new GsonBuilder().setPrettyPrinting().create().toJson(fixture) + "\n");
        println("35 complete original upgrade/stat observations exported.");
    }
}
