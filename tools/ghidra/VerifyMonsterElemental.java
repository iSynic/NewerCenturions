import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;

/** Original MD120 elemental attacks against scientist records and adjacent caller. */
public class VerifyMonsterElemental extends VerifyAllyTurnsAudio {
    StringBuilder calls,results;
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]),original=root.resolve("NewCenturions/TheNewCenturions"),scenario=original.resolve("Scenarios/The Thunder of Guns/0");
        items=Files.readAllBytes(original.resolve("Data Files/Data ID"));scenarioItems=Files.readAllBytes(scenario.resolve("Data NI"));races=Files.readAllBytes(original.resolve("Data Files/Data Race"));monsters=Files.readAllBytes(scenario.resolve("Data MD"));properties=Files.readAllBytes(original.resolve("Data Files/Data Swamp BD"));attacks=Files.readAllBytes(original.resolve("Data Files/Data S"));areas=Files.readAllBytes(original.resolve("Data Files/Data AD"));party=new byte[3138*6];String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};
        for(int i=0;i<6;i++){System.arraycopy(Files.readAllBytes(original.resolve("Warriors/"+names[i])),0,party,i*3138,3138);party[i*3138+0x9df]=1;ByteBuffer.wrap(party).putShort(i*3138+0x50,(short)10000).putShort(i*3138+0x52,(short)10000);}
        StringBuilder direct=new StringBuilder(),caller=new StringBuilder();
        for(long seed:new long[]{12,1,2,3,4,5,6,7})for(int index=0;index<2;index++)for(int variant=0;variant<8;variant++)for(int actor:new int[]{0,2})probe(seed,index,variant,actor,0,direct);
        for(long seed:new long[]{12,1,2,3,4,5,6,7})for(int points:new int[]{3,4})probe(seed,0,0,2,points,caller);
        StringBuilder identity=new StringBuilder();String[][] inputs={{"probe","tools/ghidra/VerifyMonsterElemental.java"},{"audio_probe","tools/ghidra/VerifyAllyTurnsAudio.java"},{"parent_probe","tools/ghidra/VerifyAllyTurns.java"},{"inventory_probe","tools/ghidra/VerifyStoryInventory.java"},{"harness","tools/ghidra/ActionOracleHarness.java"},{"sane_boundary","tools/ghidra/SaneBoundary.java"},{"mapped_binary","analysis/ghidra/startup/code1-mapped.bin"},{"monsters","NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data MD"},{"items","NewCenturions/TheNewCenturions/Data Files/Data ID"},{"scenario_items","NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data NI"},{"races","NewCenturions/TheNewCenturions/Data Files/Data Race"},{"properties","NewCenturions/TheNewCenturions/Data Files/Data Swamp BD"},{"attacks","NewCenturions/TheNewCenturions/Data Files/Data S"},{"areas","NewCenturions/TheNewCenturions/Data Files/Data AD"}};
        for(String[] input:inputs){if(identity.length()>0)identity.append(',');identity.append('"').append(input[0]).append("_sha256\":\"").append(ActionOracleHarness.sha256(root.resolve(input[1]))).append('"');}
        Files.writeString(root.resolve("evidence/fixtures/monster-elemental.json"),"{\"authority\":\"Complete original attack2101eb6 MD120 naturalcondition14 against scientist65 for firsttwo native attackslots. Eightseeds, retainedactor0/2, save4=100, protectionbe=1/-1, bothhalves, forcedmiss, HP1 and controlledlevel0 zerofinaldamage variants. Full three monster records, whole unchanged six-warrior party, persistent58B packet, ordered native Rand receipts and result calls. Sixteen complete adjacent checkforenemy110e96 callers use MD120body12 with two scientist65 allies, three/fouractionpoints and movement24. Native result audio code0/-1 and elementalcode-21 executes; other UI remains an acknowledged boundary. Originaldefinitions supplied without reconstructed condition behavior. No live battle claim.\",\"identity\":{"+identity+"},\"party\":\""+hex(party)+"\",\"attack_cases\":["+direct+"],\"caller_cases\":["+caller+"]}\n");
        println("256 original monster elemental attacks and16 adjacent caller cases exported");
    }
    void probe(long seed,int index,int variant,int actor,int points,StringBuilder output)throws Exception {
        ActionOracleHarness h=frame();RunResult result=new RunResult(seed);calls=new StringBuilder();results=new StringBuilder();
        try {
            byte[] source=monster(120,10000),target=monster(65,10000),other=monster(65,10000),records=new byte[864],map=terrain(),order=new byte[220];Arrays.fill(order,(byte)255);
            if(variant==1)target[0x8f]=100;if(variant==2)target[0xbe]=1;if(variant==3){target[0x8f]=100;target[0xbe]=1;}if(variant==4)target[0xbe]=(byte)-1;if(variant==5)ByteBuffer.wrap(target).putShort(8,(short)32767);if(variant==6)ByteBuffer.wrap(target).putShort(0x110,(short)1);if(variant==7){ByteBuffer.wrap(source).putShort(0,(short)0);target[0x8f]=100;target[0xbe]=1;}
            int sourceIndex=points==0?0:2,targetIndex=points==0?1:0,otherIndex=points==0?2:1;
            source[0x106]=24;source[0x109]=(byte)(points==0?7:points);
            System.arraycopy(source,0,records,sourceIndex*288,288);System.arraycopy(target,0,records,targetIndex*288,288);System.arraycopy(other,0,records,otherIndex*288,288);
            int body=sourceIndex+10,victim=targetIndex+10;
            ByteBuffer.wrap(map).putShort((25*90+25)*2,(short)body).putShort((26*90+25)*2,(short)victim).putShort((45*90+45)*2,(short)(otherIndex+10));ByteBuffer.wrap(order).putShort(2,(short)body);
            state(h,records,map);h.value(0x43874a,2,3);h.value(0x4435c4,2,3);h.value(0x447196,2,actor);h.value(0x4435d4,2,index);h.write(0x4470b8,order);
            h.value(0x446f0a+sourceIndex*4,2,20);h.value(0x446f0c+sourceIndex*4,2,20);h.value(0x446f0a+targetIndex*4,2,21);h.value(0x446f0c+targetIndex*4,2,20);h.value(0x446f0a+otherIndex*4,2,40);h.value(0x446f0c+otherIndex*4,2,40);
            h.enter(points==0?0x101eb6:0x110e96,0x6ffe00,0x900000);h.value(0x6ffe04,2,points==0?body:0);h.value(0x6ffe06,2,victim);h.value(0x6ffe08,2,index);execute(h,result,0x900000);
            if(!Arrays.equals(party,h.read(0x448aa0,party.length)))throw new Exception("Unexpected disease party write");
            if(output.length()>0)output.append(',');output.append("{\"seed_before\":").append(seed).append(",\"index\":").append(index).append(",\"variant\":").append(variant).append(",\"actor\":").append(actor+10).append(",\"source\":").append(body).append(",\"target\":").append(victim).append(",\"points\":").append(points).append(",\"records_before\":\"").append(hex(records)).append("\",\"records_after\":\"").append(hex(h.read(0x439d3c,records.length))).append("\",\"map_before\":\"").append(hex(map)).append("\",\"map_after\":\"").append(hex(h.read(0x4085b8,map.length))).append("\",\"order_before\":\"").append(hex(order)).append("\",\"order_after\":\"").append(hex(h.read(0x4470b8,order.length))).append("\",\"positions_after\":\"").append(hex(h.read(0x446f0a,12))).append("\",\"packet_after\":\"").append(hex(h.read(0x44374e,58))).append("\",\"seed_after\":").append(result.seed).append(",\"rng_draws\":").append(result.draws).append(",\"draws\":[").append(result.drawRecords).append("],\"sounds\":[").append(result.sounds).append("],\"calls\":[").append(calls).append("],\"results\":[").append(results).append("]}");
        }finally{h.cpu.dispose();}
    }
    @Override void showresultsSound(ActionOracleHarness h,RunResult result)throws Exception {
        int code=h.word(h.sp()+6);if(results.length()>0)results.append(',');results.append('[').append(h.word(h.sp()+4)).append(',').append(code).append(',').append(h.word(h.sp()+8)).append(']');
        if(code!=-21){super.showresultsSound(h,result);return;}
        String[] names={"D0","D1","D2","D3","D4","D5","D6","D7","A0","A1","A2","A3","A4","A5","A6","SP","PC","XF","NF","ZF","VF","CF"};java.math.BigInteger[] saved=new java.math.BigInteger[names.length];for(int i=0;i<names.length;i++)saved[i]=h.cpu.readRegister(names[i]);
        h.cpu.writeRegister("SP",0x6dd000);h.jump(0x158be4);boolean done=false;
        for(int step=0;step<20;step++){long pc=h.pc();if(pc==0x158bf6){done=true;break;}if(pc==0x133a18){result.sound(h);continue;}if((h.word(pc)&65535)==0xa884){h.pop(4);h.jump(pc+2);continue;}if(!h.cpu.step(monitor))throw new Exception("Disease result audio at "+Long.toHexString(pc)+": "+h.cpu.getLastError());}
        if(!done)throw new Exception("Disease result audio instruction budget");for(int i=0;i<names.length;i++)h.cpu.writeRegister(names[i],saved[i]);
    }
    @Override void execute(ActionOracleHarness h,RunResult result,long stop)throws Exception {
        for(int step=0;step<200000;step++){
            long pc=h.pc(),sp=h.sp();if(pc==stop)return;
            if(pc==0x101eb6){if(calls.length()>0)calls.append(',');calls.append('[').append(h.word(sp+4)).append(',').append(h.word(sp+6)).append(',').append(h.word(sp+8)).append(']');super.execute(h,result,pc==0x101eb6&&h.pointer(sp)==0x900000?0x900000:h.pointer(sp));continue;}
            if(pc==0x153ca4){result.rng(h);continue;}if(pc==0x400038){h.cpu.writeRegister("D0",Math.abs(h.word(sp+4)));h.ret();continue;}if(pc==0x133a18||pc==0x16474a){result.sound(h);continue;}
            if(pc==0x13154c||pc==0x11f1a8||pc==0x1205f8){h.ret();continue;}
            if(!h.cpu.step(monitor))throw new Exception("Disease caller at "+Long.toHexString(pc)+": "+h.cpu.getLastError());
        }throw new Exception("Disease caller instruction budget");
    }
}
