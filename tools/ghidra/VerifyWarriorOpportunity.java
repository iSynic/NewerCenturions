import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;

public class VerifyWarriorOpportunity extends VerifyGeneticMelee {
    String runCase(int layout,int seed,boolean forcedKill)throws Exception{
        ActionOracleHarness h=frame();RunResult result=new RunResult(seed);
        try{
            byte[] warriors=party.clone(),records=new byte[3*288],map=terrain(),order=new byte[220];Arrays.fill(order,(byte)255);ByteBuffer.wrap(order).putShort(2,(short)0);
            ByteBuffer b=ByteBuffer.wrap(warriors);for(int member=0;member<6;member++)b.putShort(member*3138+0x16,(short)0).putShort(member*3138+0x50,(short)10000).putShort(member*3138+0x52,(short)10000);
            b.putShort(0x3c,(short)(layout==12?0:30)).putShort(0x34,(short)(layout==7?4:layout==16?5:1)).putShort(0x10,(short)(layout==14?2:0)).putShort(0x8e,(short)(layout==10||layout==17||layout==18?1:0));
            int[][] positions={{19,20},{23,20},{24,20}};int dx=1,dy=0;
            if(layout==2||layout==17){positions[0]=new int[]{21,20};dx=0;dy=1;}
            if(layout==3){dx=-1;dy=1;}
            if(layout==4||layout==18)positions[0]=new int[]{22,20};
            if(layout==6){positions=new int[][]{{19,20},{19,21},{19,19}};}
            for(int enemy=0;enemy<3;enemy++){
                byte[] record=super.monster(layout==0||layout==13?65:13,100);ByteBuffer rb=ByteBuffer.wrap(record);
                rb.putShort(0xe,(short)(layout==0||layout==13?0:layout==14?2:1)).putShort(0x4e,(short)178).putShort(0x28,(short)178);
                record[0x104]=(byte)(layout==4||layout==5||layout==13?1:0);if(layout==8)record[0xb3]=1;if(layout==9)record[0xb0]=-1;if(layout==15)rb.putShort(0x110,(short)0);
                System.arraycopy(record,0,records,enemy*288,288);ByteBuffer.wrap(map).putShort(((positions[enemy][0]+5)*90+positions[enemy][1]+5)*2,(short)(10+enemy));
            }
            ByteBuffer.wrap(map).putShort((25*90+25)*2,(short)0);if(layout==11)ByteBuffer.wrap(map).putShort(((25+dx)*90+25+dy)*2,(short)1061);
            for(int member=1;member<6;member++)ByteBuffer.wrap(map).putShort(((45+member)*90+45)*2,(short)member);
            state(h,records,map);h.write(0x448aa0,warriors);h.write(0x4470b8,order);h.write(0x442934,new byte[660]);h.value(0x447194,2,1);h.value(0x4435d2,2,0);h.value(0x4435c4,2,3);h.value(0x44374c,2,0);h.value(0x43874a,2,3);h.value(0x447196,2,0);
            h.value(0x44709a,2,20);h.value(0x44709c,2,20);h.value(0x437a6e,2,1155);h.value(0x442736,1,dx);h.value(0x442737,1,dy);h.value(0x438764,2,0);h.value(0x43565f,1,0);h.value(0x443318,2,0);h.value(0x43959c,1,0);
            h.value(0x439450,4,0x680000);h.value(0x680000,4,0x680100);h.write(0x680100,new byte[32]);
            for(int enemy=0;enemy<3;enemy++){h.value(0x446f0a+enemy*4,2,positions[enemy][0]);h.value(0x446f0c+enemy*4,2,positions[enemy][1]);}
            h.cpu.writeRegister("A6",0x701000);h.cpu.writeRegister("SP",0x6ffe00);h.write(0x700c00,new byte[2048]);h.jump(0x11bfc0);
            StringBuilder calls=new StringBuilder(),phases=new StringBuilder();int advances=0;boolean done=false;
            for(int budget=0;budget<300000;budget++){
                long pc=h.pc(),sp=h.sp();if(pc==0x11ce76){done=true;break;}
                if(pc==0x110e96){if(phases.length()>0)phases.append(',');phases.append(h.word(sp+4));}
                if(pc==0x101eb6){if(calls.length()>0)calls.append(',');calls.append('[').append(h.word(sp+4)).append(',').append(h.word(sp+6)).append(',').append(h.word(sp+8)).append(',').append((short)h.cpu.readRegister("D6").intValue()).append(']');if(forcedKill){h.cpu.writeRegister("D0",1);h.ret();}else execute(h,result,h.pointer(sp));continue;}
                if(pc==0x125a3a){advances++;h.ret();continue;}
                if(pc==0x153ca4){result.rng(h);continue;}
                if(pc==0x400038){h.cpu.writeRegister("D0",(short)Math.abs(h.word(sp+4)));h.ret();continue;}
                if(pc==0x133a18){result.sound(h);continue;}
                if(pc==0x16043e||pc==0x10e6b6||pc==0x13154c||pc==0x11f1a8||pc==0x11f136||pc==0x11edc4||pc==0x1382a8){h.ret();continue;}
                int op=h.word(pc)&65535;if(op==0xa873||op==0xa958){h.pop(4);h.jump(pc+2);continue;}
                if(!h.cpu.step(monitor))throw new Exception("Warrior walk layout"+layout+" PC "+Long.toHexString(pc)+": "+h.cpu.getLastError());
            }
            if(!done)throw new Exception("Warrior opportunity instruction budget");
            return "{\"layout\":"+layout+",\"forced_kill_return\":"+forcedKill+",\"dx\":"+dx+",\"dy\":"+dy+",\"seed_before\":"+seed+",\"seed_after\":"+result.seed+",\"rng_draws\":"+result.draws+",\"warriors_before\":\""+hex(warriors)+"\",\"warriors_after\":\""+hex(h.read(0x448aa0,warriors.length))+"\",\"records_before\":\""+hex(records)+"\",\"records_after\":\""+hex(h.read(0x439d3c,records.length))+"\",\"map_before\":\""+hex(map)+"\",\"map_after\":\""+hex(h.read(0x4085b8,16200))+"\",\"enemy_positions\":[["+positions[0][0]+","+positions[0][1]+"],["+positions[1][0]+","+positions[1][1]+"],["+positions[2][0]+","+positions[2][1]+"]],\"x\":"+h.word(0x44709a)+",\"y\":"+h.word(0x44709c)+",\"under\":"+h.word(0x437a6e)+",\"calls\":["+calls+"],\"phases\":["+phases+"],\"advance_calls\":"+advances+",\"packet\":\""+hex(h.read(0x44374e,58))+"\",\"sounds\":["+result.sounds+"]}";
        }finally{h.cpu.dispose();}
    }
    public void run()throws Exception{
        Path root=Path.of(getScriptArgs()[0]),original=root.resolve("NewCenturions/TheNewCenturions"),scenario=original.resolve("Scenarios/The Thunder of Guns/0");
        items=Files.readAllBytes(original.resolve("Data Files/Data ID"));scenarioItems=Files.readAllBytes(scenario.resolve("Data NI"));races=Files.readAllBytes(original.resolve("Data Files/Data Race"));monsters=Files.readAllBytes(scenario.resolve("Data MD"));properties=Files.readAllBytes(original.resolve("Data Files/Data Swamp BD"));attacks=Files.readAllBytes(original.resolve("Data Files/Data S"));areas=Files.readAllBytes(original.resolve("Data Files/Data AD"));party=new byte[3138*6];
        String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};for(int i=0;i<6;i++){System.arraycopy(Files.readAllBytes(original.resolve("Warriors/"+names[i])),0,party,i*3138,3138);party[i*3138+0x9df]=1;}
        StringBuilder cases=new StringBuilder();int count=0;for(int layout=0;layout<=18;layout++)for(int seed:new int[]{12,1}){if(cases.length()>0)cases.append(',');cases.append(runCase(layout,seed,false));count++;}
        for(int layout:new int[]{1,5,6}){if(cases.length()>0)cases.append(',');cases.append(runCase(layout,12,true));count++;}
        StringBuilder identity=new StringBuilder();String[][] inputs={{"probe","tools/ghidra/VerifyWarriorOpportunity.java"},{"parent_probe","tools/ghidra/VerifyGeneticMelee.java"},{"harness","tools/ghidra/ActionOracleHarness.java"},{"mapped_binary","analysis/ghidra/startup/code1-mapped.bin"},{"monsters","NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data MD"},{"items","NewCenturions/TheNewCenturions/Data Files/Data ID"},{"properties","NewCenturions/TheNewCenturions/Data Files/Data Swamp BD"},{"attacks","NewCenturions/TheNewCenturions/Data Files/Data S"}};for(String[] input:inputs){if(identity.length()>0)identity.append(',');identity.append('"').append(input[0]).append("_sha256\":\"").append(ActionOracleHarness.sha256(root.resolve(input[1]))).append('"');}
        Files.writeString(root.resolve("evidence/fixtures/warrior-opportunity.json"),"{\"authority\":\"Original normal warrior walk combatchoice11bfc0..11ce76 executes originalcheckforenemy modes0/1/2, full native attack2 for controlled MD13 loaded melee178 and friendly MD65, movement cost/map/position/under-tile writes, empty collide and finalmode0. Full attack services inherit independently verified GeneticMelee preparation, SANE and native hit/miss audio boundaries. Controlled targetHP10000 avoids actual death. Three forcedkillreturn cases model ONLY attack2return1 to prove caller abort/getup ordering, and cannot establish death behavior. Class4/5, activeword8e, b3, b0 and HP eligibility, old/new adjacency, guard consumption and ascending opportunity order are native branches. Presentation boundaries intercepted; no live battle claim.\",\"identity\":{"+identity+"},\"properties\":\""+hex(properties)+"\",\"cases\":["+cases+"]}\n");println("Warrior opportunity "+count+" native caller cases exported");
    }
}
