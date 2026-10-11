import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.ByteBuffer;
import java.util.Arrays;

public class VerifyOutdoorDisease extends VerifyStoryHealth {
    void executeReduction(ActionOracleHarness h)throws Exception{
        for(int step=0;step<200000;step++){
            long pc=h.pc(),sp=h.sp();if(pc==0x900000)return;
            if(pc==0x14de26){h.jump(0x14dede);continue;}
            if(pc==0x153ca4){int bound=h.word(sp+4);seed=seed*16807%2147483647;draws++;short raw=(short)seed;if(raw==Short.MIN_VALUE)raw=0;h.cpu.writeRegister("D0",Math.abs((int)raw)*bound/32768+1);h.ret();continue;}
            if(pc==0x1287c2){if(amounts.length()>0)amounts.append(',');amounts.append('[').append(h.word(sp+4)).append(',').append(h.word(sp+6)).append(']');}
            if(pc==0x400038){h.cpu.writeRegister("D0",(short)Math.abs(h.word(sp+4)));h.ret();continue;}
            if(pc==0x133a18){event("sound",h.word(sp+4),0);h.ret();continue;}
            if(pc==0x14913e){event("party_loss",0,0);h.ret();continue;}
            if(pc==0x12e4e4){int member=h.word(sp+4);event("defeat",member,h.word(0x448aa0+member*3138+0x50)<-9?1:0);}
            if(pc==0x123d1c||pc==0x15f66c||pc==0x1600fc||pc==0x13a498||pc==0x13154c||pc==0x1205f8||pc==0x16043e||pc==0x1500d8||pc==0x10f186||pc==0x158454){h.ret();continue;}
            int op=h.word(pc)&65535;
            if(op==0xa8a7){h.pop(12);h.jump(pc+2);continue;}
            if(op==0xa8a8){h.pop(8);h.jump(pc+2);continue;}
            if(op==0xa873){h.pop(4);h.jump(pc+2);continue;}
            if(op==0xa893||op==0xa884){h.pop(4);h.jump(pc+2);continue;}
            if(!h.cpu.step(monitor))throw new Exception("Outdoor reduce PC "+Long.toHexString(pc)+": "+h.cpu.getLastError());
        }throw new Exception("Outdoor reduce instruction budget");
    }
    public void run()throws Exception{
        root=Path.of(getScriptArgs()[0]);warriors=new byte[3138*6];String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};
        for(int i=0;i<6;i++)System.arraycopy(Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Warriors/"+names[i])),0,warriors,i*3138,3138);
        for(int member=0;member<6;member++)for(int hp:new int[]{-10,0,1,20,39,32767})for(int regeneration:new int[]{0,5})for(int disease:new int[]{-32768,-1,0,1,29,32767})for(int condition:new int[]{-1,0})for(int initial:new int[]{12,1}){
            ActionOracleHarness h=new ActionOracleHarness(currentProgram);
            try{
                byte[] before=warriors.clone(),affected=new byte[12];for(int i=0;i<6;i++){word(before,i*3138+0x50,40);word(before,i*3138+0x52,40);Arrays.fill(before,i*3138+0x5e,i*3138+0xae,(byte)0);before[i*3138+0x9df]=(byte)(i%2);word(affected,i*2,11+i);}
                word(before,member*3138+0x50,hp);word(before,member*3138+0x66,regeneration);word(before,member*3138+0x5e,1);word(before,member*3138+0x60,-1);word(before,member*3138+0x62,2);word(before,member*3138+0x74,-1);word(before,member*3138+0x96,disease);word(before,member*3138+0x90,condition);
                h.write(0x448aa0,before);h.write(0x442562,new byte[20]);h.write(0x440edc,new byte[6*288]);h.write(0x442798,affected);
                h.value(0x44273a,1,5);h.value(0x438776,2,0);h.value(0x44274a,1,0);h.value(0x442739,1,hp<=0?1:0);h.value(0x442750,1,0);h.value(0x44257c,2,0);h.value(0x44273f,1,0);h.value(0x443760,1,77);h.value(0x408574,2,-7);
                seed=initial;draws=0;events=new StringBuilder();amounts=new StringBuilder();h.enter(0x14de04,0x700000,0x900000);executeReduction(h);
                if(cases.length()>0)cases.append(',');cases.append("{\"member\":").append(member).append(",\"hp\":").append(hp).append(",\"disease\":").append(disease).append(",\"condition\":").append(condition).append(",\"regeneration\":").append(regeneration).append(",\"before_changes\":").append(changes(warriors,before)).append(",\"after_changes\":").append(changes(before,h.read(0x448aa0,before.length))).append(",\"before_sha256\":\"").append(digest(before)).append("\",\"after_sha256\":\"").append(digest(h.read(0x448aa0,before.length))).append("\",\"unavailable_before\":").append(hp<=0?1:0).append(",\"unavailable_after\":").append(h.read(0x442739,1)[0]).append(",\"effect_after\":").append(h.read(0x443760,1)[0]).append(",\"last_body_after\":").append(h.word(0x408574)).append(",\"affected_before\":\"").append(hex.formatHex(affected)).append("\",\"affected_after\":\"").append(hex.formatHex(h.read(0x442798,12))).append("\",\"enemy_unchanged\":").append(Arrays.equals(h.read(0x440edc,6*288),new byte[6*288])).append(",\"buffs_unchanged\":").append(Arrays.equals(h.read(0x442562,20),new byte[20])).append(",\"seed_before\":").append(initial).append(",\"seed_after\":").append(seed).append(",\"draws\":").append(draws).append(",\"amounts\":[").append(amounts).append("],\"events\":[").append(events).append("]}");
            }finally{h.cpu.dispose();}
        }
        Files.writeString(root.resolve("evidence/fixtures/outdoor-disease.json"),"{\"authority\":\"Original outdoor reduce14de04..14e5d8, full six warrior records, zero partybuffs/enemyconditions and controlled signed disease duration and condition90, no other active damage/stun/energy channels. Original heal and disease direct HP subtraction followed by outdoorkillbody execute, including regeneration before disease, signedword overflow, availability counters and exact ordered defeat sounds/events. Intro buffpaint14de26..14dede skipped; UI short/fullupdates, controls, repaint, soundoutput and party-loss modal intercepted. No combatgetup bleeding decrement, other active damagechannels, noon/day foodrecovery, hour/fatigue/energy caller or live UI claim.\",\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyOutdoorDisease.java"))+"\",\"harness_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/ActionOracleHarness.java"))+"\",\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\",\"warriors\":\""+hex.formatHex(warriors)+"\",\"cases\":["+cases+"]}\n");println("Outdoor disease reduction cases exported");
    }
}
