import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;

/** Original spell area masks, reflection and lingering-field preparation. */
public class VerifySpellTargets extends VerifyEnemySpells {
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
        byte[] map=terrain();ByteBuffer field=ByteBuffer.wrap(map);for(int i=0;i<6;i++)field.putShort(((25+i)*90+25)*2,(short)i);field.putShort((25*90+27)*2,(short)10).putShort((27*90+27)*2,(short)11);
        StringBuilder cases=new StringBuilder();int total=0;
        for(int id:new int[]{2101,2106,2102,2203,2507,2307,2202})for(int initial:new int[]{12,1,2,3,4,5,6,7})for(int count:new int[]{1,3,7})for(int variant=0;variant<3;variant++)for(int kind:new int[]{1,2,3}) {
            if(packet(id)[1]<2&&kind==3)continue;
            ActionOracleHarness h=frame();RunResult result=new RunResult(initial);
            try {
                byte[] records=new byte[576],caster=monster(13,10000),other=monster(65,10000),beforeParty=party.clone();
                ByteBuffer.wrap(caster).putShort(0xe,(short)1);if(variant==1){other[0xd0]=1;for(int i=0;i<6;i++)ByteBuffer.wrap(beforeParty).putShort(i*3138+0x9a,(short)1);}
                System.arraycopy(caster,0,records,0,288);System.arraycopy(other,0,records,288,288);
                state(h,records,map);h.write(0x448aa0,beforeParty);h.value(0x446f0a,2,20);h.value(0x446f0c,2,22);h.value(0x446f0e,2,22);h.value(0x446f10,2,22);for(int i=0;i<6;i++){h.value(0x44709a+i*4,2,20+i);h.value(0x44709c+i*4,2,20);}
                byte[] p=packet(id);h.write(0x44374e,p);h.value(0x4435c6,2,id/1000-1);h.value(0x4435c8,2,id%1000/100-1);h.value(0x4435ca,2,id%100-1);h.value(0x4435cc,2,count);h.value(0x447194,2,1);h.value(0x4470ba,2,10);
                byte[] fields=new byte[1200];if(variant==2)for(int i=0;i<60;i++)fields[i*20+15]=1;h.write(0x442bd6,fields);
                int first=kind==1?2:kind==2?11:27,second=kind==3?25:-1;h.value(0x443722,2,first);h.value(0x443724,2,second);h.value(0x443726,2,kind);
                h.enter(0x15cb0e,0x6ffe00,0x900000);h.value(0x6ffe04,2,0);h.value(0x6ffe06,2,-1);h.value(0x6ffe08,2,0);h.value(0x6ffe0a,2,0);execute(h,result,0x151022);
                if(cases.length()>0)cases.append(',');cases.append("{\"spell_id\":").append(id).append(",\"count\":").append(count).append(",\"variant\":").append(variant).append(",\"kind\":").append(kind).append(",\"first\":").append(first).append(",\"second\":").append(second).append(",\"seed_before\":").append(initial).append(",\"packet\":\"").append(hex(p)).append("\",\"records\":\"").append(hex(records)).append("\",\"mask\":\"").append(hex(h.read(0x4434e0,220))).append("\",\"fields\":\"").append(hex(h.read(0x442bd6,1200))).append("\",\"seed_after\":").append(result.seed).append(",\"rng_draws\":").append(result.draws).append(",\"draws\":[").append(result.drawRecords).append("]}");total++;
            }finally{h.cpu.dispose();}
        }
        Files.writeString(root.resolve("evidence/fixtures/spell-targets.json"),"{\"authority\":\"Original spelltargets15cb0e through first resolvespell151022: native7x7 body mask, reflection draws and lingering20-byte field table mutation. Seven MD13/MD94 packets excluding beam, exact Data AD, six starter warriors and two controlled monsters. Target kinds warrior/monster/point, counts1/3/7; variants reflection and full60-field table. Resolution and cast projectile geometry are outside this probe.\",\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifySpellTargets.java"))+"\",\"parent_probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyEnemySpells.java"))+"\",\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\",\"areas\":\""+hex(areas)+"\",\"party\":\""+hex(party)+"\",\"map\":\""+hex(map)+"\",\"cases\":["+cases+"]}\n");println(total+" original spell target-mask cases exported");
    }
}
