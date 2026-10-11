import ghidra.app.script.GhidraScript;
import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.HexFormat;
import java.util.regex.Pattern;

/** Original NPC combat placement and postbattle retention probes. */
public class VerifyAllyCombat extends GhidraScript {
    static String hex(byte[] bytes) { return HexFormat.of().formatHex(bytes); }
    byte[] definition, properties, initialMap;
    StringBuilder cases = new StringBuilder();
    public void run() throws Exception {
        Path root = Path.of(getScriptArgs()[0]);
        byte[] monsters = Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data MD"));
        definition = Arrays.copyOfRange(monsters, 65 * 288, 66 * 288);
        properties = Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data Swamp BD"));
        var m = Pattern.compile("\\\"tiles\\\"\\s*:\\s*\\\"([0-9a-f]+)\\\"").matcher(Files.readString(root.resolve("evidence/fixtures/formation-opening.json")));
        if (!m.find()) throw new Exception("Formation missing");
        initialMap = HexFormat.of().parseHex(m.group(1));
        for (int count : new int[]{0, 1, 2}) for (int selected : new int[]{0, 3}) placement(count, selected);
        for (long seed : new long[]{12, 1}) for (int count : new int[]{1, 2, 20}) for (int dead : new int[]{0, 1, 2}) synchronization(seed, count, dead);
        for (int actor : new int[]{0, 10, 11}) for (int dead : new int[]{0, 1, 2}) targets(actor, dead);
        String authority = "Original combatsetup ally-copy/place slice11ddb4..11e0fe, placemonster and bodyfield with independently verified opening map and exact Swamp properties. Full original bodycount1047f8 executes with ModalDialog confirmation item2, original automatic retention/sorting and updatenpcmenu. Original checkforenemy110e96 mode1 builds adjacent hostile target lists for warrior/ally/hostile actors and living/dead NPCs with controlled faction words. Exact MD65 records have controlled HP2+i and stale status/selection inputs. Presentation-only services and Toolbox drawing/dialog/menu calls modeled; Rand consumes original QuickDraw seed contract. No live combat claim.";
        Files.writeString(root.resolve("evidence/fixtures/ally-combat.json"), "{\"authority\":\"" + authority + "\",\"probe_sha256\":\"" + ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyAllyCombat.java")) + "\",\"harness_sha256\":\"" + ActionOracleHarness.sha256(root.resolve("tools/ghidra/ActionOracleHarness.java")) + "\",\"mapped_binary_sha256\":\"" + ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin")) + "\",\"monsters_sha256\":\"" + ActionOracleHarness.sha256(root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data MD")) + "\",\"properties_sha256\":\"" + ActionOracleHarness.sha256(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data Swamp BD")) + "\",\"formation_fixture_sha256\":\"" + ActionOracleHarness.sha256(root.resolve("evidence/fixtures/formation-opening.json")) + "\",\"ghidra\":\"12.1.2\",\"definition\":\"" + hex(definition) + "\",\"properties\":\"" + hex(properties) + "\",\"cases\":[" + cases + "]}\n");
        println("Thirty-three original ally combat cases exported");
    }
    byte[] records(int count, int selected) {
        byte[] result = new byte[20 * 288];
        for (int i=0; i<count; i++) {
            System.arraycopy(definition,0,result,i*288,288);
            ByteBuffer.wrap(result).putShort(i*288+0x110,(short)(i+2)).putShort(i*288+0x112,(short)(i+2)).putShort(i*288+0x10c,(short)17);
            result[i*288+0x105]=(byte)selected; result[i*288+0x82]=7; result[i*288+0x104]=9;
        }
        return result;
    }
    void placement(int count, int selected) throws Exception {
        ActionOracleHarness h = new ActionOracleHarness(currentProgram);
        try {
            byte[] menu = records(count,selected);
            h.write(0x440edc,menu); h.value(0x44257c,2,count); h.value(0x408578,2,0); h.value(0x43874a,2,0);
            h.write(0x439d3c,new byte[100*288]); h.write(0x446f0a,new byte[400]); h.write(0x4085b8,initialMap);
            h.write(0x43167c,Arrays.copyOfRange(properties,64,12064)); h.value(0x442780,4,-34); h.value(0x442784,4,-10);
            h.value(0x44709a,2,39); h.value(0x44709c,2,39);
            h.write(0x6fd000,new byte[16384]); h.cpu.writeRegister("A6",0x700000); h.cpu.writeRegister("SP",0x6ffe00);
            h.value(0x6ffff8,2,5); h.value(0x6ffffa,2,5); h.value(0x6ffffc,2,85); h.value(0x6ffffe,2,85); h.jump(0x11ddb4);
            boolean stopped=false;
            for(int step=0;step<1000000;step++) {
                long pc=h.pc(),sp=h.sp(); if(pc==0x11e0fe){stopped=true;break;}
                if(pc==0x400038){h.cpu.writeRegister("D0",Math.abs(h.word(sp+4)));h.ret();continue;}
                if(pc==0x153ca4)throw new Exception("Ally placement consumed RNG");
                int op=h.word(pc)&65535;
                if(op==0xa8ad){long r=h.pointer(sp);int y=h.word(sp+4),x=h.word(sp+6);h.value(sp+8,1,y>=h.word(r)&&x>=h.word(r+2)&&y<h.word(r+4)&&x<h.word(r+6)?1:0);h.pop(8);h.jump(pc+2);continue;}
                if(!h.cpu.step(monitor))throw new Exception("Placement at "+Long.toHexString(pc)+": "+h.cpu.getLastError());
            }
            if(!stopped)throw new Exception("Ally placement budget");
            append("{\"kind\":\"placement\",\"count\":"+count+",\"selection_byte\":"+selected+",\"menu_before\":\""+hex(menu)+"\",\"menu_after\":\""+hex(h.read(0x440edc,menu.length))+"\",\"map_before\":\""+hex(initialMap)+"\",\"map_after\":\""+hex(h.read(0x4085b8,16200))+"\",\"records_after\":\""+hex(count==0?new byte[0]:h.read(0x439d3c,count*288))+"\",\"positions\":\""+hex(count==0?new byte[0]:h.read(0x446f0a,count*4))+"\",\"enemy_count\":"+h.word(0x43874a)+",\"rng_draws\":0}");
        } finally {h.cpu.dispose();}
    }
    void synchronization(long initial, int count, int dead) throws Exception {
        ActionOracleHarness h = new ActionOracleHarness(currentProgram);
        long seed=initial; int draws=0,dialogs=0; StringBuilder sounds=new StringBuilder();
        try {
            byte[] menu=records(count,3),enemies=new byte[100*288];System.arraycopy(menu,0,enemies,0,menu.length);
            for(int i=0;i<count;i++){if(dead==2||(dead==1&&i==0))ByteBuffer.wrap(enemies).putShort(i*288+0x110,(short)0);for(int o:new int[]{0xb2,0xb3,0xb4,0xb7,0xcf,0xd8})enemies[i*288+o]=2;}
            h.write(0x440edc,menu);h.write(0x439d3c,enemies);h.value(0x44257c,2,count);h.value(0x43874a,2,count);h.value(0x408578,2,0);
            h.enter(0x1047f8,0x6ffe00,0x900000);h.cpu.writeRegister("A6",0x700000);boolean done=false;
            for(int step=0;step<1000000;step++) {
                long pc=h.pc(),sp=h.sp();if(pc==0x900000){done=true;break;}
                if(pc==0x400038){h.cpu.writeRegister("D0",Math.abs(h.word(sp+4)));h.ret();continue;}
                if(pc==0x153ca4){int bound=h.word(sp+4);seed=seed*16807%2147483647;draws++;short raw=(short)seed;if(raw==Short.MIN_VALUE)raw=0;h.cpu.writeRegister("D0",Math.abs((int)raw)*bound/32768+1);h.ret();continue;}
                if(pc==0x133a18){if(sounds.length()>0)sounds.append(',');sounds.append(h.word(sp+4));h.ret();continue;}
                if(pc==0x136cfa||pc==0x400090||pc==0x13e13c||pc==0x1537ee||pc==0x153720||pc==0x10518a||pc==0x12382a){h.ret();continue;}
                int op=h.word(pc)&65535;
                if(op==0xa97c){h.value(sp+10,4,0x600000);h.pop(10);h.jump(pc+2);continue;}
                if(op==0xaa1b){h.value(sp+2,4,0);h.pop(2);h.jump(pc+2);continue;}
                if(op==0xa98d){h.value(h.pointer(sp),4,0x600100);h.value(h.pointer(sp+4),4,0x600108);h.value(h.pointer(sp+8),2,0);h.pop(18);h.jump(pc+2);continue;}
                if(op==0xa991){h.value(h.pointer(sp),2,2);dialogs++;h.pop(8);h.jump(pc+2);continue;}
                if(op==0xa032){h.jump(pc+2);continue;}
                if(op==0xa939||op==0xa93a){h.pop(6);h.jump(pc+2);continue;}
                if(op==0xa947){h.pop(10);h.jump(pc+2);continue;}
                if(op==0xa940){h.pop(8);h.jump(pc+2);continue;}
                if(op==0xa88a){h.pop(2);h.jump(pc+2);continue;}
                if(op==0xa873||op==0xaa0b||op==0xa862||op==0xa915||op==0xa8a3||op==0xa981||op==0xa863||op==0xa922||op==0xa923||op==0xa914||op==0xaa1c||op==0xaa26){h.pop(4);h.jump(pc+2);continue;}
                if(!h.cpu.step(monitor))throw new Exception("Bodycount at "+Long.toHexString(pc)+": "+h.cpu.getLastError());
            }
            if(!done)throw new Exception("Ally synchronization budget");
            append("{\"kind\":\"synchronization\",\"count\":"+count+",\"dead\":"+dead+",\"menu_before\":\""+hex(menu)+"\",\"menu_after\":\""+hex(h.read(0x440edc,menu.length))+"\",\"records_before\":\""+hex(enemies)+"\",\"records_after\":\""+hex(h.read(0x439d3c,enemies.length))+"\",\"count_after\":"+h.word(0x44257c)+",\"seed_before\":"+initial+",\"seed_after\":"+seed+",\"rng_draws\":"+draws+",\"dialogs\":"+dialogs+",\"sounds\":["+sounds+"]}");
        } finally {h.cpu.dispose();}
    }
    void append(String item){if(cases.length()>0)cases.append(',');cases.append(item);}
    void targets(int actor, int dead) throws Exception {
        ActionOracleHarness h=new ActionOracleHarness(currentProgram);
        try {
            byte[] map=new byte[16200];ByteBuffer tiles=ByteBuffer.wrap(map);for(int i=0;i<8100;i++)tiles.putShort(i*2,(short)1155);
            tiles.putShort((25*90+25)*2,(short)0).putShort((24*90+25)*2,(short)10).putShort((25*90+24)*2,(short)11);
            byte[] monsters=Arrays.copyOf(records(2,0),576);ByteBuffer.wrap(monsters).putShort(288+0xe,(short)1);
            if(dead>0)ByteBuffer.wrap(monsters).putShort((dead-1)*288+0x110,(short)0);
            byte[] warriors=new byte[3138];warriors[0x9df]=1;ByteBuffer.wrap(warriors).putShort(0x50,(short)100);
            h.write(0x4085b8,map);h.write(0x439d3c,monsters);h.write(0x448aa0,warriors);
            h.value(0x44709a,2,25);h.value(0x44709c,2,25);h.value(0x446f0a,2,24);h.value(0x446f0c,2,25);h.value(0x446f0e,2,25);h.value(0x446f10,2,24);
            h.value(0x442780,4,0);h.value(0x442784,4,0);h.value(0x447194,2,0);h.value(0x4470b8,2,actor);h.value(0x447196,2,Math.max(0,actor-10));h.value(0x4435d2,2,0);h.value(0x44374c,2,0);h.value(0x4435c4,2,1);
            h.enter(0x110e96,0x6ffe00,0x900000);h.value(0x6ffe04,2,1);boolean done=false;
            for(int step=0;step<30000;step++) {
                long pc=h.pc(),sp=h.sp();if(pc==0x900000){done=true;break;}
                if(pc==0x13154c){h.ret();continue;}
                if(pc==0x400038){h.cpu.writeRegister("D0",Math.abs(h.word(sp+4)));h.ret();continue;}
                if(pc==0x153ca4)throw new Exception("Target list consumed RNG");
                if(!h.cpu.step(monitor))throw new Exception("Targets at "+Long.toHexString(pc)+": "+h.cpu.getLastError());
            }
            if(!done)throw new Exception("Target list budget");
            append("{\"kind\":\"targets\",\"actor\":"+actor+",\"dead\":"+dead+",\"map\":\""+hex(map)+"\",\"records_before\":\""+hex(monsters)+"\",\"records_after\":\""+hex(h.read(0x439d3c,monsters.length))+"\",\"warrior_before\":\""+hex(warriors)+"\",\"warrior_after\":\""+hex(h.read(0x448aa0,warriors.length))+"\",\"target_count\":"+(short)h.cpu.readRegister("D0").intValue()+",\"targets\":\""+hex(h.read(0x4434c4,24))+"\",\"rng_draws\":0}");
        }finally{h.cpu.dispose();}
    }
}
