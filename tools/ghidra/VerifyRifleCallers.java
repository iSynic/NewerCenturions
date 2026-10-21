import com.google.gson.*;
import java.nio.file.*;
import java.nio.ByteBuffer;
import java.util.*;
/** Original scientist decision caller with authored rifle allies and controlled pre-entry state. */
public class VerifyRifleCallers extends VerifyCombatCallerBoundary {
    static byte[] bytes(JsonObject value, String key) {
        return HexFormat.of().parseHex(value.get(key).getAsString());
    }
    static void put(byte[] data, int offset, int value) {
        ByteBuffer.wrap(data).putShort(offset, (short)value);
    }
    @Override public void run() throws Exception {
        Path root = Path.of(getScriptArgs()[0]);
        Path output = root.resolve(getScriptArgs()[1]);
        if (Files.exists(output)) throw new Exception("Capture output already exists");
        JsonObject foundation = JsonParser.parseString(Files.readString(root.resolve("evidence/fixtures/ally-turns.json"))).getAsJsonObject();
        Path data = root.resolve("NewCenturions/TheNewCenturions/Data Files");
        Path scenario = root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0");
        byte[] definitions = Files.readAllBytes(scenario.resolve("Data MD"));
        byte[] items = Files.readAllBytes(data.resolve("Data ID"));
        byte[] properties = Files.readAllBytes(data.resolve("Data Swamp BD"));
        boolean full = getScriptArgs().length > 2;
        boolean turn = full && getScriptArgs()[2].equals("turn");
        JsonArray cases = new JsonArray();
        for (int[] recipe : new int[][] {
            {64, 0, 4, 0},
            {64, 1, 4, 0},
            {98, 0, 4, 0},
            {98, 1, 4, 0},
            {35, 0, 4, 0},
            {35, 1, 4, 0},
            {98, 1, 12, 0},
            {35, 1, 12, 1},
            {64, 1, 4, 2},
            {35, 1, 12, 3},
            {35, 1, 12, 4},
            {35, 1, 12, 5},
            {35, 1, 12, 6},
            {35, 1, 12, 7},
            {35, 1, 12, 8},
        }) {
            int id = recipe[0], context = recipe[1], distance = recipe[2], mode = recipe[3];
            JsonObject c = foundation.getAsJsonArray("turn_cases").get(distance == 4 ? 1 : 3).getAsJsonObject().deepCopy();
            byte[] party = bytes(c, "warriors_before"), records = bytes(c, "records_before"), map = bytes(c, "map_before");
            System.arraycopy(definitions, id * 288, records, 0, 288);
            put(records, 0xe, 0);
            put(records, 0x110, 2);
            put(records, 0x112, 2);
            put(records, 0x10c, -1);
            put(records, 0x11e, 1155);
            records[0x106] = 110;
            records[0x109] = 3;
            if (mode == 1 || mode >= 3) for (int dx = -1; dx <= 1; dx++) for (int dy = -1; dy <= 1; dy++) if (dx != 0 || dy != 0) put(map, ((25 + dx) * 90 + 25 + dy) * 2, mode == 1 ? 1062 : 1000);
            if (mode >= 4 && mode <= 7) { put(map, (24 * 90 + 25) * 2, 1062); }
            if (mode == 5 || mode == 7) put(records, 0x10c, 108);
            if (mode == 2) { records[0xb2] = 1; records[0x106] = 0; }
            c.addProperty("mode", mode);
            ActionOracleHarness h = new ActionOracleHarness(currentProgram);
            try {
                h.enter(0x1145c8, 0x6ffe00, 0x6ff000);
                h.cpu.writeRegister("A6", 0x700000);
                h.write(0x448aa0, new byte[12 * 3138]);
                h.write(0x448aa0, party);
                h.value(0x44273a, 1, 5);
                h.write(0x439d3c, records);
                h.write(0x4085b8, map);
                h.value(0x43874a, 2, 2);
                for (int cache = 0; cache < 4; cache++) h.write(new long[]{0x4537fe, 0x46225e, 0x45ad2e, 0x46978e}[cache], Arrays.copyOfRange(items, cache * 30000, (cache + 1) * 30000));
                h.write(0x470cbe, Files.readAllBytes(scenario.resolve("Data NI")));
                h.write(0x443788, Files.readAllBytes(data.resolve("Data S")));
                h.write(0x438210, Files.readAllBytes(data.resolve("Data AD")));
                h.write(0x43167c, Arrays.copyOfRange(properties, 64, 12064));
                h.value(0x442780, 4, 5);
                h.value(0x442784, 4, 5);
                h.value(0x438776, 2, 1);
                h.value(0x447196, 2, 0);
                h.value(0x447194, 2, 1);
                if (turn) {
                    for (int i = 0; i < 110; i++) h.value(0x4470b8 + i * 2, 2, -1);
                    h.value(0x4470bc, 2, 0);
                    byte[] races = Files.readAllBytes(data.resolve("Data Race"));
                    for (int i = 0; i < 10; i++) h.value(0x435664 + i, 1, races[i * 408 + 197]);
                }
                h.value(0x4470ba, 2, 10);
                h.value(0x4435c4, 2, 6);
                h.value(0x44374c, 2, 0);
                h.value(0x402f43, 1, 2);
                h.value(0x43550c, 2, 155);
                h.value(0x439568, 2, 0);
                h.write(0x442bd6, new byte[1200]);
                h.value(0x43565f, 1, 0);
                h.value(0x447d96, 2, 0);
                h.value(0x4435cc, 2, 0);
                h.value(0x45365c, 2, context);
                byte[] positions = ByteBuffer.allocate(8).putShort((short)20).putShort((short)20).putShort((short)(20 + distance)).putShort((short)20).array();
                h.write(0x446f0a, positions);
                if (mode == 3) {
                    h.write(0x447096, ByteBuffer.allocate(4).putShort((short)40).putShort((short)20).array());
                    c.add("retained_slot99", JsonParser.parseString("[40,20]"));
                }
                if (mode >= 4 && mode <= 7) {
                    byte[] retained = Arrays.copyOfRange(records, 288, 576);
                    if (mode >= 6) retained[0x82] = 1;
                    h.write(0x439d3c + 98 * 288, retained);
                    h.write(0x446f0a + 98 * 4, ByteBuffer.allocate(4).putShort((short)19).putShort((short)20).array());
                    JsonObject target = new JsonObject();
                    target.addProperty("record", HexFormat.of().formatHex(retained));
                    target.add("position", JsonParser.parseString("[19,20]"));
                    c.add("retained_target", target);
                }
                c.addProperty("positions_before", HexFormat.of().formatHex(positions));
                for (int i = 0; i < 6; i++) {
                    h.value(0x44709a + i * 4, 2, 40 + i);
                    h.value(0x44709c + i * 4, 2, 40);
                }
                seed = 12;
                draws = 0;
                JsonArray sounds = new JsonArray(), rng = new JsonArray();
                int steps = executeCombat(h, turn ? 0x115788 : full ? 0x115776 : 0x115732, full, turn, sounds, rng);
                if (mode >= 4 && mode <= 7) c.getAsJsonObject("retained_target").addProperty("after", HexFormat.of().formatHex(h.read(0x439d3c + 98 * 288, 288)));
                c.addProperty("packet_after", HexFormat.of().formatHex(h.read(0x44374e, 58)));
                c.addProperty("count_after", h.word(0x4435cc));
                c.addProperty("cursor_after", h.word(0x447194));
                c.addProperty("selected_after", h.word(0x4435d2));
                c.addProperty("item_after", HexFormat.of().formatHex(h.read(0x45363c, 150)));
                c.addProperty("weapon_after", HexFormat.of().formatHex(h.read(0x453768, 150)));
                c.addProperty("ammo_after", HexFormat.of().formatHex(h.read(0x4536d2, 150)));
                c.addProperty("monster_id", id);
                c.addProperty("item_context", context);
                c.addProperty("map_before", HexFormat.of().formatHex(map));
                c.addProperty("records_before", HexFormat.of().formatHex(records));
                c.addProperty("records_after", HexFormat.of().formatHex(h.read(0x439d3c, records.length)));
                c.addProperty("warriors_after", HexFormat.of().formatHex(h.read(0x448aa0, party.length)));
                c.addProperty("map_after", HexFormat.of().formatHex(h.read(0x4085b8, map.length)));
                c.addProperty("positions", HexFormat.of().formatHex(h.read(0x446f0a, 8)));
                c.addProperty("seed_after", seed);
                c.addProperty("rng_draws", draws);
                c.addProperty("steps", steps);
                c.add("sounds", sounds);
                c.add("rng", rng);
                c.remove("draws");
                cases.add(c);
            } finally { h.cpu.dispose(); }
        }
        JsonObject result = new JsonObject();
        result.addProperty("stop", turn ? "115788" : full ? "115776" : "115732");
        result.addProperty("authority", "Original combat1145c8..115732 scientist decision caller, authored MD64/98 and converted MD35 faction0, HP2, AP3, MOV110 (fleeing control MOV0), target49 HP100 distance4/12, seed12; mode1 surrounds the actor with original terrain62 (movement blocker2, sight blocker0), mode2 sets original fleeing flag, mode3 uses opaque terrain0 and retained NPC99 guest coordinates40,20; modes4/5 retain hostile NPC98 at19,20 behind a transparent movement barrier (mode5 begins with target body108); modes6/7 set the scan-exclusion byte while distinguishing fresh scan from cached reuse, retained item WORD32 zero/nonzero. Original targeting, weapon1/ammunition preparation, firing and movement execute. Original showresults code0/-1 audio158720..15886e/1588fc and159042..15904c executes; display services including combatupdate2 item-work writes are omitted; Rand and SANE use established boundaries. The stop field distinguishes the historical pre-retry slice from the final retry branch. In full mode combatupdate2 final selection11faa0 through original loaditem executes in the same memory; drawing is omitted. Turn mode also executes Getup125a3a..1270b6 to the following actor, with light/drawing/inwindow and OS FlushEvents delegated. No round-wrap or live UI proof.");
        result.addProperty("foundation_sha256", ActionOracleHarness.sha256(root.resolve("evidence/fixtures/ally-turns.json")));
        result.add("cases", cases);
        receipt(root, output, result);
        println("Original rifle callers captured: " + cases.size());
    }
}
