import ghidra.app.script.GhidraScript;
import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.HexFormat;

public class VerifySlowReduction extends GhidraScript {
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]);HexFormat hex=HexFormat.of();
        byte[] before=new byte[6*3138];String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};
        int[] counters={-1,0,1,2,29,32767};
        for(int i=0;i<6;i++){
            byte[] warrior=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Warriors/"+names[i]));
            ByteBuffer b=ByteBuffer.wrap(warrior);b.putShort(0x62,(short)counters[i]);b.putShort(0x50,b.getShort(0x52));
            System.arraycopy(warrior,0,before,i*3138,3138);
        }
        ActionOracleHarness h=new ActionOracleHarness(currentProgram);
        try {
            h.write(0x448aa0,before);h.write(0x442562,new byte[20]);h.write(0x440edc,new byte[6*288]);
            h.value(0x44273a,1,5);h.value(0x438776,2,1);h.value(0x44274a,1,0);h.value(0x442739,1,0);h.value(0x442750,1,0);h.value(0x44257c,2,0);
            h.enter(0x14de04,0x700000,0x900000);boolean done=false;
            for(int step=0;step<100000;step++){
                long pc=h.pc();if(pc==0x900000){done=true;break;}
                if(pc==0x14de26){h.jump(0x14dede);continue;}
                if(pc==0x15f66c||pc==0x1600fc||pc==0x158454){h.ret();continue;}
                if(pc==0x153ca4)throw new Exception("Unexpected reduction RNG");
                if(!h.cpu.step(monitor))throw new Exception("Reduction at "+Long.toHexString(pc)+": "+h.cpu.getLastError());
            }
            if(!done)throw new Exception("Slow reduction instruction budget");
            Files.writeString(root.resolve("evidence/fixtures/slow-reduction.json"),"{\"authority\":\"Original reduce14de04 full state body, presentation-only buff animation and updatechar calls omitted. Six original warriors at full health with slow signedword62 counters-1,0,1,2,29,32767; no RNG. Validates existing generic positive-duration reduction for recovered slow effect.\",\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifySlowReduction.java"))+"\",\"before\":\""+hex.formatHex(before)+"\",\"after\":\""+hex.formatHex(h.read(0x448aa0,before.length))+"\",\"rng_draws\":0}\n");
            println("Six signed slow duration cases exported");
        }finally{h.cpu.dispose();}
    }
}
