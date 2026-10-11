import ghidra.app.script.GhidraScript;
import ghidra.app.decompiler.DecompInterface;
import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.HexFormat;

public class VerifyStorySkill extends GhidraScript {
    static String hex(byte[] raw){return HexFormat.of().formatHex(raw);}
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]);byte[] original=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Warriors/Coriand"));StringBuilder cases=new StringBuilder();
        int[][] skills={{8,0,0,0,0},{8,0,-1,0,0},{8,0,1,0,5},{8,45,44,1,2},{8,45,44,0,2},{8,0,32767,1,0},{8,0,-32768,-1,0},{7,-32768,32767,1,3},{7,32767,-32768,-1,3},{0,-1,0,-1,4},{8,55,55,0,1},{7,45,45,0,1}};
        for(int[] t:skills)probe(original,t[0],t[1],0,t[2],t[3],t[4],false,cases);
        for(int mode:new int[]{1,-1})for(int index=0;index<=6;index++)for(int value:new int[]{74,75,76})probe(original,index,75,mode,value,0,index%6,false,cases);
        probe(original,8,0,0,0,0,0,true,cases);
        String identity="\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyStorySkill.java"))+"\",\"harness_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/ActionOracleHarness.java"))+"\",\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\",\"warrior_sha256\":\""+ActionOracleHarness.sha256(root.resolve("NewCenturions/TheNewCenturions/Warriors/Coriand"))+"\"";
        Files.writeString(root.resolve("evidence/fixtures/story-skill.json"),"{\"authority\":\"Original newland opcode31 144a66..144bca selectedmask scanning, signedword addition/comparison, attribute mapping and loaddoor2 dispatch. getchoice UI modeled as supplied sixwordmask, loadextracode as supplied10bytes, loaddoor2 as record8..41 copy boundary. ZeroRNG; no original selection UI or copiedscript execution claim.\",\"cache_identity\":{"+identity+"},\"rng_draws\":0,\"cases\":["+cases+"]}\n");
        DecompInterface d=new DecompInterface();try{d.openProgram(currentProgram);var f=getFunctionAt(toAddr(0x12512a));if(f!=null){var out=d.decompileFunction(f,60,monitor);if(out.decompileCompleted())Files.writeString(root.resolve("analysis/ghidra/exports/getchoice.c"),out.getDecompiledFunction().getC());}}finally{d.dispose();}
        println("Story skill cases exported");
    }
    void probe(byte[] original,int index,int threshold,int mode,int value,int modifier,int selected,boolean multiple,StringBuilder out)throws Exception{
        byte[] extra=new byte[10],warrior=original.clone();ByteBuffer.wrap(extra).putShort((short)index).putShort((short)threshold).putShort((short)mode).putShort((short)299).putShort((short)300);
        if(mode==0){ByteBuffer.wrap(warrior).putShort(0xf4+index*2,(short)value).putShort(0xb62+index*2,(short)modifier);}else{int[] fields={0x9e2,0x9e4,0x9e6,0x9e8,0x9ec,0x9ee};if(index<6)ByteBuffer.wrap(warrior).putShort(fields[index],(short)value);}
        short[] mask=new short[6];mask[selected]=(short)(multiple?-2:1);if(multiple)mask[5]=1;byte[] maskBytes=new byte[12];for(int i=0;i<6;i++)ByteBuffer.wrap(maskBytes).putShort(i*2,mask[i]);
        ActionOracleHarness h=new ActionOracleHarness(currentProgram);
        try{
            h.write(0x6ff000,new byte[8192]);h.cpu.writeRegister("A6",0x700000);h.cpu.writeRegister("SP",0x6ffe00);h.value(0x6fffea,2,568);h.value(0x6fffee,2,3);h.value(0x6fffe9,1,0);h.write(0x44330a,extra);
            byte[] party=new byte[6*3138];for(int i=0;i<6;i++)System.arraycopy(original,0,party,i*3138,3138);System.arraycopy(warrior,0,party,selected*3138,3138);h.write(0x448aa0,party);h.write(0x4434e0,maskBytes);h.jump(0x144a66);int branch=-1,choiceCalls=0,zero=0,choiceMode=-1;boolean complete=false;
            for(int s=0;s<3000;s++){
                long pc=h.pc();if(pc==0x14673c){complete=true;break;}
                if(pc==0x12512a){choiceCalls++;zero=(int)h.pointer(h.sp()+4);choiceMode=h.word(h.sp()+8);h.ret();continue;}
                if(pc==0x1383e6){h.ret();continue;}
                if(pc==0x123c60){branch=h.word(h.sp()+4);h.ret();continue;}
                if(!h.cpu.step(monitor))throw new Exception("Skill PC "+Long.toHexString(pc)+": "+h.cpu.getLastError());
            }
            if(!complete)throw new Exception("Skill budget");if(out.length()>0)out.append(',');out.append("{\"extra\":\"").append(hex(extra)).append("\",\"warrior\":\"").append(hex(warrior)).append("\",\"selected_mask\":\"").append(hex(maskBytes)).append("\",\"selected_index\":").append((short)h.cpu.readRegister("D3").intValue()).append(",\"branch\":").append(branch).append(",\"cursor_after\":").append(h.word(0x6fffee)).append(",\"mask_after\":\"").append(hex(h.read(0x4434e0,12))).append("\",\"party_unchanged\":").append(Arrays.equals(party,h.read(0x448aa0,party.length))).append(",\"choice_calls\":").append(choiceCalls).append(",\"choice_zero\":").append(zero).append(",\"choice_mode\":").append(choiceMode).append('}');
        }finally{h.cpu.dispose();}
    }
}
