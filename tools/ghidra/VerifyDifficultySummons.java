import com.google.gson.GsonBuilder;
import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;

/** Original buildmonster instructions with the existing bounded SANE service model. */
public class VerifyDifficultySummons extends VerifyStoryInventory {
    @Override
    public void run() throws Exception {
        Path root = Path.of(getScriptArgs()[0]);
        byte[] definitions = Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data MD"));
        JsonObject fixture = new JsonObject();
        fixture.addProperty("authority", "Complete original buildmonster109210..1094fc instructions; verified Rand and bounded SANE model intercepted. No port code. Ordinary definition1 and definition113 with explicit caster/aim-cap input controls; full288B records and RNG observed. This is bounded emulation, not execution of Apple's SANE ROM.");
        JsonObject identity = new JsonObject();
        for (String[] source : new String[][] {
            {"probe", "tools/ghidra/VerifyDifficultySummons.java"},
            {"parent", "tools/ghidra/VerifyStoryInventory.java"},
            {"harness", "tools/ghidra/ActionOracleHarness.java"},
            {"sane", "tools/ghidra/SaneBoundary.java"},
            {"binary", "analysis/ghidra/startup/code1-mapped.bin"}
        }) identity.addProperty(source[0] + "_sha256", ActionOracleHarness.sha256(root.resolve(source[1])));
        fixture.add("identity", identity);
        JsonArray cases = new JsonArray();
        for (int difficulty = -2; difficulty <= 2; difficulty++) {
            for (int variant = 0; variant < 2; variant++) {
                int id = variant == 0 ? 1 : 113;
                byte[] before = Arrays.copyOfRange(definitions, id * 288, (id + 1) * 288);
                if (variant == 1) {
                    ByteBuffer.wrap(before).putShort(0x10e, (short)31).putShort(0x52, (short)5).putShort(0xa, (short)99);
                }
                ActionOracleHarness harness = frame();
                RunResult result = new RunResult(12);
                try {
                    harness.write(0x440dbc, before);
                    harness.value(0x442576, 2, difficulty);
                    harness.enter(0x109210, 0x6ffe00, 0x900000);
                    harness.value(0x6ffe04, 2, 0);
                    SaneBoundary sane = new SaneBoundary(harness.cpu, currentProgram);
                    boolean finished = false;
                    for (int step = 0; step < 100000; step++) {
                        long pc = harness.pc();
                        if (pc == 0x900000) { finished = true; break; }
                        if (pc == 0x153ca4) { result.rng(harness); continue; }
                        if ((harness.word(pc) & 65535) == 0xa9eb) { sane.execute(); continue; }
                        if (!harness.cpu.step(monitor)) throw new Exception("Summon at " + Long.toHexString(pc) + ": " + harness.cpu.getLastError());
                    }
                    if (!finished) throw new Exception("Summon instruction budget");
                    JsonObject observation = new JsonObject();
                    observation.addProperty("case", "difficulty" + difficulty + "_variant" + variant);
                    observation.addProperty("difficulty", difficulty);
                    observation.addProperty("faction", 0);
                    observation.addProperty("definition_id", id);
                    observation.addProperty("variant", variant == 0 ? "unmodified original" : "original113 with psi10e31, psi525, aim99");
                    observation.addProperty("before", hex(before));
                    observation.addProperty("after", hex(harness.read(0x440dbc, 288)));
                    observation.addProperty("seed_before", 12);
                    observation.addProperty("seed_after", result.seed);
                    observation.addProperty("draws", result.draws);
                    cases.add(observation);
                } finally { harness.cpu.dispose(); }
            }
        }
        fixture.add("cases", cases);
        Files.writeString(root.resolve("evidence/fixtures/difficulty-summons.json"), new GsonBuilder().setPrettyPrinting().create().toJson(fixture) + "\n");
        println("Ten original summon difficulty observations exported.");
    }
}
