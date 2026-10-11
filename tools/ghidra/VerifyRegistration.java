import ghidra.app.script.GhidraScript;
import java.nio.file.Files;
import java.nio.file.Path;

public class VerifyRegistration extends GhidraScript {
    public void run()throws Exception {
        Path root=Path.of(getScriptArgs()[0]);StringBuilder cases=new StringBuilder();
        for(long input:new long[]{0,1,0x02000000L,0x02000001L,0x01000000L}) {
            ActionOracleHarness h=new ActionOracleHarness(currentProgram);
            try {
                h.value(0x438598,4,input);h.enter(0x13a6d4,0x700000,0x900000);boolean done=false;int tests=0;
                for(int step=0;step<500;step++) {
                    long pc=h.pc(),sp=h.sp();if(pc==0x900000){done=true;break;}
                    if((h.word(pc)&65535)==0xa85d) {
                        long bit=h.pointer(sp),pointer=h.pointer(sp+4);int value=h.read(pointer+bit/8,1)[0]&255;
                        h.value(sp+8,1,(value&(128>>((int)bit%8)))!=0?1:0);h.pop(8);h.jump(pc+2);tests++;continue;
                    }
                    if(pc==0x400028)throw new Exception("Unexpected exit during stable synthetic registration");
                    if(!h.cpu.step(monitor))throw new Exception("Registration: "+h.cpu.getLastError());
                }
                if(!done)throw new Exception("Registration budget");if(cases.length()>0)cases.append(',');
                cases.append("{\"synthetic_word\":").append(input).append(",\"accepted\":").append(h.cpu.readRegister("D0").intValue()!=0).append(",\"bit_tests\":").append(tests).append(",\"unchanged\":").append(h.pointer(0x438598)==input).append(",\"rng_draws\":0}");
            }finally{h.cpu.dispose();}
        }
        Files.writeString(root.resolve("evidence/fixtures/registration.json"),"{\"authority\":\"Original doreg13a6d4 instructions on synthetic inputs only. Toolbox BitTst A85D modeled with MSB-first memorybit numbering; bit6 is firstbyte mask2 (wordmask0x02000000). Stable word tested twice before acceptance. This fixture conveys no actual registration value and does not authorize an invented acceptance state.\",\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyRegistration.java"))+"\",\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\",\"ghidra\":\"12.1.2\",\"cases\":["+cases+"]}\n");println("Synthetic registration contract exported");
    }
}
