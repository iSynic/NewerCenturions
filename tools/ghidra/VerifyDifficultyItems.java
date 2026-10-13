import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;

/** Capture difficulty-dependent original item construction without executing port code. */
public class VerifyDifficultyItems extends VerifyRandomItems {
    int difficulty;

    @Override
    public void run() throws Exception {
        Path root = Path.of(getScriptArgs()[0]);
        Path original = root.resolve("NewCenturions/TheNewCenturions");
        items = Files.readAllBytes(original.resolve("Data Files/Data ID"));
        scenarioItems = Files.readAllBytes(original.resolve("Scenarios/The Thunder of Guns/0/Data NI"));
        StringBuilder cases = new StringBuilder();
        for (difficulty = -2; difficulty <= 2; difficulty++) {
            for (int id : new int[]{165, 400, 602}) {
                probe(12, id, 3, cases);
            }
        }
        StringBuilder identity = new StringBuilder();
        String[][] inputs = {
            {"probe", "tools/ghidra/VerifyDifficultyItems.java"},
            {"parent_probe", "tools/ghidra/VerifyRandomItems.java"},
            {"service_probe", "tools/ghidra/VerifyStoryInventory.java"},
            {"harness", "tools/ghidra/ActionOracleHarness.java"},
            {"mapped_binary", "analysis/ghidra/startup/code1-mapped.bin"},
            {"items", "NewCenturions/TheNewCenturions/Data Files/Data ID"}
        };
        for (String[] input : inputs) {
            if (identity.length() > 0) identity.append(',');
            identity.append('"').append(input[0]).append("_sha256\":\"")
                .append(ActionOracleHarness.sha256(root.resolve(input[1]))).append('"');
        }
        Files.writeString(root.resolve("evidence/fixtures/difficulty-items.json"),
            "{\"authority\":\"Original randomitem12d558 with signed difficulty word442576 at all five menu levels. Fresh22B firearm165, armor400 and equipment602 exercise ammunition odds, enhancement thresholds, damage/mastery, armor/resistance, attribute and range loop counts. Loaditem and verified Rand service boundaries are intercepted; complete22B results and ordered RNG receipts are observed. No port code executed.\",\"identity\":{" +
            identity + "},\"cases\":[" + cases + "]}\n");
        println("15 original difficulty item observations exported.");
    }

    @Override
    void probe(long seed, int id, int enhancement, StringBuilder cases) throws Exception {
        ActionOracleHarness harness = frame();
        RunResult result = new RunResult(seed);
        byte[] before = new byte[22];
        ByteBuffer.wrap(before).putShort(0, (short) id);
        byte[] definition = definition(id);
        try {
            harness.write(0x41fc74, new byte[22000]);
            harness.write(0x41fc74, before);
            harness.value(0x442576, 2, difficulty);
            harness.write(0x46225e, Arrays.copyOfRange(items, 30000, 60000));
            harness.enter(0x12d558, 0x6ffe00, 0x900000);
            harness.value(0x6ffe04, 2, 0);
            harness.value(0x6ffe06, 2, enhancement);
            boolean finished = false;
            for (int step = 0; step < 500000; step++) {
                long pc = harness.pc();
                if (pc == 0x900000) { finished = true; break; }
                if (pc == 0x130afe) { loadItem(harness); continue; }
                if (pc == 0x153ca4) { result.rng(harness); continue; }
                if (!harness.cpu.step(monitor)) {
                    throw new Exception("Difficulty item PC " + Long.toHexString(pc) + ": " + harness.cpu.getLastError());
                }
            }
            if (!finished) throw new Exception("Difficulty item instruction budget");
            if (cases.length() > 0) cases.append(',');
            cases.append("{\"case\":\"difficulty").append(difficulty).append("_item").append(id)
                .append("_seed12\",\"id\":").append(id).append(",\"enhancement\":").append(enhancement)
                .append(",\"difficulty\":").append(difficulty).append(",\"definition\":\"").append(hex(definition))
                .append("\",\"before\":\"").append(hex(before)).append("\",\"after\":\"")
                .append(hex(harness.read(0x41fc74, 22))).append("\",\"seed_before\":").append(seed)
                .append(",\"seed_after\":").append(result.seed).append(",\"rng_draws\":").append(result.draws)
                .append(",\"draws\":[").append(result.drawRecords).append("]}");
        } finally {
            harness.cpu.dispose();
        }
    }
}
