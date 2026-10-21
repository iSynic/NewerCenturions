import ghidra.app.script.GhidraScript;
import com.google.gson.*;
import java.nio.file.*;

/** Original NPC death-feedback branch; drawing and Sound Manager playback are excluded. */
public class VerifyDeathFeedback extends GhidraScript {
    @Override public void run() throws Exception {
        Path root = Path.of(getScriptArgs()[0]);
        JsonObject fixture = new JsonObject();
        fixture.addProperty("authority", "Original showresults158ea0..158efa, ending at159090. Alternate showresults158f14 is also captured for result -14. Native NPC death sound selection only; drawing prefix and playback are not executed. Equipped warrior item differs between controls but is not read by this original branch. No full shooting/killbody or live audio claim.");
        fixture.addProperty("probe_sha256", ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyDeathFeedback.java")));
        fixture.addProperty("binary_sha256", ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin")));
        JsonArray cases = new JsonArray();
        for (int weapon : new int[]{165, 2, 38, -14}) {
            try (var ignored = new Scope()) {
                ActionOracleHarness h = new ActionOracleHarness(currentProgram);
                ignored.cpu = h;
                h.cpu.writeRegister("SP", 0x6ffe00);
                h.cpu.writeRegister("D3", 10);
                h.value(0x448aa0 + 0xe4, 2, weapon);
                h.value(0x439d3c + 0x6a, 2, weapon < 0 ? 500 : 611);
                h.jump(weapon < 0 ? 0x158f14 : 0x158ea0);
                JsonArray sounds = new JsonArray();
                int steps = 0;
                while (h.pc() != 0x159090) {
                    if (++steps > 50) throw new Exception("Death feedback instruction budget");
                    if (h.pc() == 0x133a18) {
                        sounds.add(h.word(h.sp() + 4));
                        h.ret();
                    } else if (!h.cpu.step(monitor)) {
                        throw new Exception(h.cpu.getLastError());
                    }
                }
                JsonObject c = new JsonObject();
                c.addProperty("weapon", weapon);
                c.addProperty("body", 10);
                c.addProperty("record_sound", weapon < 0 ? 500 : 611);
                c.add("sounds", sounds);
                c.addProperty("steps", steps);
                fixture.addProperty("rng_calls", 0);
                if (weapon < 0) fixture.add("alternate", c); else cases.add(c);
            }
        }
        fixture.add("cases", cases);
        Path output = root.resolve(getScriptArgs()[1]);
        if (Files.exists(output)) throw new Exception("Refusing to overwrite a capture");
        Files.createDirectories(output.getParent());
        Files.writeString(output, new GsonBuilder().setPrettyPrinting().create().toJson(fixture) + "\n");
        println("Original death-feedback controls captured: " + cases.size());
    }
    static class Scope implements AutoCloseable {
        ActionOracleHarness cpu;
        @Override public void close() { if (cpu != null) cpu.cpu.dispose(); }
    }
}
