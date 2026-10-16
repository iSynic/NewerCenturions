import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.security.MessageDigest;
import com.google.gson.*;
import java.nio.ByteBuffer;
import java.nio.file.*;
import java.util.HexFormat;

/** Original selection continuation with the reference emulator's zero fault-read boundary. */
public class VerifyZeroAreaSelection extends GhidraScript {
    byte[] bytes(JsonObject object, String key) {
        return HexFormat.of().parseHex(object.get(key).getAsString());
    }
    void value(EmulatorHelper cpu, long address, int width, long value) {
        cpu.writeMemoryValue(toAddr(address), width, value);
    }
    String sha(Path path) throws Exception {
        return HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(path)));
    }
    @Override public void run() throws Exception {
        Path root = Path.of(getScriptArgs()[0]);
        JsonObject prefix = JsonParser.parseString(Files.readString(root.resolve("evidence/fixtures/enemy-spell-selection.json"))).getAsJsonObject();
        JsonArray cases = new JsonArray();
        for (int faction : new int[] {0, 1}) {
            JsonObject c = prefix.getAsJsonArray("cases").get(0).getAsJsonObject().deepCopy();
            byte[] party = bytes(c, "party"), caster = bytes(c, "caster_before");
            ByteBuffer.wrap(caster).putShort(14, (short)faction);
            for (int i = 0; i < 6; i++) ByteBuffer.wrap(party).putShort(i * 3138 + 16, (short)(1 - faction));
            c.addProperty("party", HexFormat.of().formatHex(party));
            c.addProperty("caster_before", HexFormat.of().formatHex(caster));
            EmulatorHelper cpu = new EmulatorHelper(currentProgram);
            try {
                for (int register = 3; register < 8; register++)
                    cpu.writeRegister("D" + register, 0);
                cpu.writeRegister("A6", 0);
                cpu.writeMemory(toAddr(0x439d3c), caster);
                cpu.writeMemory(toAddr(0x448aa0), party);
                cpu.writeMemory(toAddr(0x44374e), bytes(c, "packet"));
                cpu.writeMemory(toAddr(0x4435d6), bytes(c, "distances"));
                cpu.writeMemory(toAddr(0x443722), bytes(c, "targets"));
                cpu.writeMemory(toAddr(0x7481df), new byte[49]);
                value(cpu, 0x447196, 2, 0);
                value(cpu, 0x44273a, 1, 5);
                value(cpu, 0x4435bc, 2, c.get("range").getAsInt());
                value(cpu, 0x4435cc, 2, c.get("count").getAsInt());
                value(cpu, 0x4435c0, 2, c.get("target_count").getAsInt());
                value(cpu, 0x442780, 4, 5);
                value(cpu, 0x442784, 4, 5);
                for (int i = 0; i < 6; i++) {
                    value(cpu, 0x44709a + 4 * i, 2, 20 + i);
                    value(cpu, 0x44709c + 4 * i, 2, 20);
                }
                cpu.writeRegister("SP", 0x6ffe00);
                cpu.writeRegister("PC", 0x114df2);
                int step = 0;
                while (cpu.getExecutionAddress().getOffset() != 0x1150d0 && cpu.getExecutionAddress().getOffset() != 0x11510a) {
                    if (++step > 20000) throw new Exception("Selection instruction budget");
                    long pc = cpu.getExecutionAddress().getOffset();
                    if (pc == 0x11f1a8 || pc == 0x15a1f6 || pc == 0x159192) {
                        long stack = cpu.readRegister("SP").longValue();
                        cpu.writeRegister("PC", Integer.toUnsignedLong(ByteBuffer.wrap(cpu.readMemory(toAddr(stack), 4)).getInt()));
                        cpu.writeRegister("SP", stack + 4);
                        continue;
                    }
                    if (!cpu.step(monitor)) throw new Exception("Selection: " + cpu.getLastError());
                }
                c.addProperty("case", "zero_area_faction" + faction);
                c.addProperty("caster_after", HexFormat.of().formatHex(cpu.readMemory(toAddr(0x439d3c), 288)));
                c.addProperty("targets", HexFormat.of().formatHex(cpu.readMemory(toAddr(0x443722), 42)));
                c.addProperty("cast", cpu.getExecutionAddress().getOffset() == 0x1150d0);
                c.addProperty("invalid_shape", false);
                cases.add(c);
            } finally {
                cpu.dispose();
            }
        }
        JsonObject fixture = new JsonObject();
        fixture.addProperty("authority", "Original combat114df2 through cast1150d0/no-cast11510a including complete bestchoice. Independent captured getrange prefix0 supplies inputs. Zero fault-read boundary is source-derived from isolated reference emulator ignoresegv/load-zeroing and live inaccessible address; no native faulting-load or whole-turn claim. Three presentation calls return without rule effects; no RNG in this continuation. Faction0/1 controls preserve opposition. Native outputs contain complete caster and targets.");
        fixture.addProperty("probe_sha256", sha(root.resolve("tests/native/VerifyZeroAreaSelection.java")));
        fixture.addProperty("prefix_fixture_sha256", sha(root.resolve("evidence/fixtures/enemy-spell-selection.json")));
        fixture.addProperty("mapped_binary_sha256", sha(root.resolve("analysis/ghidra/startup/code1-mapped.bin")));
        fixture.addProperty("fault_policy_evidence_sha256", sha(root.resolve("analysis/thunder-completion/hive-original-fault-policy.json")));
        fixture.addProperty("live_mapping_receipt_sha256", sha(root.resolve("analysis/thunder-completion/hive-original-mapped-read.json")));
        fixture.add("areas", prefix.get("areas"));
        fixture.add("properties", prefix.get("properties"));
        fixture.add("cases", cases);
        Files.writeString(root.resolve("evidence/fixtures/zero-area-selection.json"), new GsonBuilder().setPrettyPrinting().create().toJson(fixture) + "\n");
        println("Two original selection continuations captured with explicit zero fault-read boundary");
    }
}
