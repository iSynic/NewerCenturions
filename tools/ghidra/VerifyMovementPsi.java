import ghidra.app.script.GhidraScript;
import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.HexFormat;

/** Original learned Psi transactions through the shared charge/AP caller. */
public class VerifyMovementPsi extends GhidraScript {
    public void run() throws Exception {
        Path root = Path.of(getScriptArgs()[0]);
        Path output = Path.of(getScriptArgs()[1]);
        if (Files.exists(output)) throw new Exception("Native output already exists: " + output);
        Path input = root.resolve("evidence/fixtures/player-psi.json");
        JsonObject foundation = JsonParser.parseString(Files.readString(input))
            .getAsJsonObject().getAsJsonObject("route_foundation");
        JsonObject combat = foundation.getAsJsonObject("combat");
        Path spells = root.resolve("NewCenturions/TheNewCenturions/Data Files/Data S");
        byte[] source = Files.readAllBytes(spells);
        String mode = getScriptArgs().length > 2 ? getScriptArgs()[2] : "movement";
        boolean drain = mode.equals("drain"), charm = mode.equals("charm");
        boolean outdoor = mode.equals("outdoor"), encounter = mode.equals("encounter");
        int[] ids = switch (mode) {
            case "drain" -> new int[]{1502, 1603, 3503};
            case "charm" -> new int[]{1406, 1506, 3506};
            case "outdoor" -> new int[]{1104, 1107};
            case "encounter" -> new int[]{1105, 3401};
            default -> new int[]{1106, 1403, 2305, 2604, 3105, 1106};
        };
        JsonArray cases = new JsonArray();
        for (int spell : ids) {
            int gate = cases.size() == 5 ? 0 : 1;
            ActionOracleHarness h = new ActionOracleHarness(currentProgram);
            try {
                int offset = (spell / 1000 - 1) * 2842 + (spell / 100 % 10 - 1) * 406 + (spell % 100 - 1) * 58;
                h.write(0x44374e, java.util.Arrays.copyOfRange(source, offset, offset + 58));
                ByteBuffer party = ByteBuffer.allocate(6 * 3138), map = ByteBuffer.allocate(16200), positions = ByteBuffer.allocate(24);
                for (JsonElement warrior : foundation.getAsJsonArray("warriors"))
                    for (JsonElement value : warrior.getAsJsonObject().getAsJsonArray("bytes")) party.put(value.getAsByte());
                int[][] fields = {
                    {0x58, 30000}, {0x50, 30000}, {0x40, 5}, {0x3c, 5},
                    {0xac6, 100}, {0xac8, 0}, {0x14, 0}, {0x36, spell / 1000}
                };
                for (int[] field : fields)
                    party.putShort(3138 + field[0], (short)field[1]);
                java.util.Arrays.fill(party.array(), 3138 + 0x9f0, 3138 + 0xa21, (byte)1);
                for (JsonElement value : combat.getAsJsonObject("map").getAsJsonArray("tiles")) map.putShort(value.getAsShort());
                for (JsonElement point : combat.getAsJsonObject("squad").getAsJsonArray("positions"))
                    for (JsonElement value : point.getAsJsonArray()) positions.putShort(value.getAsShort());
                if (drain || charm) {
                    ByteBuffer records = ByteBuffer.allocate(combat.getAsJsonObject("enemies").getAsJsonArray("records").size() * 288);
                    for (JsonElement monster : combat.getAsJsonObject("enemies").getAsJsonArray("records"))
                        for (JsonElement value : monster.getAsJsonObject().getAsJsonArray("bytes")) records.put(value.getAsByte());
                    java.util.Arrays.fill(records.array(), 288 + 0xa, 288 + 0xc, (byte)0);
                    java.util.Arrays.fill(records.array(), 288 + 0xc2, 288 + 0xc9, (byte)0);
                    java.util.Arrays.fill(records.array(), 288 + 0x8c, 288 + 0x8e, (byte)0);
                    records.put(288 + 0x90, (byte)0);
                    records.putShort(288 + 0x10e, (short)1000);
                    h.write(0x439d3c, records.array());
                    h.value(0x4434b8, 2, drain ? 13 : 0);
                    h.value(0x4434e0 + 22, 2, 1);
                }
                h.write(0x448aa0, party.array());
                h.write(0x4085b8, map.array());
                h.write(0x44709a, positions.array());
                h.write(0x43788a, new byte[480]);
                h.value(0x442780, 4, 26);
                h.value(0x442784, 4, -13);
                h.value(0x443722, 2, 64);
                h.value(0x443724, 2, 37);
                h.value(0x437a70, 2, combat.getAsJsonObject("squad").getAsJsonArray("under_tiles").get(1).getAsShort());
                h.value(0x438786, 2, gate);
                h.value(0x438776, 2, outdoor || encounter ? 0 : 1);
                h.value(0x438778, 2, 0);
                h.value(0x44273f, 1, 1);
                h.value(0x4435d2, 2, 1);
                h.value(0x447194, 2, 1);
                h.value(0x4470ba, 2, 1);
                h.value(0x4435c6, 2, spell / 1000 - 1);
                h.value(0x4435c8, 2, spell / 100 % 10 - 1);
                h.value(0x4435ca, 2, spell % 100 - 1);
                h.value(0x4435cc, 2, 1);
                h.enter(0x10d2b4, 0x6ffe00, 0x900000);
                h.cpu.writeRegister("A6", 0x700000);
                h.value(0x44273a, 1, 5);
                h.value(0x44257c, 2, 0);
                if (encounter) {
                    byte[] records = Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data ED2"));
                    h.write(0x443396, java.util.Arrays.copyOfRange(records, 10 * 516, 10 * 516 + 156));
                    h.value(0x44263e, 2, -1);
                    h.cpu.writeRegister("D5", spell);
                }
                JsonArray sounds = new JsonArray(), warnings = new JsonArray();
                int draws = 0;
                long randomSeed = 12;
                for (int instructions = 0; h.pc() != (encounter ? 0x122adc : outdoor ? 0x10ae30 : 0x11af72); instructions++) {
                    long pc = h.pc();
                    if (instructions == 10000) throw new Exception("Movement instruction limit");
                    if (pc == 0x10d3c2) h.jump(encounter ? 0x1228cc : outdoor ? 0x10aca0 : 0x11af0a);
                    else if (pc == 0x10adbe) {
                        h.pop(4);
                        h.jump(pc + 2);
                    }
                    else if (pc == 0x10d4fe && drain) {
                        h.value(h.sp() + 4, 2, 11);
                        h.value(h.sp() + 6, 2, h.word(0x443776));
                        h.cpu.writeRegister("A0", 0x439d3c + 288);
                        h.jump(0x150f7a);
                        if (!h.cpu.step(monitor)) throw new Exception("Original positive-damage marker failed");
                        h.jump(0x15b9d8);
                    } else if (pc == 0x10d4fe) h.jump(0x151022);
                    else if (charm && (pc == 0x150114 || pc == 0x15075a)) {
                        h.cpu.writeRegister("D0", 0);
                        h.ret();
                    }
                    else if (pc == 0x15847a) h.jump(0x15917c);
                    else if (pc == 0x133a18 || pc == 0x16474a) {
                        (pc == 0x133a18 ? sounds : warnings).add(h.word(h.sp() + 4));
                        h.ret();
                    } else if (encounter && (pc == 0x16043e || pc == 0x1600fc || pc == 0x15ec56 || pc == 0x13145c)) h.ret();
                    else if (pc == 0x13154c || pc == 0x15b2f6 || pc == 0x105572 || pc == 0x10e6b6 || pc == 0x1205f8 || pc == 0x1500d8) h.ret();
                    else if (pc == 0x400038) {
                        h.cpu.writeRegister("D0", Math.abs(h.word(h.sp() + 4)));
                        h.ret();
                    } else if (pc == 0x153cb2) {
                        if (++draws > (encounter ? 2 : 1)) throw new Exception("Unexpected Psi random service count");
                        randomSeed = randomSeed * 16807 % 2147483647;
                        h.value(h.sp(), 2, (short)randomSeed);
                        h.jump(pc + 2);
                    } else if (!h.cpu.step(monitor)) throw new Exception(h.cpu.getLastError());
                }
                if (h.sp() != (outdoor || encounter ? 0x6ffe04 : 0x6ffdfc)) throw new Exception("Psi caller stack mismatch");
                JsonObject receipt = new JsonObject();
                if (outdoor || encounter) {
                    receipt.addProperty("warriors_before", HexFormat.of().formatHex(party.array()));
                    receipt.addProperty("mask_after", HexFormat.of().formatHex(h.read(0x4434e0, 220)));
                }
                if (encounter) {
                    receipt.addProperty("scenario_duration_after", h.word(0x44263e));
                    receipt.addProperty("selector", h.cpu.readRegister("D4").byteValue());
                    receipt.addProperty("seed_after", randomSeed);
                }
                if (drain || charm) {
                    int count = combat.getAsJsonObject("enemies").getAsJsonArray("records").size();
                    receipt.addProperty("damage_before", drain ? 13 : 0);
                    receipt.addProperty("records_after", HexFormat.of().formatHex(h.read(0x439d3c, count * 288)));
                }
                receipt.addProperty("spell", spell);
                receipt.addProperty("gate", gate);
                receipt.addProperty("draws", draws);
                receipt.addProperty("warriors_after", HexFormat.of().formatHex(h.read(0x448aa0, 6 * 3138)));
                receipt.addProperty("map_after", HexFormat.of().formatHex(h.read(0x4085b8, 16200)));
                receipt.addProperty("positions_after", HexFormat.of().formatHex(h.read(0x44709a, 24)));
                receipt.addProperty("memory_after", HexFormat.of().formatHex(h.read(0x43788a, 480)));
                receipt.addProperty("gate_after", h.word(0x438786));
                receipt.add("sounds", sounds);
                receipt.add("warnings", warnings);
                cases.add(receipt);
            } finally {
                h.cpu.dispose();
            }
        }
        JsonObject result = new JsonObject();
        result.addProperty("authority", "Original charge10d2b4..10d3c2/AP caller11af0a..11af72, resolvespell151022 and bodyfield. Original showresults prologue executes its gate clear; remaining presentation, location logging and sound services acknowledged. Cast entry jumps to resolvespell with an accepted target. QuickDraw Random supplies one signed WORD from seed12->201684. No original selection, whole animation/audio or live claim.");
        if (drain) result.addProperty("authority", "Original charge10d2b4..10d3c2/AP caller11af0a..11af72 then original positive-damage marker150f7a and spelllist15b9d8. Controlled damage13 is a source-command observation, not native damage/RNG proof. Accepted target11 and prepared damage bypass original targeting/resistance/damage/duration RNG/animation. Complete party and NPC outputs are original instructions; no live/full-cast claim.");
        if (charm) result.addProperty("authority", "Original charge10d2b4..10d3c2/AP caller11af0a..11af72 and resolvespell151022/spelllist15b9d8 with accepted mask target11/effect51. Resistance and damage services return0. QuickDraw Random supplies one duration WORD; original showresults gate clear executes, remaining drawing/delay/audio acknowledged. Complete party/NPC/gate outputs are original instructions; no original targeting/resistance/damage/full-animation/live claim.");
        if (outdoor) result.addProperty("authority", "Original accepted charge10d2b4, outdoor type11 caller10aca0..10ae30, resolvespell/resist/resolvedamage/spelllist. Six supplied party records, no saved allies; original mask, charge, learned memory, RNG consumption and ordered sound requests. QuickDraw Random supplies one signed duration WORD for seed12; QuickDraw port, resettextarea, animation and result drawing acknowledged after the original gate clear. Original UI selection/cancellation/delay/refresh and live acceptance excluded.");
        if (encounter) result.addProperty("authority", "Original charge10d2b4 and encounter2 caller1228cc..122adc, cleartarget, resolvespell and original selector matching. Native ED2 record10; scenario-duration WORD44263e starts-1. QuickDraw Random uses the retained seed12 Park-Miller service model. updatecontrols/updatecharshort/tickcheck drawing and unchanged selected loadspell2 packet acknowledged. Original chooser/admission, elapsed periodic ticks, continuation presentation and live acceptance excluded.");
        for (String name : new String[]{"VerifyMovementPsi.java", "ActionOracleHarness.java"})
            result.addProperty(name + "_sha256", ActionOracleHarness.sha256(root.resolve("tools/ghidra/" + name)));
        result.addProperty("code_sha256", ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin")));
        result.addProperty("input_sha256", ActionOracleHarness.sha256(input));
        result.addProperty("spells_sha256", ActionOracleHarness.sha256(spells));
        result.add("cases", cases);
        Files.writeString(output, result + "\n");
    }
}
