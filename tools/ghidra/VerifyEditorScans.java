import java.nio.file.Files;
import java.nio.file.Path;
import com.google.gson.*;

/** Original scan-selection mutations with controlled dialog input. */
public class VerifyEditorScans extends VerifyLevelPromotion {
    void scan(ActionOracleHarness h, RunResult result, int[] controls, JsonArray questions) throws Exception {
        int next = 0;
        for (int step = 0; step < 100000; step++) {
            long pc = h.pc(), sp = h.sp();
            if (pc == 0x900000) return;
            if (pc == 0x13a7c4) { loadProfile(h); continue; }
            if (pc == 0x15aa42) {
                h.pop(8); h.cpu.writeRegister("A2", 0x600000); h.cpu.writeRegister("D5", 0);
                h.jump(0x15ac7c); continue;
            }
            if (pc == 0x15acf4) { h.pop(4); h.jump(0x15ad0a); continue; }
            if (pc == 0x15ad0a) {
                if (next == controls.length) throw new Exception("Dialog input exhausted");
                h.value(0x4434b2, 2, controls[next++]); h.jump(0x15ad3e); continue;
            }
            if (pc == 0x15adc0) { h.jump(0x15adf8); continue; }
            if (pc == 0x15ae0a) { h.pop(2); h.jump(0x15af14); continue; }
            if (pc == 0x15af2c) { h.pop(-44); h.jump(0x15afba); continue; }
            if (pc == 0x15b014) { h.pop(-8); h.jump(0x15b058); continue; }
            if (pc == 0x15b062) { h.jump(0x15b098); continue; }
            if (pc == 0x15b106) { h.pop(-8); h.jump(0x15b14a); continue; }
            if (pc == 0x15b154) { h.jump(0x15b18a); continue; }
            if (pc == 0x14cff6) {
                questions.add(h.word(sp + 4)); h.cpu.writeRegister("D0", 1); h.ret(); continue;
            }
            if (pc == 0x133a18 || pc == 0x16474a) { result.sound(h); continue; }
            if (pc == 0x14ce2a || pc == 0x153d56) { h.ret(); continue; }
            int op = h.word(pc) & 65535;
            if (op == 0xa983) { h.pop(4); h.jump(pc + 2); continue; }
            if (!h.cpu.step(monitor)) throw new Exception("Scan PC " + Long.toHexString(pc) + ": " + h.cpu.getLastError());
        }
        throw new Exception("Scan instruction budget");
    }
    @Override public void run() throws Exception {
        Path root = Path.of(getScriptArgs()[0]);
        Path original = root.resolve("NewCenturions/TheNewCenturions");
        races = Files.readAllBytes(original.resolve("Data Files/Data Race"));
        profiles = Files.readAllBytes(original.resolve("Data Files/Data MOS"));
        attacks = Files.readAllBytes(original.resolve("Data Files/Data S"));
        JsonObject fixture = new JsonObject();
        fixture.addProperty("authority", "Original scanselect15a9cc initialization, profile-tier limit, normalization, add/remove affordability and WORD counters, Done/question17, loadspell1314fc, opcode57 and shortcut cleanup execute. Drawing, description-file reads and dialog input are controlled by explicit PC boundaries; caller ac6 eligibility is separately composed. Actual original sound calls captured. No live visual or registration claim.");
        for (String[] p : new String[][]{{"probe", "tools/ghidra/VerifyEditorScans.java"}, {"parent", "tools/ghidra/VerifyLevelPromotion.java"}, {"binary", "analysis/ghidra/startup/code1-mapped.bin"}, {"harness", "tools/ghidra/ActionOracleHarness.java"}, {"races", "NewCenturions/TheNewCenturions/Data Files/Data Race"}, {"profiles", "NewCenturions/TheNewCenturions/Data Files/Data MOS"}, {"attacks", "NewCenturions/TheNewCenturions/Data Files/Data S"}})
            fixture.addProperty(p[0] + "_sha256", ActionOracleHarness.sha256(root.resolve(p[1])));
        JsonArray cases = new JsonArray();
        JsonObject packets = new JsonObject();
        for (String name : new String[]{"Coriand", "Solestri"}) {
            int variant = 0;
            for (int[] controls : new int[][]{{1}, {2, 10, 11, 1}, {2, 10, 10, 1}, {8, 10, 1}}) {
                byte[] before = Files.readAllBytes(original.resolve("Warriors/" + name));
                ActionOracleHarness h = frame(); RunResult result = new RunResult(12);
                try {
                    h.write(0x44721c, before); h.write(0x443788, attacks);
                    byte[] packetBefore = h.read(0x44374e, 58);
                    h.write(0x43788a, new byte[480]);
                    h.write(0x43788a, java.util.Arrays.copyOfRange(before, 0xa76, 0xac6));
                    byte[] memoryBefore = h.read(0x43788a, 480);
                    h.enter(0x15a9cc, 0x6ffe00, 0x900000);
                    JsonArray questions = new JsonArray(); scan(h, result, controls, questions);
                    JsonObject c = new JsonObject(); c.addProperty("case", name + "_selection" + variant++);
                    c.addProperty("before", hex(before)); c.addProperty("after", hex(h.read(0x44721c, 3138)));
                    c.add("controls", new Gson().toJsonTree(controls)); c.add("questions", questions);
                    c.add("sounds", JsonParser.parseString("[" + result.sounds + "]")); cases.add(c);
                    JsonObject packet = new JsonObject(); packet.addProperty("before", hex(packetBefore)); packet.addProperty("after", hex(h.read(0x44374e, 58)));
                    h.cpu.writeRegister("D3", 0); h.jump(0x1628e4);
                    int copySteps = 0;
                    while (h.pc() != 0x1629e8) {
                        if (++copySteps > 10000 || !h.cpu.step(monitor)) throw new Exception("Original scan caller copy failed");
                    }
                    packet.addProperty("memory_before", hex(memoryBefore));
                    packet.addProperty("memory_after", hex(h.read(0x43788a, 480)));
                    packets.add(c.get("case").getAsString(), packet);
                } finally { h.cpu.dispose(); }
            }
        }
        fixture.add("cases", cases);
        fixture.add("attack_packets", packets);
        fixture.addProperty("caller_copy", "Original upgradewarrior1628e4..1629e8 copies the selected record and all ten shortcut slots; member0 with explicit initial shortcut buffer. No copied effect is modeled in the probe.");
        Files.writeString(root.resolve("evidence/fixtures/editor-scans.json"), new GsonBuilder().setPrettyPrinting().create().toJson(fixture) + "\n");
        println("Original scan-selection observations exported: " + cases.size());
    }
}
