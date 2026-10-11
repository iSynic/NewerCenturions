import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;

/** Native death cleanup, deferred scripts and nested encounter continuation. */
public class VerifyDeathScripts extends VerifyAllyTurns {
    byte[] doors,extraCodes;
    StringBuilder cases=new StringBuilder(),events;
    int childMode,abort;
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]);Path original=root.resolve("NewCenturions/TheNewCenturions");Path scenario=original.resolve("Scenarios/The Thunder of Guns/0");
        items=Files.readAllBytes(original.resolve("Data Files/Data ID"));races=Files.readAllBytes(original.resolve("Data Files/Data Race"));scenarioItems=Files.readAllBytes(scenario.resolve("Data NI"));monsters=Files.readAllBytes(scenario.resolve("Data MD"));properties=Files.readAllBytes(original.resolve("Data Files/Data Swamp BD"));attacks=Files.readAllBytes(original.resolve("Data Files/Data S"));areas=Files.readAllBytes(original.resolve("Data Files/Data AD"));doors=Files.readAllBytes(scenario.resolve("Data ED3"));extraCodes=Files.readAllBytes(scenario.resolve("Data EDCD"));
        party=new byte[6*3138];String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};for(int i=0;i<6;i++)System.arraycopy(Files.readAllBytes(original.resolve("Warriors/"+names[i])),0,party,i*3138,3138);
        for(int id:new int[]{66,60})for(int queued:new int[]{0,1,2})for(int child:new int[]{0,1})for(int stopped:new int[]{0,1})if(id==66||child==0&&stopped==0)probe(id,queued,child,stopped,false);
        for(int child:new int[]{0,1})for(int stopped:new int[]{0,1})probe(66,1,child,stopped,true);
        String identity="\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyDeathScripts.java"))+"\",\"parent_probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyAllyTurns.java"))+"\",\"harness_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/ActionOracleHarness.java"))+"\",\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\",\"monsters_sha256\":\""+ActionOracleHarness.sha256(scenario.resolve("Data MD"))+"\",\"scripts_sha256\":\""+ActionOracleHarness.sha256(scenario.resolve("Data ED3"))+"\",\"extras_sha256\":\""+ActionOracleHarness.sha256(scenario.resolve("Data EDCD"))+"\"";
        identity+=",\"inventory_probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyStoryInventory.java"))+"\",\"items_sha256\":\""+ActionOracleHarness.sha256(original.resolve("Data Files/Data ID"))+"\",\"races_sha256\":\""+ActionOracleHarness.sha256(original.resolve("Data Files/Data Race"))+"\",\"scenario_items_sha256\":\""+ActionOracleHarness.sha256(scenario.resolve("Data NI"))+"\",\"properties_sha256\":\""+ActionOracleHarness.sha256(original.resolve("Data Files/Data Swamp BD"))+"\",\"attacks_sha256\":\""+ActionOracleHarness.sha256(original.resolve("Data Files/Data S"))+"\",\"areas_sha256\":\""+ActionOracleHarness.sha256(original.resolve("Data Files/Data AD"))+"\"";
        Files.writeString(root.resolve("evidence/fixtures/death-scripts.json"),"{\"authority\":\"Complete original killbody12e4e4 and emptyque121524 execute with exact MD66/60 and ED3 scripts51/229. Full original newland executes authored text/sound actions and opcode2/47. Loaddoor2/loadextracode supply exact authored records at explicit file-service boundaries. Nested combat114378 and booty105748 are controlled callback acknowledgements, not nested battle or reward proof; child variant mutates queue/body/level and abort to establish native parent continuation and restoration. Native bodyfield large-body cleanup executes; graphical routines and text presentation are intercepted. No live gameplay claim.\",\"identity\":{"+identity+"},\"script51\":\""+hex(Arrays.copyOfRange(doors,51*42,52*42))+"\",\"script229\":\""+hex(Arrays.copyOfRange(doors,229*42,230*42))+"\",\"extra104\":\""+hex(Arrays.copyOfRange(extraCodes,104*10,105*10))+"\",\"cases\":["+cases+"]}\n");println("Death-script native continuation cases exported");
    }
    void event(String value){if(events.length()>0)events.append(',');events.append(value);}
    void executeDeath(ActionOracleHarness h,RunResult result,long stop) throws Exception {
        for(int step=0;step<400000;step++){
            long pc=h.pc(),sp=h.sp();if(pc==stop)return;
            if(pc==0x123c60){int id=h.word(sp+4);byte[] record=Arrays.copyOfRange(doors,id*42,(id+1)*42);System.arraycopy(h.read(0x479eaa,8),0,record,0,8);h.write(0x479eaa,record);event("[\"load_script\","+id+"]");h.ret();continue;}
            if(pc==0x1383e6){int id=h.word(sp+4);h.write(0x44330a,Arrays.copyOfRange(extraCodes,id*10,(id+1)*10));event("[\"load_extra\","+id+"]");h.ret();continue;}
            if(pc==0x1400e4){event("[\"newland\","+h.word(sp+12)+","+h.word(sp+14)+","+h.word(sp+16)+"]");}
            if(pc==0x114378){event("[\"combat\","+h.word(0x43874e)+","+h.word(0x43800c)+"]");if(childMode!=0){h.value(0x43800c,2,0);h.value(0x437a90,2,0);byte[] mutation=new byte[42];Arrays.fill(mutation,(byte)0x6a);h.write(0x479eaa,mutation);}h.value(0x4386b2,1,abort);h.ret();continue;}
            if(pc==0x105748){event("[\"booty\","+h.word(sp+4)+","+h.word(sp+6)+"]");h.ret();continue;}
            if(pc==0x13107a){event("[\"load_map\","+(int)h.pointer(sp+4)+"]");h.ret();continue;}
            if(pc==0x13bb20){event("[\"update_main\"]");h.ret();continue;}
            if(pc==0x15dc34){event("[\"text\","+h.word(sp+6)+"]");h.ret();continue;}
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
    void probe(int id,int queued,int child,int stopped,boolean second) throws Exception {
        ActionOracleHarness h=frame();RunResult result=new RunResult(12);events=new StringBuilder();childMode=child;abort=stopped;
        try{
            byte[] records=new byte[100*288],record=monster(id,0);ByteBuffer r=ByteBuffer.wrap(record);r.putShort(0x118,(short)1151).putShort(0x11a,(short)1152).putShort(0x11c,(short)1153).putShort(0x11e,(short)1154);for(int b=0xb2;b<0xda;b++)record[b]=(byte)0x5a;record[0x106]=100;System.arraycopy(record,0,records,0,288);
            byte[] map=terrain();ByteBuffer m=ByteBuffer.wrap(map);for(int x=24;x<=25;x++)for(int y=24;y<=25;y++)m.putShort((x*90+y)*2,(short)10);state(h,records,map);h.value(0x43874a,2,1);h.value(0x446f0a,2,20);h.value(0x446f0c,2,20);h.write(0x437a94,new byte[14*100]);h.value(0x43800c,2,0);h.value(0x437a90,4,0x12345678);h.value(0x438585,1,queued==1?1:0);h.value(0x438760,2,queued==2?1:0);h.value(0x438584,1,0);h.value(0x438582,2,7);h.value(0x4386b2,1,0);h.value(0x435658,1,0);h.value(0x447194,2,1);h.value(0x442739,1,0);h.value(0x4427b3,1,1);h.value(0x442774,4,4);h.write(0x44258c,new byte[128]);h.write(0x4470b8,new byte[220]);for(int i=0;i<110;i++)h.value(0x4470b8+i*2,2,-1);h.value(0x4470ba,2,10);
            byte[] parent=new byte[42];for(int i=0;i<42;i++)parent[i]=(byte)(0x20+i);h.write(0x479eaa,parent);h.enter(0x12e4e4,0x6ffe00,0x900000);h.value(0x6ffe04,2,10);h.value(0x6ffe06,2,0);executeDeath(h,result,0x900000);
            byte[] queuedAfter=h.read(0x437a94,1400);int count=h.word(0x43800c),kills=h.word(0x44374c);byte[] cleanup=h.read(0x439d3c,288),mapCleanup=h.read(0x4085b8,16200),bodyCleanup=h.read(0x479eaa,42);
            if(queued!=0){if(second){h.value(0x43800c,2,2);h.write(0x437a94+14,ByteBuffer.allocate(14).putShort((short)60).putShort((short)1).putShort((short)229).putInt(25).putInt(25).array());}h.value(0x438585,1,0);h.value(0x438760,2,0);h.enter(0x121524,0x6ffe00,0x900000);executeDeath(h,result,0x900000);}
            if(cases.length()>0)cases.append(',');cases.append("{\"monster_id\":").append(id).append(",\"queued\":").append(queued).append(",\"child_mutation\":").append(child).append(",\"child_abort\":").append(stopped).append(",\"second_entry\":").append(second).append(",\"record_before\":\"").append(hex(record)).append("\",\"record_cleanup\":\"").append(hex(cleanup)).append("\",\"kills_cleanup\":").append(kills).append(",\"map_before\":\"").append(hex(map)).append("\",\"map_cleanup\":\"").append(hex(mapCleanup)).append("\",\"queue_cleanup\":\"").append(hex(queuedAfter)).append("\",\"queue_count_cleanup\":").append(count).append(",\"queue_after\":\"").append(hex(h.read(0x437a94,1400))).append("\",\"queue_count_after\":").append(h.word(0x43800c)).append(",\"queue_pointer_after\":").append(h.pointer(0x437a90)).append(",\"parent_body\":\"").append(hex(parent)).append("\",\"body_cleanup\":\"").append(hex(bodyCleanup)).append("\",\"body_after\":\"").append(hex(h.read(0x479eaa,42))).append("\",\"flags_after\":\"").append(hex(h.read(0x44258c,128))).append("\",\"warriors_before\":\"").append(hex(party)).append("\",\"warriors_after\":\"").append(hex(h.read(0x448aa0,party.length))).append("\",\"order_after\":\"").append(hex(h.read(0x4470b8,220))).append("\",\"control_after\":\"").append(hex(h.read(0x438582,4))).append("\",\"participation_after\":").append(h.read(0x402f43,1)[0]).append(",\"rng_draws\":").append(result.draws).append(",\"events\":[").append(events).append("]}");
        }finally{h.cpu.dispose();}
    }
}
