import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;

public class VerifyRemoveAllies extends VerifyStoryAllies {
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]);StringBuilder cases=new StringBuilder();int total=0;
        for(int count:new int[]{0,1,2,5,6,19,20})for(int wanted:new int[]{-128,-1,0,112,121,127,128})for(int layout=0;layout<4;layout++){
            byte[] before=new byte[20*288];
            for(int i=0;i<20;i++){
                for(int j=0;j<288;j++)before[i*288+j]=(byte)(i*17+j*3);
                int value=layout==0?wanted:layout==1?(i%2==0?wanted:126):layout==2?(i==4||i==5||i==19?wanted:126):126;
                before[i*288+0x103]=(byte)value;before[i*288+0x105]=(byte)(i+1);
                ByteBuffer.wrap(before).putShort(i*288+0x10c,(short)(200+i));
            }
            ActionOracleHarness h=new ActionOracleHarness(currentProgram);
            try{
                h.write(0x440edc,before);h.value(0x44257c,2,count);h.cpu.writeRegister("A6",0x700000);h.cpu.writeRegister("SP",0x6ffe00);h.value(0x6fffea,2,wanted);h.jump(0x142146);boolean done=false;
                for(int step=0;step<200000;step++){
                    long pc=h.pc();if(pc==0x14673c){done=true;break;}
                    int op=h.word(pc)&65535;
                    if(op==0xa939||op==0xa93a){h.pop(6);h.jump(pc+2);continue;}
                    if(op==0xa947){h.pop(10);h.jump(pc+2);continue;}
                    if(op==0xa940){h.pop(8);h.jump(pc+2);continue;}
                    if(!h.cpu.step(monitor))throw new Exception("Remove allies at "+Long.toHexString(pc)+": "+h.cpu.getLastError());
                }
                if(!done)throw new Exception("Remove allies instruction budget");
                if(cases.length()>0)cases.append(',');cases.append("{\"count_before\":").append(count).append(",\"count_after\":").append(h.word(0x44257c)).append(",\"class\":").append(wanted).append(",\"layout\":").append(layout).append(",\"before\":\"").append(hex(before)).append("\",\"after\":\"").append(hex(h.read(0x440edc,before.length))).append("\"}");total++;
            }finally{h.cpu.dispose();}
        }
        Files.writeString(root.resolve("evidence/fixtures/remove-allies.json"),"{\"authority\":\"Original newland opcode88 at142146..1421b2 and complete updatenpcmenu1046e0 execute. Parameter is signed class ID directly. Native copy loop below slot5 repeatedly copies only the next record, and advancing the outer index skips adjacent matches after copying. Slots5..19 decrement count without copying. Menu presentation Toolbox boundaries modeled; no RNG or file service. Full twenty records retained including stale tail. Bounded instruction evidence, not live gameplay.\",\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyRemoveAllies.java"))+"\",\"harness_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/ActionOracleHarness.java"))+"\",\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\",\"ghidra\":\"12.1.2\",\"cases\":["+cases+"]}\n");println("Remove allies "+total+" original cases exported");
    }
}
