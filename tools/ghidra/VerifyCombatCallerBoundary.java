import com.google.gson.*;
import java.util.*;

/** Finite original-instruction caller boundary; shared drawing, SANE and audio delegates. */
public abstract class VerifyCombatCallerBoundary extends VerifyRetainedCallers {
    void resultAudio(ActionOracleHarness h, JsonArray sounds) throws Exception {
        int code = h.word(h.sp() + 6), attacker = h.word(h.sp() + 8);
        if (code != 0 && code != -1) return;
        String[] names = {"D0", "D1", "D2", "D3", "D4", "D5", "D6", "D7", "A0", "A1", "A2", "A3", "A4", "A5", "A6", "SP", "PC", "XF", "NF", "ZF", "VF", "CF"};
        java.math.BigInteger[] saved = new java.math.BigInteger[names.length];
        for (int i = 0; i < names.length; i++) saved[i] = h.cpu.readRegister(names[i]);
        h.write(0x6dc000, new byte[8192]);
        h.cpu.writeRegister("SP", 0x6dd000);
        h.cpu.writeRegister("D5", attacker);
        h.jump(code == 0 ? 0x158720 : 0x159042);
        int steps = 0;
        while (h.pc() != 0x15886e && h.pc() != 0x1588fc && h.pc() != 0x15904c) {
            if (++steps > 150) throw new Exception("Result audio instruction budget");
            if (h.pc() == 0x133a18) { sounds.add(h.word(h.sp() + 4)); h.ret(); }
            else if ((h.word(h.pc()) & 65535) == 0xa873 || (h.word(h.pc()) & 65535) == 0xa884) { h.pop(4); h.jump(h.pc() + 2); }
            else if (!h.cpu.step(monitor)) throw new Exception(h.cpu.getLastError());
        }
        h.value(0x438784, 2, 0);
        for (int i = 0; i < names.length; i++) h.cpu.writeRegister(names[i], saved[i]);
    }
    int executeCombat(ActionOracleHarness h, long stop, boolean full, boolean turn, JsonArray sounds, JsonArray rng) throws Exception {
        int steps = 0;
        SaneBoundary sane = new SaneBoundary(h.cpu, currentProgram);
        while (h.pc() != stop) {
            if (++steps > 1500000) throw new Exception("Combat caller instruction budget");
            long pc = h.pc();
            int op = h.word(pc) & 65535;
            if (pc == 0x400038) { h.cpu.writeRegister("D0", Math.abs(h.word(h.sp() + 4))); h.ret(); }
            else if (pc == 0x153ca4) {
                int bound = h.word(h.sp() + 4);
                seed = seed * 16807 % 2147483647;
                int raw = (short)seed;
                h.cpu.writeRegister("D0", Math.abs(raw == -32768 ? 0 : raw) * bound / 32768 + 1);
                JsonObject draw = new JsonObject();
                draw.addProperty("bound", bound);
                draw.addProperty("result", h.cpu.readRegister("D0").intValue());
                draw.addProperty("return_address", Long.toHexString(h.pointer(h.sp())));
                rng.add(draw);
                draws++;
                h.ret();
            } else if (pc == 0x133a18 || pc == 0x16474a) { sounds.add(h.word(h.sp() + 4)); h.ret(); }
            else if (pc == 0x158454) { resultAudio(h, sounds); h.ret(); }
            else if (turn && pc == 0x12946c) { h.cpu.writeRegister("D0", 1); h.ret(); }
            else if (turn && pc == 0x160fb2) h.ret();
            else if (full && pc == 0x11f1a8) { displayItemWriter(h); h.ret(); }
            else if (pc == 0x159192) { h.value(0x44719a, 2, 0); h.cpu.writeRegister("D0", 0); h.ret(); }
            else if (Set.of(0x120c14L, 0x105572L, 0x10e6b6L, 0x133d46L, 0x13154cL, 0x11f1a8L, 0x11edc4L, 0x14a02cL, 0x13a2c4L, 0x1205f8L, 0x16043eL, 0x1382a8L, 0x11f136L).contains(pc)) h.ret();
            else if (op == 0xa9eb) {
                if ((h.word(h.sp()) & 65535) == 0x12) {
                    long destination = h.pointer(h.sp() + 2);
                    sane.extended(destination, Math.sqrt(sane.extended(destination)));
                    h.pop(6);
                    h.jump(pc + 2);
                } else sane.execute();
            } else if (op == 0xa873 || op == 0xaa1c || op == 0xa862 || op == 0xa863 || op == 0xa89b || op == 0xa893 || op == 0xa891 || op == 0xa884 || op == 0xa8b7) { h.pop(4); h.jump(pc + 2); }
            else if (op == 0xa8ad) {
                long sp = h.sp(), rect = h.pointer(sp);
                int x = h.word(sp + 4), y = h.word(sp + 6);
                boolean inside = x >= h.word(rect) && x < h.word(rect + 4) && y >= h.word(rect + 2) && y < h.word(rect + 6);
                h.value(sp + 8, 1, inside ? 1 : 0);
                h.pop(8);
                h.jump(pc + 2);
            } else if (op == 0xa8aa) {
                long sp = h.sp(), out = h.pointer(sp), b = h.pointer(sp + 4), a = h.pointer(sp + 8);
                int top = Math.max(h.word(a), h.word(b)), left = Math.max(h.word(a + 2), h.word(b + 2));
                int bottom = Math.min(h.word(a + 4), h.word(b + 4)), right = Math.min(h.word(a + 6), h.word(b + 6));
                boolean empty = top >= bottom || left >= right;
                h.value(out, 2, empty ? 0 : top);
                h.value(out + 2, 2, empty ? 0 : left);
                h.value(out + 4, 2, empty ? 0 : bottom);
                h.value(out + 6, 2, empty ? 0 : right);
                h.value(sp + 12, 1, empty ? 0 : 1);
                h.pop(12);
                h.jump(pc + 2);
            } else if (op == 0xa8a7) {
                long rect = h.pointer(h.sp() + 8);
                h.write(rect, h.read(h.sp() + 4, 4));
                h.write(rect + 4, h.read(h.sp(), 4));
                h.pop(12);
                h.jump(pc + 2);
            } else if (turn && op == 0xa032) h.jump(pc + 2);
            else if (op == 0xa9b4) h.jump(pc + 2);
            else if (op == 0xa974) { h.cpu.writeRegister("SP", h.sp() - 2); h.value(h.sp(), 2, 0); h.jump(pc + 2); }
            else if (op == 0xa89c) { h.pop(2); h.jump(pc + 2); }
            else if (op == 0xa8ec) { h.pop(22); h.jump(pc + 2); }
            else if (!h.cpu.step(monitor)) throw new Exception("Combat caller " + Long.toHexString(pc) + ": " + h.cpu.getLastError());
        }
        return steps;
    }
}
