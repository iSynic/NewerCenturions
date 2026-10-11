import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;

/** Enemy firearm preparation and complete resolution against a friendly body. */
public class VerifyEnemyFirearmBodies extends VerifyEnemySpells {
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]);
        items=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data ID"));
        races=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data Race"));
        monsters=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data MD"));
        properties=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data Swamp BD"));
        attacks=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data S"));
        areas=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data AD"));
        scenarioItems=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data NI"));
        party=new byte[6*3138];String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};
        for(int i=0;i<6;i++)System.arraycopy(Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Warriors/"+names[i])),0,party,i*3138,3138);
        StringBuilder cases=new StringBuilder();int total=0;
        for(int weapon:new int[]{680,681})for(int casterId:new int[]{12,26,50})for(int initial:new int[]{12,1,2,3,4,5,6,7})for(int variant=0;variant<4;variant++) {
            ActionOracleHarness h=frame();RunResult result=new RunResult(initial);
            try {
                byte[] caster=monster(casterId,10000),target=monster(65,10000),records=new byte[576];
                ByteBuffer.wrap(caster).putShort(0xe,(short)1).putShort(0x26,(short)weapon).putShort(0x4c,(short)0);
                if(variant==1)target[0xb3]=1;
                if(variant==2)ByteBuffer.wrap(target).putShort(0x56,(short)80).putShort(0x58,(short)300);
                if(variant==3){ByteBuffer.wrap(target).putShort(0x62,(short)1000);target[0x104]=7;}
                System.arraycopy(caster,0,records,0,288);System.arraycopy(target,0,records,288,288);
                state(h,records,terrain());h.value(0x43874a,2,2);h.value(0x45365c,2,0);h.cpu.writeRegister("D5",0);h.jump(0x1146ee);execute(h,result,0x114b78);
                byte[] prepared=h.read(0x44374e,58),preparedCaster=h.read(0x439d3c,288);int shots=h.word(0x4537b4),sound=h.word(0x453776);long prepDraws=result.draws;
                h.write(0x4434e0,new byte[220]);h.value(0x4434e0+22,2,1);h.value(0x4435cc,2,shots);h.value(0x438760,2,1);h.value(0x443580,1,0);h.value(0x4358c2,1,0);h.write(0x435884,new byte[128]);
                h.enter(0x151022,0x6ffe00,0x900000);execute(h,result,0x900000);
                if(cases.length()>0)cases.append(',');cases.append("{\"weapon_id\":").append(weapon).append(",\"caster_id\":").append(casterId).append(",\"variant\":").append(variant).append(",\"seed_before\":").append(initial).append(",\"packet\":\"").append(hex(prepared)).append("\",\"shots\":").append(shots).append(",\"weapon_sound\":").append(sound).append(",\"preparation_draws\":").append(prepDraws).append(",\"caster_before\":\"").append(hex(preparedCaster)).append("\",\"caster_after\":\"").append(hex(h.read(0x439d3c,288))).append("\",\"target_before\":\"").append(hex(target)).append("\",\"target_after\":\"").append(hex(h.read(0x439d3c+288,288))).append("\",\"seed_after\":").append(result.seed).append(",\"rng_draws\":").append(result.draws).append(",\"sounds\":[").append(result.sounds).append("]}");total++;
            }finally{h.cpu.dispose();}
        }
        Files.writeString(root.resolve("evidence/fixtures/enemy-firearm-bodies.json"),"{\"authority\":\"Original enemy preparation1146ee..114b78 and complete resolvespell151022 with explicit single scientist65 target mask and enemy caster12/26/50. Weapons680/681, no ammo, controlled target HP10000, variants immobilization/shield/very high defense. Original resist/resolvedamage, full caster/target records, sound order and RNG. Target selection, death and live battle outside probe.\",\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyEnemyFirearmBodies.java"))+"\",\"parent_probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyEnemySpells.java"))+"\",\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\",\"cases\":["+cases+"]}\n");println(total+" original enemy firearm body cases exported");
    }
}
