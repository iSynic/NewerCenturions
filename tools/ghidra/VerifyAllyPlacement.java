import ghidra.app.script.GhidraScript;
import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.HexFormat;

/** Original combatsetup caller for Thunder's three recruited large-body definitions. */
public class VerifyAllyPlacement extends GhidraScript {
    public void run() throws Exception {
        Path root = Path.of(getScriptArgs()[0]), output = Path.of(getScriptArgs()[1]);
        if (Files.exists(output)) throw new Exception("Native output already exists: " + output);
        Path monsters = root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data MD");
        Path properties = root.resolve("NewCenturions/TheNewCenturions/Data Files/Data Swamp BD");
        Path foundation = root.resolve("evidence/fixtures/formation-opening.json");
        byte[] definitions = Files.readAllBytes(monsters), terrain = Files.readAllBytes(properties);
        byte[] initialMap = HexFormat.of().parseHex(JsonParser.parseString(Files.readString(foundation))
            .getAsJsonObject().getAsJsonArray("cases").get(0).getAsJsonObject().get("tiles").getAsString());
        JsonArray cases = new JsonArray();
        for (int[] ids : new int[][]{{99}, {100}, {101}, {99, 99, 99, 100, 100, 100, 101}}) {
            ActionOracleHarness h = new ActionOracleHarness(currentProgram);
            try {
                byte[] menu = new byte[20 * 288];
                for (int i = 0; i < ids.length; i++) {
                    System.arraycopy(definitions, ids[i] * 288, menu, i * 288, 288);
                    ByteBuffer.wrap(menu).putShort(i * 288 + 0x110, (short)(i + 2));
                    menu[i * 288 + 0x82] = 7;
                    menu[i * 288 + 0x104] = 9;
                }
                h.write(0x440edc, menu);
                h.value(0x44257c, 2, ids.length);
                h.value(0x408578, 2, 0);
                h.value(0x43874a, 2, 0);
                h.write(0x439d3c, new byte[100 * 288]);
                h.write(0x446f0a, new byte[400]);
                h.write(0x4085b8, initialMap);
                h.write(0x43167c, Arrays.copyOfRange(terrain, 64, 12064));
                h.value(0x442780, 4, -34);
                h.value(0x442784, 4, -10);
                h.value(0x44709a, 2, 39);
                h.value(0x44709c, 2, 39);
                h.enter(0x11ddb4, 0x6ffe00, 0x900000);
                h.cpu.writeRegister("A6", 0x700000);
                for (int i = 0; i < 4; i++) h.value(0x6ffff8 + 2 * i, 2, i < 2 ? 5 : 85);
                for (int step = 0; h.pc() != 0x11e0fe; step++) {
                    if (step == 1000000) throw new Exception("Ally placement instruction limit");
                    long pc = h.pc(), sp = h.sp();
                    if (pc == 0x153ca4) throw new Exception("Ally placement consumed RNG");
                    if (pc == 0x400038) {
                        h.cpu.writeRegister("D0", Math.abs(h.word(sp + 4)));
                        h.ret();
                    } else if ((h.word(pc) & 65535) == 0xa8ad) {
                        long rect = h.pointer(sp);
                        int y = h.word(sp + 4), x = h.word(sp + 6);
                        h.value(sp + 8, 1, y >= h.word(rect) && x >= h.word(rect + 2)
                            && y < h.word(rect + 4) && x < h.word(rect + 6) ? 1 : 0);
                        h.pop(8);
                        h.jump(pc + 2);
                    } else if (!h.cpu.step(monitor)) throw new Exception(h.cpu.getLastError());
                }
                JsonObject receipt = new JsonObject();
                receipt.addProperty("kind", "placement");
                receipt.addProperty("count", ids.length);
                receipt.addProperty("menu_before", HexFormat.of().formatHex(menu));
                receipt.addProperty("map_before", HexFormat.of().formatHex(initialMap));
                receipt.addProperty("records_after", HexFormat.of().formatHex(h.read(0x439d3c, ids.length * 288)));
                receipt.addProperty("map_after", HexFormat.of().formatHex(h.read(0x4085b8, 16200)));
                receipt.addProperty("positions", HexFormat.of().formatHex(h.read(0x446f0a, ids.length * 4)));
                receipt.addProperty("enemy_count", h.word(0x43874a));
                cases.add(receipt);
            } finally { h.cpu.dispose(); }
        }
        JsonObject result = new JsonObject();
        result.addProperty("authority", "Original combatsetup11ddb4..11e0fe, placemonster and bodyfield. Original MD99/100/101 inputs, controlled living HP and stale status bytes; independently captured opening map and exact Swamp properties. PtInRect/abs services modeled. No random service accepted. Recruitment/initiative/full UI/live are separate evidence.");
        for (Path input : new Path[]{monsters, properties, foundation, root.resolve("tools/ghidra/VerifyAllyPlacement.java"), root.resolve("tools/ghidra/ActionOracleHarness.java"), root.resolve("analysis/ghidra/startup/code1-mapped.bin")})
            result.addProperty(input.getFileName() + "_sha256", ActionOracleHarness.sha256(input));
        result.add("cases", cases);
        Files.writeString(output, result + "\n");
    }
}
