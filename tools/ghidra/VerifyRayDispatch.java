import ghidra.app.script.GhidraScript;
import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.HexFormat;

/** Original cast ray geometry and ordered dispatch, excluding spell resolution. */
public class VerifyRayDispatch extends GhidraScript {
    public void run() throws Exception {
        Path root = Path.of(getScriptArgs()[0]);
        Path output = Path.of(getScriptArgs()[1]);
        if (Files.exists(output)) throw new Exception("Native output already exists: " + output);
        Path spells = root.resolve("NewCenturions/TheNewCenturions/Data Files/Data S");
        byte[] source = Files.readAllBytes(spells);
        JsonArray cases = new JsonArray();
        int[][] inputs = {
            {2103, -1, 0, 0, 0},
            {2506, 1, 0, 50, 0},
            {2607, 0, -1, 0, 0},
            {2707, 0, 1, 50, 0},
            {3104, -1, -1, 0, 0},
            {2103, -1, 1, 50, 0},
            {2506, 1, -1, 0, 0},
            {2607, 1, 1, 50, 0},
            {2103, -1, 0, 0, 1},
            {3104, 1, 0, 50, 1}
        };
        for (int[] input : inputs) {
            ActionOracleHarness h = new ActionOracleHarness(currentProgram);
            try {
                int id = input[0], dx = input[1], dy = input[2], delay = input[3];
                int offset = (id / 1000 - 1) * 2842 + (id / 100 % 10 - 1) * 406 + (id % 100 - 1) * 58;
                h.write(0x44374e, java.util.Arrays.copyOfRange(source, offset, offset + 58));
                ByteBuffer map = ByteBuffer.allocate(16200);
                for (int cell = 0; cell < 8100; cell++) map.putShort((short)1155);
                map.putShort((44 * 90 + 55) * 2, (short)1);
                for (int step = 1; step <= 3; step++)
                    map.putShort(((44 + step * dx) * 90 + 55 + step * dy) * 2,
                        (short)(step == 1 ? -10 : step == 2 ? (input[4] == 0 ? 11 : 1061) : 0));
                h.write(0x4085b8, map.array());
                h.write(0x4434e0, new byte[220]);
                h.value(0x442780, 4, 5);
                h.value(0x442784, 4, 5);
                h.value(0x447194, 2, 1);
                h.value(0x4470ba, 2, 1);
                h.enter(0x10db26, 0x6ffe00, 0x900000);
                h.cpu.writeRegister("A6", 0x700000);
                h.cpu.writeRegister("D6", 128 + delay);
                h.value(0x700000 - 6, 2, 39 * 32);
                h.value(0x700000 - 8, 2, 50 * 32);
                h.value(0x700000 - 14, 2, (39 + 4 * dx) * 32);
                h.value(0x700000 - 16, 2, (50 + 4 * dy) * 32);
                SaneBoundary sane = new SaneBoundary(h.cpu, currentProgram);
                JsonArray targets = new JsonArray();
                int iterations = 0;
                for (int instructions = 0; ; instructions++) {
                    long pc = h.pc();
                    if (instructions == 100000) throw new Exception("Ray instruction limit");
                    if (pc == 0x10dc60) h.jump(0x10e07a);
                    else if (pc == 0x10e1f4 || pc == 0x10e230)
                        h.jump(pc == 0x10e1f4 ? 0x10e22a : 0x10e266);
                    else if (pc == 0x151022) {
                        targets.add(h.cpu.readRegister("D5").intValue() & 0xffff);
                        h.ret();
                    } else if (pc == 0x10e31e) {
                        if (++iterations == 128 + delay) break;
                        h.jump(0x10e07a);
                    } else if (pc == 0x400038) {
                        h.cpu.writeRegister("D0", Math.abs(h.word(h.sp() + 4)));
                        h.ret();
                    } else if (h.word(pc) == (short)0xa9eb) sane.execute();
                    else if (!h.cpu.step(monitor)) throw new Exception(h.cpu.getLastError());
                }
                if (!java.util.Arrays.equals(map.array(), h.read(0x4085b8, 16200)) || h.sp() != 0x6ffe02)
                    throw new Exception("Ray changed controlled map or unbalanced its stack");
                JsonObject receipt = new JsonObject();
                receipt.addProperty("spell", id);
                receipt.addProperty("dx", dx);
                receipt.addProperty("dy", dy);
                receipt.addProperty("delay", delay);
                receipt.addProperty("map", HexFormat.of().formatHex(map.array()));
                receipt.add("targets", targets);
                cases.add(receipt);
            } finally {
                h.cpu.dispose();
            }
        }
        JsonObject result = new JsonObject();
        result.addProperty("authority", "Original cast10db26..10dc60 and10e07a..10e31e; graphics and resolvespell acknowledged, abs modeled, SANE bounded arithmetic model. Controlled geometry/HP is functionality assistance. No whole damage/payment/RNG/audio or live claim.");
        for (String name : new String[]{"VerifyRayDispatch.java", "ActionOracleHarness.java", "SaneBoundary.java"})
            result.addProperty(name + "_sha256", ActionOracleHarness.sha256(root.resolve("tools/ghidra/" + name)));
        result.addProperty("code_sha256", ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin")));
        result.addProperty("spells_sha256", ActionOracleHarness.sha256(spells));
        result.add("cases", cases);
        Files.writeString(output, result + "\n");
    }
}
