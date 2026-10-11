import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.HexFormat;

public class VerifyInactiveBattleReturn extends VerifyStoryParty {
    public void run()throws Exception{
        root=Path.of(getScriptArgs()[0]);warriors=new byte[6*3138];String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};
        for(int i=0;i<6;i++)System.arraycopy(Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Warriors/"+names[i])),0,warriors,i*3138,3138);
        StringBuilder cases=new StringBuilder();
        for(int variant=0;variant<2;variant++)for(int abort=0;abort<2;abort++)for(int skip=0;skip<2;skip++){
            ActionOracleHarness h=new ActionOracleHarness(currentProgram);
            try{
                byte[] before=warriors.clone(),body=new byte[32];for(int i=0;i<32;i++)body[i]=(byte)(i*7+3);
                for(int i=0;i<6;i++){word(before,i*3138+0x50,i%2==variant?-3:40);before[i*3138+0x9df]=(byte)new int[]{0,1,255,128,0,1}[i];}
                h.write(0x448aa0,before);h.write(0x479eb2,body);h.value(0x479ed2,1,7);h.value(0x44273a,1,5);h.value(0x442774,4,1);h.value(0x4386b2,1,0);h.value(0x435658,1,skip);
                long frame=0x701000;h.write(frame-4096,new byte[8192]);h.cpu.writeRegister("A6",frame);h.cpu.writeRegister("SP",0x700000);h.value(frame-0x12,2,3);h.value(frame-0x16,2,141);h.jump(0x142836);
                StringBuilder events=new StringBuilder();boolean done=false;
                for(int step=0;step<100000;step++){
                    long pc=h.pc(),sp=h.sp();if(pc==0x14673c||pc==0x14694c){done=true;break;}
                    String event=null;int arg=0;
                    if(pc==0x1383e6){h.write(0x44330a,extra(25,0,0,0,0));h.ret();continue;}
                    if(pc==0x114378){event="combat";arg=(int)h.pointer(sp+4);h.value(0x4386b2,1,abort);h.write(0x479eb2,new byte[32]);h.value(0x479ed2,1,99);h.value(frame-0x12,2,87);}
                    if(pc==0x105748){event="booty";arg=h.word(sp+4);}
                    if(pc==0x13107a){event="land";arg=(int)h.pointer(sp+4);}
                    if(pc==0x13bb20){event="update";arg=(int)h.pointer(sp+4);}
                    if(event!=null){if(events.length()>0)events.append(',');events.append("[\"").append(event).append("\",").append(arg).append(']');h.ret();continue;}
                    if(!h.cpu.step(monitor))throw new Exception("Ordinary return PC "+Long.toHexString(pc)+": "+h.cpu.getLastError());
                }
                if(!done)throw new Exception("Ordinary return budget");if(cases.length()>0)cases.append(',');cases.append("{\"variant\":").append(variant).append(",\"aborted\":").append(abort!=0).append(",\"skip_booty\":").append(skip!=0).append(",\"before\":\"").append(hex.formatHex(before)).append("\",\"after\":\"").append(hex.formatHex(h.read(0x448aa0,before.length))).append("\",\"body_before\":\"").append(hex.formatHex(body)).append("\",\"body_after\":\"").append(hex.formatHex(h.read(0x479eb2,32))).append("\",\"level_after\":").append(h.read(0x479ed2,1)[0]).append(",\"cursor_after\":").append(h.word(frame-0x12)).append(",\"events\":[").append(events).append("],\"draws\":0}");
            }finally{h.cpu.dispose();}
        }
        Files.writeString(root.resolve("evidence/fixtures/inactive-battle-return.json"),"{\"authority\":\"Original opcode2 caller142836..142994 and abort branch14694c. Combat and booty are intercepted service boundaries; their internal lifecycle is outside this fixture. Original non-abort caller sets every warrior participation byte9df=1 after booty, restores32 actionbody bytes plus level byte40 and cursor, with no health or RNG changes. Abort bypasses restoration. Loadland/update are presentation/persistence boundaries. Two alternating unconscious party patterns, varied participation bytes, abort and booty-skip flags.\",\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyInactiveBattleReturn.java"))+"\",\"harness_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/ActionOracleHarness.java"))+"\",\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\",\"cases\":["+cases+"]}\n");println("Eight ordinary battle return cases exported");
    }
}
