import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import com.google.gson.JsonParser;
import com.google.gson.JsonArray;

/** Original loaded melee178 against starter and generated-route foundations. */
public class VerifyGeneticMelee extends VerifyFinalEnemyWeapons {
    byte[] routeParty,routeEnemy;
    byte[] array(JsonArray values){byte[] result=new byte[values.size()];for(int i=0;i<result.length;i++)result[i]=(byte)values.get(i).getAsInt();return result;}
    @Override byte[] monster(int id,int hp){byte[] result=id<0?routeEnemy.clone():super.monster(id,hp);ByteBuffer.wrap(result).putShort(0x4e,(short)178);return result;}
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]),original=root.resolve("NewCenturions/TheNewCenturions"),scenario=original.resolve("Scenarios/The Thunder of Guns/0");items=Files.readAllBytes(original.resolve("Data Files/Data ID"));scenarioItems=Files.readAllBytes(scenario.resolve("Data NI"));races=Files.readAllBytes(original.resolve("Data Files/Data Race"));monsters=Files.readAllBytes(scenario.resolve("Data MD"));properties=Files.readAllBytes(original.resolve("Data Files/Data Swamp BD"));attacks=Files.readAllBytes(original.resolve("Data Files/Data S"));areas=Files.readAllBytes(original.resolve("Data Files/Data AD"));party=new byte[3138*6];String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};for(int i=0;i<6;i++){System.arraycopy(Files.readAllBytes(original.resolve("Warriors/"+names[i])),0,party,i*3138,3138);party[i*3138+0x9df]=1;ByteBuffer.wrap(party).putShort(i*3138+0x50,(short)100).putShort(i*3138+0x52,(short)100);}
        Path routePath=root.resolve("analysis/genetic-melee-source.json");
        String routeSnapshotHash;
        if(Files.exists(routePath)){
            var route=JsonParser.parseString(Files.readString(routePath)).getAsJsonObject().getAsJsonObject("session");
            routeParty=new byte[6*3138];var routeWarriors=route.getAsJsonArray("warriors");for(int i=0;i<6;i++)System.arraycopy(array(routeWarriors.get(i).getAsJsonObject().getAsJsonArray("bytes")),0,routeParty,i*3138,3138);
            routeEnemy=array(route.getAsJsonObject("combat").getAsJsonObject("enemies").getAsJsonArray("records").get(7).getAsJsonObject().getAsJsonArray("bytes"));
            routeSnapshotHash=ActionOracleHarness.sha256(routePath);
        }else{
            var foundation=JsonParser.parseString(Files.readString(root.resolve("evidence/fixtures/genetic-melee.json"))).getAsJsonObject().getAsJsonObject("route_foundation");
            routeParty=java.util.HexFormat.of().parseHex(foundation.get("warriors").getAsString());
            routeEnemy=java.util.HexFormat.of().parseHex(foundation.get("enemy").getAsString());
            routeSnapshotHash=foundation.get("snapshot_sha256").getAsString();
        }
        if(ByteBuffer.wrap(routeEnemy).getShort(0x28)!=178||ByteBuffer.wrap(routeEnemy).getShort(0x4e)!=178)throw new Exception("Route snapshot no longer contains loaded melee178");
        for(long seed:new long[]{12,1,2,3,4,5,6,7})for(int id:new int[]{13,36})for(int member=0;member<6;member++)for(int variant=0;variant<3;variant++)for(int index=0;index<2;index++)attack(seed,id,member,variant,index);
        byte[] starterParty=party;
        party=routeParty;for(long seed:new long[]{12,1,2,3,4,5,6,7})for(int member:new int[]{1,4})for(int index=0;index<2;index++)attack(seed,-1,member,0,index);party=starterParty;
        for(long seed:new long[]{12,1,2,3,4,5,6,7})for(int id:new int[]{13,36})for(int points:new int[]{1,2,3,4,6})caller(seed,id,points,0);
        party=routeParty;for(long seed:new long[]{12,1,2,3,4,5,6,7})for(int member:new int[]{1,4})for(int points:new int[]{1,2,3,4,6})caller(seed,-1,points,member);party=starterParty;
        StringBuilder identity=new StringBuilder();String[][] inputs={{"probe","tools/ghidra/VerifyGeneticMelee.java"},{"parent_probe","tools/ghidra/VerifyFinalEnemyWeapons.java"},{"weapon_probe","tools/ghidra/VerifyRequiredEnemyWeapons.java"},{"turn_probe","tools/ghidra/VerifyAllyTurns.java"},{"inventory_probe","tools/ghidra/VerifyStoryInventory.java"},{"harness","tools/ghidra/ActionOracleHarness.java"},{"sane_boundary","tools/ghidra/SaneBoundary.java"},{"mapped_binary","analysis/ghidra/startup/code1-mapped.bin"},{"monsters","NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data MD"},{"items","NewCenturions/TheNewCenturions/Data Files/Data ID"},{"races","NewCenturions/TheNewCenturions/Data Files/Data Race"}};for(String[] input:inputs){if(identity.length()>0)identity.append(',');identity.append('"').append(input[0]).append("_sha256\":\"").append(ActionOracleHarness.sha256(root.resolve(input[1]))).append('"');}
        Files.writeString(root.resolve("evidence/fixtures/genetic-melee.json"),"{\"authority\":\"Original complete attack2 at101eb6 and adjacent checkforenemy110e96 for loaded melee178. Original MD13/36 with explicitly loaded current178 and exact generated level3 route enemy bytes exercise both attack indices, full six warrior records, defenses, ordered sounds from native showresults hit/miss audio branches and RNG. Route-derived foundations preserve authored/runtime bytes and raise living target HP to10000 to isolate weapon behavior; other dead route warriors remain dead. Caller controls points1/2/3/4/6, movement24, target adjacency and zero guards. Original route positions, other enemies, scripts and live observations are outside this controlled fixture. showresults UI is intercepted after executing its native audio slice158776..15886e for code0 or159042..15904c for code-1; remaining presentation is outside the fixture. No natural condition10/unloaded melee recovery claim.\",\"identity\":{"+identity+"},\"route_foundation\":{\"snapshot_sha256\":\""+routeSnapshotHash+"\",\"warriors\":\""+hex(routeParty)+"\",\"enemy\":\""+hex(routeEnemy)+"\"},\"weapon\":\""+hex(Arrays.copyOfRange(items,178*150,179*150))+"\",\"melee_cases\":["+directCases+"],\"caller_cases\":["+callerCases+"]}\n");println("608 genetic melee direct and160 caller cases exported");
    }
    @Override byte[] defended(int member,int variant,boolean ranged){
        byte[] result=super.defended(member,variant,ranged);
        if(party==routeParty){result=routeParty.clone();ByteBuffer.wrap(result).putShort(member*3138+0x50,(short)10000).putShort(member*3138+0x52,(short)10000);}
        return result;
    }
    void showresultsSound(ActionOracleHarness h,RunResult result)throws Exception{
        int code=h.word(h.sp()+6),attacker=h.word(h.sp()+8);
        if(code!=0&&code!=-1)throw new Exception("Unexpected melee178 showresults code "+code);
        String[] names={"D0","D1","D2","D3","D4","D5","D6","D7","A0","A1","A2","A3","A4","A5","A6","SP","PC","XF","NF","ZF","VF","CF"};
        java.math.BigInteger[] saved=new java.math.BigInteger[names.length];for(int i=0;i<names.length;i++)saved[i]=h.cpu.readRegister(names[i]);
        h.cpu.writeRegister("SP",0x6dd000);h.cpu.writeRegister("D5",attacker);h.jump(code==0?0x158776:0x159042);
        boolean done=false;
        for(int step=0;step<100;step++){
            long pc=h.pc();if(pc==0x15886e||pc==0x15904c){done=true;break;}
            if(pc==0x133a18){result.sound(h);continue;}
            if((h.word(pc)&65535)==0xa873){h.pop(4);h.jump(pc+2);continue;}
            if(!h.cpu.step(monitor))throw new Exception("Melee178 native audio at "+Long.toHexString(pc)+": "+h.cpu.getLastError());
        }
        if(!done)throw new Exception("Melee178 audio instruction budget");
        for(int i=0;i<names.length;i++)h.cpu.writeRegister(names[i],saved[i]);
    }
    @Override void execute(ActionOracleHarness h,RunResult result,long stop) throws Exception {
        SaneBoundary sane=new SaneBoundary(h.cpu,currentProgram);boolean done=false;
        for(int step=0;step<1500000;step++) {
            long pc=h.pc(),sp=h.sp();if(pc==stop){done=true;break;}
            if(pc==0x400038){h.cpu.writeRegister("D0",Math.abs(h.word(sp+4)));h.ret();continue;}
            if(pc==0x153ca4){result.rng(h);continue;}
            if(pc==0x133a18||pc==0x16474a){result.sound(h);continue;}
            if(pc==0x158454){showresultsSound(h,result);h.ret();continue;}
            if(pc==0x159192){h.value(0x44719a,2,0);h.cpu.writeRegister("D0",0);h.ret();continue;}
            if(pc==0x120c14||pc==0x105572||pc==0x10e6b6||pc==0x133d46||pc==0x13154c||pc==0x11f1a8||pc==0x11edc4||pc==0x14a02c||pc==0x13a2c4||pc==0x1205f8||pc==0x16043e||pc==0x1382a8||pc==0x11f136){h.ret();continue;}
            int op=h.word(pc)&65535;
            if(op==0xa9eb){
                // Apple Numerics Manual FSQRTX uses one extended destination address.
                if((h.word(sp)&65535)==0x12){long destination=h.pointer(sp+2);sane.extended(destination,Math.sqrt(sane.extended(destination)));h.pop(6);h.jump(pc+2);}else sane.execute();
                continue;
            }
            if(op==0xa873||op==0xaa1c||op==0xa862||op==0xa863||op==0xa89b||op==0xa893||op==0xa891||op==0xa884){h.pop(4);h.jump(pc+2);continue;}
            if(op==0xa89c){h.pop(2);h.jump(pc+2);continue;}
            if(op==0xa8b7){h.pop(4);h.jump(pc+2);continue;}
            if(op==0xa8ec){h.pop(22);h.jump(pc+2);continue;}
            if(op==0xa8a9){int vertical=h.word(sp),horizontal=h.word(sp+2);long rect=h.pointer(sp+4);h.value(rect,2,h.word(rect)+vertical);h.value(rect+2,2,h.word(rect+2)+horizontal);h.value(rect+4,2,h.word(rect+4)-vertical);h.value(rect+6,2,h.word(rect+6)-horizontal);h.pop(8);h.jump(pc+2);continue;}
            if(op==0xa8aa){long output=h.pointer(sp),b=h.pointer(sp+4),a=h.pointer(sp+8);int top=Math.max(h.word(a),h.word(b)),left=Math.max(h.word(a+2),h.word(b+2)),bottom=Math.min(h.word(a+4),h.word(b+4)),right=Math.min(h.word(a+6),h.word(b+6));boolean empty=top>=bottom||left>=right;h.value(output,2,empty?0:top);h.value(output+2,2,empty?0:left);h.value(output+4,2,empty?0:bottom);h.value(output+6,2,empty?0:right);h.value(sp+12,1,empty?0:1);h.pop(12);h.jump(pc+2);continue;}
            if(op==0xa8a7){long rectangle=h.pointer(sp+8);h.write(rectangle,h.read(sp+4,4));h.write(rectangle+4,h.read(sp,4));h.pop(12);h.jump(pc+2);continue;}
            if(op==0xaa1e){h.value(sp+2,4,0);h.pop(2);h.jump(pc+2);continue;}
            if(op==0xa8ad){long rect=h.pointer(sp);int x=h.word(sp+4),y=h.word(sp+6);h.value(sp+8,1,x>=h.word(rect)&&x<h.word(rect+4)&&y>=h.word(rect+2)&&y<h.word(rect+6)?1:0);h.pop(8);h.jump(pc+2);continue;}
            if(op==0xa9b4){h.jump(pc+2);continue;}
            if(op==0xa974){h.cpu.writeRegister("SP",sp-2);h.value(sp-2,2,0);h.jump(pc+2);continue;}
            if(!h.cpu.step(monitor))throw new Exception("Ally action at "+Long.toHexString(pc)+": "+h.cpu.getLastError());
        }
        if(!done)throw new Exception("Ally action instruction budget");
    }
    void caller(long seed,int id,int points,int member)throws Exception{
        ActionOracleHarness h=frame();RunResult result=new RunResult(seed);
        try{
            byte[] record=monster(id,100),warriors=defended(member,0),map=terrain(),order=new byte[220];Arrays.fill(order,(byte)255);ByteBuffer.wrap(order).putShort(2,(short)10);record[0x106]=24;record[0x109]=(byte)points;
            for(int i=0;i<6;i++)ByteBuffer.wrap(warriors).putShort(i*3138+0x16,(short)0);
            ByteBuffer.wrap(map).putShort((25*90+25)*2,(short)10).putShort((26*90+25)*2,(short)member);state(h,record,map);h.write(0x448aa0,warriors);h.write(0x4470b8,order);h.value(0x446f0a,2,20);h.value(0x446f0c,2,20);h.value(0x44709a+member*4,2,21);h.value(0x44709c+member*4,2,20);h.value(0x43874a,2,1);h.value(0x4435d2,2,0);h.value(0x4435c4,2,1);h.value(0x44374c,2,0);h.enter(0x110e96,0x6ffe00,0x900000);h.value(0x6ffe04,2,0);StringBuilder calls=new StringBuilder();executeCaller(h,result,calls);
            if(callerCases.length()>0)callerCases.append(',');callerCases.append("{\"seed_before\":").append(seed).append(",\"monster_id\":").append(id).append(",\"member\":").append(member).append(",\"points\":").append(points).append(",\"warriors_before\":\"").append(hex(warriors)).append("\",\"warriors_after\":\"").append(hex(h.read(0x448aa0,party.length))).append("\",\"enemy_before\":\"").append(hex(record)).append("\",\"enemy_after\":\"").append(hex(h.read(0x439d3c,288))).append("\",\"map_before\":\"").append(hex(map)).append("\",\"map_after\":\"").append(hex(h.read(0x4085b8,16200))).append("\",\"order_before\":\"").append(hex(order)).append("\",\"order_after\":\"").append(hex(h.read(0x4470b8,220))).append("\",\"calls\":[").append(calls).append("],\"seed_after\":").append(result.seed).append(",\"rng_draws\":").append(result.draws).append(",\"draws\":[").append(result.drawRecords).append("],\"sounds\":[").append(result.sounds).append("]}");
        }finally{h.cpu.dispose();}
    }
}
