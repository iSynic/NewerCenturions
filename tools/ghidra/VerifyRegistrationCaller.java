import ghidra.app.script.GhidraScript;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.HexFormat;

public class VerifyRegistrationCaller extends GhidraScript {
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]); StringBuilder cases=new StringBuilder();
        byte[] authored=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data ED3"));
        byte[] script=Arrays.copyOfRange(authored,230*42,231*42);
        for(long value:new long[]{0,1,0x02000000L,0x02000001L,0x01000000L}) {
            ActionOracleHarness h=new ActionOracleHarness(currentProgram);
            try {
                h.write(0x6fc000,new byte[24576]); h.cpu.writeRegister("A6",0x700000); h.cpu.writeRegister("SP",0x6ffe00);
                h.value(0x438598,4,value); h.value(0x442764,4,5); h.value(0x442768,4,50);
                h.value(0x442778,4,40); h.value(0x44277c,4,-6); h.value(0x442736,1,0); h.value(0x442737,1,1);
                h.value(0x4085b8+(45*90+45)*2,2,61); h.write(0x479eaa,script); h.value(0x41f716,4,0);
                for(long flag:new long[]{0x442755,0x44273b,0x442756,0x442753,0x442757,0x442758}) h.value(flag,1,0);
                h.jump(0x141b2e); int tests=0,messages=0,moves=0,exits=0; boolean done=false,accepted=false;
                for(int step=0;step<5000;step++) {
                    long pc=h.pc(),sp=h.sp();
                    if(pc==0x14673c || pc==0x14694c) {done=true; accepted=pc==0x14673c; break;}
                    if(pc==0x123d1c) { messages++; h.ret(); continue; }
                    if(pc==0x13ea82) moves++;
                    if(pc==0x400028) {exits++; h.ret(); continue;}
                    if(pc==0x400038) {h.cpu.writeRegister("D0",(short)Math.abs(h.word(sp+4)));h.ret();continue;}
                    if(pc==0x16043e || pc==0x10f186 || pc==0x1205f8) {h.ret();continue;}
                    int opcode=h.word(pc)&65535;
                    if(opcode==0xa85d) {
                        long bit=h.pointer(sp),pointer=h.pointer(sp+4); int current=h.read(pointer+bit/8,1)[0]&255;
                        h.value(sp+8,1,(current&(128>>((int)bit%8)))!=0?1:0);h.pop(8);h.jump(pc+2);tests++;continue;
                    }
                    if(opcode==0xa975) {h.value(sp,4,100);h.jump(pc+2);continue;}
                    if(opcode==0xa873) {h.pop(4);h.jump(pc+2);continue;}
                    if(!h.cpu.step(monitor)) throw new Exception("Registration caller PC "+Long.toHexString(pc)+": "+h.cpu.getLastError());
                }
                if(!done) throw new Exception("Registration caller budget");
                if(cases.length()>0) cases.append(',');
                cases.append("{\"synthetic_word\":").append(value).append(",\"accepted\":").append(accepted)
                    .append(",\"x_after\":").append(h.pointer(0x442764)).append(",\"y_after\":").append(h.pointer(0x442768))
                    .append(",\"messages\":").append(messages).append(",\"move_calls\":").append(moves).append(",\"exit_calls\":").append(exits)
                    .append(",\"bit_tests\":").append(tests).append(",\"return\":").append(h.cpu.readRegister("D0").intValue())
                    .append(",\"script_before\":\"").append(HexFormat.of().formatHex(script)).append("\",\"script_after\":\"")
                    .append(HexFormat.of().formatHex(h.read(0x479eaa,42))).append("\",\"rng_draws\":0}");
            } finally {h.cpu.dispose();}
        }
        Files.writeString(root.resolve("evidence/fixtures/registration-caller.json"),"{\"authority\":\"Original newland opcode98 caller141b2e..141b68 plus complete stable doreg13a6d4 and moveparty13ea82 instructions on synthetic registration words. Toolbox BitTst is byte-oriented; message/drawing/delay/TickCount/abs-short/exit presentation boundaries intercepted. Effective accepted state must come separately from a verified original service receipt. No actual registration value stored.\",\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyRegistrationCaller.java"))+"\",\"harness_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/ActionOracleHarness.java"))+"\",\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\",\"ghidra\":\"12.1.2\",\"cases\":["+cases+"]}\n");
        println("Registration caller cases exported");
    }
}
