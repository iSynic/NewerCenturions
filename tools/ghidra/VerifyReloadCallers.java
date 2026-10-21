import com.google.gson.*;
import java.nio.file.*;
import java.nio.ByteBuffer;
import java.util.*;

/** Original occupied-cell preparation/reload branch with assisted actor and ammunition inputs. */
public class VerifyReloadCallers extends VerifyRetainedCallers {
    static final String[] KEYS = {
        "party", "reserves", "item_work", "weapon_work", "ammo_work", "selected_record",
        "packet", "identity", "order", "party_positions", "npc_positions", "npcs", "map",
        "party_buffs", "fields", "field_used", "flags", "memory", "references", "crate", "loot",
        "loot_quality", "markers", "mask", "targets", "item_cache0", "item_cache1",
        "item_cache2", "item_cache3", "scenario_cache"
    };
    static final long[] ADDRESSES = {
        0x448aa0, 0x42dbe4, 0x45363c, 0x453768, 0x4536d2, 0x41fc48,
        0x44374e, 0x4435c6, 0x4470b8, 0x44709a, 0x446f0a, 0x439d3c, 0x4085b8,
        0x442562, 0x442bd6, 0x435884, 0x44258c, 0x43788a, 0x440edc, 0x4285f4, 0x41fc74,
        0x42df04, 0x442798, 0x4434e0, 0x443722, 0x4537fe, 0x46225e,
        0x45ad2e, 0x46978e, 0x470cbe
    };
    byte[] decoded(JsonObject fixture, JsonElement value) {
        return HexFormat.of().parseHex((value.isJsonPrimitive() ? value : fixture.getAsJsonObject("shared_rule_buffers").get(value.getAsJsonObject().get("shared_buffer_sha256").getAsString())).getAsString());
    }
    JsonObject snapshot(ActionOracleHarness h, JsonObject template, int member) {
        JsonObject result = template.deepCopy();
        for (int i = 0; i < KEYS.length; i++) result.addProperty(KEYS[i], HexFormat.of().formatHex(h.read(ADDRESSES[i], decoded(template, template.get(KEYS[i])).length)));
        result.addProperty("member_record", HexFormat.of().formatHex(h.read(0x448aa0 + member * 3138, 3138)));
        result.addProperty("seed", seed);
        result.addProperty("draws", draws);
        result.addProperty("count", h.word(0x4435cc));
        return result;
    }
    @Override public void run() throws Exception {
        Path root = Path.of(getScriptArgs()[0]), output = root.resolve(getScriptArgs()[1]);
        if (Files.exists(output)) throw new Exception("Capture output exists");
        JsonObject fixture = JsonParser.parseString(Files.readString(root.resolve("evidence/fixtures/ammunition-management.json"))).getAsJsonObject();
        JsonObject template = fixture.getAsJsonArray("cases").get(50).getAsJsonObject().getAsJsonObject("before_command").deepCopy();
        for (String key : KEYS) template.addProperty(key, HexFormat.of().formatHex(decoded(fixture, template.get(key))));
        byte[] items = Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data ID"));
        JsonArray cases = new JsonArray();
        for (int[] recipe : new int[][] {
            {0, 100, 1, 265, 99},
            {5, 100, 1, 265, 0},
            {0, 100, 0, 265, 1},
            {5, 38, 0, 203, 3}
        }) {
            int member = recipe[0], weapon = recipe[1], loaded = recipe[2], ammo = recipe[3];
            seed = 12;
            draws = 0;
            JsonArray sounds = new JsonArray();
            ActionOracleHarness h = new ActionOracleHarness(currentProgram);
            try {
                h.enter(0x11c236, 0x6ffe00, 0x6ff000);
                h.cpu.writeRegister("A6", 0x700000);
                for (int i = 0; i < KEYS.length; i++) h.write(ADDRESSES[i], bytes(template, KEYS[i]));
                long actor = 0x448aa0 + member * 3138;
                h.value(actor + 0xe4, 2, weapon);
                h.value(actor + 0x9dd, 1, 1);
                h.value(actor + 0x9e0, 1, 0);
                h.value(actor + 0x40, 2, 5);
                h.value(actor + 0x90, 2, 0);
                h.write(actor + 0x132, h.read(0x448aa0 + 4 * 3138 + 0x132, 22));
                h.value(actor + 0x132, 2, weapon);
                h.value(actor + 0x134, 2, loaded);
                h.value(actor + 0x136, 2, ammo);
                int capacity = ByteBuffer.wrap(items).getShort(weapon * 150 + 90);
                if (capacity == 0) capacity = ByteBuffer.wrap(items).getShort(weapon * 150 + 24);
                h.value(actor + 0x138, 2, capacity);
                h.write(0x42dbe4, new byte[804]);
                h.value(0x42dbe4 + (ammo - 200) * 4, 4, recipe[4]);
                h.value(0x4435d2, 2, member);
                h.value(0x44273a, 1, 5);
                h.value(0x442750, 1, 0);
                h.value(0x438776, 2, 1);
                h.value(0x442780, 4, 19);
                h.value(0x442784, 4, 19);
                h.value(0x4470b8, 2, member);
                h.value(0x447194, 2, 0);
                h.value(0x439d3c + 0x110, 2, 100);
                h.value(0x439d3c + 0xb3, 1, 1);
                byte[] previous = h.read(0x44709a + member * 4, 4);
                h.write(0x44709a + member * 4, h.read(0x44709a + 16, 4));
                h.write(0x44709a + 16, previous);
                for (int x = 0; x < 90; x++) for (int y = 0; y < 90; y++) {
                    long address = 0x4085b8 + (x * 90 + y) * 2;
                    int tile = h.word(address);
                    if (tile == member) h.value(address, 2, 4);
                    else if (tile == 4) h.value(address, 2, member);
                }
                h.write(0x443788, Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data S")));
                JsonObject c = new JsonObject();
                c.addProperty("kind", "move_reload");
                c.addProperty("member", member);
                c.addProperty("weapon", weapon);
                c.add("before_command", snapshot(h, template, member));
                execute(h, 0x11c28e, sounds);
                c.add("after", snapshot(h, template, member));
                c.add("feedback_arguments", sounds);
                cases.add(c);
            } finally { h.cpu.dispose(); }
        }
        JsonObject result = new JsonObject();
        result.addProperty("authority", "Original combatchoice11c236..11c28e including both prepattack calls and doreload/refit, assisted actors0/5, burst weapon100 partial/empty loaded counts and reserves full/partial/zero, plus weapon38 actor5. Original party/definition/cache frame; facing and inactive adjacent NPC established before the measured branch. Reactions and mouse event loop are outside this capture. Location log and warning presentation delegated; warning arguments retained. No output or RNG assistance during action.");
        result.add("cases", cases);
        receipt(root, output, result);
        println("Original occupied reload callers captured: " + cases.size());
    }
}
