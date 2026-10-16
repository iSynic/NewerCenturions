import ghidra.app.script.GhidraScript;
import com.google.gson.JsonObject;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.HexFormat;

/** Complete original CODE3 tolower, without substituting a host character conversion. */
public class VerifyCaseFold extends GhidraScript {
    public void run() throws Exception {
        Path root = Path.of(getScriptArgs()[0]);
        Path code = root.resolve("analysis/extracted/resources/106/434f4445_3.bin");
        Path data = root.resolve("analysis/ghidra/startup/a5-initial.bin");
        byte[] initial = Files.readAllBytes(data);
        byte[] folded = new byte[256];
        ActionOracleHarness h = new ActionOracleHarness(currentProgram);
        try {
            h.write(0x300000, Files.readAllBytes(code));
            h.write(0x3fe210, initial);
            h.cpu.writeRegister("A5", 0x400000);
            for (int input = -1; input < 256; input++) {
                h.enter(0x300a58, 0x6ffe00, 0x900000);
                h.value(0x6ffe04, 2, input);
                for (int step = 0; h.pc() != 0x900000; step++) {
                    if (step == 64 || h.pc() < 0x300a58 || h.pc() > 0x300a90)
                        throw new Exception("Unexpected case-fold instruction boundary");
                    if (!h.cpu.step(monitor))
                        throw new Exception("Original case-fold instruction: " + h.cpu.getLastError());
                }
                int result = (short)h.cpu.readRegister("D0").intValue();
                if (h.sp() != 0x6ffe04 || h.cpu.readRegister("A5").longValue() != 0x400000)
                    throw new Exception("Original case-fold return ABI");
                if (input == -1 && result != -1 || input >= 0 && (result < 0 || result > 255))
                    throw new Exception("Original case-fold result width: " + input + " -> " + result);
                if (input >= 0) folded[input] = (byte)result;
            }
            if (!Arrays.equals(initial, h.read(0x3fe210, initial.length)))
                throw new Exception("Original case-fold mutated DATA");
            JsonObject receipt = new JsonObject();
            receipt.addProperty("authority", "Complete original CODE3 tolower300a58 to RTS for EOF and all256 bytes, using original initialized DATA; no services, host conversion or live desktop observation.");
            receipt.addProperty("probe_sha256", ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyCaseFold.java")));
            receipt.addProperty("harness_sha256", ActionOracleHarness.sha256(root.resolve("tools/ghidra/ActionOracleHarness.java")));
            receipt.addProperty("code_sha256", ActionOracleHarness.sha256(code));
            receipt.addProperty("data_sha256", ActionOracleHarness.sha256(data));
            receipt.addProperty("lowercase256", HexFormat.of().formatHex(folded));
            receipt.addProperty("eof", -1);
            Files.writeString(root.resolve("evidence/fixtures/case-fold.json"), receipt + "\n");
        } finally {
            h.cpu.dispose();
        }
    }
}
