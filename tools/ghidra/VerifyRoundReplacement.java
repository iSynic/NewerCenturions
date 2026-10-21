import ghidra.app.script.GhidraScript;
import com.google.gson.*;
import java.nio.file.*;
import java.nio.ByteBuffer;

/** Original opcode126 predicates; script loading and successor actions are outside the slice. */
public class VerifyRoundReplacement extends GhidraScript {
    @Override public void run() throws Exception {
        Path root = Path.of(getScriptArgs()[0]), output = root.resolve(getScriptArgs()[1]);
        if (Files.exists(output)) throw new Exception("Native output already exists");
        Path base = root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0");
        byte[] scripts = Files.readAllBytes(base.resolve("Data ED3"));
        byte[] extras = Files.readAllBytes(base.resolve("Data EDCD"));
        JsonArray cases = new JsonArray();
        for (int script : new int[]{10, 35, 38, 97, 331, 333, 338, 340, 363, 364}) {
            int parameter = ByteBuffer.wrap(scripts).getShort(script * 42 + 24);
            byte[] extra = java.util.Arrays.copyOfRange(extras, parameter * 10, parameter * 10 + 10);
            for (int outcome = 0; outcome < 2; outcome++) {
                int counter = ByteBuffer.wrap(extra).getShort() == 0 ? ByteBuffer.wrap(extra).getShort(2) + outcome : 1;
                long seed = outcome == 0 ? 2 : 12, initialSeed = seed;
                int selected = -1, steps = 0, draws = 0;
                ActionOracleHarness h = new ActionOracleHarness(currentProgram);
                try {
                    h.enter(0x141040, 0x6ffe00, 0x6ff000);
                    h.cpu.writeRegister("A6", 0x700000);
                    h.write(0x44330a, extra);
                    h.value(0x44255c, 1, counter);
                    h.value(0x408576, 2, 0);
                    h.value(0x439cec, 2, script);
                    while (h.pc() != 0x14694c && h.pc() != 0x14032e) {
                        if (++steps > 500) throw new Exception("Round predicate instruction budget");
                        long pc = h.pc();
                        if (pc == 0x123c60) {
                            selected = h.word(h.sp() + 4);
                            h.ret();
                        } else if ((h.word(pc) & 65535) == 0xa861) {
                            seed = seed * 16807 % 2147483647;
                            h.value(h.sp(), 2, (short)seed == -32768 ? 0 : (short)seed);
                            draws++;
                            h.jump(pc + 2);
                        } else if (!h.cpu.step(monitor)) throw new Exception(h.cpu.getLastError());
                    }
                    JsonObject c = new JsonObject();
                    c.addProperty("script", script);
                    c.addProperty("counter", counter);
                    c.addProperty("seed_before", initialSeed);
                    c.addProperty("selected", selected);
                    c.addProperty("slot_after", h.word(0x439cec));
                    c.addProperty("seed_after", seed);
                    c.addProperty("rng_draws", draws);
                    c.addProperty("steps", steps);
                    cases.add(c);
                } finally { h.cpu.dispose(); }
            }
        }
        JsonObject receipt = new JsonObject();
        receipt.addProperty("authority", "Original newland126 141040 through14694c/14032e, exact authored extras for ten Thunder owners. Records selected loaddoor2 argument without loading its successor. Native round/slot predicates and Rand arithmetic execute; QuickDraw seed kernel supplied. No successor effects or whole turn claim.");
        receipt.addProperty("probe_sha256", ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyRoundReplacement.java")));
        receipt.addProperty("binary_sha256", ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin")));
        receipt.add("cases", cases);
        Files.writeString(output, new GsonBuilder().setPrettyPrinting().create().toJson(receipt) + "\n");
        println("Original round predicates captured: " + cases.size());
    }
}
