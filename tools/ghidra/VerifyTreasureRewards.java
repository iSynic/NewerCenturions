import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;

public class VerifyTreasureRewards extends VerifyStoryInventory {
    public void run()throws Exception{
        Path root=Path.of(getScriptArgs()[0]);
        items=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data ID"));
        scenarioItems=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data NI"));
        races=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data Race"));
        treasures=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data TD"));
        party=new byte[3138*6];String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};
        for(int i=0;i<6;i++)System.arraycopy(Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Warriors/"+names[i])),0,party,i*3138,3138);
        StringBuilder cases=new StringBuilder();
        for(int variant=0;variant<6;variant++)for(int initial:new int[]{12,1}){
            byte[] before=party.clone(),table=Arrays.copyOfRange(treasures,8*88,9*88);
            for(int member=0;member<6;member++){
                before[member*3138+0x9df]=(byte)(variant==1?0:variant==2?member%2:1);
                if(variant==3&&member==0)ByteBuffer.wrap(before).putShort(member*3138+0x50,(short)0);
                if(variant==4&&member==0)ByteBuffer.wrap(before).putShort(member*3138+0x90,(short)-1);
                if(variant==5){ByteBuffer.wrap(before).putShort(member*3138+0x10,(short)17).putShort(member*3138+0x5e,(short)3).putShort(member*3138+0x60,(short)-1).putShort(member*3138+0x62,(short)4).putShort(member*3138+0x64,(short)-3).putShort(member*3138+0x90,(short)7);before[member*3138+0x9de]=9;}
            }
            ActionOracleHarness h=frame();RunResult result=new RunResult(initial);SaneBoundary sane=new SaneBoundary(h.cpu,currentProgram);
            try{
                common(h,before);h.value(0x6fffea,2,8);h.jump(0x143540);long seek=-1;int read=0;
                for(int step=0;step<1000;step++){
                    long pc=h.pc(),sp=h.sp();if(pc==0x105748)break;
                    if(pc==0x136cfa||pc==0x400068){h.ret();continue;}
                    if(pc==0x400070){h.cpu.writeRegister("A0",0x600000);h.ret();continue;}
                    if(pc==0x400078){seek=h.pointer(sp+8);h.ret();continue;}
                    if(pc==0x400048){if(h.pointer(sp+8)!=88||h.pointer(sp+12)!=1)throw new Exception("Treasure read size");h.write(h.pointer(sp+4),table);read++;h.cpu.writeRegister("D0",1);h.ret();continue;}
                    if(!h.cpu.step(monitor))throw new Exception("Treasure caller PC "+Long.toHexString(pc)+": "+h.cpu.getLastError());
                }
                if(h.pc()!=0x105748||seek!=704||read!=1||h.word(h.sp()+4)!=1||h.word(h.sp()+6)!=0)throw new Exception("Treasure caller boundary");
                h.cpu.writeRegister("A6",0x700000);h.cpu.writeRegister("SP",0x6ffe00);h.value(0x700008,2,1);h.value(0x70000a,2,0);h.jump(0x1058ec);
                for(int step=0;step<1000000;step++){
                    long pc=h.pc();if(pc==0x1064c2)break;
                    if(pc==0x130afe){loadItem(h);continue;}
                    if(pc==0x153ca4){result.rng(h);continue;}
                    if(pc==0x133a18){result.sound(h);continue;}
                    if(pc==0x13a7c4||pc==0x1046e0){h.ret();continue;}
                    if(pc==0x400038){h.cpu.writeRegister("D0",(short)Math.abs(h.word(h.sp()+4)));h.ret();continue;}
                    int op=h.word(pc)&65535;
                    if(op==0xa9eb){sane.execute();continue;}
                    if(op==0xa939){h.pop(6);h.jump(pc+2);continue;}
                    if(op==0xaa1c||op==0xa93c){h.pop(4);h.jump(pc+2);continue;}
                    if(op==0xa937){h.jump(pc+2);continue;}
                    if(!h.cpu.step(monitor))throw new Exception("Treasure preparation PC "+Long.toHexString(pc)+": "+h.cpu.getLastError());
                }
                if(h.pc()!=0x1064c2)throw new Exception("Treasure preparation budget");
                if(cases.length()>0)cases.append(',');cases.append("{\"variant\":").append(variant).append(",\"treasure\":8,\"seek\":").append(seek).append(",\"table_before\":\"").append(hex(table)).append("\",\"table_after\":\"").append(hex(h.read(0x44331c,88))).append("\",\"before\":\"").append(hex(before)).append("\",\"after\":\"").append(hex(h.read(0x448aa0,party.length))).append("\",\"crate_after\":\"").append(hex(h.read(0x41fc74,22000))).append("\",\"share\":").append((int)h.pointer(0x6fff6a)).append(",\"currency\":").append(h.pointer(0x442788)).append(",\"seed_before\":").append(initial).append(",\"seed_after\":").append(result.seed).append(",\"rng_draws\":").append(result.draws).append(",\"draws\":[").append(result.drawRecords).append("],\"sounds\":[").append(result.sounds).append("]}");
            }finally{h.cpu.dispose();}
        }
        Files.writeString(root.resolve("evidence/fixtures/treasure-rewards.json"),"{\"authority\":\"Original opcode10 caller143540..1435bc reads exact authoredTD8 at offset704, clearscombat/enemycount and callsbooty1,0; stdio source service intercepted. Originalbooty1058ec..1064c2 cleanup, item initialization in authoredorder, rewardeligibility and SANE experience arithmetic to firstlootUIallocation. Same bounded no-enemy setup as verifiedstoryinventory; actualrandomitem executes, exactloaditem and SANE boundary services modeled. Subsequentpickup, upgrade UI, warning dialogs, fullbootyreturn and livecampaign excluded. Field80 is experience signed32=4000;field84 currency signed32=0. Existing participation9df remainsunchanged forordinarytreasure, unlikeopcode11.\",\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyTreasureRewards.java"))+"\",\"harness_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/ActionOracleHarness.java"))+"\",\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\",\"cases\":["+cases+"]}\n");println("Treasure reward cases exported");
    }
}
