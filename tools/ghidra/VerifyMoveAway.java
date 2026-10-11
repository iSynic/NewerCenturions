import ghidra.app.script.GhidraScript;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.HexFormat;

public class VerifyMoveAway extends GhidraScript {
    Path root;StringBuilder cases=new StringBuilder();
    int[] flags={0x442755,0x44273b,0x442756,0x442753,0x442757,0x442758};
    String hex(byte[] bytes){return HexFormat.of().formatHex(bytes);}
    void step(ActionOracleHarness h)throws Exception{if(!h.cpu.step(monitor))throw new Exception("Move-away PC "+Long.toHexString(h.pc())+": "+h.cpu.getLastError());}
    void runCase(boolean caller,int answer,int x,int y,int dx,int dy,int floor,int variant)throws Exception{
        ActionOracleHarness h=new ActionOracleHarness(currentProgram);
        byte[] source=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data DD"));
        byte[] script=Arrays.copyOfRange(source,4200+6*42,4200+7*42),beforeFlags=new byte[6];
        try{
            h.write(0x6fc000,new byte[24576]);h.cpu.writeRegister("A6",0x700000);h.cpu.writeRegister("SP",0x6ffe00);
            h.value(0x442764,4,x);h.value(0x442768,4,y);h.value(0x442778,4,45-x-dx);h.value(0x44277c,4,45-y-dy);
            h.value(0x442736,1,dx);h.value(0x442737,1,dy);h.value(0x4085b8+(45*90+45)*2,2,floor);
            h.value(0x4434b6,2,123);h.value(0x442754,1,77);h.value(0x4358c3,1,12);h.value(0x4358c4,1,13);
            for(int i=0;i<6;i++){beforeFlags[i]=(byte)(variant==0?0:i+1);h.value(flags[i],1,beforeFlags[i]);}
            h.write(0x479eaa,script);h.write(0x478e42+6*42,script);h.value(0x6fffea,2,124);h.value(0x6fffee,2,2);
            h.value(0x41f716,4,0);int controls=0,center=0,delay=0,movement=0,loads=0;StringBuilder labels=new StringBuilder();
            if(caller)h.jump(0x14299a);else{h.enter(0x13ea82,0x6ffe00,0x900000);h.value(0x6ffe04,2,-1);}
            long stop=caller?(answer==0?0x14694c:0x14673c):0x900000;
            for(int s=0;s<5000;s++){
                long pc=h.pc(),sp=h.sp();if(pc==stop)break;
                if(pc==0x1383e6){byte[] extra=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data EDCD"));h.write(0x44330a,Arrays.copyOfRange(extra,1240,1250));loads++;h.ret();continue;}
                if(pc==0x14d21a){labels.append('[').append(h.word(sp+4)).append(',').append(h.word(sp+6)).append(']');h.cpu.writeRegister("D0",answer);h.ret();continue;}
                if(pc==0x1500d8){h.ret();continue;}
                if(pc==0x13ea82)movement++;
                if(pc==0x400038){h.cpu.writeRegister("D0",(short)Math.abs(h.word(sp+4)));h.ret();continue;}
                if(pc==0x16043e){controls++;h.ret();continue;}
                if(pc==0x10f186){center++;h.ret();continue;}
                if(pc==0x1205f8){delay++;h.ret();continue;}
                int op=h.word(pc)&65535;
                if(op==0xa873){h.pop(4);h.jump(pc+2);continue;}
                if(op==0xa975){h.value(sp,4,100);h.jump(pc+2);continue;}
                step(h);
            }
            if(h.pc()!=stop)throw new Exception("Move-away budget "+Long.toHexString(h.pc()));
            byte[] afterFlags=new byte[6];for(int i=0;i<6;i++)afterFlags[i]=h.read(flags[i],1)[0];
            if(cases.length()>0)cases.append(',');cases.append("{\"caller\":").append(caller).append(",\"answer\":").append(answer!=0).append(",\"x\":").append(x).append(",\"y\":").append(y).append(",\"dx\":").append(dx).append(",\"dy\":").append(dy).append(",\"floor\":").append(floor).append(",\"flags_before\":\"").append(hex(beforeFlags)).append("\",\"flags_after\":\"").append(hex(afterFlags)).append("\",\"x_after\":").append((int)h.pointer(0x442764)).append(",\"y_after\":").append((int)h.pointer(0x442768)).append(",\"floor_after\":").append(h.word(0x4434b6)).append(",\"mark_after\":").append(h.read(0x442754,1)[0]).append(",\"saved_delta\":\"").append(hex(h.read(0x4358c3,2))).append("\",\"working_before\":\"").append(hex(script)).append("\",\"working_after\":\"").append(hex(h.read(0x479eaa,42))).append("\",\"header_after\":\"").append(hex(h.read(0x478e42+6*42,42))).append("\",\"controls\":").append(controls).append(",\"center\":").append(center).append(",\"delay\":").append(delay).append(",\"movement_calls\":").append(movement).append(",\"extra_loads\":").append(loads).append(",\"question_arguments\":").append(labels.length()==0?"[]":labels).append(",\"cursor_after\":").append(h.word(0x6fffee)).append(",\"return\":").append(h.cpu.readRegister("D0").intValue()).append('}');
        }finally{h.cpu.dispose();}
    }
    public void run()throws Exception{
        root=Path.of(getScriptArgs()[0]);
        for(int variant=0;variant<2;variant++)for(int[] d:new int[][]{{0,1},{1,0},{1,1},{-1,-1},{0,0},{-128,127}})for(int floor:new int[]{-32768,-17,0,106})runCase(false,0,68,83,d[0],d[1],floor,variant);
        runCase(false,0,Integer.MIN_VALUE,Integer.MAX_VALUE,1,-1,-17,1);
        for(int answer=0;answer<2;answer++)for(int[] d:new int[][]{{0,1},{1,0},{-1,-1}})runCase(true,answer,68,83,d[0],d[1],-17,1);
        Files.writeString(root.resolve("evidence/fixtures/move-away.json"),"{\"authority\":\"Original moveparty13ea82..13ebd0 with signed-byte deltas and signed32 wrapping coordinates; opcode3 extra124 caller14299a..142a04 through immediate return1 on No, Continue on Yes. UI question, text reset, repaint, controls, delay, TickCount and abs-short C boundaries intercepted; actual moveparty executes. Original working42 and persisted DD42 compared unchanged. Floor read is sampled at current view position plus next delta, including view-origin offset; no collision check or regional RNG. Live UI, transport-state meaning and complete movement command excluded.\",\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyMoveAway.java"))+"\",\"harness_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/ActionOracleHarness.java"))+"\",\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\",\"draws\":0,\"cases\":["+cases+"]}\n");println("Move-away cases exported");
    }
}
