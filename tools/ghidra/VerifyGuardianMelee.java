import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;

/** Original high guardian melee and natural poison against warriors and monsters. */
public class VerifyGuardianMelee extends VerifyAllyTurnsAudio {
    StringBuilder results,calls,steps;
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]),original=root.resolve("NewCenturions/TheNewCenturions"),scenario=original.resolve("Scenarios/The Thunder of Guns/0");
        items=Files.readAllBytes(original.resolve("Data Files/Data ID"));scenarioItems=Files.readAllBytes(scenario.resolve("Data NI"));races=Files.readAllBytes(original.resolve("Data Files/Data Race"));monsters=Files.readAllBytes(scenario.resolve("Data MD"));properties=Files.readAllBytes(original.resolve("Data Files/Data Swamp BD"));attacks=Files.readAllBytes(original.resolve("Data Files/Data S"));areas=Files.readAllBytes(original.resolve("Data Files/Data AD"));party=new byte[3138*6];String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};
        for(int i=0;i<6;i++){System.arraycopy(Files.readAllBytes(original.resolve("Warriors/"+names[i])),0,party,i*3138,3138);party[i*3138+0x9df]=1;ByteBuffer.wrap(party).putShort(i*3138+0x50,(short)10000).putShort(i*3138+0x52,(short)10000);}
        StringBuilder cases=new StringBuilder(),caller=new StringBuilder();
        for(long seed:new long[]{12,1,3,7})for(int kind=0;kind<2;kind++)for(int loaded=0;loaded<2;loaded++)for(int index:new int[]{0,3})for(int variant=0;variant<8;variant++)for(int actor:new int[]{16,22})probe(seed,kind,loaded,index,variant,16,actor,true,true,0,cases);
        for(long seed:new long[]{12,1,3,7})for(int kind=0;kind<2;kind++)for(int loaded=0;loaded<2;loaded++)for(int index:new int[]{0,3})probe(seed,kind,loaded,index,0,21,22,true,false,0,cases);
        for(long seed:new long[]{12,1,3,7})for(int kind=0;kind<2;kind++)for(int index:new int[]{0,3})for(int actor:new int[]{16,22})probe(seed,kind,0,index,0,16,actor,false,false,0,cases);
        for(long seed:new long[]{12,1,3,7})for(int body:new int[]{16,21})for(int points:new int[]{3,4})probe(seed,1,1,0,0,body,22,true,false,points,caller);
        StringBuilder identity=new StringBuilder();String[][] inputs={{"probe","tools/ghidra/VerifyGuardianMelee.java"},{"audio_probe","tools/ghidra/VerifyAllyTurnsAudio.java"},{"audio_boundary","tools/ghidra/ResultAudioBoundary.java"},{"parent_probe","tools/ghidra/VerifyAllyTurns.java"},{"inventory_probe","tools/ghidra/VerifyStoryInventory.java"},{"harness","tools/ghidra/ActionOracleHarness.java"},{"sane_boundary","tools/ghidra/SaneBoundary.java"},{"mapped_binary","analysis/ghidra/startup/code1-mapped.bin"},{"monsters","NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data MD"},{"items","NewCenturions/TheNewCenturions/Data Files/Data ID"},{"scenario_items","NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data NI"},{"races","NewCenturions/TheNewCenturions/Data Files/Data Race"},{"properties","NewCenturions/TheNewCenturions/Data Files/Data Swamp BD"},{"attacks","NewCenturions/TheNewCenturions/Data Files/Data S"},{"areas","NewCenturions/TheNewCenturions/Data Files/Data AD"}};
        for(String[] input:inputs){if(identity.length()>0)identity.append(',');identity.append('"').append(input[0]).append("_sha256\":\"").append(ActionOracleHarness.sha256(root.resolve(input[1]))).append('"');}for(int i=0;i<6;i++)identity.append(",\"warrior").append(i).append("_sha256\":\"").append(ActionOracleHarness.sha256(original.resolve("Warriors/"+names[i]))).append('"');
        Files.writeString(root.resolve("evidence/fixtures/guardian-melee.json"),"{\"authority\":\"Complete original attack2101eb6 uses authoredDataMD29 highguardian defaultweapon187/naturalcondition5, targetwarrior0 or authoredMD32body22. Seeds12/1/3/7, current0/187, indices0/3, retainedactor16/22, plusattacker21retained22. Native loadweapon187 firstband baseDataID supplies explicit preloadedworkcontext; separate coldworkzero cases independently compare retainedwork context. Full13monsters,6warriors,58bytepacket,loaded150byteweapon,positions,map,orderedRNG/audio/results captured. Controlled save100,negativecondition,positive28/29warrior or126/127monster,warriorrace3/monstersaveMIN,forcedmiss anddamagearmor95 variants. Native resultaudio0/-1 andpoisoncode-28 execute; remainingpresentation acknowledged. Sixteen complete checkforenemy110e96 callers capture loaded guardian reactions with activebody22 and actionpoints3/4, then original counterattacks. No live claim.\",\"identity\":{"+identity+"},\"cases\":["+cases+"],\"caller_cases\":["+caller+"]}\n");println("576 original guardian melee cases and16 reaction callers exported");
    }
    void probe(long seed,int kind,int loaded,int index,int variant,int body,int actor,boolean preload,boolean sentinel,int points,StringBuilder output)throws Exception {
        ActionOracleHarness h=frame();RunResult result=new RunResult(seed);results=new StringBuilder();calls=new StringBuilder();steps=new StringBuilder();
        try {
            byte[] source=monster(29,10000),records=new byte[13*288],warriors=party.clone(),map=terrain(),positions=new byte[13*4];
            for(int i=0;i<13;i++){byte[] record=monster(32,10000);System.arraycopy(record,0,records,i*288,288);ByteBuffer.wrap(positions).putShort(i*4,(short)(40+i)).putShort(i*4+2,(short)40);}
            ByteBuffer.wrap(source).putShort(0x4e,(short)(loaded!=0?187:0));if(points!=0){source[0x104]=1;source[0x106]=24;source[0x109]=7;}System.arraycopy(source,0,records,(body-10)*288,288);
            if(sentinel&&actor!=body)ByteBuffer.wrap(records).putShort((actor-10)*288+0x4e,(short)1);
            if(points!=0){records[12*288+0x106]=24;records[12*288+0x109]=(byte)points;}
            int target=kind==0?0:22,offset=kind==0?0:(target-10)*288;byte[] victim=kind==0?warriors:records;ByteBuffer v=ByteBuffer.wrap(victim);
            int condition=kind==0?0x64:0xb5,save=kind==0?0x114:0x8c;
            if(variant==1){if(kind==0)v.putShort(offset+save,(short)100);else victim[offset+save]=100;}
            if(variant==2){if(kind==0)v.putShort(offset+condition,(short)-1);else victim[offset+condition]=(byte)-1;}
            if(variant==3){if(kind==0)v.putShort(offset+condition,(short)28);else victim[offset+condition]=126;}
            if(variant==4){if(kind==0)v.putShort(offset+condition,(short)29);else victim[offset+condition]=127;}
            if(variant==5){if(kind==0)v.putShort(offset+0x32,(short)3);else victim[offset+save]=(byte)-100;}
            if(variant==6)v.putShort(offset+(kind==0?0xa:8),(short)32767);
            if(variant==7)v.putShort(offset+(kind==0?0x2c:8),(short)95);
            ByteBuffer.wrap(positions).putShort((body-10)*4,(short)20).putShort((body-10)*4+2,(short)20).putShort(12*4,(short)21).putShort(12*4+2,(short)20);
            state(h,records,map);h.write(0x448aa0,warriors);h.value(0x43874a,2,13);h.value(0x4435c4,2,13);h.value(0x447196,2,actor-10);h.value(0x4435d4,2,index);h.write(0x446f0a,positions);h.value(0x44709a,2,21);h.value(0x44709c,2,20);
            if(preload){h.enter(0x130d38,0x6ffe00,0x900000);h.value(0x6ffe04,2,187);super.execute(h,result,0x900000);}
            byte[] order=new byte[220];Arrays.fill(order,(byte)255);ByteBuffer.wrap(order).putShort(2,(short)22);h.write(0x4470b8,order);if(points!=0){ByteBuffer.wrap(map).putShort((25*90+25)*2,(short)body).putShort((26*90+25)*2,(short)22);h.write(0x4085b8,map);}
            byte[] weaponBefore=h.read(0x453768,150);h.enter(points==0?0x101eb6:0x110e96,0x6ffe00,0x900000);h.value(0x6ffe04,2,points==0?body:0);h.value(0x6ffe06,2,target);h.value(0x6ffe08,2,index);execute(h,result,0x900000);
            if(output.length()>0)output.append(',');output.append("{\"seed_before\":").append(seed).append(",\"target_kind\":").append(kind).append(",\"loaded\":").append(loaded).append(",\"index\":").append(index).append(",\"variant\":").append(variant).append(",\"points\":").append(points).append(",\"source\":").append(body).append(",\"target\":").append(target).append(",\"actor\":").append(actor).append(",\"preloaded_work\":").append(preload).append(",\"actor_sentinel\":").append(sentinel).append(",\"records_before\":\"").append(hex(records)).append("\",\"records_after\":\"").append(hex(h.read(0x439d3c,records.length))).append("\",\"warriors_before\":\"").append(hex(warriors)).append("\",\"warriors_after\":\"").append(hex(h.read(0x448aa0,warriors.length))).append("\",\"positions_before\":\"").append(hex(positions)).append("\",\"positions_after\":\"").append(hex(h.read(0x446f0a,positions.length))).append("\",\"map_before\":\"").append(hex(map)).append("\",\"map_after\":\"").append(hex(h.read(0x4085b8,map.length))).append("\",\"weapon_before\":\"").append(hex(weaponBefore)).append("\",\"weapon_after\":\"").append(hex(h.read(0x453768,150))).append("\",\"packet_after\":\"").append(hex(h.read(0x44374e,58))).append("\",\"result\":").append((short)h.cpu.readRegister("D0").intValue()).append(",\"seed_after\":").append(result.seed).append(",\"rng_draws\":").append(result.draws).append(",\"draws\":[").append(result.drawRecords).append("],\"sounds\":[").append(result.sounds).append("],\"results\":[").append(results).append("],\"calls\":[").append(calls).append("],\"attack_steps\":[").append(steps).append("],\"order_after\":\"").append(hex(h.read(0x4470b8,220))).append("\"}");
        }finally{h.cpu.dispose();}
    }
    @Override void execute(ActionOracleHarness h,RunResult result,long stop)throws Exception {
        for(int step=0;step<200000;step++){
            long pc=h.pc(),sp=h.sp();if(pc==stop)return;
            if(pc==0x101eb6){int source=h.word(sp+4),target=h.word(sp+6),index=h.word(sp+8),actor=h.word(0x447196)+10;long actorWord=0x439d3c+(actor-10)*288+0x4e;int current=h.word(actorWord),primary=h.word(0x439d3c+(source-10)*288+0x26),fallback=h.word(0x439d3c+(source-10)*288+0x28);long seed=result.seed;int draws=result.draws;
                if(calls.length()>0)calls.append(',');calls.append('[').append(source).append(',').append(target).append(',').append(index).append(']');super.execute(h,result,h.pointer(sp));
                if(steps.length()>0)steps.append(',');steps.append("{\"source\":").append(source).append(",\"target\":").append(target).append(",\"index\":").append(index).append(",\"actor\":").append(actor).append(",\"actor_current_before\":").append(current).append(",\"actor_current_after\":").append(h.word(actorWord)).append(",\"source_primary\":").append(primary).append(",\"source_default\":").append(fallback).append(",\"loaded_class_after\":").append(h.word(0x453782)).append(",\"packet_after\":\"").append(hex(h.read(0x44374e,58))).append("\",\"seed_before\":").append(seed).append(",\"seed_after\":").append(result.seed).append(",\"draws_before\":").append(draws).append(",\"draws_after\":").append(result.draws).append('}');continue;}
            if(pc==0x153ca4){result.rng(h);continue;}if(pc==0x400038){h.cpu.writeRegister("D0",Math.abs(h.word(sp+4)));h.ret();continue;}if(pc==0x133a18||pc==0x16474a){result.sound(h);continue;}
            if(pc==0x13154c||pc==0x11f1a8||pc==0x1205f8){h.ret();continue;}
            if(!h.cpu.step(monitor))throw new Exception("Guardian caller at "+Long.toHexString(pc)+": "+h.cpu.getLastError());
        }throw new Exception("Guardian caller budget");
    }
    @Override void showresultsSound(ActionOracleHarness h,RunResult result)throws Exception {
        int code=h.word(h.sp()+6);if(results.length()>0)results.append(',');results.append('[').append(h.word(h.sp()+4)).append(',').append(code).append(',').append(h.word(h.sp()+8)).append(']');
        if(code!=-28){ResultAudioBoundary.execute(h,result,monitor);return;}
        String[] names={"D0","D1","D2","D3","D4","D5","D6","D7","A0","A1","A2","A3","A4","A5","A6","SP","PC","XF","NF","ZF","VF","CF"};java.math.BigInteger[] saved=new java.math.BigInteger[names.length];for(int i=0;i<names.length;i++)saved[i]=h.cpu.readRegister(names[i]);
        h.cpu.writeRegister("SP",0x6dd000);h.jump(0x158a26);boolean done=false;
        for(int step=0;step<20;step++){long pc=h.pc();if(pc==0x158a3e){done=true;break;}if(pc==0x133a18){result.sound(h);continue;}if((h.word(pc)&65535)==0xa884){h.pop(4);h.jump(pc+2);continue;}if(!h.cpu.step(monitor))throw new Exception("Poison resultaudio at "+Long.toHexString(pc)+": "+h.cpu.getLastError());}
        if(!done)throw new Exception("Poison resultaudio budget");for(int i=0;i<names.length;i++)h.cpu.writeRegister(names[i],saved[i]);
    }
}
