import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;

/** Opcode 56 caller ordering, restoration and escape continuation. */
public class VerifyNpcBattleCaller extends VerifyStoryInventory {
    byte[] doors,extraCodes;
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]),scenario=root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0");doors=Files.readAllBytes(scenario.resolve("Data ED3"));extraCodes=Files.readAllBytes(scenario.resolve("Data EDCD"));party=new byte[6*3138];String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};for(int i=0;i<6;i++)System.arraycopy(Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Warriors/"+names[i])),0,party,i*3138,3138);
        StringBuilder cases=new StringBuilder();for(long seed:new long[]{12,1})for(int variant=0;variant<3;variant++)for(int result=0;result<3;result++)probe(seed,variant,result,cases);
        String identity="\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyNpcBattleCaller.java"))+"\",\"inventory_probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyStoryInventory.java"))+"\",\"harness_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/ActionOracleHarness.java"))+"\",\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\",\"extras_sha256\":\""+ActionOracleHarness.sha256(scenario.resolve("Data EDCD"))+"\",\"scripts_sha256\":\""+ActionOracleHarness.sha256(scenario.resolve("Data ED3"))+"\"";
        Files.writeString(root.resolve("evidence/fixtures/npc-battle-caller.json"),"{\"authority\":\"Original opcode56 caller145efc..146108 including native randrange13849e executes exact authored extra391 or explicit synthetic range/escape continuation variants. Combat114378, booty105748, loadland13107a and moveparty13ea82 are recorded controlled callback boundaries; combat callback overwrites all42 story bytes and local action index to establish caller restoration. No original battle117 turns/rewards/map loading or NPC selection claim. Authored ED3 supplied when escape requests script360, preserving original first8 header bytes. Six exact original warriors have varied participation bytes and controlled signed level/XP counters to establish MULS.W then EXT.L lowword truncation and ADD.L wrap. Ordinary success and script escape reach participation reset then update_main; abort and default escape return bypass that suffix. Captures complete party/header/control bytes and ordered calls.\",\"identity\":{"+identity+"},\"extra391\":\""+hex(Arrays.copyOfRange(extraCodes,3910,3920))+"\",\"cases\":["+cases+"]}\n");println("Eighteen original opcode56 caller cases exported");
    }
    void probe(long seed,int variant,int outcome,StringBuilder cases) throws Exception {
        ActionOracleHarness h=frame();RunResult result=new RunResult(seed);
        try {
            byte[] warriors=party.clone(),header=new byte[42];for(int i=0;i<42;i++)header[i]=(byte)(i*7+3);int[] levels={-32768,-1,0,1,164,32767};for(int i=0;i<6;i++){ByteBuffer.wrap(warriors).putShort(i*3138+0x3a,(short)levels[i]).putInt(i*3138+0x9ce,0xfffffff0+i);warriors[i*3138+0x9df]=(byte)new int[]{0,1,255,128,0,1}[i];}
            byte[] extra=variant==0?Arrays.copyOfRange(extraCodes,3910,3920):ByteBuffer.allocate(10).putShort((short)117).putShort((short)(variant==1?118:0)).putShort((short)(variant==2?360:-1)).putShort((short)(variant==1?6000:0)).putShort((short)(variant==1?1320:0)).array();
            h.write(0x448aa0,warriors);h.write(0x479eaa,header);h.value(0x44273a,1,5);h.value(0x442774,4,4);h.value(0x4386b2,1,0);h.value(0x4386fc,1,0);h.value(0x4427b3,1,1);h.value(0x402f43,1,2);h.value(0x442736,1,9);h.value(0x442737,1,10);h.value(0x4358c3,1,0xfe);h.value(0x4358c4,1,3);long frame=0x701000;h.write(frame-4096,new byte[8192]);h.cpu.writeRegister("A6",frame);h.cpu.writeRegister("SP",0x700000);h.value(frame-0x12,2,3);h.value(frame-0x16,2,391);h.jump(0x145efc);
            StringBuilder events=new StringBuilder();String bodyAtBooty="",partyAtBooty="";boolean done=false;
            for(int step=0;step<100000;step++){
                long pc=h.pc(),sp=h.sp();if(pc==0x14673c||pc==0x14694c){done=true;break;}String event=null;
                if(pc==0x1383e6){event="[\"extra\",391]";h.write(0x44330a,extra);}
                else if(pc==0x114378){event="[\"combat\","+h.word(0x43874e)+","+h.pointer(sp+4)+"]";h.value(0x4386b2,1,outcome==1?1:0);h.value(0x4386fc,1,outcome==2?1:0);byte[] child=new byte[42];Arrays.fill(child,(byte)0x6a);h.write(0x479eaa,child);h.value(frame-0x12,2,87);}
                else if(pc==0x105748){event="[\"booty\","+h.word(sp+4)+","+h.word(sp+6)+"]";bodyAtBooty=hex(h.read(0x479eaa,42));partyAtBooty=hex(h.read(0x448aa0,party.length));}
                else if(pc==0x13107a)event="[\"load_map\","+h.pointer(sp+4)+"]";
                else if(pc==0x13bb20)event="[\"update_main\","+h.pointer(sp+4)+"]";
                else if(pc==0x133a18)event="[\"sound\","+h.word(sp+4)+"]";
                else if(pc==0x16474a)event="[\"warning\","+h.word(sp+4)+"]";
                else if(pc==0x15dc34)event="[\"text\","+h.word(sp+6)+"]";
                else if(pc==0x13ea82)event="[\"move_party\","+h.word(sp+4)+"]";
                else if(pc==0x123c60){int id=h.word(sp+4);event="[\"load_script\","+id+"]";byte[] replacement=Arrays.copyOfRange(doors,id*42,(id+1)*42);System.arraycopy(h.read(0x479eaa,8),0,replacement,0,8);h.write(0x479eaa,replacement);}
                if(event!=null){if(events.length()>0)events.append(',');events.append(event);h.ret();continue;}
                if(pc==0x153ca4){result.rng(h);continue;}
                if(!h.cpu.step(monitor))throw new Exception("Opcode56 at "+Long.toHexString(pc)+": "+h.cpu.getLastError());
            }if(!done)throw new Exception("Opcode56 caller budget");if(cases.length()>0)cases.append(',');cases.append("{\"seed\":").append(seed).append(",\"seed_after\":").append(result.seed).append(",\"rng_draws\":").append(result.draws).append(",\"draws\":[").append(result.drawRecords).append("],\"variant\":").append(variant).append(",\"outcome\":").append(outcome).append(",\"extra\":\"").append(hex(extra)).append("\",\"battle\":").append(h.word(0x43874e)).append(",\"warriors_before\":\"").append(hex(warriors)).append("\",\"warriors_after\":\"").append(hex(h.read(0x448aa0,party.length))).append("\",\"header_before\":\"").append(hex(header)).append("\",\"header_after\":\"").append(hex(h.read(0x479eaa,42))).append("\",\"header_at_booty\":\"").append(bodyAtBooty).append("\",\"warriors_at_booty\":\"").append(partyAtBooty).append("\",\"cursor_after\":").append(h.word(frame-0x12)).append(",\"move_after\":\"").append(hex(h.read(0x442736,2))).append("\",\"terrain_context_after\":").append(h.read(0x402f43,1)[0]&255).append(",\"map_dirty_after\":").append(h.read(0x4427b3,1)[0]&255).append(",\"continuation\":\"").append(h.pc()==0x14673c?"continue":"return").append("\",\"return_value\":").append(h.cpu.readRegister("D0").intValue()).append(",\"events\":[").append(events).append("]}");
        }finally{h.cpu.dispose();}
    }
}
