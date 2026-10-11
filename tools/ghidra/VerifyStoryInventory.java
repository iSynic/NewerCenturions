import ghidra.app.script.GhidraScript;
import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.HexFormat;

public class VerifyStoryInventory extends GhidraScript {
    static String hex(byte[] value){return HexFormat.of().formatHex(value);}
    byte[] items,scenarioItems,races,treasures,party;
    ActionOracleHarness frame(){ActionOracleHarness h=new ActionOracleHarness(currentProgram);h.write(0x6fc000,new byte[24576]);h.cpu.writeRegister("A6",0x700000);h.cpu.writeRegister("SP",0x6ffe00);return h;}
    void common(ActionOracleHarness h,byte[] warriors) {
        h.write(0x448aa0,warriors);h.value(0x44273a,1,5);h.write(0x41fc32,new byte[22]);h.write(0x41fc74,new byte[22000]);h.write(0x42df04,new byte[1000]);h.write(0x46225e,Arrays.copyOfRange(items,30000,60000));
        for(int i=0;i<10;i++)h.value(0x435664+i,1,races[i*408+197]);
        h.value(0x43874a,2,0);h.value(0x4427a6,2,0);h.value(0x442576,2,0);h.value(0x44257c,2,0);h.value(0x44257e,2,0);h.value(0x442788,4,0);h.value(0x44278c,4,0);h.value(0x443392,4,0);h.value(0x4386fc,1,0);
    }
    byte[] add(byte[] warriors,int member,int id,int quantity,boolean duplicate) {
        byte[] result=warriors.clone();int base=member*3138,count=ByteBuffer.wrap(result).getShort(base+0x1a);
        int amount=duplicate?2:1;
        for(int i=0;i<amount;i++){int slot=base+0x132+(count+i)*22;Arrays.fill(result,slot,slot+22,(byte)0);ByteBuffer.wrap(result).putShort(slot,(short)id).putShort(slot+2,(short)quantity);result[slot+10]=23;}
        ByteBuffer.wrap(result).putShort(base+0x1a,(short)(count+amount));return result;
    }
    class RunResult {
        long seed;int draws;StringBuilder sounds=new StringBuilder(),drawRecords=new StringBuilder();
        RunResult(long seed){this.seed=seed;}
        void rng(ActionOracleHarness h){int bound=h.word(h.sp()+4);seed=seed*16807%2147483647;draws++;short raw=(short)seed;if(raw==Short.MIN_VALUE)raw=0;int result=Math.abs((int)raw)*bound/32768+1;h.cpu.writeRegister("D0",result);if(drawRecords.length()>0)drawRecords.append(',');drawRecords.append("{\"bound\":").append(bound).append(",\"result\":").append(result).append('}');h.ret();}
        void sound(ActionOracleHarness h){if(sounds.length()>0)sounds.append(',');sounds.append(h.word(h.sp()+4));h.ret();}
    }
    void loadItem(ActionOracleHarness h){int id=Math.abs(h.word(h.sp()+4));byte[] table=id>=800?scenarioItems:items;int slot=id>=800?id-800:id;h.write(0x45363c,Arrays.copyOfRange(table,slot*150,slot*150+150));h.ret();}
    public void run()throws Exception {
        Path root=Path.of(getScriptArgs()[0]);items=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data ID"));scenarioItems=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data NI"));races=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data Race"));treasures=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data TD"));party=new byte[3138*6];
        String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};for(int i=0;i<6;i++)System.arraycopy(Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Warriors/"+names[i])),0,party,i*3138,3138);
        StringBuilder consume=new StringBuilder(),query=new StringBuilder(),awards=new StringBuilder(),loot=new StringBuilder();
        for(int variant=0;variant<5;variant++) {
            int id=variant==4?804:833,limit=variant==1?1:variant==2?3:1;byte[] before=variant==3?party.clone():add(party,variant==4?4:0,id,1,variant==1||variant==2);
            byte[] extra=ByteBuffer.allocate(10).putShort((short)id).putShort((short)limit).putShort((short)1).putShort((short)0).putShort((short)0).array();
            ActionOracleHarness h=frame();RunResult result=new RunResult(12);SaneBoundary sane=new SaneBoundary(h.cpu,currentProgram);
            try {
                common(h,before);h.write(0x44330a,extra);h.jump(0x146560);boolean done=false;
                for(int step=0;step<500000;step++) {
                    long pc=h.pc();if(pc==0x14673c){done=true;break;}
                    if(pc==0x130afe){loadItem(h);continue;}
                    if(pc==0x133a18){result.sound(h);continue;}
                    if(pc==0x153ca4){result.rng(h);continue;}
                    if((h.word(pc)&65535)==0xa9eb){sane.execute();continue;}
                    if(!h.cpu.step(monitor))throw new Exception("Consume "+variant+" PC "+Long.toHexString(pc)+": "+h.cpu.getLastError());
                }
                if(!done)throw new Exception("Consumption instruction budget");if(consume.length()>0)consume.append(',');
                consume.append("{\"extra\":\"").append(hex(extra)).append("\",\"before\":\"").append(hex(before)).append("\",\"after\":\"").append(hex(h.read(0x448aa0,party.length))).append("\",\"consumed\":").append(h.cpu.readRegister("D6").intValue()).append(",\"sounds\":[").append(result.sounds).append("],\"seed\":").append(result.seed).append(",\"rng_draws\":").append(result.draws).append(",\"crate_after\":\"").append(hex(h.read(0x41fc74,22000))).append("\"}");
            }finally{h.cpu.dispose();}
        }
        for(int variant=0;variant<6;variant++) {
            int id=variant==5?804:833,mode=variant%3;byte[] before=variant<3?party.clone():add(party,4,id,0,false);
            byte[] extra=ByteBuffer.allocate(10).putShort((short)id).putShort((short)mode).putShort((short)0).putShort((short)53).putShort((short)0).array();ActionOracleHarness h=frame();
            try {
                common(h,before);h.write(0x44330a,extra);h.jump(0x144f96);boolean done=false;
                for(int step=0;step<5000;step++) {
                    long pc=h.pc();if(pc==0x144fe4||pc==0x14673c){done=true;break;}
                    if(!h.cpu.step(monitor))throw new Exception("Query PC "+Long.toHexString(pc)+": "+h.cpu.getLastError());
                }
                if(!done)throw new Exception("Query budget");if(query.length()>0)query.append(',');
                query.append("{\"extra\":\"").append(hex(extra)).append("\",\"before\":\"").append(hex(before)).append("\",\"after\":\"").append(hex(h.read(0x448aa0,party.length))).append("\",\"matched\":").append(h.pc()==0x144fe4).append(",\"holder\":").append(h.word(0x4471ea)).append(",\"slot\":").append(h.word(0x438748)).append(",\"rng_draws\":0}");
            }finally{h.cpu.dispose();}
        }
        int[] awardAmounts={8000,2000,-1,8000};
        for(int variant=0;variant<7;variant++)for(int initial:new int[]{12,1}) {
            boolean award=variant<4;int treasure=variant==4?5:variant==5?6:85,amount=award?awardAmounts[variant]:0;
            byte[] before=party.clone();if(variant==3)ByteBuffer.wrap(before).putShort(0x50,(short)0);
            for(int j=0;j<6;j++)before[j*3138+0x9df]=0;
            byte[] table=award?new byte[88]:Arrays.copyOfRange(treasures,treasure*88,treasure*88+88);
            ActionOracleHarness h=frame();RunResult result=new RunResult(initial);SaneBoundary sane=new SaneBoundary(h.cpu,currentProgram);
            try {
                common(h,before);h.write(0x44331c,table);
                if(award) {
                    h.value(0x6fffea,2,amount);h.jump(0x1435d8);boolean entered=false;
                    for(int step=0;step<500;step++) {
                        if(h.pc()==0x105748){if(h.word(h.sp()+4)!=12||h.word(h.sp()+6)!=0)throw new Exception("Unexpected upgradebooty mode");entered=true;break;}
                        if(!h.cpu.step(monitor))throw new Exception("Original11 setup: "+h.cpu.getLastError());
                    }
                    if(!entered)throw new Exception("Upgrade setup budget");
                    // Continue the separately bounded original booty preparation with its own frame.
                    h.cpu.writeRegister("A6",0x700000);h.cpu.writeRegister("SP",0x6ffe00);
                }
                h.value(0x700008,2,award?12:1);h.value(0x70000a,2,0);h.jump(0x1058ec);boolean done=false;
                for(int step=0;step<1000000;step++) {
                    long pc=h.pc();if(pc==0x1064c2){done=true;break;}
                    if(pc==0x130afe){loadItem(h);continue;}
                    if(pc==0x153ca4){result.rng(h);continue;}
                    if(pc==0x133a18){result.sound(h);continue;}
                    if(pc==0x13a7c4||pc==0x1046e0){h.ret();continue;}
                    if(pc==0x400038){h.cpu.writeRegister("D0",Math.abs(h.word(h.sp()+4)));h.ret();continue;}
                    int op=h.word(pc)&65535;
                    if(op==0xa9eb){sane.execute();continue;}
                    if(op==0xa939){h.pop(6);h.jump(pc+2);continue;}
                    if(op==0xaa1c||op==0xa93c){h.pop(4);h.jump(pc+2);continue;}
                    if(op==0xa937){h.jump(pc+2);continue;}
                    if(!h.cpu.step(monitor))throw new Exception("Booty "+variant+" PC "+Long.toHexString(pc)+": "+h.cpu.getLastError());
                }
                if(!done)throw new Exception("Booty prep budget");StringBuilder output=award?awards:loot;if(output.length()>0)output.append(',');
                output.append("{\"amount\":").append(amount).append(",\"treasure\":").append(award?-1:treasure).append(",\"table_before\":\"").append(hex(table)).append("\",\"table_after\":\"").append(hex(h.read(0x44331c,88))).append("\",\"before\":\"").append(hex(before)).append("\",\"after\":\"").append(hex(h.read(0x448aa0,party.length))).append("\",\"crate_before\":\"").append(hex(new byte[22000])).append("\",\"crate_after\":\"").append(hex(h.read(0x41fc74,22000))).append("\",\"share\":").append((int)h.pointer(0x6fff6a)).append(",\"currency\":").append(h.pointer(0x442788)).append(",\"seed_before\":").append(initial).append(",\"seed_after\":").append(result.seed).append(",\"rng_draws\":").append(result.draws).append(",\"draws\":[").append(result.drawRecords).append("],\"sounds\":[").append(result.sounds).append("]}");
            }finally{h.cpu.dispose();}
        }
        StringBuilder movements=new StringBuilder();for(int i=0;i<10;i++){if(i>0)movements.append(',');movements.append(races[i*408+197]);}
        Files.writeString(root.resolve("evidence/fixtures/story-inventory.json"),"{\"authority\":\"Original newland22 mode1 including complete unequipped dropitem/movecalc; original38 query and branchpredicate includingcheckforitem; originalbooty1058ec..1064c2 preparation and upgrade arithmetic to first UI allocation. Originalrandomitem executes for authoredtreasures5/6/85 sequentially. Loaditem serves exact originalID definitions; SANE arithmetic modeled; bodycount executes no-enemy input; profileload and presentation boundaries intercepted. No livepickup or fullrouteclaim.\",\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyStoryInventory.java"))+"\",\"harness_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/ActionOracleHarness.java"))+"\",\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\",\"items_sha256\":\""+ActionOracleHarness.sha256(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data ID"))+"\",\"scenario_items_sha256\":\""+ActionOracleHarness.sha256(root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data NI"))+"\",\"races_sha256\":\""+ActionOracleHarness.sha256(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data Race"))+"\",\"treasures_sha256\":\""+ActionOracleHarness.sha256(root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data TD"))+"\",\"ghidra\":\"12.1.2\",\"race_movements\":["+movements+"],\"consume_cases\":["+consume+"],\"query_cases\":["+query+"],\"award_cases\":["+awards+"],\"treasure_cases\":["+loot+"]}\n");println("Storyinventory cases exported");
    }
}
