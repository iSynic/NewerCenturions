import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;

/** Final encounter getup stories with original construction, placement and removal. */
public class VerifyFinalTurnStories extends VerifyDeathScripts {
    int fileOffset;
    long finish;
    void executeFinal(ActionOracleHarness h,RunResult result,long stop) throws Exception {
        SaneBoundary sane=new SaneBoundary(h.cpu,currentProgram);
        for(int step=0;step<1500000;step++){
            long pc=h.pc(),sp=h.sp();if(pc==stop||stop==0x126c3e&&pc==0x125ca0){finish=pc;return;}
            if(pc==0x123c60){int id=h.word(sp+4);byte[] record=Arrays.copyOfRange(doors,id*42,(id+1)*42);System.arraycopy(h.read(0x479eaa,8),0,record,0,8);h.write(0x479eaa,record);event("[\"load_script\","+id+"]");h.ret();continue;}
            if(pc==0x1383e6){int id=h.word(sp+4);h.write(0x44330a,Arrays.copyOfRange(extraCodes,id*10,(id+1)*10));event("[\"load_extra\","+id+"]");h.ret();continue;}
            if(pc==0x136cfa){h.ret();continue;}
            if(pc==0x400070){h.cpu.writeRegister("A0",0x600000);h.ret();continue;}
            if(pc==0x400078){fileOffset=(int)h.pointer(sp+8);h.ret();continue;}
            if(pc==0x400048){int length=(int)h.pointer(sp+8);if(length!=288)throw new Exception("Unexpected MD read length");h.write(h.pointer(sp+4),Arrays.copyOfRange(monsters,fileOffset,fileOffset+length));h.cpu.writeRegister("D0",1);h.ret();continue;}
            if(pc==0x400068){h.ret();continue;}
            if(pc==0x109210){event("[\"build\","+fileOffset/288+","+h.word(sp+4)+"]");}
            if(pc==0x12e4e4){event("[\"kill\","+h.word(sp+4)+"]");}
            if(pc==0x15dc34){event("[\"text\","+h.word(sp+6)+"]");h.ret();continue;}
            if(pc==0x15b2f6){h.ret();continue;}
            if(pc==0x153ca4){result.rng(h);continue;}
            if(pc==0x133a18||pc==0x16474a){event("[\"sound\","+h.word(sp+4)+"]");result.sound(h);continue;}
            if(pc==0x400038){h.cpu.writeRegister("D0",Math.abs(h.word(sp+4)));h.ret();continue;}
            if(pc==0x120c14||pc==0x105572||pc==0x10e6b6||pc==0x133d46||pc==0x158454||pc==0x13154c||pc==0x11f1a8||pc==0x11edc4||pc==0x14a02c||pc==0x13a2c4||pc==0x1205f8||pc==0x16043e||pc==0x1382a8||pc==0x11f136||pc==0x11f690){h.ret();continue;}
            int op=h.word(pc)&65535;
            if(op==0xa9eb){sane.execute();continue;}
            if(op==0xa8ad){long rect=h.pointer(sp);int x=h.word(sp+4),y=h.word(sp+6);h.value(sp+8,1,x>=h.word(rect)&&x<h.word(rect+4)&&y>=h.word(rect+2)&&y<h.word(rect+6)?1:0);h.pop(8);h.jump(pc+2);continue;}
            if(op==0xa873||op==0xaa1c||op==0xa862||op==0xa863||op==0xa89b||op==0xa893||op==0xa891||op==0xa884){h.pop(4);h.jump(pc+2);continue;}
            if(op==0xaa1e){h.value(sp+2,4,0);h.pop(2);h.jump(pc+2);continue;}
            if(op==0xa032){h.jump(pc+2);continue;}
            if(!h.cpu.step(monitor))throw new Exception("Final turn at "+Long.toHexString(pc)+": "+h.cpu.getLastError());
        }
        throw new Exception("Final turn instruction budget at "+Long.toHexString(h.pc()));
    }
    void probeFinal(int id,long seed,int faction,int layout,int count,boolean negative) throws Exception {
        ActionOracleHarness h=frame();RunResult result=new RunResult(seed);events=new StringBuilder();
        try{
            byte[] records=new byte[100*288],active=monster(id,100);ByteBuffer.wrap(active).putShort(0xe,(short)faction);if(negative)ByteBuffer.wrap(active).putShort(0x5c,(short)-25);System.arraycopy(active,0,records,0,288);
            byte[] map=terrain();int x=layout==2?3:20,y=x;
            if(layout==1){int blocked=-1;for(int i=0;i<300;i++)if(ByteBuffer.wrap(properties).getShort(64+i*40+4)==2){blocked=1000+i;break;}if(blocked<0)throw new Exception("No blocked tile");for(int xx=x-3;xx<=x+2;xx++)for(int yy=y-3;yy<=y+2;yy++)ByteBuffer.wrap(map).putShort(((xx+5)*90+yy+5)*2,(short)blocked);}
            state(h,records,map);h.value(0x43874a,2,count);h.value(0x446f0a,2,x);h.value(0x446f0c,2,y);h.write(0x437a94,new byte[1400]);h.value(0x43800c,2,0);h.value(0x438584,1,0);h.value(0x438583,1,0);h.value(0x438585,1,0);h.value(0x438760,2,0);h.value(0x43800e,2,0);h.value(0x4386b2,1,0);h.value(0x4427b3,1,1);h.value(0x442774,4,4);h.write(0x4470b8,new byte[220]);for(int i=0;i<110;i++)h.value(0x4470b8+i*2,2,-1);h.value(0x4470ba,2,10);h.value(0x4435c4,2,faction==0?0:1);h.value(0x44374c,2,0);
            h.enter(0x149e0e,0x6ffe00,0x900000);h.value(0x6ffe04,2,x);h.value(0x6ffe06,2,y);h.value(0x6ffe08,2,0);executeFinal(h,result,0x900000);events=new StringBuilder();
            byte[] before=h.read(0x439d3c,count*288),mapBefore=h.read(0x4085b8,16200),positionsBefore=h.read(0x446f0a,count*4);h.write(0x479eaa,new byte[42]);h.cpu.writeRegister("A6",0x700000);h.cpu.writeRegister("SP",0x6ffe00);h.jump(0x126938);executeFinal(h,result,0x126c3e);int after=h.word(0x43874a);
            if(cases.length()>0)cases.append(',');cases.append("{\"monster_id\":").append(id).append(",\"seed_before\":").append(seed).append(",\"faction\":").append(faction).append(",\"layout\":").append(layout).append(",\"count_before\":").append(count).append(",\"negative_chance\":").append(negative).append(",\"records_before\":\"").append(hex(before)).append("\",\"records_after\":\"").append(hex(h.read(0x439d3c,after*288))).append("\",\"map_before\":\"").append(hex(mapBefore)).append("\",\"map_after\":\"").append(hex(h.read(0x4085b8,16200))).append("\",\"positions_before\":\"").append(hex(positionsBefore)).append("\",\"positions_after\":\"").append(hex(h.read(0x446f0a,after*4))).append("\",\"warriors_before\":\"").append(hex(party)).append("\",\"warriors_after\":\"").append(hex(h.read(0x448aa0,party.length))).append("\",\"order_after\":\"").append(hex(h.read(0x4470b8,220))).append("\",\"count_after\":").append(after).append(",\"hostile_count_after\":").append(h.word(0x4435c4)).append(",\"kills_after\":").append(h.word(0x44374c)).append(",\"actor_after\":").append(h.word(0x43800e)).append(",\"skipped_turn\":").append(finish==0x125ca0).append(",\"seed_after\":").append(result.seed).append(",\"rng_draws\":").append(result.draws).append(",\"draws\":[").append(result.drawRecords).append("],\"events\":[").append(events).append("]}");println("Final turn "+id+"/"+seed+"/"+faction+"/"+layout+"/"+count+"/"+negative);
        }finally{h.cpu.dispose();}
    }
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]);Path original=root.resolve("NewCenturions/TheNewCenturions"),scenario=original.resolve("Scenarios/The Thunder of Guns/0");items=Files.readAllBytes(original.resolve("Data Files/Data ID"));races=Files.readAllBytes(original.resolve("Data Files/Data Race"));scenarioItems=Files.readAllBytes(scenario.resolve("Data NI"));monsters=Files.readAllBytes(scenario.resolve("Data MD"));properties=Files.readAllBytes(original.resolve("Data Files/Data Swamp BD"));attacks=Files.readAllBytes(original.resolve("Data Files/Data S"));areas=Files.readAllBytes(original.resolve("Data Files/Data AD"));doors=Files.readAllBytes(scenario.resolve("Data ED3"));extraCodes=Files.readAllBytes(scenario.resolve("Data EDCD"));party=new byte[6*3138];String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};for(int i=0;i<6;i++)System.arraycopy(Files.readAllBytes(original.resolve("Warriors/"+names[i])),0,party,i*3138,3138);
        for(int id:new int[]{107,108,109})for(long seed:new long[]{12,1})for(int faction:new int[]{0,2})for(int layout:new int[]{0,1,2})for(int count:new int[]{1,100})probeFinal(id,seed,faction,layout,count,false);
        for(int id:new int[]{107,108,109})for(int faction:new int[]{0,2}){probeFinal(id,12,faction,0,96,false);probeFinal(id,12,faction,0,1,true);}
        StringBuilder identity=new StringBuilder();String[] paths={"tools/ghidra/VerifyFinalTurnStories.java","tools/ghidra/VerifyDeathScripts.java","tools/ghidra/VerifyAllyTurns.java","tools/ghidra/VerifyStoryInventory.java","tools/ghidra/ActionOracleHarness.java","tools/ghidra/SaneBoundary.java","analysis/ghidra/startup/code1-mapped.bin","NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data MD","NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data ED3","NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data EDCD","NewCenturions/TheNewCenturions/Data Files/Data Swamp BD"};String[] labels={"probe_sha256","parent_probe_sha256","ally_probe_sha256","inventory_probe_sha256","harness_sha256","sane_boundary_sha256","mapped_binary_sha256","monsters_sha256","scripts_sha256","extras_sha256","properties_sha256"};for(int i=0;i<paths.length;i++){if(identity.length()>0)identity.append(',');identity.append('"').append(labels[i]).append("\":\"").append(ActionOracleHarness.sha256(root.resolve(paths[i]))).append('"');}
        Files.writeString(root.resolve("evidence/fixtures/final-turn-stories.json"),"{\"authority\":\"Original getup126938..126c3e or early125ca0, complete authored newland345/276/277 with original opcode124/buildmonster109210/placemonster149e0e/bodyfield105366 and opcode120/killbody12e4e4. Authored MD, ED3 and EDCD supplied at file-service boundaries; default difficulty zero. Controlled flat/blocked/edge maps, factions0/2, capacity1/96/100 and negative chance. Original QuickDraw Rand and finite SANE boundary modeled. Text, sound and presentation routines intercepted; no live-gameplay claim.\",\"identity\":{"+identity+"},\"monsters\":\""+hex(monsters)+"\",\"scripts\":\""+hex(doors)+"\",\"extras\":\""+hex(extraCodes)+"\",\"properties\":\""+hex(Arrays.copyOfRange(properties,64,12064))+"\",\"cases\":["+cases+"]}\n");println("Final turn story cases exported");
    }
}
