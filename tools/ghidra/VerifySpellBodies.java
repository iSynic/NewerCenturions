import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;

/** Original direct spell resolution in ascending body order. */
public class VerifySpellBodies extends VerifyEnemySpells {
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
        for(int id:new int[]{2103,2101,2106,2102,2203,2507,2307,2202})for(int initial:new int[]{12,1,2,3})for(int count:new int[]{1,3,7})for(int variant=0;variant<6;variant++) {
            ActionOracleHarness h=frame();RunResult result=new RunResult(initial);
            try {
                byte[] records=new byte[864];int[] ids={13,65,94};
                for(int i=0;i<3;i++){
                    byte[] record=monster(ids[i],10000);
                    if(variant==1)record[0xc2]=1;
                    if(variant==2)ByteBuffer.wrap(record).putShort(0xa,(short)100);
                    if(variant==3)record[0x8b+packet(id)[19]]=101;
                    if(variant==4)ByteBuffer.wrap(record).putShort(0x56,(short)80).putShort(0x58,(short)300);
                    System.arraycopy(record,0,records,i*288,288);
                }
                byte[] mask=new byte[220];for(int i=0;i<6;i++)if(variant!=5||i%2==0)ByteBuffer.wrap(mask).putShort(i*2,(short)1);for(int i=0;i<3;i++)if(variant!=5||i!=1)ByteBuffer.wrap(mask).putShort((i+10)*2,(short)1);
                state(h,records,terrain());h.value(0x43874a,2,3);h.write(0x448aa0,party);h.write(0x44374e,packet(id));h.value(0x4435c6,2,id/1000-1);h.value(0x4435c8,2,id%1000/100-1);h.value(0x4435ca,2,id%100-1);h.value(0x4435cc,2,count);
                h.write(0x453768,new byte[150]);h.write(0x4536d2,new byte[150]);h.write(0x4434e0,mask);
                h.value(0x438760,2,1);h.value(0x443580,1,0);h.value(0x4358c2,1,0);h.write(0x435884,new byte[128]);
                h.enter(0x151022,0x6ffe00,0x900000);execute(h,result,0x900000);
                if(cases.length()>0)cases.append(',');cases.append("{\"spell_id\":").append(id).append(",\"count\":").append(count).append(",\"variant\":").append(variant).append(",\"seed_before\":").append(initial).append(",\"packet\":\"").append(hex(packet(id))).append("\",\"mask\":\"").append(hex(mask)).append("\",\"warriors_before\":\"").append(hex(party)).append("\",\"warriors_after\":\"").append(hex(h.read(0x448aa0,party.length))).append("\",\"enemies_before\":\"").append(hex(records)).append("\",\"enemies_after\":\"").append(hex(h.read(0x439d3c,records.length))).append("\",\"seed_after\":").append(result.seed).append(",\"rng_draws\":").append(result.draws).append(",\"sounds\":[").append(result.sounds).append("]}");total++;
            }finally{h.cpu.dispose();}
        }
        Files.writeString(root.resolve("evidence/fixtures/spell-bodies.json"),"{\"authority\":\"Complete original resolvespell151022 through return, six high-HP starter warriors followed by controlled MD13/65/94 monsters. Eight mandatory MD13/94 packets, six variants normal/immunity/threshold100/typeimmunity/shield/subset mask. Explicit enemy caster and zero loaded weapon/ammo NIword12/14. Includes native resist, resolvedamage, full records, ordered sounds and RNG. No death, cast projectile or live battle claim. Presentation intercepted through VerifyEnemySpells; QuickDraw Rand and SANE modeled.\",\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifySpellBodies.java"))+"\",\"parent_probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyEnemySpells.java"))+"\",\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\",\"cases\":["+cases+"]}\n");println(total+" original spell body cases exported");
    }
}
