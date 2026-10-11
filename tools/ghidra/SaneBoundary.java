import ghidra.app.emulator.EmulatorHelper;
import ghidra.program.model.address.Address;
import ghidra.program.model.listing.Program;
import java.nio.ByteBuffer;

// Bounded arithmetic model for the finite gameplay cases used by the probes.
// This is an intercepted Toolbox boundary, not execution of Apple's SANE ROM.
public final class SaneBoundary {
    final EmulatorHelper cpu; final Program program;
    public SaneBoundary(EmulatorHelper cpu,Program program){this.cpu=cpu;this.program=program;}
    Address address(long value){return program.getAddressFactory().getDefaultAddressSpace().getAddress(value);}
    byte[] read(long pointer,int size){return cpu.readMemory(address(pointer),size);}
    int word(long pointer){return Short.toUnsignedInt(ByteBuffer.wrap(read(pointer,2)).getShort());}
    long pointer(long pointer){return Integer.toUnsignedLong(ByteBuffer.wrap(read(pointer,4)).getInt());}
    double extended(long pointer){
        byte[] bytes=read(pointer,10);int signExponent=word(pointer);long fraction=ByteBuffer.wrap(bytes,2,8).getLong();
        if((signExponent&0x7fff)==0 && fraction==0)return 0;
        double significand=(double)(fraction>>>1)*2.0+(fraction&1);
        return Math.scalb(significand,(signExponent&0x7fff)-16383-63)*((signExponent&0x8000)!=0?-1:1);
    }
    void extended(long pointer,double value){
        if(!Double.isFinite(value))throw new IllegalArgumentException("Nonfinite SANE boundary");
        ByteBuffer bytes=ByteBuffer.allocate(10);
        if(value!=0){long bits=Double.doubleToRawLongBits(value);int exponent=(int)((bits>>>52)&0x7ff)-1023;
            long fraction=((bits&0xfffffffffffffL)|0x10000000000000L)<<11;
            bytes.putShort((short)((value<0?0x8000:0)|(exponent+16383)));bytes.putLong(fraction);}
        cpu.writeMemory(address(pointer),bytes.array());
    }
    double source(long pointer,int format){return switch(format){
        case 0->extended(pointer);
        case 0x1000->ByteBuffer.wrap(read(pointer,4)).getFloat();
        case 0x2000->ByteBuffer.wrap(read(pointer,2)).getShort();
        case 0x2800->ByteBuffer.wrap(read(pointer,4)).getInt();
        default->throw new IllegalArgumentException("SANE format "+Integer.toHexString(format));};}
    public void execute() {
        long sp=cpu.readRegister("SP").longValue();int opcode=word(sp),operation=opcode&0xff,format=opcode&0xff00;
        long destination=pointer(sp+2);
        if(operation==0x16) {extended(destination,(long)extended(destination));cpu.writeRegister("SP",sp+6);}
        else {
            long source=pointer(sp+6);double value=source(source,operation==0x10?0:format);
            if(operation==0x10){
                byte[] result=switch(format){
                    case 0x1000->ByteBuffer.allocate(4).putFloat((float)value).array();
                    case 0x2800->ByteBuffer.allocate(4).putInt((int)Math.rint(value)).array();
                    default->throw new IllegalArgumentException("SANE conversion "+Integer.toHexString(opcode));};
                cpu.writeMemory(address(destination),result);
            }else if(operation==0xe)extended(destination,value);
            else {
                double current=extended(destination);
                switch(operation){
                    case 0->extended(destination,current+value);
                    case 2->extended(destination,current-value);
                    case 4->extended(destination,current*value);
                    case 6->extended(destination,current/value);
                    case 8->{cpu.writeRegister("NF",current<value?1:0);cpu.writeRegister("ZF",current==value?1:0);cpu.writeRegister("VF",0);cpu.writeRegister("CF",0);}
                    default->throw new IllegalArgumentException("SANE opcode "+Integer.toHexString(opcode));
                }
            }
            cpu.writeRegister("SP",sp+10);
        }
        cpu.writeRegister("PC",cpu.getExecutionAddress().getOffset()+2);
    }
}
