import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.HexFormat;
import java.util.regex.Pattern;

public class VerifyLootCollection extends VerifyStoryInventory {
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]);items=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data ID"));scenarioItems=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data NI"));races=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data Race"));
        party=new byte[3138*6];String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};for(int i=0;i<6;i++)System.arraycopy(Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Warriors/"+names[i])),0,party,i*3138,3138);
        var matcher=Pattern.compile("\\\"treasure\\\":85,[^}]*\\\"crate_after\\\":\\\"([0-9a-f]+)\\\"").matcher(Files.readString(root.resolve("evidence/fixtures/story-inventory.json")));if(!matcher.find())throw new Exception("Missing original treasure85 item");byte[] loot=HexFormat.of().parseHex(matcher.group(1).substring(0,44));
        StringBuilder cases=new StringBuilder();for(int member=0;member<6;member++)probe(member,0,loot,cases);for(int variant=1;variant<6;variant++)probe(0,variant,loot,cases);for(int id=804;id<=809;id++){byte[] quest=loot.clone();ByteBuffer.wrap(quest).putShort(0,(short)id);quest[10]=(byte)ByteBuffer.wrap(scenarioItems).getShort((id-800)*150+0x1a);probe(0,id-798,quest,cases);}
        String identity="\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyLootCollection.java"))+"\",\"inventory_probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyStoryInventory.java"))+"\",\"inventory_fixture_sha256\":\""+ActionOracleHarness.sha256(root.resolve("evidence/fixtures/story-inventory.json"))+"\",\"harness_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/ActionOracleHarness.java"))+"\",\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\",\"items_sha256\":\""+ActionOracleHarness.sha256(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data ID"))+"\",\"scenario_items_sha256\":\""+ActionOracleHarness.sha256(root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data NI"))+"\",\"races_sha256\":\""+ActionOracleHarness.sha256(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data Race"))+"\",\"ghidra\":\"12.1.2\"";
        Files.writeString(root.resolve("evidence/fixtures/loot-collection.json"),"{\"authority\":\"Original booty default collection108c9e through loot ID clear109198, including original twixt, signed definition checks, capacity99, unsigned weight comparison, installitem12c874 mode-2, calcw139192 and movecalc136994. Input833 comes from independently executed original treasure85 seed12; inputs804..809 are synthetic ordinary quantity1 records with exact original definition kinds. Modifier BitAnd Toolbox returns no modifiers; quiet, sound, warning and updatebooty presentation are recorded boundaries. Complete supplied warriors and original item/race definitions used. No UI pointer or live pickup claim.\",\"cache_identity\":{"+identity+"},\"rng_draws\":0,\"cases\":["+cases+"]}\n");println("Original loot collection cases exported");
    }
    void probe(int member,int variant,byte[] loot,StringBuilder cases) throws Exception {
        byte[] before=party.clone();int base=member*3138,count=ByteBuffer.wrap(before).getShort(base+0x1a),slot=base+0x132+count*22;
        before[slot+11]=(byte)0x5a;before[slot+12]=(byte)0x9f;
        if(variant==1)ByteBuffer.wrap(before).putShort(base+0x1a,(short)99);
        if(variant==2)ByteBuffer.wrap(before).putShort(base+0x9d6,(short)0xffff).putShort(base+0x9d8,(short)0x7fff);
        if(variant==3)ByteBuffer.wrap(before).putShort(base+0x9d6,(short)0x8000).putShort(base+0x9d8,(short)0xffff);
        if(variant==4)ByteBuffer.wrap(before).putShort(base+0x9d6,(short)0).putShort(base+0x9d8,(short)0);
        if(variant==5)ByteBuffer.wrap(before).putShort(base+0x9d6,(short)0x7fff).putShort(base+0x9d8,(short)0x7fff);
        ActionOracleHarness h=frame();RunResult result=new RunResult(12);SaneBoundary sane=new SaneBoundary(h.cpu,currentProgram);StringBuilder warnings=new StringBuilder();
        try {
            common(h,before);h.value(0x44273f,1,member);h.value(0x6fffce,2,0);h.value(0x6fff9f,1,0);h.value(0x70000a,2,0);h.value(0x447218,2,0);h.write(0x41fc74,loot);h.jump(0x108c9e);boolean done=false;
            for(int step=0;step<100000;step++) {
                long pc=h.pc();if(pc==0x10919c||pc==0x106dba){done=true;break;}
                if(pc==0x130afe){loadItem(h);continue;}
                if(pc==0x400038){h.cpu.writeRegister("D0",Math.abs(h.word(h.sp()+4)));h.ret();continue;}
                if(pc==0x133a18){result.sound(h);continue;}
                if(pc==0x153ca4){result.rng(h);continue;}
                if(pc==0x16474a){if(warnings.length()>0)warnings.append(',');warnings.append(h.word(h.sp()+4));h.ret();continue;}
                if(pc==0x13b56c||pc==0x15f4b8){h.ret();continue;}
                int op=h.word(pc)&65535;if(op==0xa9eb){sane.execute();continue;}
                if(op==0xa858){h.pop(8);h.value(h.sp(),4,0);h.jump(pc+2);continue;}
                if(!h.cpu.step(monitor))throw new Exception("Collection PC "+Long.toHexString(pc)+": "+h.cpu.getLastError());
            }
            if(!done)throw new Exception("Collection budget");if(cases.length()>0)cases.append(',');
            byte[] after=h.read(0x448aa0,party.length),lootAfter=h.read(0x41fc74,22);int race=ByteBuffer.wrap(before).getShort(base+0x32)-1;
            cases.append("{\"member\":").append(member).append(",\"variant\":").append(variant).append(",\"race_movement\":").append(races[race*408+197]).append(",\"before\":\"").append(hex(Arrays.copyOfRange(before,base,base+3138))).append("\",\"after\":\"").append(hex(Arrays.copyOfRange(after,base,base+3138))).append("\",\"party_before\":\"").append(hex(before)).append("\",\"party_after\":\"").append(hex(after)).append("\",\"loot_before\":\"").append(hex(loot)).append("\",\"loot_after\":\"").append(hex(lootAfter)).append("\",\"collected\":").append(h.pc()==0x10919c).append(",\"sounds\":[").append(result.sounds).append("],\"warnings\":[").append(warnings).append("],\"rng_draws\":").append(result.draws).append('}');
        }finally{h.cpu.dispose();}
    }
}
