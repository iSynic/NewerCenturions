import com.google.gson.GsonBuilder;
import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;

public class VerifyPartialInitiative extends VerifyStoryParty {
    String digest(byte[] bytes) throws Exception {
        return hex.formatHex(java.security.MessageDigest.getInstance("SHA-256").digest(bytes));
    }

    JsonArray changes(byte[] before, byte[] after) {
        JsonArray result = new JsonArray();
        for (int index = 0; index < before.length; index++) {
            if (before[index] != after[index]) {
                JsonArray change = new JsonArray();
                change.add(index);
                change.add(after[index] & 255);
                result.add(change);
            }
        }
        return result;
    }

    @Override
    public void run() throws Exception {
        root = Path.of(getScriptArgs()[0]);
        warriors = new byte[6 * 3138];
        String[] names = {"Cindred", "Coriand", "Kevlar", "PDP 11", "Sifini", "Solestri"};
        int[] speeds = {12, 10, 9, 8, 8, 9};
        int[] enemies = {8, 7, 6, 5, 9, 7, 9, 5, 8};
        for (int member = 0; member < names.length; member++) {
            byte[] record = Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Warriors/" + names[member]));
            System.arraycopy(record, 0, warriors, member * 3138, 3138);
            word(warriors, member * 3138 + 0x50, 30);
            word(warriors, member * 3138 + 0x90, 0);
            word(warriors, member * 3138 + 0x9e8, speeds[member]);
            word(warriors, member * 3138 + 0x20, -1);
        }
        JsonObject fixture = new JsonObject();
        fixture.addProperty("authority", "Original getbattleorder 124d9e..125128; selected-count byte 44273a varies. Only Rand is intercepted. Complete selected records, unused-record preservation, order and RNG observed.");
        fixture.addProperty("probe_sha256", ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyPartialInitiative.java")));
        fixture.addProperty("harness_sha256", ActionOracleHarness.sha256(root.resolve("tools/ghidra/ActionOracleHarness.java")));
        fixture.addProperty("mapped_binary_sha256", ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin")));
        fixture.addProperty("warriors", hex.formatHex(warriors));
        JsonArray enemySpeeds = new JsonArray();
        for (int speed : enemies) enemySpeeds.add(speed);
        fixture.add("enemy_speeds", enemySpeeds);
        JsonArray cases = new JsonArray();
        for (int count : new int[]{1, 3, 5}) {
            for (int mode : new int[]{-1, 0, 1}) {
                ActionOracleHarness harness = new ActionOracleHarness(currentProgram);
                try {
                    harness.write(0x448aa0, warriors);
                    harness.value(0x442739, 1, 0);
                    harness.value(0x44273a, 1, count - 1);
                    harness.value(0x43874a, 2, enemies.length);
                    for (int index = 0; index < enemies.length; index++) {
                        harness.value(0x439d3c + index * 288 + 4, 2, enemies[index]);
                    }
                    seed = 12;
                    draws = 0;
                    harness.enter(0x124d9e, 0x700000, 0x900000);
                    harness.value(0x700004, 2, mode);
                    boolean finished = false;
                    for (int step = 0; step < 100000; step++) {
                        long pc = harness.pc();
                        if (pc == 0x900000) { finished = true; break; }
                        if (pc == 0x153ca4) {
                            int bound = harness.word(harness.sp() + 4);
                            seed = seed * 16807 % 2147483647;
                            draws++;
                            short raw = (short) seed;
                            if (raw == Short.MIN_VALUE) raw = 0;
                            harness.cpu.writeRegister("D0", Math.abs((int) raw) * bound / 32768 + 1);
                            harness.ret();
                        } else if (!harness.cpu.step(monitor)) {
                            throw new Exception("Partial initiative PC " + Long.toHexString(pc) + ": " + harness.cpu.getLastError());
                        }
                    }
                    if (!finished) throw new Exception("Partial initiative instruction budget");
                    byte[] fullAfter = harness.read(0x448aa0, warriors.length);
                    if (!Arrays.equals(Arrays.copyOfRange(warriors, count * 3138, warriors.length),
                                       Arrays.copyOfRange(fullAfter, count * 3138, fullAfter.length))) {
                        throw new Exception("Original modified an unselected warrior");
                    }
                    byte[] before = Arrays.copyOf(warriors, count * 3138);
                    byte[] after = Arrays.copyOf(fullAfter, count * 3138);
                    JsonObject observation = new JsonObject();
                    observation.addProperty("case", "count" + count + "_mode" + mode + "_seed12");
                    observation.addProperty("party_count", count);
                    observation.addProperty("mode", mode);
                    observation.addProperty("unavailable", 0);
                    observation.add("before_changes", new JsonArray());
                    observation.add("after_changes", changes(before, after));
                    observation.addProperty("before_sha256", digest(before));
                    observation.addProperty("after_sha256", digest(after));
                    observation.addProperty("order", hex.formatHex(harness.read(0x4470b8, 220)));
                    observation.addProperty("seed_before", 12);
                    observation.addProperty("seed_after", seed);
                    observation.addProperty("draws", draws);
                    cases.add(observation);
                } finally {
                    harness.cpu.dispose();
                }
            }
        }
        fixture.add("cases", cases);
        Files.writeString(root.resolve("evidence/fixtures/partial-initiative.json"),
            new GsonBuilder().setPrettyPrinting().create().toJson(fixture) + "\n");
        println("Nine original partial-party initiative observations exported.");
    }
}
