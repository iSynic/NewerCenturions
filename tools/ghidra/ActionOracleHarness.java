import ghidra.app.emulator.EmulatorHelper;
import ghidra.program.model.listing.Program;
import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.MessageDigest;
import java.util.HexFormat;

/** Shared primitives for bounded original-instruction action probes. */
public final class ActionOracleHarness {
    public final EmulatorHelper cpu;
    private final Program program;
    public ActionOracleHarness(Program program) {
        this.program = program;
        cpu = new EmulatorHelper(program);
        for (String register : new String[]{"D0","D1","D2","D3","D4","D5","D6","D7","A0","A1","A2","A3","A4","A5","A6","XF","NF","ZF","VF","CF"}) cpu.writeRegister(register, 0);
    }
    public byte[] read(long address, int length) { return cpu.readMemory(program.getAddressFactory().getDefaultAddressSpace().getAddress(address), length); }
    public void write(long address, byte[] bytes) { cpu.writeMemory(program.getAddressFactory().getDefaultAddressSpace().getAddress(address), bytes); }
    public void value(long address, int width, long value) { cpu.writeMemoryValue(program.getAddressFactory().getDefaultAddressSpace().getAddress(address), width, value); }
    public int word(long address) { return ByteBuffer.wrap(read(address, 2)).getShort(); }
    public long pointer(long address) { return Integer.toUnsignedLong(ByteBuffer.wrap(read(address, 4)).getInt()); }
    public long sp() { return cpu.readRegister("SP").longValue(); }
    public long pc() { return cpu.getExecutionAddress().getOffset(); }
    public void jump(long address) { cpu.writeRegister("PC", address); }
    public void pop(int bytes) { cpu.writeRegister("SP", sp() + bytes); }
    public void ret() { long stack = sp(); jump(pointer(stack)); cpu.writeRegister("SP", stack + 4); }
    public void enter(long address, long stack, long sentinel) { write(stack - 4096, new byte[8192]); value(stack, 4, sentinel); cpu.writeRegister("SP", stack); jump(address); }
    public static String sha256(Path path) throws Exception { return HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(path))); }
}
