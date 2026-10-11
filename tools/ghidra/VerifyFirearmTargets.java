import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;

/** Original enemy firearm targeting with separated near and far warriors. */
public class VerifyFirearmTargets extends VerifyEnemySpells {
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
        for(int i=0;i<6;i++){System.arraycopy(Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Warriors/"+names[i])),0,party,i*3138,3138);party[i*3138+0x9df]=1;ByteBuffer.wrap(party).putShort(i*3138+0x50,(short)10000).putShort(i*3138+0x52,(short)10000);}
        StringBuilder cases=new StringBuilder();int total=0;
        for(int weapon:new int[]{680,681})for(int initial:new int[]{12,1,2,3,4,5,6,7})for(int distance:new int[]{0,2,8,12}) {
            ActionOracleHarness h=frame();RunResult result=new RunResult(initial);
            try {
                byte[] caster=monster(76,10000);ByteBuffer.wrap(caster).putShort(0xe,(short)1).putShort(0x10e,(short)350).putShort(0x26,(short)weapon).putShort(0x4c,(short)0);caster[0x106]=20;caster[0x109]=7;
                byte[] map=terrain();ByteBuffer field=ByteBuffer.wrap(map);field.putShort((25*90+25)*2,(short)10);for(int i=0;i<6;i++)field.putShort(((25+distance+i*4)*90+25)*2,(short)i);
                state(h,caster,map);h.value(0x43874a,2,1);h.value(0x446f0a,2,20);h.value(0x446f0c,2,20);for(int i=0;i<6;i++){h.value(0x44709a+i*4,2,20+distance+i*4);h.value(0x44709c+i*4,2,20);}
                h.value(0x45365c,2,0);h.cpu.writeRegister("D5",0);h.jump(0x1146ee);execute(h,result,0x114b78);
                byte[] preparedCaster=h.read(0x439d3c,288),preparedPacket=h.read(0x44374e,58);int shots=h.word(0x4537b4);execute(h,result,0x1150d0);
                if(cases.length()>0)cases.append(',');cases.append("{\"weapon_id\":").append(weapon).append(",\"distance\":").append(distance).append(",\"seed_before\":").append(initial).append(",\"packet\":\"").append(hex(preparedPacket)).append("\",\"shots\":").append(shots).append(",\"caster_before\":\"").append(hex(preparedCaster)).append("\",\"caster_after\":\"").append(hex(h.read(0x439d3c,288))).append("\",\"map\":\"").append(hex(map)).append("\",\"party\":\"").append(hex(party)).append("\",\"range\":").append(h.word(0x4435bc)).append(",\"nearest\":").append(h.word(0x4436b4)).append(",\"distances\":\"").append(hex(h.read(0x4435d6,224))).append("\",\"count\":").append(h.word(0x4435cc)).append(",\"target_count\":").append(h.word(0x4435c0)).append(",\"targets\":\"").append(hex(h.read(0x443722,42))).append("\",\"cast\":").append(h.pc()==0x1150d0).append(",\"seed_after\":").append(result.seed).append(",\"rng_draws\":").append(result.draws).append(",\"draws\":[").append(result.drawRecords).append("]}");total++;
            }finally{h.cpu.dispose();}
        }
        Files.writeString(root.resolve("evidence/fixtures/firearm-targets.json"),"{\"authority\":\"Original enemy weapon preparation1146ee through114b78, then shared combat ranged targeting through first cast1150d0 or no-cast11510a. Weapons680/681 with loaded work-item word32 zero and separated warrior positions4 cells apart; full original getrange/checkrange/cansee and caster mutation. Controlled flat Swamp terrain. No cast or live battle claim.\",\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyFirearmTargets.java"))+"\",\"parent_probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyEnemySpells.java"))+"\",\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\",\"areas\":\""+hex(areas)+"\",\"properties\":\""+hex(properties)+"\",\"cases\":["+cases+"]}\n");println(total+" original firearm target cases exported");
    }
}
