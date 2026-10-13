import com.google.gson.GsonBuilder;
import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;

public class VerifyUpgradeCosts extends VerifyStoryInventory {
    @Override public void run() throws Exception {
        Path root = Path.of(getScriptArgs()[0]);
        byte[] profiles = Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data MOS"));
        JsonObject fixture = new JsonObject();
        fixture.addProperty("authority", "Original upgradewarrior162bb2..1631e6 cost instructions, complete SANE ordering through the existing bounded arithmetic model; numeric dialog rendering acknowledged. Original MOS bytes, all33 table selectors, all six supplied warriors, explicit counter controls. Not native SANE ROM execution.");
        fixture.addProperty("probe_sha256", ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyUpgradeCosts.java")));
        fixture.addProperty("sane_sha256", ActionOracleHarness.sha256(root.resolve("tools/ghidra/SaneBoundary.java")));
        fixture.addProperty("binary_sha256", ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin")));
        fixture.addProperty("parent_sha256", ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyStoryInventory.java")));
        fixture.addProperty("harness_sha256", ActionOracleHarness.sha256(root.resolve("tools/ghidra/ActionOracleHarness.java")));
        fixture.addProperty("mos_sha256", ActionOracleHarness.sha256(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data MOS")));
        ActionOracleHarness constants = frame();
        try { fixture.addProperty("base_words", hex(constants.read(0x402f4a, 66))); fixture.addProperty("percent_words", hex(constants.read(0x402fae, 66))); }
        finally { constants.cpu.dispose(); }
        JsonArray cases = new JsonArray();
        String[] names = {"Cindred", "Coriand", "Kevlar", "PDP 11", "Sifini", "Solestri"};
        for (int member = 0; member < names.length; member++) {
            int[] selectors = member == 0 ? java.util.stream.IntStream.range(0, 33).toArray() : new int[]{7,16,20,23,32};
            for (int selector : selectors) {
                int[] counts = member == 0 && selector == 0 ? new int[]{-1,0,1,3,12,200} : new int[]{3};
                for (int count : counts) {
                    byte[] before = Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Warriors/" + names[member]));
                    ByteBuffer.wrap(before).putShort(0xace + selector * 2, (short)count);
                    int mos = Short.toUnsignedInt(ByteBuffer.wrap(before).getShort(0x34));
                    byte[] profile = Arrays.copyOfRange(profiles, (mos - 1) * 576, mos * 576);
                    ActionOracleHarness h = frame();
                    try {
                        h.write(0x448aa0, before); h.write(0x41f9f2, profile); h.value(0x4042da, 2, selector);
                        h.cpu.writeRegister("D3", 0); h.jump(0x162bb2);
                        SaneBoundary sane = new SaneBoundary(h.cpu, currentProgram); JsonArray numbers = new JsonArray(); boolean finished = false;
                        for (int step = 0; step < 100000; step++) {
                            if (h.pc() == 0x1631e6) { finished = true; break; }
                            if (h.pc() == 0x153cf4) {
                                JsonObject number = new JsonObject(); number.addProperty("item", h.word(h.sp()+4)); number.addProperty("value", ByteBuffer.wrap(h.read(h.sp()+6,4)).getInt()); numbers.add(number); h.ret(); continue;
                            }
                            if ((h.word(h.pc()) & 65535) == 0xa9eb) { sane.execute(); continue; }
                            if (!h.cpu.step(monitor)) throw new Exception("Cost at " + Long.toHexString(h.pc()) + ": " + h.cpu.getLastError());
                        }
                        if (!finished) throw new Exception("Cost instruction budget");
                        JsonObject c = new JsonObject(); c.addProperty("case", names[member]+"_selector"+selector+"_count"+count); c.addProperty("selector",selector); c.addProperty("count",count); c.addProperty("warrior",hex(before)); c.addProperty("profile",hex(profile)); c.add("numbers",numbers); cases.add(c);
                    } finally { h.cpu.dispose(); }
                }
            }
        }
        fixture.add("cases", cases);
        Files.writeString(root.resolve("evidence/fixtures/upgrade-costs.json"), new GsonBuilder().setPrettyPrinting().create().toJson(fixture)+"\n");
        println("Original cost observations exported: " + cases.size());
    }
}
