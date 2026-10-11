import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;

/** Original initiative-wrap caller and the authored BD146 round scripts. */
public class VerifyRoundStories extends VerifyFinalTurnStories {
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
            if(pc==0x158454){event("[\"spawn\","+h.word(sp+4)+","+fileOffset/288+"]");h.ret();continue;}
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
    void probeRound(int round,long seed,int layout,int count,int calls) throws Exception {
        ActionOracleHarness h=frame();RunResult result=new RunResult(seed);events=new StringBuilder();
        try {
            byte[] records=new byte[100*288],active=monster(115,100);System.arraycopy(active,0,records,0,288);
            byte[] map=terrain();int x=layout==2?3:20,y=x;
            if(layout==1){int blocked=-1;for(int i=0;i<300;i++)if(ByteBuffer.wrap(properties).getShort(64+i*40+4)==2){blocked=1000+i;break;}if(blocked<0)throw new Exception("No blocked tile");for(int xx=x-3;xx<=x+2;xx++)for(int yy=y-3;yy<=y+2;yy++)ByteBuffer.wrap(map).putShort(((xx+5)*90+yy+5)*2,(short)blocked);}
            state(h,records,map);h.value(0x43874a,2,count);h.value(0x446f0a,2,x);h.value(0x446f0c,2,y);h.write(0x437a94,new byte[1400]);h.value(0x43800c,2,0);h.value(0x438584,1,0);h.value(0x438583,1,0);h.value(0x438585,1,0);h.value(0x438760,2,0);h.value(0x43800e,2,0);h.value(0x4386b2,1,0);h.value(0x4427b3,1,1);h.value(0x442774,4,4);h.write(0x4470b8,new byte[220]);for(int i=0;i<110;i++)h.value(0x4470b8+i*2,2,-1);h.value(0x4470ba,2,10);h.value(0x4435c4,2,1);h.value(0x44374c,2,0);
            h.enter(0x149e0e,0x6ffe00,0x900000);h.value(0x6ffe04,2,x);h.value(0x6ffe06,2,y);h.value(0x6ffe08,2,0);executeFinal(h,result,0x900000);events=new StringBuilder();
            byte[] slots=ByteBuffer.allocate(10).putShort((short)363).putShort((short)364).putShort((short)338).putShort((short)340).putShort((short)0).array();h.write(0x439cec,slots);h.value(0x44255c,1,round);
            byte[] before=h.read(0x439d3c,count*288),mapBefore=h.read(0x4085b8,16200),positionsBefore=h.read(0x446f0a,count*4);h.write(0x479eaa,new byte[42]);
            for(int call=0;call<calls;call++){h.cpu.writeRegister("A6",0x700000);h.cpu.writeRegister("SP",0x6ffe00);h.jump(0x125d34);executeFinal(h,result,0x125db8);}
            int after=h.word(0x43874a);if(cases.length()>0)cases.append(',');cases.append("{\"round_before\":").append(round).append(",\"round_after\":").append(h.read(0x44255c,1)[0]).append(",\"seed_before\":").append(seed).append(",\"layout\":").append(layout).append(",\"count_before\":").append(count).append(",\"calls\":").append(calls).append(",\"slots_before\":\"").append(hex(slots)).append("\",\"slots_after\":\"").append(hex(h.read(0x439cec,10))).append("\",\"records_before\":\"").append(hex(before)).append("\",\"records_after\":\"").append(hex(h.read(0x439d3c,after*288))).append("\",\"map_before\":\"").append(hex(mapBefore)).append("\",\"map_after\":\"").append(hex(h.read(0x4085b8,16200))).append("\",\"positions_before\":\"").append(hex(positionsBefore)).append("\",\"positions_after\":\"").append(hex(h.read(0x446f0a,after*4))).append("\",\"warriors_after\":\"").append(hex(h.read(0x448aa0,party.length))).append("\",\"order_after\":\"").append(hex(h.read(0x4470b8,220))).append("\",\"count_after\":").append(after).append(",\"hostile_count_after\":").append(h.word(0x4435c4)).append(",\"actor_after\":").append(h.word(0x43800e)).append(",\"round_flag_after\":").append(h.read(0x438583,1)[0]).append(",\"seed_after\":").append(result.seed).append(",\"rng_draws\":").append(result.draws).append(",\"draws\":[").append(result.drawRecords).append("],\"events\":[").append(events).append("]}");println("Round "+round+"/"+seed+"/"+layout+"/"+count+"/"+calls);
        }finally{h.cpu.dispose();}
    }
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]);Path original=root.resolve("NewCenturions/TheNewCenturions"),scenario=original.resolve("Scenarios/The Thunder of Guns/0");items=Files.readAllBytes(original.resolve("Data Files/Data ID"));races=Files.readAllBytes(original.resolve("Data Files/Data Race"));scenarioItems=Files.readAllBytes(scenario.resolve("Data NI"));monsters=Files.readAllBytes(scenario.resolve("Data MD"));properties=Files.readAllBytes(original.resolve("Data Files/Data Swamp BD"));attacks=Files.readAllBytes(original.resolve("Data Files/Data S"));areas=Files.readAllBytes(original.resolve("Data Files/Data AD"));doors=Files.readAllBytes(scenario.resolve("Data ED3"));extraCodes=Files.readAllBytes(scenario.resolve("Data EDCD"));party=new byte[6*3138];String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};for(int i=0;i<6;i++)System.arraycopy(Files.readAllBytes(original.resolve("Warriors/"+names[i])),0,party,i*3138,3138);
        for(int round:new int[]{0,1,2,3,4,5,6,127,-128})for(long seed:new long[]{12,1})for(int layout:new int[]{0,1,2})for(int count:new int[]{1,100})probeRound(round,seed,layout,count,1);
        for(long seed:new long[]{12,1})for(int layout:new int[]{0,1,2})probeRound(1,seed,layout,1,6);
        StringBuilder identity=new StringBuilder();String[] paths={"tools/ghidra/VerifyRoundStories.java","tools/ghidra/VerifyFinalTurnStories.java","tools/ghidra/VerifyDeathScripts.java","tools/ghidra/VerifyAllyTurns.java","tools/ghidra/VerifyStoryInventory.java","tools/ghidra/ActionOracleHarness.java","tools/ghidra/SaneBoundary.java","analysis/ghidra/startup/code1-mapped.bin","NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data MD","NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data ED3","NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data EDCD","NewCenturions/TheNewCenturions/Data Files/Data Swamp BD"};String[] labels={"probe_sha256","turn_probe_sha256","parent_probe_sha256","ally_probe_sha256","inventory_probe_sha256","harness_sha256","sane_boundary_sha256","mapped_binary_sha256","monsters_sha256","scripts_sha256","extras_sha256","properties_sha256"};for(int i=0;i<paths.length;i++){if(identity.length()>0)identity.append(',');identity.append('"').append(labels[i]).append("\":\"").append(ActionOracleHarness.sha256(root.resolve(paths[i]))).append('"');}
        Files.writeString(root.resolve("evidence/fixtures/round-stories.json"),"{\"authority\":\"Original getup125d34..125db8 initiative-wrap caller executes increment of signed-byte round counter and five BD slots, ordered authored363/364/338/340, native126 predicates and opcode124/buildmonster/placemonster/bodyfield. Round flag438583 preserves authored factions. Controlled flat/blocked/edge maps, capacity1/100, signed127/-128 wrap and six consecutive rounds. Original authored file records supplied; QuickDraw RNG and SANE boundary modeled. UI/sound/text intercepted. Stops before hourly reduction. No live gameplay claim.\",\"identity\":{"+identity+"},\"monsters\":\""+hex(monsters)+"\",\"scripts\":\""+hex(doors)+"\",\"extras\":\""+hex(extraCodes)+"\",\"properties\":\""+hex(Arrays.copyOfRange(properties,64,12064))+"\",\"warriors_before\":\""+hex(party)+"\",\"cases\":["+cases+"]}\n");println("Round story cases exported");
    }
}
