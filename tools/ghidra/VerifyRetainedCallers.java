import ghidra.app.script.GhidraScript;
import com.google.gson.*;
import java.nio.file.*;
import java.nio.ByteBuffer;
import java.util.*;

/** Original selected-row writers joined to original story consumption without resetting work. */
public class VerifyRetainedCallers extends GhidraScript {
    long seed = 1;
    int draws;
    static byte[] bytes(JsonObject row, String key) {
        return HexFormat.of().parseHex(row.get(key).getAsString());
    }
    void execute(ActionOracleHarness h, long stop, JsonArray sounds) throws Exception {
        SaneBoundary sane = new SaneBoundary(h.cpu, currentProgram);
        for (int steps = 0; h.pc() != stop; steps++) {
            if (steps > 500000) throw new Exception("Retained caller instruction budget");
            long pc = h.pc();
            if (pc == 0x153ca4) {
                int bound = h.word(h.sp() + 4);
                seed = seed * 16807 % 2147483647;
                int raw = (short)seed;
                h.cpu.writeRegister("D0", Math.abs(raw == -32768 ? 0 : raw) * bound / 32768 + 1);
                draws++;
                h.ret();
            } else if (pc == 0x13a7c4) {
                Path data = Path.of(getScriptArgs()[0], "NewCenturions/TheNewCenturions/Data Files");
                for (int part = 0; part < 2; part++) {
                    int index = h.word(h.sp() + 4 + part * 2), size = part == 0 ? 408 : 576;
                    if (index != 0) h.write(part == 0 ? 0x41f85a : 0x41f9f2, Arrays.copyOfRange(Files.readAllBytes(data.resolve(part == 0 ? "Data Race" : "Data MOS")), (index - 1) * size, index * size));
                }
                h.ret();
            } else if ((h.word(pc) & 65535) == 0xa85d) {
                int bit = (int)h.pointer(h.sp());
                int value = h.read(h.pointer(h.sp() + 4) + (bit >> 3), 1)[0] & (128 >> (bit & 7));
                h.value(h.sp() + 8, 1, value == 0 ? 0 : 1);
                h.pop(8);
                h.jump(pc + 2);
            } else if (pc == 0x14a074) { displayItemWriter(h, true); h.ret(); }
            else if (pc == 0x15498e) h.ret();
            else if (Set.of(0x160fb2L,0x11f1a8L,0x16043eL,0x10e6b6L).contains(pc)) h.ret();
            else if (pc == 0x13154c) h.ret();
            else if (Set.of(0xa873,0xa862,0xa863).contains(h.word(pc) & 65535)) { h.pop(4); h.jump(pc + 2); }
            else if (pc == 0x12946c) { h.cpu.writeRegister("D0", 1); h.ret(); }
            else if (pc == 0x133a18 || pc == 0x16474a) { sounds.add(h.word(h.sp() + 4)); h.ret(); }
            else if (pc == 0x400038) { h.cpu.writeRegister("D0", Math.abs(h.word(h.sp() + 4))); h.ret(); }
            else if ((h.word(pc) & 65535) == 0xa9eb) sane.execute();
            else if (!h.cpu.step(monitor)) throw new Exception("Retained caller " + Long.toHexString(pc) + ": " + h.cpu.getLastError());
        }
    }
    void displayItemWriter(ActionOracleHarness h) throws Exception {
        displayItemWriter(h, false);
    }
    void displayItemWriter(ActionOracleHarness h, boolean icon) throws Exception {
        Map<String, java.math.BigInteger> saved = new LinkedHashMap<>();
        for (String name : new String[]{"D0","D1","D2","D3","D4","D5","D6","D7","A0","A1","A2","A3","A4","A5","A6","SP","PC","XF","NF","ZF","VF","CF"}) saved.put(name, h.cpu.readRegister(name));
        int body = h.word(h.sp() + 4);
        h.enter(icon ? 0x14a074 : 0x11faa0, 0x6da000, 0x6ff000);
        if (icon) h.value(h.sp() + 4, 2, body);
        h.cpu.writeRegister("D3", body);
        Set<Long> stops = Set.of(0x11faecL, 0x11fbbeL, 0x11fd6eL, 0x11fdb2L, 0x14a0daL);
        for (int steps = 0; !stops.contains(h.pc()); steps++) {
            if (steps > 600) throw new Exception("combatupdate2 item-writer budget");
            if (!h.cpu.step(monitor)) throw new Exception("Item writer " + Long.toHexString(h.pc()) + ": " + h.cpu.getLastError());
        }
        for (Map.Entry<String, java.math.BigInteger> entry : saved.entrySet()) h.cpu.writeRegister(entry.getKey(), entry.getValue());
    }
    void receipt(Path root, Path output, JsonObject result) throws Exception {
        result.addProperty("probe_sha256", ActionOracleHarness.sha256(root.resolve("tools/ghidra/" + getClass().getSimpleName() + ".java")));
        result.addProperty("parent_probe_sha256", ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyRetainedCallers.java")));
        result.addProperty("binary_sha256", ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin")));
        Files.writeString(Files.createFile(output), new GsonBuilder().setPrettyPrinting().create().toJson(result) + "\n");
    }
    @Override public void run() throws Exception {
        Path root = Path.of(getScriptArgs()[0]), output = root.resolve(getScriptArgs()[1]);
        if (Files.exists(output)) throw new Exception("Capture output exists");
        Path data = root.resolve("NewCenturions/TheNewCenturions/Data Files");
        byte[] items = Files.readAllBytes(data.resolve("Data ID")), races = Files.readAllBytes(data.resolve("Data Race"));
        byte[] scenario = Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data NI"));
        JsonArray cases = new JsonArray();
        boolean refit = getScriptArgs().length > 2;
        boolean right = refit && getScriptArgs()[2].endsWith("right");
        boolean battery = refit && getScriptArgs()[2].startsWith("battery");
        boolean charged = battery && getScriptArgs()[2].endsWith("loaded");
        int rounds = battery && !charged ? 2 : 1;
        for (int target : battery ? new int[]{820} : new int[]{820, 243}) for (boolean stock : refit ? new boolean[]{false} : new boolean[]{false, true}) {
            String source = target == 820 ? "story-retained-equipment.json" : "story-equipped-depletion.json";
            JsonObject old = JsonParser.parseString(Files.readString(root.resolve("evidence/fixtures/" + source))).getAsJsonObject().getAsJsonArray("consume_cases").get(0).getAsJsonObject();
            byte[] party = bytes(old, "before"), extra = bytes(old, "extra");
            if (battery) {
                JsonObject shield = JsonParser.parseString(Files.readString(root.resolve("evidence/fixtures/personal-shield.json"))).getAsJsonObject();
                party = bytes(shield.getAsJsonArray("equipment_cases").get(0).getAsJsonObject().getAsJsonObject("battery_worn"), "party");
                for (int member = 0; member < 6; member++) ByteBuffer.wrap(party).putShort(member * 3138 + 0x3a, (short)99);
                if (charged) ByteBuffer.wrap(party).putShort(3 * 3138 + 0x132 + 5 * 22 + 2, (short)1).putShort(3 * 3138 + 0x132 + 5 * 22 + 4, (short)307).putShort(3 * 3138 + 0x26, (short)1);
            }
            if (target == 820) {
                ByteBuffer.wrap(party).putShort(0x132, (short)820).putShort(0x102, (short)820);
                ByteBuffer.wrap(extra).putShort(0, (short)820);
            }
            if (refit && target == 820) ByteBuffer.wrap(party).putShort(0x136, (short)201);
            JsonArray sounds = new JsonArray();
            JsonObject c = new JsonObject();
            ActionOracleHarness h = new ActionOracleHarness(currentProgram);
            try {
                h.enter(0x139eca, 0x6ffe00, 0x6ff000);
                h.write(0x448aa0, party);
                h.write(0x44721c, Arrays.copyOfRange(party, 0, 3138));
                h.value(0x44273a, 1, 5);
                h.value(0x4470b2, 2, stock ? 8 : 0);
                h.value(0x44273d, 1, right ? 1 : 0);
                h.write(0x41fc32, new byte[44]);
                h.write(0x41fc74, new byte[22000]);
                h.write(0x42df04, new byte[1000]);
                for (int family = 0; family < 5; family++) h.write(new long[]{0x4537fe,0x46225e,0x45ad2e,0x46978e,0x470cbe}[family], Arrays.copyOfRange(family == 4 ? scenario : items, family == 4 ? 0 : family * 30000, family == 4 ? 30000 : (family + 1) * 30000));
                for (int race = 0; race < 10; race++) h.value(0x435664 + race, 1, races[race * 408 + 197]);
                h.value(0x439404, 4, right ? 2 : 1);
                h.value(0x4393fc, 4, 2);
                execute(h, refit ? 0x139f6c : 0x139f4a, sounds);
                if (stock) {
                    h.value(0x4470b2, 2, 0);
                    h.value(0x4471a8, 2, 0);
                    h.value(0x478288, 2, 1);
                    h.value(0x478a58, 1, 1);
                    h.jump(0x139d34);
                    execute(h, 0x139e24, sounds);
                }
                if (refit) {
                    c.addProperty("item_before_prefix", HexFormat.of().formatHex(h.read(0x45363c, 150)));
                    c.addProperty("ammo_before_prefix", HexFormat.of().formatHex(h.read(0x4536d2, 150)));
                    c.addProperty("weapon_before_prefix", HexFormat.of().formatHex(h.read(0x453768, 150)));
                    c.addProperty("before_prefix", HexFormat.of().formatHex(party));
                    h.write(0x42dbe4, new byte[804]);
                    h.value(0x42dbe4 + (battery ? 107 * 4 : target == 820 ? 4 : 0), 4, battery || target == 820 ? 2 : 1);
                    c.addProperty("reserves_before_prefix", HexFormat.of().formatHex(h.read(0x42dbe4, 804)));
                    h.value(0x438776, 2, 0);
                    h.value(0x442750, 1, 0);
                    c.addProperty("refit_rounds", rounds);
                    c.addProperty("side", right ? 1 : 0);
                    for (int round = 0; round < rounds; round++) for (int member = 0; member < 6; member++) {
                        h.enter(0x14e5e4, 0x6ffe00, 0x6ff000);
                        h.value(h.sp() + 4, 2, member);
                        h.value(h.sp() + 6, 2, 1);
                        h.value(h.sp() + 8, 2, 1);
                        h.value(h.sp() + 10, 2, 0);
                        execute(h, 0x6ff000, sounds);
                    }
                    party = h.read(0x448aa0, party.length);
                    c.addProperty("reserves_after_prefix", HexFormat.of().formatHex(h.read(0x42dbe4, 804)));
                    c.addProperty("prefix_ammo", HexFormat.of().formatHex(h.read(0x4536d2, 150)));
                    c.addProperty("prefix_item", HexFormat.of().formatHex(h.read(0x45363c, 150)));
                    c.addProperty("prefix_weapon", HexFormat.of().formatHex(h.read(0x453768, 150)));
                }
                c.add("prefix_sounds", sounds.deepCopy());
                sounds = new JsonArray();
                c.addProperty("selected_before", HexFormat.of().formatHex(h.read(0x41fc48, 22)));
                h.cpu.writeRegister("A6", 0x700000);
                h.cpu.writeRegister("SP", 0x6ffe00);
                h.write(0x44330a, extra);
                h.jump(0x146560);
                execute(h, 0x14673c, sounds);
                c.addProperty("after", HexFormat.of().formatHex(h.read(0x448aa0, party.length)));
                c.addProperty("selected_after", HexFormat.of().formatHex(h.read(0x41fc48, 22)));
                c.addProperty("item_after", HexFormat.of().formatHex(h.read(0x45363c, 150)));
                c.addProperty("consumed", h.cpu.readRegister("D6").intValue());
            } finally { h.cpu.dispose(); }
            c.addProperty("case", "selected_" + (battery ? "battery_" + (charged ? "loaded_" : "empty_") : refit ? "refit_" : stock ? "stock_" : "owned_") + target);
            c.addProperty("writer", refit ? "refit" : stock ? "stock" : "owned");
            c.addProperty("before", HexFormat.of().formatHex(party));
            c.addProperty("extra", HexFormat.of().formatHex(extra));
            c.add("sounds", sounds);
            cases.add(c);
        }
        JsonObject receipt = new JsonObject();
        receipt.addProperty("authority", "Original updateshop selected-owned139eca..139f4a and stock139d34..139e24 writers, followed by original newland146560..14673c/removeitem/dropitem/movecalc in the same emulator memory. No retained-record reset between writer and consumer. Original definitions/race movements and finite SANE supplied; sound/warning arguments captured. Setup uses preserved native party controls, replacing the first equipped row and class15 slot with820 for removal, or preserved equipped243 depletion. Input polls, complete inventory dialog, transfer/acquisition history and story presentation are outside this bounded join; no live UI claim.");
        JsonArray movement = new JsonArray();
        for (int race = 0; race < 10; race++) movement.add(races[race * 408 + 197]);
        receipt.add("race_movements", movement);
        if (refit) receipt.addProperty("refit_authority", "Original owned selection139eca..139f6c including ploticon/loaditemnotype14a074..14a0da, six refit14e5e4 calls with flags110, then original820/243 consumer without resetting retained work or party. Assisted820 uses compatible ammunition201/reserve2;243 control has ammunition200/reserve1. Prefix changes magazine quantities, while the selected22-byte record stays stale. Left-owned window context1/right2, declared initial work bytes; quantity drawing is delegated. No original drag or menu event loop claim.");
        receipt.add("consume_cases", cases);
        receipt(root, output, receipt);
        println("Original retained writer/consumer joins captured: " + cases.size());
    }
}
