import ghidra.app.script.GhidraScript;
import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.HexFormat;
import java.security.MessageDigest;

public class VerifyStoryHealth extends GhidraScript {
    Path root;byte[] warriors;long seed,draws;StringBuilder cases=new StringBuilder(),events,amounts;
    HexFormat hex=HexFormat.of();
    void word(byte[] bytes,int offset,int value){ByteBuffer.wrap(bytes).putShort(offset,(short)value);}
    byte[] extra(int... values){ByteBuffer b=ByteBuffer.allocate(10);for(int value:values)b.putShort((short)value);return b.array();}
    String changes(byte[] before,byte[] after){StringBuilder s=new StringBuilder("[");for(int i=0;i<before.length;i++)if(before[i]!=after[i]){if(s.length()>1)s.append(',');s.append('[').append(i).append(',').append(after[i]&255).append(']');}return s.append(']').toString();}
    String digest(byte[] bytes) throws Exception{return hex.formatHex(MessageDigest.getInstance("SHA-256").digest(bytes));}
    void event(String name,int first,int second){if(events.length()>0)events.append(',');events.append("[\"").append(name).append("\",").append(first).append(',').append(second).append(']');}
    void execute(ActionOracleHarness h,long stop) throws Exception {
        for(int step=0;step<150000;step++) {
            long pc=h.pc(),sp=h.sp();if(pc==stop)return;
            if(pc==0x153ca4){int bound=h.word(sp+4);seed=seed*16807%2147483647;draws++;short raw=(short)seed;if(raw==Short.MIN_VALUE)raw=0;h.cpu.writeRegister("D0",Math.abs((int)raw)*bound/32768+1);h.ret();continue;}
            if(pc==0x1287c2){if(amounts.length()>0)amounts.append(',');amounts.append('[').append(h.word(sp+4)).append(',').append(h.word(sp+6)).append(']');}
            if(pc==0x1383e6){h.ret();continue;}
            if(pc==0x400038){h.cpu.writeRegister("D0",(short)Math.abs(h.word(sp+4)));h.ret();continue;}
            if(pc==0x15b2f6){event("animation",h.word(sp+4),h.read(0x443760,1)[0]);h.ret();continue;}
            if(pc==0x133a18){event("sound",h.word(sp+4),0);h.ret();continue;}
            if(pc==0x15dc34){event("dialogue",h.word(sp+6),0);h.ret();continue;}
            if(pc==0x14913e){event("party_loss",0,0);h.ret();continue;}
            if(pc==0x12e4e4){int member=h.word(sp+4);event("defeat",member,h.word(0x448aa0+member*3138+0x50)<-9?1:0);}
            if(pc==0x1600fc||pc==0x13a498||pc==0x13154c||pc==0x1205f8||pc==0x16043e||pc==0x1500d8||pc==0x10f186){h.ret();continue;}
            int op=h.word(pc)&65535;
            if(op==0xa893||op==0xa884){h.pop(4);h.jump(pc+2);continue;}
            if(!h.cpu.step(monitor))throw new Exception("Story health PC "+Long.toHexString(pc)+": "+h.cpu.getLastError());
        }throw new Exception("Story health instruction budget");
    }
    void runCase(String kind,int member,int current,int maximum,int amount,int animate,int flags,int variant,int unavailable,long initial,int[] input,int[] selection) throws Exception {
        ActionOracleHarness h=new ActionOracleHarness(currentProgram);
        try {
            byte[] before=warriors.clone(),record=extra(input),mask=new byte[12],affected=extra(11,12,13,14,15),affectedSix=new byte[12];System.arraycopy(affected,0,affectedSix,0,10);word(affectedSix,10,16);
            for(int index=0;index<6;index++) {
                word(before,index*3138+0x50,kind.equals("heal")?(index==member?current:100):(variant==0?500:variant==1?new int[]{1,20,0,-9,-10,-11}[index]:32760));
                word(before,index*3138+0x52,kind.equals("heal")?(index==member?maximum:100):(variant==2?32767:600));
                before[index*3138+0x9df]=(byte)(index%2);word(mask,index*2,selection[index]);
            }
            h.write(0x448aa0,before);h.write(0x44330a,record);h.write(0x4434e0,mask);h.write(0x442798,affectedSix);
            h.value(0x44273a,1,5);h.value(0x442739,1,unavailable);h.value(0x44273f,1,member);h.value(0x438776,2,0);h.value(0x447198,2,1);h.value(0x442750,1,flags);h.value(0x443760,1,77);h.value(0x408574,2,-7);
            h.write(0x6fc000,new byte[24576]);h.cpu.writeRegister("A6",0x700000);h.cpu.writeRegister("SP",0x6ffe00);seed=initial;draws=0;events=new StringBuilder();amounts=new StringBuilder();
            if(kind.equals("heal")){h.enter(0x1287c2,0x700000,0x900000);h.value(0x700004,2,member);h.value(0x700006,2,amount);h.value(0x700008,2,animate);execute(h,0x900000);}
            else {h.value(0x6fffea,2,400);h.jump(kind.equals("all")?0x143984:0x1438e2);execute(h,0x14673c);}
            if(cases.length()>0)cases.append(',');cases.append("{\"kind\":\"").append(kind).append("\",\"member\":").append(member).append(",\"amount\":").append(amount).append(",\"animate\":").append(animate!=0).append(",\"marks_affected\":").append(flags!=0).append(",\"variant\":").append(variant).append(",\"extra\":\"").append(hex.formatHex(record)).append("\",\"mask\":\"").append(hex.formatHex(mask)).append("\",\"mask_after\":\"").append(hex.formatHex(h.read(0x4434e0,12))).append("\",\"before_changes\":").append(changes(warriors,before)).append(",\"after_changes\":").append(changes(before,h.read(0x448aa0,before.length))).append(",\"before_sha256\":\"").append(digest(before)).append("\",\"after_sha256\":\"").append(digest(h.read(0x448aa0,before.length))).append("\",\"unavailable_before\":").append((byte)unavailable).append(",\"unavailable_after\":").append(h.read(0x442739,1)[0]).append(",\"effect_after\":").append(h.read(0x443760,1)[0]).append(",\"last_body_after\":").append(h.word(0x408574)).append(",\"affected_before\":\"").append(hex.formatHex(affectedSix)).append("\",\"affected_after\":\"").append(hex.formatHex(h.read(0x442798,12))).append("\",\"seed_before\":").append(initial).append(",\"seed_after\":").append(seed).append(",\"draws\":").append(draws).append(",\"amounts\":[").append(amounts).append("],\"events\":[").append(events).append("]}");
        }finally{h.cpu.dispose();}
    }
    public void run() throws Exception {
        root=Path.of(getScriptArgs()[0]);warriors=new byte[3138*6];String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};for(int i=0;i<6;i++)System.arraycopy(Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Warriors/"+names[i])),0,warriors,i*3138,3138);
        int[] all={1,1,1,1,1,1},zero={0,0,0,0,0,0},mixed={0,-1,0,1,0,32767};
        for(long initial:new long[]{12,1}) {
            for(int current:new int[]{100,1,0,-1,-9,-10,-11,32760,-32768})for(int amount:new int[]{-20,0,15,32767,-32768})runCase("heal",2,current,120,amount,1,1,0,current<1?1:0,initial,new int[]{0,0,0,0,0},all);
            for(int member=0;member<6;member++)runCase("heal",member,1,120,-1,0,0,0,5,initial,new int[]{0,0,0,0,0},all);
            runCase("heal",1,-1,-3,2,0,0,0,1,initial,new int[]{0,0,0,0,0},all);
            runCase("heal",1,1,120,-1,1,0,0,255,initial,new int[]{0,0,0,0,0},all);
            for(String kind:new String[]{"all","selected"})for(int variant=0;variant<3;variant++)for(int[] selection:new int[][]{all,zero,mixed})for(int[] input:new int[][]{{-3,5,10,624,0},{0,5,10,0,-851},{1,-3,5,-624,851},{32767,2,2,0,0}})runCase(kind,0,0,0,0,1,1,variant,variant==1?4:0,initial,input,selection);
        }
        Files.writeString(root.resolve("evidence/fixtures/story-health.json"),"{\"authority\":\"Original outdoor heal1287c2..1289f0 and killbody outdoor branch; original opcode15 selected-mask and opcode16 all-member callers, original randrange arithmetic. Full six original warrior records copied and compared. ParkMiller/Rand service modeled using previously verified boundary; drawing, sound output, short updates and modal party-loss service intercepted and recorded. Party-loss service itself, combat body removal, live pointer UI and campaign survival excluded. Permanent-dead members still consume caller range RNG and animation; direct heal uses signed-word wrap/clamp and signed-byte unavailable-counter.\",\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyStoryHealth.java"))+"\",\"harness_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/ActionOracleHarness.java"))+"\",\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\",\"warriors\":\""+hex.formatHex(warriors)+"\",\"cases\":["+cases+"]}\n");println("Story health original cases exported");
    }
}
