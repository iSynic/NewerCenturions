import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.regex.Pattern;

/** Original bodycount and booty preparation execute as one sequence. */
public class VerifyAllyVictory extends VerifyGeneticVictory {
    byte[] enemyTemplate,allyDefinition;
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]);items=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data ID"));scenarioItems=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data NI"));races=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data Race"));party=new byte[3138*6];
        String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};for(int i=0;i<6;i++)System.arraycopy(Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Warriors/"+names[i])),0,party,i*3138,3138);
        byte[] md=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data MD"));allyDefinition=Arrays.copyOfRange(md,65*288,66*288);
        var matcher=Pattern.compile("\\{\\\"battle_id\\\":8,[^}]+\\}").matcher(Files.readString(root.resolve("evidence/fixtures/genetic-enemies.json")));if(!matcher.find())throw new Exception("Genetic input missing");enemyTemplate=bytes(matcher.group(),"records");
        StringBuilder cases=new StringBuilder();for(long seed:new long[]{12,1})for(int variant=0;variant<4;variant++)victory(seed,variant,cases);
        Files.writeString(root.resolve("evidence/fixtures/ally-victory.json"),"{\"authority\":\"Original booty1057da..1064c2 including full bodycount1047f8, automatic important scientist selection and explicit modal confirmation item2, complete updatenpcmenu, currency rolls, level0/1 XP branches, dropitem and randomitem and rewards. Controlled two original genetic BD8 generated records have HP110 zeroed; zero or two scientist65 records with HP2/4, first/all deaths. Original map flat1155 and participation/killcount controlled. Original QuickDraw RNG and SANE modeled; item table/profile/presentation boundaries documented. This establishes composite survivor and zero-level reward effects, not a live battle.\",\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyAllyVictory.java"))+"\",\"parent_probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyGeneticVictory.java"))+"\",\"inventory_probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyStoryInventory.java"))+"\",\"harness_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/ActionOracleHarness.java"))+"\",\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\",\"setup_fixture_sha256\":\""+ActionOracleHarness.sha256(root.resolve("evidence/fixtures/genetic-enemies.json"))+"\",\"items_sha256\":\""+ActionOracleHarness.sha256(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data ID"))+"\",\"monsters_sha256\":\""+ActionOracleHarness.sha256(root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data MD"))+"\",\"scenario_items_sha256\":\""+ActionOracleHarness.sha256(root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data NI"))+"\",\"cases\":["+cases+"]}\n");println("Eight original composite ally victory cases exported");
    }
    void victory(long seed,int variant,StringBuilder cases) throws Exception {
        int allies=variant==0?0:2,count=enemyTemplate.length/288+allies;byte[] enemies=new byte[100*288],menu=new byte[20*288];
        for(int i=0;i<allies;i++){System.arraycopy(allyDefinition,0,menu,i*288,288);ByteBuffer.wrap(menu).putShort(i*288+0x110,(short)(2+i*2)).putShort(i*288+0x112,(short)(2+i*2));System.arraycopy(menu,i*288,enemies,i*288,288);if(variant==3||(variant==2&&i==0))ByteBuffer.wrap(enemies).putShort(i*288+0x110,(short)0);}
        System.arraycopy(enemyTemplate,0,enemies,allies*288,enemyTemplate.length);for(int i=allies;i<count;i++)ByteBuffer.wrap(enemies).putShort(i*288+0x110,(short)0);
        byte[] warriors=party.clone();for(int i=0;i<6;i++){warriors[i*3138+0x9df]=1;ByteBuffer.wrap(warriors).putShort(i*3138+0x20,(short)(i+1)).putShort(i*3138+0x18,(short)2);}
        byte[] map=new byte[16200],order=new byte[220];ByteBuffer tiles=ByteBuffer.wrap(map),orders=ByteBuffer.wrap(order);for(int i=0;i<8100;i++)tiles.putShort(i*2,(short)1155);for(int i=0;i<110;i++)orders.putShort(i*2,(short)-1);for(int i=0;i<6;i++){tiles.putShort(((25+i)*90+25)*2,(short)i);orders.putShort((i+1)*2,(short)i);}for(int i=0;i<count;i++){tiles.putShort(((35+i)*90+35)*2,(short)(i+10));orders.putShort((i+7)*2,(short)(i+10));}
        ActionOracleHarness h=frame();RunResult result=new RunResult(seed);SaneBoundary sane=new SaneBoundary(h.cpu,currentProgram);int dialogs=0;
        try {
            common(h,warriors);h.write(0x439d3c,enemies);h.write(0x440edc,menu);h.value(0x44257c,2,allies);h.write(0x4085b8,map);h.write(0x4470b8,order);h.value(0x438776,2,1);h.value(0x4435c4,2,1);h.value(0x43874a,2,count);h.value(0x442780,4,5);h.value(0x442784,4,5);
            h.write(0x4537fe,Arrays.copyOfRange(items,0,30000));h.write(0x46978e,Arrays.copyOfRange(items,90000,120000));
            for(int i=0;i<6;i++){h.value(0x44709a+i*4,2,20+i);h.value(0x44709c+i*4,2,20);h.value(0x437a6e+i*2,2,1155);}for(int i=0;i<count;i++){h.value(0x446f0a+i*4,2,30+i);h.value(0x446f0c+i*4,2,30);}
            h.value(0x700008,2,0);h.value(0x70000a,2,0);h.jump(0x1057da);boolean done=false;
            for(int step=0;step<1000000;step++) {
                long pc=h.pc(),sp=h.sp();if(pc==0x1064c2){done=true;break;}
                if(pc==0x130afe){loadItem(h);continue;}if(pc==0x153ca4){result.rng(h);continue;}if(pc==0x133a18){result.sound(h);continue;}if(pc==0x400038){h.cpu.writeRegister("D0",Math.abs(h.word(sp+4)));h.ret();continue;}
                if(pc==0x400090||pc==0x13e13c||pc==0x13a7c4||pc==0x13154c||pc==0x13a2c4||pc==0x12382a||pc==0x160fb2||pc==0x158454||pc==0x1205f8||pc==0x105572||pc==0x1382a8||pc==0x1537ee||pc==0x153720||pc==0x10518a){h.ret();continue;}
                int op=h.word(pc)&65535;if(op==0xa9eb){sane.execute();continue;}
                if(op==0xa97c){h.value(sp+10,4,0x600000);h.pop(10);h.jump(pc+2);continue;}if(op==0xaa1b){h.value(sp+2,4,0);h.pop(2);h.jump(pc+2);continue;}
                if(op==0xa98d){h.value(h.pointer(sp),4,0x600100);h.value(h.pointer(sp+4),4,0x600108);h.value(h.pointer(sp+8),2,0);h.pop(18);h.jump(pc+2);continue;}if(op==0xa991){h.value(h.pointer(sp),2,2);dialogs++;h.pop(8);h.jump(pc+2);continue;}
                if(op==0xa032||op==0xa937||op==0xa89b){h.jump(pc+2);continue;}
                if(op==0xa939||op==0xa93a){h.pop(6);h.jump(pc+2);continue;}if(op==0xa947){h.pop(10);h.jump(pc+2);continue;}if(op==0xa940){h.pop(8);h.jump(pc+2);continue;}if(op==0xa88a){h.pop(2);h.jump(pc+2);continue;}
                if(op==0xaa1e){h.value(sp+2,4,0);h.pop(2);h.jump(pc+2);continue;}
                if(op==0xaa1c||op==0xa93c||op==0xa873||op==0xaa0b||op==0xa862||op==0xa915||op==0xa8a3||op==0xa981||op==0xa863||op==0xa922||op==0xa923||op==0xa914||op==0xaa26){h.pop(4);h.jump(pc+2);continue;}
                if(!h.cpu.step(monitor))throw new Exception("Ally victory "+variant+" at "+Long.toHexString(pc)+": "+h.cpu.getLastError());
            }
            if(!done)throw new Exception("Ally victory budget");if(cases.length()>0)cases.append(',');
            cases.append("{\"variant\":").append(variant).append(",\"count\":").append(count).append(",\"ally_count\":").append(allies).append(",\"ally_count_after\":").append(h.word(0x44257c)).append(",\"dialogs\":").append(dialogs).append(",\"menu_before\":\"").append(hex(menu)).append("\",\"menu_after\":\"").append(hex(h.read(0x440edc,menu.length))).append("\",\"warriors_before\":\"").append(hex(warriors)).append("\",\"warriors_after\":\"").append(hex(h.read(0x448aa0,warriors.length))).append("\",\"enemies_before\":\"").append(hex(enemies)).append("\",\"enemies_after\":\"").append(hex(h.read(0x439d3c,enemies.length))).append("\",\"map_before\":\"").append(hex(map)).append("\",\"map_after\":\"").append(hex(h.read(0x4085b8,map.length))).append("\",\"order_before\":\"").append(hex(order)).append("\",\"order_after\":\"").append(hex(h.read(0x4470b8,order.length))).append("\",\"loot_after\":\"").append(hex(h.read(0x41fc74,22000))).append("\",\"loot_flags\":\"").append(hex(h.read(0x42df04,1000))).append("\",\"seed_before\":").append(seed).append(",\"seed_after\":").append(result.seed).append(",\"rng_draws\":").append(result.draws).append(",\"draws\":[").append(result.drawRecords).append("],\"share\":").append((int)h.pointer(0x6fff6a)).append(",\"currency\":").append(h.pointer(0x442788)).append(",\"sounds\":[").append(result.sounds).append("]}");
        }finally{h.cpu.dispose();}
    }
}
