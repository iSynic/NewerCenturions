import ghidra.app.script.GhidraScript;
import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;

public class VerifyStoryDelay extends GhidraScript {
    void step(ActionOracleHarness h)throws Exception {if(!h.cpu.step(monitor))throw new Exception("Story delay PC "+Long.toHexString(h.pc())+": "+h.cpu.getLastError());}
    String caller(byte[] script,int slot,int parameter)throws Exception {
        ActionOracleHarness h=new ActionOracleHarness(currentProgram);
        try {
            h.write(0x6ff000,new byte[8192]);h.cpu.writeRegister("A6",0x700000);h.cpu.writeRegister("SP",0x700500);
            h.write(0x479eaa,script);h.value(0x6fffee,2,slot);h.value(0x44330a,2,16384);
            byte[][] before={h.read(0x448aa0,18828),h.read(0x4085b8,16200),h.read(0x40c500,16200),h.read(0x478e42,4200),h.read(0x4428de,16),h.read(0x44258c,128)};
            long[] addresses={0x448aa0,0x4085b8,0x40c500,0x478e42,0x4428de,0x44258c};
            h.jump(0x140336);int delays=0,argument=0,next=-1;boolean done=false;
            for(int i=0;i<200;i++) {
                long pc=h.pc();if(pc==0x140336&&delays==1){next=h.word(0x6fffee);done=true;break;}
                if(pc==0x1205f8){argument=h.word(h.sp()+4);delays++;h.ret();continue;}
                if(pc==0x153ca4)throw new Exception("Unexpected delay RNG");step(h);
            }
            if(!done||delays!=1||argument!=parameter||next!=slot+1)throw new Exception("Delay dispatch/continuation");
            for(int i=0;i<before.length;i++)if(!Arrays.equals(before[i],h.read(addresses[i],before[i].length)))throw new Exception("Delay changed rules bytes");
            return "{\"slot\":"+slot+",\"parameter\":"+parameter+",\"delay_argument\":"+argument+",\"next_slot\":"+next+",\"next_opcode\":"+ByteBuffer.wrap(script).getShort(8+next*2)+",\"next_parameter\":"+ByteBuffer.wrap(script).getShort(24+next*2)+",\"rng_draws\":0,\"rules_unchanged\":true}";
        }finally {h.cpu.dispose();}
    }
    String loop(int parameter,int defaultTicks,int variant)throws Exception {
        ActionOracleHarness h=new ActionOracleHarness(currentProgram);
        try {
            h.enter(0x1205f8,0x700000,0x900000);h.value(0x700004,2,parameter);h.value(0x439568,2,defaultTicks);
            int ticks=0,steps=0;long start=variant==1?0xfffffff0L:100;boolean done=false;
            for(;steps<100000;steps++) {
                long pc=h.pc();if(pc==0x900000){done=true;break;}
                if((h.word(pc)&65535)==0xa975){long value=variant==2&&ticks>0?start+0xffff8001L:start+ticks;h.value(h.sp(),4,value);ticks++;if(parameter==-1&&ticks==8)break;h.jump(pc+2);continue;}
                if(pc==0x153ca4)throw new Exception("Unexpected delay-loop RNG");step(h);
            }
            if(steps==100000)throw new Exception("Delay loop budget");
            return "{\"parameter\":"+parameter+",\"default_ticks\":"+defaultTicks+",\"variant\":"+variant+",\"tick_reads\":"+ticks+",\"returned\":"+done+",\"rng_draws\":0}";
        }finally {h.cpu.dispose();}
    }
    public void run()throws Exception {
        Path root=Path.of(getScriptArgs()[0]),scripts=root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data ED3");byte[] table=Files.readAllBytes(scripts);
        StringBuilder callers=new StringBuilder(),loops=new StringBuilder();
        for(int parameter:new int[]{-32768,-1,0,1,75,32767}){byte[] script=new byte[42];ByteBuffer.wrap(script).putShort(8,(short)73).putShort(24,(short)parameter);if(callers.length()>0)callers.append(',');callers.append(caller(script,0,parameter));}
        byte[] original=Arrays.copyOfRange(table,88*42,89*42);callers.append(',').append(caller(original,6,75));
        int[][] values={{1,7,0},{75,7,0},{0,7,0},{0,0,0},{75,7,1},{-32768,7,2},{-1,7,0}};
        for(int[] value:values){if(loops.length()>0)loops.append(',');loops.append(loop(value[0],value[1],value[2]));}
        String identity="\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyStoryDelay.java"))+"\",\"harness_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/ActionOracleHarness.java"))+"\",\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\",\"scripts_sha256\":\""+ActionOracleHarness.sha256(scripts)+"\"";
        Files.writeString(root.resolve("evidence/fixtures/story-delay.json"),"{\"authority\":\"Native newland dispatch140336 opcode73 calls delay1205f8 with direct signedword parameter then advances14673c to next slot. Actual ED3 script88 slot6 parameter75 resumes slot7 opcode39 parameter89. Rule snapshots unchanged and no RNG; presentation delay boundary intercepted. Separate native delay body1205f8..120632 uses controlled QuickDraw TickCount trapA975 values, signed parameter0 default439568, unsigned elapsed strictly greater than sign-extended duration and wrapping tick subtraction. Negative-1 cannot satisfy unsigned comparison; bounded eight-tick observation does not terminate that loop. No wallclock timing or live gameplay claim.\",\"identity\":{"+identity+"},\"caller_cases\":["+callers+"],\"loop_cases\":["+loops+"]}\n");println("Story delay seven callers and seven native loops exported");
    }
}
