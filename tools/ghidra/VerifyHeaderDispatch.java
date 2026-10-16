import ghidra.app.script.GhidraScript;
import com.google.gson.JsonObject;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.HexFormat;

/** Original opcode8 copy loop, with distinct current-map and Data ED3 records. */
public class VerifyHeaderDispatch extends GhidraScript {
    public void run() throws Exception {
        Path root = Path.of(getScriptArgs()[0]);
        Path source = root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data DD");
        byte[] headers = Arrays.copyOf(Files.readAllBytes(source), 4200);
        byte[] before = Arrays.copyOfRange(headers, 13 * 42, 14 * 42);
        Arrays.fill(headers, 15 * 42 + 8, 15 * 42 + 40, (byte)0);
        headers[15 * 42 + 9] = 1;
        headers[15 * 42 + 25] = 124;
        headers[15 * 42 + 41] = 0x7b;
        ActionOracleHarness h = new ActionOracleHarness(currentProgram);
        try {
            h.write(0x478e42, headers);
            h.write(0x479eaa, before);
            h.enter(0x1434d4, 0x6ffe00, 0x900000);
            h.cpu.writeRegister("A6", 0x700000);
            h.value(0x700000 - 0x16, 2, 15);
            for (int step = 0; h.pc() != 0x1402fa; step++) {
                if (step == 256 || h.pc() < 0x1434d4 || h.pc() > 0x14352c)
                    throw new Exception("Unexpected opcode8 instruction boundary");
                if (!h.cpu.step(monitor)) throw new Exception(h.cpu.getLastError());
            }
            if (!Arrays.equals(headers, h.read(0x478e42, 4200)) || h.sp() != 0x6ffe00)
                throw new Exception("Original opcode8 changed its input table or stack");
            JsonObject receipt = new JsonObject();
            receipt.addProperty("authority", "Original opcode8 1434d4..14352c to1402fa, complete42-byte header. Controlled target header15 action/dialogue and stamp differ from Data ED3; no services/RNG or whole newland/live claim.");
            receipt.addProperty("probe_sha256", ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyHeaderDispatch.java")));
            receipt.addProperty("harness_sha256", ActionOracleHarness.sha256(root.resolve("tools/ghidra/ActionOracleHarness.java")));
            receipt.addProperty("code_sha256", ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin")));
            receipt.addProperty("source_sha256", ActionOracleHarness.sha256(source));
            receipt.addProperty("headers", HexFormat.of().formatHex(headers));
            receipt.addProperty("before", HexFormat.of().formatHex(before));
            receipt.addProperty("after", HexFormat.of().formatHex(h.read(0x479eaa, 42)));
            Files.writeString(root.resolve("evidence/fixtures/header-dispatch.json"), receipt + "\n");
        } finally {
            h.cpu.dispose();
        }
    }
}
