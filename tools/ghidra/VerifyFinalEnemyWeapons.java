import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;

/** Original weapons required by the final authored mission battles. */
public class VerifyFinalEnemyWeapons extends VerifyRequiredEnemyWeapons {
    StringBuilder directCases=new StringBuilder(),callerCases=new StringBuilder(),bodyCases=new StringBuilder();
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]),original=root.resolve("NewCenturions/TheNewCenturions"),scenario=original.resolve("Scenarios/The Thunder of Guns/0");items=Files.readAllBytes(original.resolve("Data Files/Data ID"));scenarioItems=Files.readAllBytes(scenario.resolve("Data NI"));races=Files.readAllBytes(original.resolve("Data Files/Data Race"));monsters=Files.readAllBytes(scenario.resolve("Data MD"));properties=Files.readAllBytes(original.resolve("Data Files/Data Swamp BD"));attacks=Files.readAllBytes(original.resolve("Data Files/Data S"));areas=Files.readAllBytes(original.resolve("Data Files/Data AD"));party=new byte[3138*6];String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};for(int i=0;i<6;i++){System.arraycopy(Files.readAllBytes(original.resolve("Warriors/"+names[i])),0,party,i*3138,3138);party[i*3138+0x9df]=1;ByteBuffer.wrap(party).putShort(i*3138+0x50,(short)100).putShort(i*3138+0x52,(short)100);}
        for(int id:new int[]{2,3,4,5,102,103,104,105,106})for(int body=10;body<28;body++)for(int context=0;context<2;context++)preparation(id,body,context);
        for(long seed:new long[]{12,1})for(int id:new int[]{2,3,4,5,102,103,104,105,106})for(int member=0;member<6;member++)for(int variant=0;variant<6;variant++)firearm(seed,id,member,variant);
        for(long seed:new long[]{12,1})for(int id:new int[]{1,103,104,105})for(int member=0;member<6;member++)for(int variant=0;variant<3;variant++)for(int index=0;index<2;index++)attack(seed,id,member,variant,index);
        for(long seed:new long[]{12,1})for(int id:new int[]{1,103,104,105})for(int points:new int[]{1,2,3,4,6})caller(seed,id,points);
        for(long seed:new long[]{12,1,2,3,4,5,6,7})for(int id:new int[]{2,3,4,5,102,103,104,105,106})for(int context=0;context<2;context++)for(int variant=0;variant<4;variant++)body(seed,id,context,variant);
        StringBuilder identity=new StringBuilder();String[][] inputs={{"probe","tools/ghidra/VerifyFinalEnemyWeapons.java"},{"weapon_probe","tools/ghidra/VerifyRequiredEnemyWeapons.java"},{"parent_probe","tools/ghidra/VerifyAllyTurns.java"},{"inventory_probe","tools/ghidra/VerifyStoryInventory.java"},{"harness","tools/ghidra/ActionOracleHarness.java"},{"sane_boundary","tools/ghidra/SaneBoundary.java"},{"mapped_binary","analysis/ghidra/startup/code1-mapped.bin"},{"monsters","NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data MD"},{"items","NewCenturions/TheNewCenturions/Data Files/Data ID"},{"scenario_items","NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data NI"},{"races","NewCenturions/TheNewCenturions/Data Files/Data Race"},{"properties","NewCenturions/TheNewCenturions/Data Files/Data Swamp BD"},{"attacks","NewCenturions/TheNewCenturions/Data Files/Data S"},{"areas","NewCenturions/TheNewCenturions/Data Files/Data AD"},{"warrior0","NewCenturions/TheNewCenturions/Warriors/Cindred"},{"warrior1","NewCenturions/TheNewCenturions/Warriors/Coriand"},{"warrior2","NewCenturions/TheNewCenturions/Warriors/Kevlar"},{"warrior3","NewCenturions/TheNewCenturions/Warriors/PDP 11"},{"warrior4","NewCenturions/TheNewCenturions/Warriors/Sifini"},{"warrior5","NewCenturions/TheNewCenturions/Warriors/Solestri"}};for(String[] input:inputs){if(identity.length()>0)identity.append(',');identity.append('"').append(input[0]).append("_sha256\":\"").append(ActionOracleHarness.sha256(root.resolve(input[1]))).append('"');}
        Files.writeString(root.resolve("evidence/fixtures/final-enemy-weapons.json"),"{\"authority\":\"Original ranged preparation1146ee..114b78 with MD2/3/4/5/102/103/104/105/106 authored firearms and item context0/1 at body10..27; complete resolvespell151022 with six original warrior targets at HP10000 and explicit selected flag. Complete attack2 and adjacent checkforenemy with MD1/103/104/105, default melee179/180, original action count and attack indices. Complete ranged resolution additionally selects friendly monster65 HP10000 with shield/unconscious/high-defense variations. Native record writes, SANE, damage, savings and RNG retained. Presentation, killbody and target-mask choice intercepted; no original game battle claim.\",\"identity\":{"+identity+"},\"preparation_cases\":["+preparationCases+"],\"firearm_cases\":["+firearmCases+"],\"melee_cases\":["+directCases+"],\"caller_cases\":["+callerCases+"],\"body_cases\":["+bodyCases+"]}\n");println("Final authored weapon native cases exported");
    }
    @Override byte[] defended(int member,int variant,boolean ranged){
        byte[] result=party.clone();ByteBuffer b=ByteBuffer.wrap(result);for(int i=0;i<6;i++)b.putShort(i*3138+0x50,(short)10000).putShort(i*3138+0x52,(short)10000);int base=member*3138;
        if(variant>0&&ranged)b.putShort(base+0x60,(short)1);
        if(variant==2){for(int n=0x112;n<=0x120;n+=2)b.putShort(base+n,(short)100);for(int n=0x6e;n<=0x7c;n+=2)b.putShort(base+n,(short)1);b.putShort(base+0x32,(short)3);}
        return result;
    }
    byte[] defended(int member,int variant){return defended(member,variant,false);}
    void attack(long seed,int id,int member,int variant,int index)throws Exception{
        ActionOracleHarness h=frame();RunResult result=new RunResult(seed);
        try{
            byte[] warriors=defended(member,variant),record=monster(id,100);state(h,record,terrain());h.write(0x448aa0,warriors);h.value(0x4435d4,2,index);h.enter(0x101eb6,0x6ffe00,0x900000);h.value(0x6ffe04,2,10);h.value(0x6ffe06,2,member);h.value(0x6ffe08,2,index);execute(h,result,0x900000);
            if(directCases.length()>0)directCases.append(',');directCases.append("{\"seed_before\":").append(seed).append(",\"monster_id\":").append(id).append(",\"member\":").append(member).append(",\"variant\":").append(variant).append(",\"attack\":").append(index).append(",\"warriors_before\":\"").append(hex(warriors)).append("\",\"warriors_after\":\"").append(hex(h.read(0x448aa0,party.length))).append("\",\"enemy_before\":\"").append(hex(record)).append("\",\"enemy_after\":\"").append(hex(h.read(0x439d3c,288))).append("\",\"packet\":\"").append(hex(h.read(0x44374e,58))).append("\",\"seed_after\":").append(result.seed).append(",\"rng_draws\":").append(result.draws).append(",\"draws\":[").append(result.drawRecords).append("],\"sounds\":[").append(result.sounds).append("]}");
        }finally{h.cpu.dispose();}
    }
    void executeCaller(ActionOracleHarness h,RunResult result,StringBuilder calls)throws Exception{
        for(int step=0;step<200000;step++){
            long pc=h.pc(),sp=h.sp();if(pc==0x900000)return;
            if(pc==0x101eb6){if(calls.length()>0)calls.append(',');calls.append('[').append(h.word(sp+4)).append(',').append(h.word(sp+6)).append(',').append(h.word(sp+8)).append(']');execute(h,result,h.pointer(sp));continue;}
            if(pc==0x153ca4){result.rng(h);continue;}
            if(pc==0x400038){h.cpu.writeRegister("D0",Math.abs(h.word(sp+4)));h.ret();continue;}
            if(pc==0x133a18||pc==0x16474a){result.sound(h);continue;}
            if(pc==0x13154c||pc==0x11f1a8||pc==0x1205f8){h.ret();continue;}
            if(!h.cpu.step(monitor))throw new Exception("Required melee caller at "+Long.toHexString(pc)+": "+h.cpu.getLastError());
        }throw new Exception("Required melee caller instruction budget");
    }
    void caller(long seed,int id,int points)throws Exception{
        ActionOracleHarness h=frame();RunResult result=new RunResult(seed);
        try{
            byte[] record=monster(id,100),warriors=party.clone(),map=terrain(),order=new byte[220];Arrays.fill(order,(byte)255);ByteBuffer.wrap(order).putShort(2,(short)10);record[0x106]=24;record[0x109]=(byte)points;
            for(int i=0;i<6;i++)ByteBuffer.wrap(warriors).putShort(i*3138+0x16,(short)0);
            ByteBuffer.wrap(map).putShort((25*90+25)*2,(short)10).putShort((26*90+25)*2,(short)0);state(h,record,map);h.write(0x448aa0,warriors);h.write(0x4470b8,order);h.value(0x446f0a,2,20);h.value(0x446f0c,2,20);h.value(0x44709a,2,21);h.value(0x44709c,2,20);h.value(0x43874a,2,1);h.value(0x4435d2,2,0);h.value(0x4435c4,2,1);h.value(0x44374c,2,0);h.enter(0x110e96,0x6ffe00,0x900000);h.value(0x6ffe04,2,0);StringBuilder calls=new StringBuilder();executeCaller(h,result,calls);
            if(callerCases.length()>0)callerCases.append(',');callerCases.append("{\"seed_before\":").append(seed).append(",\"monster_id\":").append(id).append(",\"points\":").append(points).append(",\"warriors_before\":\"").append(hex(warriors)).append("\",\"warriors_after\":\"").append(hex(h.read(0x448aa0,party.length))).append("\",\"enemy_before\":\"").append(hex(record)).append("\",\"enemy_after\":\"").append(hex(h.read(0x439d3c,288))).append("\",\"map_before\":\"").append(hex(map)).append("\",\"map_after\":\"").append(hex(h.read(0x4085b8,16200))).append("\",\"order_before\":\"").append(hex(order)).append("\",\"order_after\":\"").append(hex(h.read(0x4470b8,220))).append("\",\"calls\":[").append(calls).append("],\"seed_after\":").append(result.seed).append(",\"rng_draws\":").append(result.draws).append(",\"draws\":[").append(result.drawRecords).append("],\"sounds\":[").append(result.sounds).append("]}");
        }finally{h.cpu.dispose();}
    }
    void body(long seed,int id,int context,int variant)throws Exception{
        ActionOracleHarness h=frame();RunResult result=new RunResult(seed);
        try{
            byte[] attacker=monster(id,10000),target=monster(65,10000),records=new byte[576];
            if(variant==1)target[0xb3]=1;
            if(variant==2)ByteBuffer.wrap(target).putShort(0x56,(short)80).putShort(0x58,(short)300);
            if(variant==3){ByteBuffer.wrap(target).putShort(0x62,(short)1000);target[0x104]=7;}
            System.arraycopy(attacker,0,records,0,288);System.arraycopy(target,0,records,288,288);state(h,records,terrain());h.value(0x43874a,2,2);h.value(0x45365c,2,context);h.cpu.writeRegister("D5",0);h.cpu.writeRegister("A6",0x701000);h.cpu.writeRegister("SP",0x700000);h.jump(0x1146ee);executeRanged(h,result,0x114b78);
            byte[] packet=h.read(0x44374e,58),prepared=h.read(0x439d3c,288);int shots=h.word(0x4537b4),sound=h.word(0x453776);
            h.write(0x4434e0,new byte[220]);h.value(0x4434e0+22,2,1);h.value(0x4435cc,2,shots);h.value(0x438760,2,1);h.value(0x443580,1,0);h.value(0x4358c2,1,0);h.write(0x435884,new byte[128]);h.enter(0x151022,0x6ffe00,0x900000);executeRanged(h,result,0x900000);
            if(bodyCases.length()>0)bodyCases.append(',');bodyCases.append("{\"monster_id\":").append(id).append(",\"context\":").append(context).append(",\"variant\":").append(variant).append(",\"seed_before\":").append(seed).append(",\"packet\":\"").append(hex(packet)).append("\",\"shots\":").append(shots).append(",\"weapon_sound\":").append(sound).append(",\"enemy_before\":\"").append(hex(prepared)).append("\",\"enemy_after\":\"").append(hex(h.read(0x439d3c,288))).append("\",\"target_before\":\"").append(hex(target)).append("\",\"target_after\":\"").append(hex(h.read(0x439d3c+288,288))).append("\",\"seed_after\":").append(result.seed).append(",\"rng_draws\":").append(result.draws).append(",\"draws\":[").append(result.drawRecords).append("],\"sounds\":[").append(result.sounds).append("]}");
        }finally{h.cpu.dispose();}
    }
}
