import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import ghidra.util.task.TaskMonitor;

/** Native music submission with explicit Sound Manager hardware replies. */
public final class SubmitMusicBoundary {
    private SubmitMusicBoundary() {}
    @FunctionalInterface public interface Draw { void run() throws Exception; }
    public static void execute(ActionOracleHarness h, TaskMonitor monitor, int busy,
            JsonArray receipts, Draw draw) throws Exception {
        if (h.pc()!=0x133d46 || (busy!=0 && busy!=1)) throw new Exception("Music entry/status contract");
        long entrySp=h.sp(),returnPc=h.pointer(entrySp),entryFrame=h.cpu.readRegister("A6").longValue();
        JsonObject function=new JsonObject();function.addProperty("entry","133d46");function.addProperty("entry_sp",entrySp);function.addProperty("return_address",returnPc);function.addProperty("argument",h.word(entrySp+4));function.addProperty("disabled_word",h.word(0x439594));function.addProperty("hardware_busy",busy);
        JsonArray calls=new JsonArray();function.add("services",calls);receipts.add(function);
        for(int steps=0;steps<20000;steps++) {
            long pc=h.pc(),sp=h.sp();
            if(pc==returnPc && sp==entrySp+4) {
                if(h.cpu.readRegister("A6").longValue()!=entryFrame)throw new Exception("Music return frame");
                function.addProperty("return_sp",sp);function.addProperty("return_frame",entryFrame);function.addProperty("d0_return_word",(short)h.cpu.readRegister("D0").intValue());return;
            }
            if(pc==0x153ca4) { JsonObject e=new JsonObject();e.addProperty("address","153ca4");e.addProperty("bound",h.word(sp+4));calls.add(e);draw.run();continue; }
            int op=h.word(pc)&65535;
            if(op==0xa800 && pc==0x133dc0) {
                long destination=h.pointer(sp);int length=h.word(sp+4);long channel=h.pointer(sp+6);
                if(length!=24 || destination!=h.cpu.readRegister("A6").longValue()-0x20 || h.cpu.readRegister("D0").longValue()!=0x05100008L)throw new Exception("SndChannelStatus ABI");
                byte[] status=new byte[24];status[12]=(byte)busy;h.write(destination,status);h.value(sp+10,2,0);h.pop(10);h.jump(pc+2);
                JsonObject e=new JsonObject();e.addProperty("address",Long.toHexString(pc));e.addProperty("service","SndChannelStatus");e.addProperty("selector","05100008");e.addProperty("destination",destination);e.addProperty("length",length);e.addProperty("channel",channel);e.addProperty("status24",VerifyStoryInventory.hex(status));e.addProperty("result_word",0);e.addProperty("sp_before",sp);e.addProperty("sp_after",h.sp());calls.add(e);continue;
            }
            if(op==0xa804 && (pc==0x133d76 || pc==0x133d9a)) {
                long command=h.pointer(sp),channel=h.pointer(sp+4);byte[] bytes=h.read(command,8);int expected=pc==0x133d76?4:3;
                if(command!=h.cpu.readRegister("A6").longValue()-8 || h.word(command)!=expected || h.word(command+2)!=0 || h.pointer(command+4)!=0)throw new Exception("SndDoImmediate ABI");
                h.value(sp+8,2,0);h.pop(8);h.jump(pc+2);
                JsonObject e=new JsonObject();e.addProperty("address",Long.toHexString(pc));e.addProperty("service","SndDoImmediate");e.addProperty("command_pointer",command);e.addProperty("channel",channel);e.addProperty("command8",VerifyStoryInventory.hex(bytes));e.addProperty("result_word",0);e.addProperty("sp_before",sp);e.addProperty("sp_after",h.sp());calls.add(e);continue;
            }
            if(op>=0xa000 && op<=0xafff || pc==0x14cff6 || pc==0x123d1c || pc==0x16474a || pc==0x13e13c)throw new Exception("Unqualified music error/playback service at "+Long.toHexString(pc));
            if(!h.cpu.step(monitor))throw new Exception("Music instruction at "+Long.toHexString(pc)+":"+h.cpu.getLastError());
        }
        throw new Exception("Music native instruction budget");
    }
}
