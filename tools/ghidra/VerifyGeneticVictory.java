import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.HexFormat;
import java.util.regex.Pattern;

public class VerifyGeneticVictory extends VerifyStoryInventory {
    long number(String object,String key) throws Exception {
        var m=Pattern.compile("\\\""+key+"\\\":(-?[0-9]+)").matcher(object);if(!m.find())throw new Exception("Missing "+key);return Long.parseLong(m.group(1));
    }
    byte[] bytes(String object,String key) throws Exception {
        var m=Pattern.compile("\\\""+key+"\\\":\\\"([0-9a-f]+)\\\"").matcher(object);if(!m.find())throw new Exception("Missing "+key);return HexFormat.of().parseHex(m.group(1));
    }
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]);items=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data ID"));scenarioItems=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data NI"));races=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data Race"));party=new byte[3138*6];
        String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};for(int i=0;i<6;i++)System.arraycopy(Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Warriors/"+names[i])),0,party,i*3138,3138);
        String input=Files.readString(root.resolve("evidence/fixtures/genetic-enemies.json"));var matcher=Pattern.compile("\\{\\\"battle_id\\\":[^}]+\\}").matcher(input);StringBuilder cases=new StringBuilder();int total=0;
        while(matcher.find()){probe(matcher.group(),cases);total++;}if(total!=12)throw new Exception("Expected twelve genetic setup cases, got "+total);
        String identity="\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyGeneticVictory.java"))+"\",\"inventory_probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyStoryInventory.java"))+"\",\"setup_fixture_sha256\":\""+ActionOracleHarness.sha256(root.resolve("evidence/fixtures/genetic-enemies.json"))+"\",\"harness_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/ActionOracleHarness.java"))+"\",\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\",\"items_sha256\":\""+ActionOracleHarness.sha256(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data ID"))+"\",\"scenario_items_sha256\":\""+ActionOracleHarness.sha256(root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data NI"))+"\",\"ghidra\":\"12.1.2\"";
        Files.writeString(root.resolve("evidence/fixtures/genetic-victory.json"),"{\"authority\":\"Original booty victory1057da..1064c2 executes bodycount cleanup, actual drop IDs, randomitem initialization, full original loot sort and reward arithmetic. Enemies are independently generated BD8/11/16/19/22/25 records with only HP110 forced0 and originalmaxHP112 preserved. Supplied warriors have controlled participation1 and killcounter=count. Randomness continues each original setupafterseed. Item loading serves exact definitions; presentation, mapdrawing and profileloading are recorded boundaries; SANE modeled. No executed battle or livevictory claim.\",\"cache_identity\":{"+identity+"},\"cases\":["+cases+"]}\n");println("Twelve genetic victory cases exported");
    }
    void probe(String input,StringBuilder cases) throws Exception {
        int battle=(int)number(input,"battle_id"),count=(int)number(input,"count");long setupSeed=number(input,"seed_before"),initial=number(input,"seed_after");byte[] enemies=bytes(input,"records");for(int i=0;i<count;i++)ByteBuffer.wrap(enemies).putShort(i*288+0x110,(short)0);
        byte[] warriors=party.clone();for(int i=0;i<6;i++){warriors[i*3138+0x9df]=1;ByteBuffer.wrap(warriors).putShort(i*3138+0x20,(short)(i+1)).putShort(i*3138+0x18,(short)count);}
        byte[] map=bytes(input,"tiles"),positions=bytes(input,"positions"),order=new byte[220];ByteBuffer ob=ByteBuffer.wrap(order);for(int i=0;i<110;i++)ob.putShort(i*2,(short)-1);for(int i=0;i<6;i++)ob.putShort((i+1)*2,(short)i);for(int i=0;i<count;i++)ob.putShort((i+7)*2,(short)(10+i));
        ActionOracleHarness h=frame();RunResult result=new RunResult(initial);SaneBoundary sane=new SaneBoundary(h.cpu,currentProgram);
        try {
            common(h,warriors);h.write(0x439d3c,enemies);h.write(0x4085b8,map);h.write(0x446f0a,positions);h.write(0x4470b8,order);h.value(0x438776,2,1);h.value(0x4435c4,2,1);h.value(0x43874a,2,count);h.value(0x442780,4,-34);h.value(0x442784,4,-10);
            h.write(0x4537fe,Arrays.copyOfRange(items,0,30000));h.write(0x46978e,Arrays.copyOfRange(items,90000,120000));
            for(int i=0;i<6;i++){h.value(0x44709a+i*4,2,20+i);h.value(0x44709c+i*4,2,20);h.value(0x437a6e+i*2,2,1155);}
            h.value(0x700008,2,0);h.value(0x70000a,2,0);h.jump(0x1057da);boolean done=false;
            for(int step=0;step<1000000;step++) {
                long pc=h.pc();if(pc==0x1064c2){done=true;break;}
                if(pc==0x130afe){loadItem(h);continue;}
                if(pc==0x153ca4){result.rng(h);continue;}
                if(pc==0x133a18){result.sound(h);continue;}
                if(pc==0x400038){h.cpu.writeRegister("D0",Math.abs(h.word(h.sp()+4)));h.ret();continue;}
                if(pc==0x1046e0||pc==0x13e13c||pc==0x13a7c4||pc==0x13154c||pc==0x13a2c4||pc==0x12382a||pc==0x160fb2||pc==0x158454||pc==0x1205f8||pc==0x105572||pc==0x1382a8){h.ret();continue;}
                int op=h.word(pc)&65535;if(op==0xa9eb){sane.execute();continue;}
                if(op==0xa939){h.pop(6);h.jump(pc+2);continue;}if(op==0xaa1c||op==0xa93c||op==0xa873){h.pop(4);h.jump(pc+2);continue;}if(op==0xa937||op==0xa89b){h.jump(pc+2);continue;}
                if(op==0xaa1e){h.value(h.sp()+2,4,0);h.pop(2);h.jump(pc+2);continue;}
                if(!h.cpu.step(monitor))throw new Exception("Genetic victory BD"+battle+" PC "+Long.toHexString(pc)+": "+h.cpu.getLastError());
            }
            if(!done)throw new Exception("Genetic victory budget");if(cases.length()>0)cases.append(',');
            cases.append("{\"battle_id\":").append(battle).append(",\"setup_seed\":").append(setupSeed).append(",\"count\":").append(count).append(",\"warriors_before\":\"").append(hex(warriors)).append("\",\"warriors_after\":\"").append(hex(h.read(0x448aa0,warriors.length))).append("\",\"enemies_before\":\"").append(hex(enemies)).append("\",\"enemies_after\":\"").append(hex(h.read(0x439d3c,enemies.length))).append("\",\"map_before\":\"").append(hex(map)).append("\",\"map_after\":\"").append(hex(h.read(0x4085b8,16200))).append("\",\"positions\":\"").append(hex(positions)).append("\",\"order_before\":\"").append(hex(order)).append("\",\"order_after\":\"").append(hex(h.read(0x4470b8,220))).append("\",\"loot_after\":\"").append(hex(h.read(0x41fc74,22000))).append("\",\"loot_flags\":\"").append(hex(h.read(0x42df04,1000))).append("\",\"seed_before\":").append(initial).append(",\"seed_after\":").append(result.seed).append(",\"draws\":").append(result.draws).append(",\"share\":").append((int)h.pointer(0x6fff6a)).append(",\"sounds\":[").append(result.sounds).append("]}");
        }finally{h.cpu.dispose();}
    }
}
