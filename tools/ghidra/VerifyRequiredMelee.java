import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;

/** Original default melee 181 and its adjacent attack caller. */
public class VerifyRequiredMelee extends VerifyAllyTurns {
    StringBuilder cases=new StringBuilder(),callerCases=new StringBuilder();
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]),original=root.resolve("NewCenturions/TheNewCenturions"),scenario=original.resolve("Scenarios/The Thunder of Guns/0");
        items=Files.readAllBytes(original.resolve("Data Files/Data ID"));scenarioItems=Files.readAllBytes(scenario.resolve("Data NI"));races=Files.readAllBytes(original.resolve("Data Files/Data Race"));monsters=Files.readAllBytes(scenario.resolve("Data MD"));properties=Files.readAllBytes(original.resolve("Data Files/Data Swamp BD"));attacks=Files.readAllBytes(original.resolve("Data Files/Data S"));areas=Files.readAllBytes(original.resolve("Data Files/Data AD"));party=new byte[3138*6];
        String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};
        for(int i=0;i<6;i++){System.arraycopy(Files.readAllBytes(original.resolve("Warriors/"+names[i])),0,party,i*3138,3138);party[i*3138+0x9df]=1;ByteBuffer.wrap(party).putShort(i*3138+0x50,(short)100).putShort(i*3138+0x52,(short)100);}
        for(long seed:new long[]{12,1})for(int id:new int[]{24,25,26,27})for(int member=0;member<6;member++)for(int variant=0;variant<3;variant++)for(int index=0;index<2;index++)attack(seed,id,member,variant,index);
        for(long seed:new long[]{12,1})for(int id:new int[]{24,25,26,27})for(int points:new int[]{1,2,3,4,6})caller(seed,id,points);
        StringBuilder identity=new StringBuilder();String[][] inputs={{"probe","tools/ghidra/VerifyRequiredMelee.java"},{"parent_probe","tools/ghidra/VerifyAllyTurns.java"},{"inventory_probe","tools/ghidra/VerifyStoryInventory.java"},{"harness","tools/ghidra/ActionOracleHarness.java"},{"sane_boundary","tools/ghidra/SaneBoundary.java"},{"mapped_binary","analysis/ghidra/startup/code1-mapped.bin"},{"monsters","NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data MD"},{"items","NewCenturions/TheNewCenturions/Data Files/Data ID"},{"scenario_items","NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data NI"},{"races","NewCenturions/TheNewCenturions/Data Files/Data Race"},{"properties","NewCenturions/TheNewCenturions/Data Files/Data Swamp BD"},{"attacks","NewCenturions/TheNewCenturions/Data Files/Data S"},{"areas","NewCenturions/TheNewCenturions/Data Files/Data AD"}};
        for(String[] input:inputs){if(identity.length()>0)identity.append(',');identity.append('"').append(input[0]).append("_sha256\":\"").append(ActionOracleHarness.sha256(root.resolve(input[1]))).append('"');}
        for(int i=0;i<6;i++)identity.append(",\"warrior").append(i).append("_sha256\":\"").append(ActionOracleHarness.sha256(original.resolve("Warriors/"+names[i]))).append('"');
        Files.writeString(root.resolve("evidence/fixtures/required-melee.json"),"{\"authority\":\"Original complete attack2 at101eb6 for MD24..27 with authored default melee181/current primary weapons and six original warriors at controlled HP100, each seed12/1, attack indices0/1 and save/protection variants. Variant0 retains original defenses; variant1 sets elemental save word114=100; variant2 adds protection word70=1 and race3. Original SANE and all item caches supplied. Forty complete checkforenemy110e96 callers with one adjacent warrior0, no warrior guard, original byte8a attack counts, controlled action points1/2/3/4/6 and movement24 execute full attack2 calls and record indices. Native target choice, damage, record writes and RNG execute; presentation showresults and killbody are acknowledged boundaries. Full party, attacker, packet and ordered RNG observed; no live battle claim.\",\"identity\":{"+identity+"},\"attack_cases\":["+cases+"],\"caller_cases\":["+callerCases+"]}\n");println("288 original melee181 attacks and40 adjacent caller cases exported");
    }
    byte[] defended(int member,int variant){byte[] result=party.clone();ByteBuffer b=ByteBuffer.wrap(result);int base=member*3138;if(variant>0)b.putShort(base+0x114,(short)100);if(variant==2)b.putShort(base+0x70,(short)1).putShort(base+0x32,(short)3);return result;}
    void attack(long seed,int id,int member,int variant,int index)throws Exception{
        ActionOracleHarness h=frame();RunResult result=new RunResult(seed);
        try{
            byte[] warriors=defended(member,variant),record=monster(id,100);state(h,record,terrain());h.write(0x448aa0,warriors);h.value(0x4435d4,2,index);h.enter(0x101eb6,0x6ffe00,0x900000);h.value(0x6ffe04,2,10);h.value(0x6ffe06,2,member);h.value(0x6ffe08,2,index);execute(h,result,0x900000);
            if(cases.length()>0)cases.append(',');cases.append("{\"seed_before\":").append(seed).append(",\"monster_id\":").append(id).append(",\"member\":").append(member).append(",\"variant\":").append(variant).append(",\"attack\":").append(index).append(",\"warriors_before\":\"").append(hex(warriors)).append("\",\"warriors_after\":\"").append(hex(h.read(0x448aa0,party.length))).append("\",\"enemy_before\":\"").append(hex(record)).append("\",\"enemy_after\":\"").append(hex(h.read(0x439d3c,288))).append("\",\"packet\":\"").append(hex(h.read(0x44374e,58))).append("\",\"seed_after\":").append(result.seed).append(",\"rng_draws\":").append(result.draws).append(",\"draws\":[").append(result.drawRecords).append("],\"sounds\":[").append(result.sounds).append("]}");
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
}
