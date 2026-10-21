import com.google.gson.*;
import java.nio.file.*;
import java.util.HexFormat;

/** Original heal resolution on an assisted battle141 input, excluding presentation services. */
public class VerifyNpcHealing extends VerifyCombatCallerBoundary {
    byte[] records(JsonArray rows) {
        java.io.ByteArrayOutputStream out = new java.io.ByteArrayOutputStream();
        for (JsonElement row : rows) for (JsonElement value : row.getAsJsonObject().getAsJsonArray("bytes")) out.write(value.getAsInt());
        return out.toByteArray();
    }
    @Override public void run() throws Exception {
        Path root = Path.of(getScriptArgs()[0]);
        Path input = root.resolve(getScriptArgs()[1]);
        JsonObject session = JsonParser.parseString(Files.readString(input)).getAsJsonObject();
        JsonObject combat = session.getAsJsonObject("combat");
        byte[] party = records(session.getAsJsonArray("warriors"));
        boolean overflow = getScriptArgs().length > 3 && getScriptArgs()[3].equals("overflow");
        if (overflow) {
            for (int offset : new int[]{0x50, 0x52}) { party[offset] = 127; party[offset + 1] = (byte)255; }
            party[0x9a] = party[0x9b] = 0;
        }
        byte[] npcs = records(combat.getAsJsonObject("enemies").getAsJsonArray("records"));
        boolean portrait = getScriptArgs().length > 3 && getScriptArgs()[3].startsWith("portrait");
        int spell = portrait ? Integer.parseInt(getScriptArgs()[3].substring(8)) : 1101;
        int packetOffset = ((spell / 1000 - 1) * 49 + (spell % 1000 / 100 - 1) * 7 + spell % 100 - 1) * 58;
        byte[] packet = java.util.Arrays.copyOfRange(Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data S")), packetOffset, packetOffset + 58);
        ActionOracleHarness h = new ActionOracleHarness(currentProgram);
        JsonObject out = new JsonObject();
        try {
            h.enter(0x151022, 0x6ffe00, 0x6ff000);
            h.write(0x448aa0, party);
            h.write(0x439d3c, npcs);
            h.write(0x44374e, packet);
            h.write(0x4434e0, new byte[220]);
            h.write(0x435884, new byte[128]);
            h.value(0x4434e0 + (portrait ? 1 : overflow ? 0 : 11) * 2, 2, 1);
            h.value(0x4435cc, 2, portrait ? 1 : 2);
            h.value(0x438776, 2, 1);
            h.value(0x438778, 2, 1);
            h.value(0x435662, 1, 0);
            h.value(0x4358c2, 1, 0);
            h.value(0x4435d2, 2, portrait ? 5 : 10);
            h.value(0x447194, 2, 0);
            h.value(0x4470b8, 2, 10);
            h.value(0x45375a, 2, 0);
            h.value(0x4537f0, 2, 0);
            h.value(0x453776, 2, 0);
            long seed = 12;
            int draws = 0, steps = 0;
            JsonArray sounds = new JsonArray();
            JsonArray kills = new JsonArray();
            while (h.pc() != 0x6ff000) {
                if (++steps > 20000) throw new Exception("NPC heal instruction budget at " + Long.toHexString(h.pc()));
                long pc = h.pc();
                if (overflow && pc == 0x12e4e4) { kills.add(h.word(h.sp() + 4)); h.ret(); }
                else if (pc == 0x13a498) h.ret();
                else if (pc == 0x400038) { h.cpu.writeRegister("D0", Math.abs(h.word(h.sp() + 4))); h.ret(); }
                else if (pc == 0x133a18) { sounds.add(h.word(h.sp() + 4)); h.ret(); }
                else if (pc == 0x13154c || pc == 0x1107cc || pc == 0x15b2f6 || pc == 0x158454 || pc == 0x1205f8 || pc == 0x16043e || pc == 0x11f1a8 || pc == 0x159192 || pc == 0x11edc4) h.ret();
                else if ((h.word(pc) & 65535) == 0xa861) {
                    seed = seed * 16807 % 2147483647;
                    int value = (short) seed;
                    h.value(h.sp(), 2, value == -32768 ? 0 : value);
                    draws++;
                    h.jump(pc + 2);
                } else if ((h.word(pc) & 65535) == 0xa85d) {
                    int bit = (int) h.pointer(h.sp());
                    int value = h.read(h.pointer(h.sp() + 4) + (bit >> 3), 1)[0] & (128 >> (bit & 7));
                    h.value(h.sp() + 8, 1, value == 0 ? 0 : 1);
                    h.pop(8);
                    h.jump(pc + 2);
                } else if (!h.cpu.step(monitor)) throw new Exception("NPC heal " + Long.toHexString(pc) + ": " + h.cpu.getLastError());
            }
            if (portrait) {
                out.addProperty("party_after_resolution", HexFormat.of().formatHex(h.read(0x448aa0, party.length)));
                out.addProperty("draws_before_return", draws);
                this.seed = seed;
                this.draws = draws;
                String[] fields = {"map", "order", "party_positions", "npc_positions"};
                long[] addresses = {0x4085b8, 0x4470b8, 0x44709a, 0x446f0a};
                JsonArray[] values = {combat.getAsJsonObject("map").getAsJsonArray("tiles"), combat.getAsJsonArray("order"), combat.getAsJsonObject("squad").getAsJsonArray("positions"), combat.getAsJsonObject("enemies").getAsJsonArray("positions")};
                for (int field = 0; field < fields.length; field++) {
                    java.nio.ByteBuffer buffer = java.nio.ByteBuffer.allocate(values[field].size() * (field < 2 ? 2 : 4));
                    for (JsonElement value : values[field]) {
                        if (field < 2) buffer.putShort(value.getAsShort());
                        else for (JsonElement coordinate : value.getAsJsonArray()) buffer.putShort(coordinate.getAsShort());
                    }
                    h.write(addresses[field], buffer.array());
                }
                byte[] definitions = Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data ID"));
                for (int cache = 0; cache < 4; cache++) h.write(new long[]{0x4537fe,0x46225e,0x45ad2e,0x46978e}[cache], java.util.Arrays.copyOfRange(definitions, cache * 30000, (cache + 1) * 30000));
                byte[] properties = Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data Swamp BD"));
                h.write(0x43167c, java.util.Arrays.copyOfRange(properties, 64, 12064));
                byte[] races = Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data Race"));
                for (int race = 0; race < 10; race++) h.value(0x435664 + race, 1, races[race * 408 + 197]);
                h.value(0x43874a, 2, npcs.length / 288);
                h.value(0x4435c4, 2, combat.getAsJsonObject("enemies").getAsJsonArray("records").size());
                h.value(0x44273a, 1, 5);
                h.value(0x447194, 2, combat.get("cursor").getAsInt());
                h.value(0x44374c, 2, 0);
                h.value(0x442780, 4, combat.getAsJsonObject("map").getAsJsonArray("shift").get(0).getAsInt() + 5);
                h.value(0x442784, 4, combat.getAsJsonObject("map").getAsJsonArray("shift").get(1).getAsInt() + 5);
                h.cpu.writeRegister("SP", 0x6ffe00);
                h.cpu.writeRegister("A6", 0x700000);
                h.cpu.writeRegister("D3", 1);
                h.write(0x6ffff6, java.nio.ByteBuffer.allocate(8).putShort((short)7).putShort((short)7).putShort((short)83).putShort((short)83).array());
                h.jump(0x119eb0);
                executeCombat(h, 0x11ce76, true, true, sounds, new JsonArray());
                seed = this.seed;
                draws = this.draws;
                out.addProperty("positions_after", HexFormat.of().formatHex(h.read(0x44709a, 24)));
                out.addProperty("map_after", HexFormat.of().formatHex(h.read(0x4085b8, 16200)));
            }
            out.addProperty("party_after", HexFormat.of().formatHex(h.read(0x448aa0, party.length)));
            out.addProperty("npcs_after", HexFormat.of().formatHex(h.read(0x439d3c, npcs.length)));
            out.addProperty("seed_after", seed);
            out.addProperty("rng_draws", draws);
            out.addProperty("steps", steps);
            out.add("sounds_before_showresults", sounds);
            if (overflow) out.add("killbody_arguments", kills);
        } finally { h.cpu.dispose(); }
        out.addProperty("authority", "Original resolvespell151022/resolvedamage on battle141 NPC11, Heal Mk I power2, seed12. QuickDraw Random uses the independently established seed kernel; BitTst follows the Toolbox bit-array convention. Original Rand arithmetic executes. Presentation, showresults feedback/gate, delays and redraws are excluded; captured sounds exclude showresults. This is not full NPC targeting or a whole AI turn.");
        out.addProperty("probe_sha256", ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyNpcHealing.java")));
        out.addProperty("binary_sha256", ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin")));
        out.addProperty("input_sha256", ActionOracleHarness.sha256(input));
        out.add("input", session);
        if (portrait) out.addProperty("portrait_authority", "Original resolvespell with authored spell" + spell + " power1, target1 from the captured paid-selection input, seed12 and retained armor filters0. Original portrait restoration119eb0..11ce76 and fullGetup execute after the resolver in the same memory. Eligibility119dd2..119e16 is captured separately. Drawing/music/device services are delegated; no whole-cast or live claim. Setup assistance precedes this input, not the measured action.");
        if (overflow) out.addProperty("overflow_authority", "Party target0 HP/max32767, nonreflecting, accepted mask and Heal Mk I power2; original resolvespell/resolvedamage/heal signed arithmetic and killbody argument execute. killbody cleanup and in synchronization are excluded; existing independent lifecycle receipts own cleanup. Normal targeting, costs, showresults/drawing and whole cast remain excluded. The input Session is the unmodified foundation; these declared record/mask overrides are applied before entry.");
        Files.writeString(Files.createFile(root.resolve(getScriptArgs()[2])), new GsonBuilder().setPrettyPrinting().create().toJson(out) + "\n");
        println("Original NPC heal resolution captured");
    }
}
