import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;

public class VerifyMissionDeaths extends VerifyFinalTurnStories {
    byte[] phaseRecordsBefore,phaseRecordsAfter,phasePartyBefore,phasePartyAfter,phaseMapBefore,phaseMapAfter,phaseOrderBefore,phaseOrderAfter;
    int unavailableBefore,unavailableAfter,actorBefore,processingBefore,returned;
    void capture(ActionOracleHarness h,boolean before) {
        byte[] records=h.read(0x439d3c,100*288),warriors=h.read(0x448aa0,6*3138),map=h.read(0x4085b8,16200),order=h.read(0x4470b8,220);
        if(before){phaseRecordsBefore=records;phasePartyBefore=warriors;phaseMapBefore=map;phaseOrderBefore=order;unavailableBefore=h.read(0x442739,1)[0];actorBefore=h.word(0x43800e);processingBefore=h.read(0x438584,1)[0];}
        else{phaseRecordsAfter=records;phasePartyAfter=warriors;phaseMapAfter=map;phaseOrderAfter=order;unavailableAfter=h.read(0x442739,1)[0];}
    }
    void executeMission(ActionOracleHarness h,RunResult result,long stop) throws Exception {
        for(int step=0;step<400000;step++){
            long pc=h.pc(),sp=h.sp();if(pc==stop)return;
            if(pc==0x123c60){int id=h.word(sp+4);byte[] record=Arrays.copyOfRange(doors,id*42,(id+1)*42);System.arraycopy(h.read(0x479eaa,8),0,record,0,8);h.write(0x479eaa,record);event("[\"load_script\","+id+"]");h.ret();continue;}
            if(pc==0x1383e6){int id=h.word(sp+4);h.write(0x44330a,Arrays.copyOfRange(extraCodes,id*10,(id+1)*10));event("[\"load_extra\","+id+"]");h.ret();continue;}
            if(pc==0x1400e4){capture(h, true);event("[\"newland\","+h.word(sp+12)+","+h.word(sp+14)+","+h.word(sp+16)+"]");}
            if(pc==0x12ea7e||pc==0x12eac8){capture(h, false);returned=h.cpu.readRegister("D0").intValue();}
            if(pc==0x114378){event("[\"combat\","+h.word(0x43874e)+","+h.word(0x43800c)+"]");if(childMode!=0){h.value(0x43800c,2,0);h.value(0x437a90,2,0);byte[] mutation=new byte[42];Arrays.fill(mutation,(byte)0x6a);h.write(0x479eaa,mutation);}h.value(0x4386b2,1,abort);h.ret();continue;}
            if(pc==0x105748){event("[\"booty\","+h.word(sp+4)+","+h.word(sp+6)+"]");h.ret();continue;}
            if(pc==0x13107a){event("[\"load_map\","+(int)h.pointer(sp+4)+"]");h.ret();continue;}
            if(pc==0x13bb20){event("[\"update_main\"]");h.ret();continue;}
            if(pc==0x15dc34){event("[\"text\","+h.word(sp+6)+"]");h.ret();continue;}
            if(pc==0x14913e||pc==0x125a3a){event("[\"caller_boundary\","+pc+"]");h.ret();continue;}
            if(pc==0x149e0e)throw new Exception("Unexpected placement during death story");
            if(pc==0x153ca4){result.rng(h);continue;}
            if(pc==0x133a18||pc==0x16474a){event("[\"sound\","+h.word(sp+4)+"]");result.sound(h);continue;}
            if(pc==0x400038){h.cpu.writeRegister("D0",Math.abs(h.word(sp+4)));h.ret();continue;}
            if(pc==0x120c14||pc==0x105572||pc==0x10e6b6||pc==0x133d46||pc==0x158454||pc==0x13154c||pc==0x11f1a8||pc==0x11edc4||pc==0x14a02c||pc==0x13a2c4||pc==0x1205f8||pc==0x16043e||pc==0x1382a8||pc==0x11f136||pc==0x11f690){h.ret();continue;}
            int op=h.word(pc)&65535;
            if(op==0xa873||op==0xaa1c||op==0xa862||op==0xa863||op==0xa89b||op==0xa893||op==0xa891||op==0xa884){h.pop(4);h.jump(pc+2);continue;}
            if(op==0xaa1e){h.value(sp+2,4,0);h.pop(2);h.jump(pc+2);continue;}
            if(op==0xa032){h.jump(pc+2);continue;}
            if(!h.cpu.step(monitor))throw new Exception("Death-script action at "+Long.toHexString(pc)+": "+h.cpu.getLastError());
        }
        throw new Exception("Death-script instruction budget");
    }
    void probeMission(int id,int queued,int faction,int unavailable) throws Exception {
        ActionOracleHarness h=frame();RunResult result=new RunResult(12);events=new StringBuilder();phaseRecordsBefore=null;phaseRecordsAfter=null;returned=0;
        try{
            byte[] records=new byte[100*288];int[] classes={-128,-1,0,113,114,115,116,119,121,127};
            for(int i=0;i<100;i++){byte[] record=monster(113,20+i);record[0x103]=(byte)classes[i%10];ByteBuffer r=ByteBuffer.wrap(record);r.putShort(0xe,(short)(i%3)).putShort(0x46,(short)(200+i)).putShort(0x10c,(short)(i%4==0?13:-1)).putShort(0x110,(short)(i%4==0?0:20+i));record[0xb2]=(byte)(i+1);System.arraycopy(record,0,records,i*288,288);}
            byte[] active=monster(id,0);ByteBuffer.wrap(active).putShort(0xe,(short)faction).putShort(0x110,(short)0);for(int i=0xb2;i<0xda;i++)active[i]=(byte)0x5a;active[0x106]=100;System.arraycopy(active,0,records,3*288,288);
            state(h,records,terrain());h.value(0x43874a,2,100);h.value(0x446f0a+3*4,2,20);h.value(0x446f0c+3*4,2,20);h.write(0x437a94,new byte[1400]);h.value(0x43800c,2,0);h.value(0x437a90,4,0x12345678);h.value(0x438585,1,queued==1?1:0);h.value(0x438760,2,queued==2?1:0);h.value(0x438584,1,0);h.value(0x438583,1,0);h.value(0x43800e,2,0);h.value(0x438582,2,7);h.value(0x4386b2,1,0);h.value(0x435658,1,0);h.value(0x447194,2,1);h.value(0x442739,1,unavailable);h.value(0x44273a,1,5);h.value(0x4427b3,1,1);h.value(0x442774,4,4);h.write(0x4470b8,new byte[220]);for(int i=0;i<110;i++)h.value(0x4470b8+i*2,2,-1);h.value(0x4470ba,2,13);
            for(int i=0;i<6;i++){h.value(0x448aa0+i*3138+0x18,2,13);h.value(0x448aa0+i*3138+0x50,2,unavailable>5?-10-i:30+i);h.value(0x448aa0+i*3138+0x90,2,(i%2==0?-5:5));}
            h.enter(0x149e0e,0x6ffe00,0x900000);h.value(0x6ffe04,2,20);h.value(0x6ffe06,2,20);h.value(0x6ffe08,2,3);executeFinal(h,result,0x900000);events=new StringBuilder();
            byte[] parent=new byte[42];for(int i=0;i<42;i++)parent[i]=(byte)(0x20+i);h.write(0x479eaa,parent);h.enter(0x12e4e4,0x6ffe00,0x900000);h.value(0x6ffe04,2,13);h.value(0x6ffe06,2,0);executeMission(h,result,0x900000);int queuedCount=h.word(0x43800c);byte[] cleanup=h.read(0x439d3c,100*288),cleanupMap=h.read(0x4085b8,16200),cleanupWarriors=h.read(0x448aa0,party.length);
            if(queued!=0){h.value(0x438585,1,0);h.value(0x438760,2,0);h.enter(0x121524,0x6ffe00,0x900000);executeMission(h,result,0x900000);}
            if(phaseRecordsBefore==null||phaseRecordsAfter==null)throw new Exception("No story phase snapshots");
            if(cases.length()>0)cases.append(',');cases.append("{\"monster_id\":").append(id).append(",\"queued\":").append(queued).append(",\"faction\":").append(faction).append(",\"unavailable_before\":").append(unavailableBefore).append(",\"unavailable_after\":").append(unavailableAfter).append(",\"actor_before\":").append(actorBefore).append(",\"processing_before\":").append(processingBefore).append(",\"return_value\":").append(returned).append(",\"records_before\":\"").append(hex(phaseRecordsBefore)).append("\",\"records_after\":\"").append(hex(phaseRecordsAfter)).append("\",\"warriors_before\":\"").append(hex(phasePartyBefore)).append("\",\"warriors_after\":\"").append(hex(phasePartyAfter)).append("\",\"map_before\":\"").append(hex(phaseMapBefore)).append("\",\"map_after\":\"").append(hex(phaseMapAfter)).append("\",\"order_before\":\"").append(hex(phaseOrderBefore)).append("\",\"order_after\":\"").append(hex(phaseOrderAfter)).append("\",\"cleanup_records\":\"").append(hex(cleanup)).append("\",\"cleanup_warriors\":\"").append(hex(cleanupWarriors)).append("\",\"cleanup_map\":\"").append(hex(cleanupMap)).append("\",\"queued_count\":").append(queuedCount).append(",\"queue_count_after\":").append(h.word(0x43800c)).append(",\"actor_after\":").append(h.word(0x43800e)).append(",\"control_after\":\"").append(hex(h.read(0x438582,4))).append("\",\"rng_draws\":").append(result.draws).append(",\"events\":[").append(events).append("]}");println("Mission death "+id+"/"+queued+"/"+faction+"/"+unavailable);
        }finally{h.cpu.dispose();}
    }
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]);Path original=root.resolve("NewCenturions/TheNewCenturions"),scenario=original.resolve("Scenarios/The Thunder of Guns/0");items=Files.readAllBytes(original.resolve("Data Files/Data ID"));races=Files.readAllBytes(original.resolve("Data Files/Data Race"));scenarioItems=Files.readAllBytes(scenario.resolve("Data NI"));monsters=Files.readAllBytes(scenario.resolve("Data MD"));properties=Files.readAllBytes(original.resolve("Data Files/Data Swamp BD"));attacks=Files.readAllBytes(original.resolve("Data Files/Data S"));areas=Files.readAllBytes(original.resolve("Data Files/Data AD"));doors=Files.readAllBytes(scenario.resolve("Data ED3"));extraCodes=Files.readAllBytes(scenario.resolve("Data EDCD"));party=new byte[6*3138];String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};for(int i=0;i<6;i++)System.arraycopy(Files.readAllBytes(original.resolve("Warriors/"+names[i])),0,party,i*3138,3138);
        for(int id:new int[]{119,121,117})for(int queued:new int[]{0,1,2})for(int faction:new int[]{0,2})for(int unavailable:new int[]{-128,-1,0,5,6,127})probeMission(id,queued,faction,unavailable);
        StringBuilder identity=new StringBuilder();String[] paths={"tools/ghidra/VerifyMissionDeaths.java","tools/ghidra/VerifyFinalTurnStories.java","tools/ghidra/VerifyDeathScripts.java","tools/ghidra/VerifyAllyTurns.java","tools/ghidra/VerifyStoryInventory.java","tools/ghidra/ActionOracleHarness.java","analysis/ghidra/startup/code1-mapped.bin","NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data MD","NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data ED3","NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data EDCD","NewCenturions/TheNewCenturions/Data Files/Data Swamp BD"};String[] labels={"probe_sha256","turn_probe_sha256","parent_probe_sha256","ally_probe_sha256","inventory_probe_sha256","harness_sha256","mapped_binary_sha256","monsters_sha256","scripts_sha256","extras_sha256","properties_sha256"};for(int i=0;i<paths.length;i++){if(identity.length()>0)identity.append(',');identity.append('"').append(labels[i]).append("\":\"").append(ActionOracleHarness.sha256(root.resolve(paths[i]))).append('"');}
        Files.writeString(root.resolve("evidence/fixtures/mission-deaths.json"),"{\"authority\":\"Complete original killbody12e4e4 direct or deferred emptyque121524 executes authored MD119/121/117 death336/337 and native newland119/123. Actor3 direct versus retained globalactor0 queued; signed unavailable counters and faction0/2, signed classes and all100 record slots including HPzero. Full native before/after story snapshots and caller cleanup retained. File-service exact records supplied, UI/sound/text and final party-loss/getup services acknowledged as ordered boundaries. No RNG draws or live gameplay claim.\",\"identity\":{"+identity+"},\"scripts\":\""+hex(doors)+"\",\"extras\":\""+hex(extraCodes)+"\",\"cases\":["+cases+"]}\n");println("Mission death cases exported");
    }
}
