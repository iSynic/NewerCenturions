import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;

/** Original ranged-weapon rejection before a natural melee attack. */
public class VerifyRequiredWeaponFallback extends VerifyAllyTurns {
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]),original=root.resolve("NewCenturions/TheNewCenturions"),scenario=original.resolve("Scenarios/The Thunder of Guns/0");
        items=Files.readAllBytes(original.resolve("Data Files/Data ID"));scenarioItems=Files.readAllBytes(scenario.resolve("Data NI"));races=Files.readAllBytes(original.resolve("Data Files/Data Race"));monsters=Files.readAllBytes(scenario.resolve("Data MD"));properties=Files.readAllBytes(original.resolve("Data Files/Data Swamp BD"));attacks=Files.readAllBytes(original.resolve("Data Files/Data S"));areas=Files.readAllBytes(original.resolve("Data Files/Data AD"));party=new byte[3138*6];
        String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};
        for(int i=0;i<6;i++){System.arraycopy(Files.readAllBytes(original.resolve("Warriors/"+names[i])),0,party,i*3138,3138);party[i*3138+0x9df]=1;ByteBuffer.wrap(party).putShort(i*3138+0x50,(short)100).putShort(i*3138+0x52,(short)100);}
        StringBuilder cases=new StringBuilder();
        for(long seed:new long[]{12,1})for(int id:new int[]{24,25,26})for(int member=0;member<6;member++){
            ActionOracleHarness h=frame();RunResult result=new RunResult(seed);
            try{
                byte[] record=monster(id,100);ByteBuffer.wrap(record).putShort(0x28,(short)0);record[0x107]=1;
                state(h,record,terrain());h.enter(0x101eb6,0x6ffe00,0x900000);h.value(0x6ffe04,2,10);h.value(0x6ffe06,2,member);h.value(0x6ffe08,2,0);execute(h,result,0x900000);
                if(cases.length()>0)cases.append(',');cases.append("{\"seed_before\":").append(seed).append(",\"monster_id\":").append(id).append(",\"member\":").append(member).append(",\"attack\":0,\"warriors_before\":\"").append(hex(party)).append("\",\"warriors_after\":\"").append(hex(h.read(0x448aa0,party.length))).append("\",\"enemy_before\":\"").append(hex(record)).append("\",\"enemy_after\":\"").append(hex(h.read(0x439d3c,288))).append("\",\"packet\":\"").append(hex(h.read(0x44374e,58))).append("\",\"seed_after\":").append(result.seed).append(",\"rng_draws\":").append(result.draws).append(",\"draws\":[").append(result.drawRecords).append("],\"sounds\":[").append(result.sounds).append("]}");
            }finally{h.cpu.dispose();}
        }
        StringBuilder identity=new StringBuilder();String[][] inputs={{"probe","tools/ghidra/VerifyRequiredWeaponFallback.java"},{"parent_probe","tools/ghidra/VerifyAllyTurns.java"},{"inventory_probe","tools/ghidra/VerifyStoryInventory.java"},{"harness","tools/ghidra/ActionOracleHarness.java"},{"sane_boundary","tools/ghidra/SaneBoundary.java"},{"mapped_binary","analysis/ghidra/startup/code1-mapped.bin"},{"monsters","NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data MD"},{"items","NewCenturions/TheNewCenturions/Data Files/Data ID"},{"scenario_items","NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data NI"},{"races","NewCenturions/TheNewCenturions/Data Files/Data Race"},{"properties","NewCenturions/TheNewCenturions/Data Files/Data Swamp BD"},{"attacks","NewCenturions/TheNewCenturions/Data Files/Data S"},{"areas","NewCenturions/TheNewCenturions/Data Files/Data AD"}};
        for(String[] input:inputs){if(identity.length()>0)identity.append(',');identity.append('"').append(input[0]).append("_sha256\":\"").append(ActionOracleHarness.sha256(root.resolve(input[1]))).append('"');}
        for(int i=0;i<6;i++)identity.append(",\"warrior").append(i).append("_sha256\":\"").append(ActionOracleHarness.sha256(original.resolve("Warriors/"+names[i]))).append('"');
        Files.writeString(root.resolve("evidence/fixtures/required-weapon-fallback.json"),"{\"authority\":\"Original complete attack2 at101eb6 for MD24/25/26 with authored current weapons26/50/12 and controlled default melee word28 zero, facing byte107 one, original six warriors at controlledHP100. Original loadweapon rejects class15 for melee and clears current weapon word4e before the native natural damage path. Both seeds12/1 and every warrior target; original SANE and all item caches supplied. Target choice, presentation showresults and killbody are acknowledged boundaries. Full party, attacker, packet and ordered RNG observed; no live battle claim.\",\"identity\":{"+identity+"},\"cases\":["+cases+"]}\n");println("Thirty-six original ranged-to-natural fallback cases exported");
    }
}
