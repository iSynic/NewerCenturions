import ghidra.app.script.GhidraScript;
import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.HexFormat;

/** Bounded original attack branch; only presentation and QuickDraw Random are serviced. */
public class VerifyMechanicalCapture extends GhidraScript {
    public void run() throws Exception {
        Path root = Path.of(getScriptArgs()[0]), output = Path.of(getScriptArgs()[1]);
        if (Files.exists(output)) throw new Exception("Receipt already exists");
        JsonArray cases = new JsonArray();
        // Native status, skill, level, random seed, attack limit, actor faction.
        int[][] inputs = {
            {1, 100, 1, 12, 10, 0}, {1, 1, 1, 12, 10, 0},
            {255, 100, 1, 12, 10, 0}, {255, 1, 1, 12, 10, 0},
            {0, 100, 1, 12, 10, 0}, {1, 100, 1, 12, 0, 0},
            {255, 100, 1, 12, 10, 1}, {1, 94, 1000, 39, 10, 0},
            {255, 94, 1000, 39, 10, 0}
        };
        for (int[] input : inputs) {
            ActionOracleHarness h = new ActionOracleHarness(currentProgram);
            try {
                byte[] warrior = new byte[3138], enemy = new byte[288];
                ByteBuffer w = ByteBuffer.wrap(warrior), m = ByteBuffer.wrap(enemy);
                w.putShort(0x10, (short)input[5]);
                for (int field : new int[]{0xf4, 0xf6}) w.putShort(field, (short)input[1]);
                w.putShort(0x40, (short)6);
                w.putShort(0xaca, (short)input[4]);
                m.putShort(0, (short)input[2]);
                m.putShort(0xe, (short)1);
                m.putShort(0x110, (short)100);
                m.putShort(0x112, (short)100);
                m.put(0xb0, (byte)input[0]);
                h.write(0x448aa0, warrior);
                h.write(0x439d3c, enemy);
                h.value(0x43565f, 1, 1);
                h.value(0x4435d2, 2, 0);
                h.value(0x4435c4, 2, 1);
                h.enter(0x100af0, 0x6ffe00, 0x900000);
                h.value(h.sp() + 4, 2, 0);
                h.value(h.sp() + 6, 2, 10);
                long seed = input[3];
                int draws = 0;
                JsonArray effects = new JsonArray();
                for (int steps = 0; h.pc() != 0x900000; steps++) {
                    if (steps >= 10000) throw new Exception("Capture instruction limit at " + Long.toHexString(h.pc()));
                    long pc = h.pc();
                    if (pc == 0x1368c8) {
                        h.value(0x43565f, 1, 0);
                        effects.add("exit");
                        h.ret();
                    } else if (pc == 0x11f1a8 || pc == 0x10e6b6 || pc == 0x16043e) {
                        h.ret();
                    } else if (pc == 0x133a18 || pc == 0x16474a) {
                        effects.add((pc == 0x133a18 ? "sound:" : "warning:") + h.word(h.sp() + 4));
                        h.ret();
                    } else if (pc == 0x123d1c) {
                        effects.add("flash:" + Long.toHexString(h.pointer(h.sp() + 4)));
                        h.ret();
                    } else if (pc == 0x153cb2) {
                        seed = seed * 16807 % 2147483647;
                        short value = (short)seed;
                        h.value(h.sp(), 2, value == Short.MIN_VALUE ? 0 : value);
                        draws++;
                        h.jump(pc + 2);
                    } else if (!h.cpu.step(monitor)) throw new Exception(h.cpu.getLastError());
                }
                if (h.sp() != 0x6ffe04) throw new Exception("Capture stack mismatch");
                JsonObject c = new JsonObject();
                c.addProperty("id", "capture-" + cases.size());
                c.addProperty("warrior_before", HexFormat.of().formatHex(warrior));
                c.addProperty("enemy_before", HexFormat.of().formatHex(enemy));
                c.addProperty("warrior_after", HexFormat.of().formatHex(h.read(0x448aa0, 3138)));
                c.addProperty("enemy_after", HexFormat.of().formatHex(h.read(0x439d3c, 288)));
                c.addProperty("seed_before", input[3]);
                c.addProperty("seed_after", seed);
                c.addProperty("draws", draws);
                c.addProperty("capture_mode", h.read(0x43565f, 1)[0]);
                c.add("effects", effects);
                cases.add(c);
            } finally { h.cpu.dispose(); }
        }
        JsonObject result = new JsonObject();
        result.addProperty("authority", "Original attack100af0 through return, including native pin and Rand wrappers. Exit-mode, redraw, controls, flash, audio and warning services acknowledged; QuickDraw Random supplied deterministic WORDs. No live presentation claim.");
        for (String file : new String[]{"tools/ghidra/VerifyMechanicalCapture.java", "tools/ghidra/ActionOracleHarness.java", "analysis/ghidra/startup/code1-mapped.bin"})
            result.addProperty(file, ActionOracleHarness.sha256(root.resolve(file)));
        result.add("cases", cases);
        Files.writeString(output, result.toString() + "\n");
        println("Captured " + cases.size() + " mechanical branches");
    }
}
