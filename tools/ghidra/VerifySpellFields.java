import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;

/** Original field timer, usage reset and complete lingering collision. */
public class VerifySpellFields extends VerifyAdditionalSpells {
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
        StringBuilder ticks=new StringBuilder();
        for(int cursor:new int[]{-1,0,5,63,126,127,255,32767}) {
            ActionOracleHarness h=frame();RunResult result=new RunResult(12);
            try {
                byte[] fields=new byte[1200],used=new byte[128];for(int i=0;i<128;i++)used[i]=(byte)(i+1);
                for(int i=0;i<60;i++){fields[i*20+15]=(byte)(i%3==0?0:i%3==1?1:255);fields[i*20+18]=(byte)(i%2==0?cursor+1:cursor+2);}
                h.write(0x442bd6,fields);h.write(0x435884,used);h.value(0x447194,2,cursor);h.value(0x44374c,2,0);h.value(0x4435c4,2,1);h.jump(0x125ca0);execute(h,result,0x125d18);
                if(ticks.length()>0)ticks.append(',');ticks.append("{\"cursor_before\":").append(cursor).append(",\"cursor_after\":").append(h.word(0x447194)).append(",\"fields_before\":\"").append(hex(fields)).append("\",\"fields_after\":\"").append(hex(h.read(0x442bd6,1200))).append("\",\"used_before\":\"").append(hex(used)).append("\",\"used_after\":\"").append(hex(h.read(0x435884,128))).append("\",\"rng_draws\":").append(result.draws).append("}");
            }finally{h.cpu.dispose();}
        }
        byte[] map=terrain();ByteBuffer field=ByteBuffer.wrap(map);for(int i=0;i<6;i++)field.putShort(((25+i)*90+25)*2,(short)i);field.putShort((25*90+27)*2,(short)10).putShort((27*90+27)*2,(short)11);
        StringBuilder cases=new StringBuilder();int total=0;
        for(int id:new int[]{2101,2102})for(int initial:new int[]{12,1,2,3})for(int count:new int[]{1,3,7})for(int variant=0;variant<4;variant++)for(int body:new int[]{2,10}) {
            ActionOracleHarness h=frame();RunResult result=new RunResult(initial);
            try {
                byte[] records=new byte[576],caster=monster(13,10000),other=monster(65,10000),fields=new byte[1200],used=new byte[128];ByteBuffer.wrap(caster).putShort(0xe,(short)1);System.arraycopy(caster,0,records,0,288);System.arraycopy(other,0,records,288,288);
                for(int slot:new int[]{0,1,59}){int o=slot*20;ByteBuffer.wrap(fields).putShort(o,(short)22).putShort(o+2,(short)22).putShort(o+4,(short)29).putShort(o+6,(short)29).putShort(o+8,(short)id).putShort(o+10,(short)8);fields[o+12]=22;fields[o+13]=22;fields[o+14]=packet(id)[31];fields[o+15]=3;fields[o+16]=(byte)count;fields[o+19]=(byte)(id%1000/100-1);}
                if(variant==1)used[0]=1;if(variant==2)used[1]=1;if(variant==3)used[60]=1;
                state(h,records,map);h.value(0x43874a,2,2);h.value(0x446f0a,2,20);h.value(0x446f0c,2,22);h.value(0x446f0e,2,22);h.value(0x446f10,2,22);for(int i=0;i<6;i++){h.value(0x44709a+i*4,2,20+i);h.value(0x44709c+i*4,2,20);}
                h.write(0x442bd6,fields);h.write(0x435884,used);h.value(0x438760,2,1);h.value(0x443580,1,0);h.value(0x4358c2,1,0);h.write(0x453768,new byte[150]);h.write(0x4536d2,new byte[150]);h.value(0x4435cc,2,6);
                h.enter(0x1140ae,0x6ffe00,0x900000);h.value(0x6ffe04,2,body);h.value(0x6ffe06,2,0);execute(h,result,0x900000);
                if(cases.length()>0)cases.append(',');cases.append("{\"spell_id\":").append(id).append(",\"count\":").append(count).append(",\"variant\":").append(variant).append(",\"body\":").append(body).append(",\"seed_before\":").append(initial).append(",\"warriors_before\":\"").append(hex(party)).append("\",\"warriors_after\":\"").append(hex(h.read(0x448aa0,party.length))).append("\",\"enemies_before\":\"").append(hex(records)).append("\",\"enemies_after\":\"").append(hex(h.read(0x439d3c,records.length))).append("\",\"fields_before\":\"").append(hex(fields)).append("\",\"fields_after\":\"").append(hex(h.read(0x442bd6,1200))).append("\",\"used_before\":\"").append(hex(used)).append("\",\"used_after\":\"").append(hex(h.read(0x435884,128))).append("\",\"count_after\":").append(h.word(0x4435cc)).append(",\"level_after\":").append(h.word(0x4435c8)).append(",\"seed_after\":").append(result.seed).append(",\"rng_draws\":").append(result.draws).append(",\"sounds\":[").append(result.sounds).append("]}");total++;
            }finally{h.cpu.dispose();}
        }
        Files.writeString(root.resolve("evidence/fixtures/spell-fields.json"),"{\"authority\":\"Original getup125ca0..125d18 field timer and first60-byte usage reset; exact all60fields/128usage including preserved usage60. Complete native collide1140ae through return, original spelltargets/resolvespell/reflect/resist/damage, overlapping fieldslots0/1/59 and explicitly stale usage0/1/60. Required2101/2102, controlled highHPstarterparty/MD13/65 bodies, no deaths or live battle claim.\",\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifySpellFields.java"))+"\",\"parent_probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyAdditionalSpells.java"))+"\",\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\",\"areas\":\""+hex(areas)+"\",\"map\":\""+hex(map)+"\",\"ticks\":["+ticks+"],\"cases\":["+cases+"]}\n");println("Eight field ticks and "+total+" complete native collisions exported");
    }
}
