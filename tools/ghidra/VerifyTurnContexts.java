import com.google.gson.*;
import java.nio.file.*;
import java.nio.ByteBuffer;
import java.util.HexFormat;

/** Original Getup condition/faction combinations with a following actor available for a skip. */
public class VerifyTurnContexts extends VerifyRetainedCallers {
    @Override public void run() throws Exception {
        Path root = Path.of(getScriptArgs()[0]), output = root.resolve(getScriptArgs()[1]);
        if (Files.exists(output)) throw new Exception("Capture output exists");
        JsonObject base = JsonParser.parseString(Files.readString(root.resolve("evidence/fixtures/turn.json"))).getAsJsonObject().getAsJsonArray("cases").get(0).getAsJsonObject();
        JsonArray cases = new JsonArray();
        for (int[] condition : new int[][] {
            {1, 0, 0, 0},
            {-1, 0, 0, 0},
            {0, 1, 0, 0},
            {0, 1, -1, 0},
            {0, 1, 0, 1},
            {0, 1, 0, 0},
        }) {
            byte[] party = bytes(base,"warriors"), enemy = bytes(base,"enemy");
            ByteBuffer b = ByteBuffer.wrap(party);
            b.putShort(0x5e,(short)condition[0]).putShort(0x98,(short)condition[1]);
            b.putShort(0x90,(short)condition[2]).putShort(0x10,(short)condition[3]);
            b.putShort(3138 + 0x40,(short)0);
            seed = cases.size() == 5 ? 4 : 1;
            int initialSeed = (int)seed;
            draws = 0;
            JsonArray sounds = new JsonArray();
            JsonObject c = new JsonObject();
            ActionOracleHarness h = new ActionOracleHarness(currentProgram);
            try {
                h.enter(0x125ca0,0x700000,0x6ff000);
                h.cpu.writeRegister("A6",0x701000);
                h.write(0x700e00,new byte[1024]);
                h.write(0x448aa0,party);
                h.write(0x439d3c,enemy);
                h.value(0x44273a,1,5);
                h.value(0x4435c4,2,6);
                h.value(0x44374c,2,0);
                h.write(0x4470b8,new byte[220]);
                for (int i = 0; i < 110; i++) h.value(0x4470b8 + i * 2,2,-1);
                h.value(0x447194,2,1);
                h.value(0x4470ba,2,1);
                h.value(0x4470bc,2,0);
                h.value(0x4470be,2,2);
                h.value(0x43800c,2,0);
                h.value(0x447196,2,0);
                h.value(0x4435d2,2,1);
                h.write(0x435664,new byte[]{12,14,12,14,10,12,10,10,14,16,0,12});
                execute(h,0x127090,sounds);
                c.addProperty("after_warriors",HexFormat.of().formatHex(h.read(0x448aa0,party.length)));
                c.addProperty("after_enemy",HexFormat.of().formatHex(h.read(0x439d3c,enemy.length)));
                c.addProperty("cursor",h.word(0x447194));
                c.addProperty("unavailable",h.read(0x442739,1)[0]);
                c.addProperty("selected_warrior",h.word(0x4435d2));
            } finally { h.cpu.dispose(); }
            c.addProperty("warriors",HexFormat.of().formatHex(party));
            c.addProperty("enemy",HexFormat.of().formatHex(enemy));
            c.addProperty("seed_before",initialSeed);
            c.addProperty("seed",seed);
            c.addProperty("draws",draws);
            c.add("sounds",sounds);
            cases.add(c);
        }
        JsonObject receipt = new JsonObject();
        receipt.addProperty("authority","Original Getup125ca0..127090, signed fear, confusion with ordinary/permanent automatic marker and friendly/hostile faction, seeds1/4. Supplied original party/MD49 and ordered following actors0/2. Native condition decisions, marker/faction/AP/movement and cursor selection execute; light/drawing/camera/inwindow are presentation delegates, finite SANE/Rand modeled. No battle setup/round-wrap or live UI claim.");
        receipt.add("cases",cases);
        receipt(root, output, receipt);
        println("Original Getup condition callers captured: " + cases.size());
    }
}
