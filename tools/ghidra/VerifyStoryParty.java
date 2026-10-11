import ghidra.app.script.GhidraScript;
import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.HexFormat;
import java.util.regex.Pattern;

public class VerifyStoryParty extends GhidraScript {
    Path root;byte[] warriors;long seed,draws;
    StringBuilder selection=new StringBuilder(),groups=new StringBuilder(),returns=new StringBuilder();
    HexFormat hex=HexFormat.of();
    byte[] extra(int... values){ByteBuffer b=ByteBuffer.allocate(10);for(int value:values)b.putShort((short)value);return b.array();}
    void word(byte[] bytes,int offset,int value){ByteBuffer.wrap(bytes).putShort(offset,(short)value);}
    String field(String json,String name){var m=Pattern.compile("\\\""+name+"\\\"\\s*:\\s*\\\"([^\\\"]*)\\\"").matcher(json);if(!m.find())throw new IllegalArgumentException(name);return m.group(1);}
    void execute(ActionOracleHarness h,long stop,int budget) throws Exception {
        for(int step=0;step<budget;step++) {
            long pc=h.pc();if(pc==stop)return;
            if(pc==0x153ca4){int bound=h.word(h.sp()+4);seed=seed*16807%2147483647;draws++;short raw=(short)seed;if(raw==Short.MIN_VALUE)raw=0;h.cpu.writeRegister("D0",Math.abs((int)raw)*bound/32768+1);h.ret();continue;}
            if(pc==0x160fb2||pc==0x13154c||pc==0x105572){h.ret();continue;}
            if(!h.cpu.step(monitor))throw new Exception("Story party PC "+Long.toHexString(pc)+": "+h.cpu.getLastError());
        }throw new Exception("Story party instruction budget");
    }
    void select(int kind,int value,int eligible,int selected,int variant,long initial,int casting,int packetType,int bonus,int perAttack,int attackCount,int shield) throws Exception {
        ActionOracleHarness h=new ActionOracleHarness(currentProgram);
        try {
            byte[] before=warriors.clone();
            for(int member=0;member<6;member++) {
                int base=member*3138;
                word(before,base+0x50,member==1?0:member==2?-1:100);
                word(before,base+0x3e,member==0?-1:member*3);
                for(int index=0;index<6;index++)word(before,base+new int[]{0x9e2,0x9e4,0x9e6,0x9e8,0x9ec,0x9ee}[index],member==5?8192:member*8+index);
                for(int index=0;index<9;index++)word(before,base+0x112+index*2,member*19+index);
                word(before,base+0x1a,3);for(int slot=0;slot<=3;slot++){word(before,base+0x132+slot*22,slot==0?828:slot==1?809:0);before[base+0x13b+slot*22]=0;}
                if(member==2)before[base+0x13b]=1;
                if(member==3){word(before,base+0x132+3*22,828);before[base+0x13b+3*22]=1;}
                if(variant==1&&member==4)word(before,base+0x1a,-1);
                if(variant==2)word(before,base+0x112+value*2,32767);
            }
            byte[] input=extra(kind,value,eligible,0,0);
            h.write(0x448aa0,before);h.write(0x44330a,input);h.write(0x4434e0,extra(1,1,1,1,1));h.value(0x4434ea,2,1);
            h.value(0x44273a,1,5);h.value(0x44273f,1,selected);h.value(0x438760,2,casting);h.value(0x443757,1,packetType);h.value(0x443754,1,bonus);h.value(0x443756,1,perAttack);h.value(0x4435cc,2,attackCount);h.value(0x442570,2,shield);
            h.write(0x6fc000,new byte[24576]);h.cpu.writeRegister("A6",0x700000);h.cpu.writeRegister("SP",0x6ffe00);h.value(h.sp(),2,0);h.jump(0x1458e8);seed=initial;draws=0;execute(h,0x14673c,100000);
            if(selection.length()>0)selection.append(',');selection.append("{\"extra\":\"").append(hex.formatHex(input)).append("\",\"selected\":").append(selected).append(",\"variant\":").append(variant).append(",\"context\":{").append("\"casting\":").append(casting!=0).append(",\"packet_type\":").append(packetType).append(",\"bonus\":").append(bonus).append(",\"bonus_per_attack\":").append(perAttack).append(",\"attack_count\":").append(attackCount).append(",\"mind_shield\":").append(shield!=0).append("},\"before\":\"").append(hex.formatHex(before)).append("\",\"after\":\"").append(hex.formatHex(h.read(0x448aa0,before.length))).append("\",\"mask\":\"").append(hex.formatHex(h.read(0x4434e0,12))).append("\",\"seed_before\":").append(initial).append(",\"seed_after\":").append(seed).append(",\"draws\":").append(draws).append('}');
        }finally{h.cpu.dispose();}
    }
    void group(String source,int group,int variant,int[] mask) throws Exception {
        ActionOracleHarness h=new ActionOracleHarness(currentProgram);
        try {
            byte[] before=hex.parseHex(field(source,"warriors")),tiles=hex.parseHex(field(source,"tiles")),squad=hex.parseHex(field(source,"squad")),under=hex.parseHex(field(source,"under")),order=hex.parseHex(field(source,"order"));
            if(variant==1){word(before,3138+0x50,0);word(before,2*3138+0x50,-1);}
            h.write(0x448aa0,before);h.write(0x4085b8,tiles);h.write(0x44709a,squad);h.write(0x437a6e,under);h.write(0x4470b8,order);h.value(0x442780,4,-10);h.value(0x442784,4,-16);h.value(0x44273a,1,5);h.value(0x4434aa,2,0);for(int i=0;i<6;i++)h.value(0x4434e0+i*2,2,mask[i]);
            long frame=0x701000;h.write(frame-4096,new byte[8192]);h.cpu.writeRegister("A6",frame);h.cpu.writeRegister("SP",0x700000);h.value(frame+12,2,group);h.jump(0x11ec2a);execute(h,0x11ed1c,100000);
            if(groups.length()>0)groups.append(',');groups.append("{\"group\":").append(group!=0).append(",\"variant\":").append(variant).append(",\"shift\":[-15,-21],\"mask\":[");for(int i=0;i<6;i++){if(i>0)groups.append(',');groups.append(mask[i]);}groups.append(']');
            String[] fields={"warriors","tiles","squad","under","order"};byte[][] inputs={before,tiles,squad,under,order};long[] addresses={0x448aa0,0x4085b8,0x44709a,0x437a6e,0x4470b8};
            for(int i=0;i<fields.length;i++)groups.append(",\"").append(fields[i]).append("_before\":\"").append(hex.formatHex(inputs[i])).append("\",\"").append(fields[i]).append("_after\":\"").append(hex.formatHex(h.read(addresses[i],inputs[i].length))).append('"');groups.append(",\"draws\":0}");
        }finally{h.cpu.dispose();}
    }
    void groupReturn(int[] input,int retreat,int abort,long initial) throws Exception {
        ActionOracleHarness h=new ActionOracleHarness(currentProgram);
        try {
            byte[] record=extra(input),script=new byte[33],before=warriors.clone();for(int i=0;i<script.length;i++)script[i]=(byte)(i*7+2);for(int i=0;i<6;i++)before[i*3138+0x9df]=(byte)(i%2);
            h.write(0x448aa0,before);h.write(0x479eb2,script);h.value(0x44273a,1,5);h.value(0x442774,4,8);h.value(0x4386b2,1,0);h.value(0x4386fc,1,0);h.write(0x6fc000,new byte[24576]);h.cpu.writeRegister("A6",0x700000);h.cpu.writeRegister("SP",0x6ffe00);h.value(0x6fffee,2,3);h.value(0x6fffea,2,405);h.jump(0x141178);
            seed=initial;draws=0;StringBuilder events=new StringBuilder();boolean done=false;int target=0;
            for(int step=0;step<100000;step++) {
                long pc=h.pc(),sp=h.sp();if(pc==0x14673c||pc==0x14032e||pc==0x14694c){done=true;break;}
                String event=null;int argument=0;
                if(pc==0x1383e6){h.write(0x44330a,record);h.ret();continue;}
                if(pc==0x133a18){event="sound";argument=h.word(sp+4);}
                if(pc==0x15dc34){event="message";argument=h.word(sp+6);}
                if(pc==0x114378){event="combat";argument=(int)h.pointer(sp+4);h.value(0x4386fc,1,retreat);h.value(0x4386b2,1,abort);h.write(0x479eb2,new byte[33]);h.value(0x6fffee,2,87);}
                if(pc==0x13107a){event="land";argument=(int)h.pointer(sp+4);}
                if(pc==0x105748){event="booty";argument=(int)h.pointer(sp+4);}
                if(pc==0x123c60){event="script";argument=h.word(sp+4);target=argument;}
                if(pc==0x13bb20){event="update";argument=(int)h.pointer(sp+4);}
                if(event!=null){if(events.length()>0)events.append(',');events.append("[\"").append(event).append("\",").append(argument).append(']');h.ret();continue;}
                if(pc==0x153ca4){int bound=h.word(sp+4);seed=seed*16807%2147483647;draws++;short raw=(short)seed;if(raw==Short.MIN_VALUE)raw=0;h.cpu.writeRegister("D0",Math.abs((int)raw)*bound/32768+1);h.ret();continue;}
                if(!h.cpu.step(monitor))throw new Exception("Group return PC "+Long.toHexString(pc)+": "+h.cpu.getLastError());
            }
            if(!done)throw new Exception("Group return budget");if(returns.length()>0)returns.append(',');returns.append("{\"extra\":\"").append(hex.formatHex(record)).append("\",\"retreated\":").append(retreat!=0).append(",\"aborted\":").append(abort!=0).append(",\"seed_before\":").append(initial).append(",\"seed_after\":").append(seed).append(",\"draws\":").append(draws).append(",\"battle\":").append(h.word(0x43874e)).append(",\"target\":").append(target).append(",\"cursor\":").append(h.word(0x6fffee)).append(",\"continuation\":\"").append(h.pc()==0x14032e?"jump":h.pc()==0x14694c?"abort":"continue").append("\",\"events\":[").append(events).append("],\"before\":\"").append(hex.formatHex(before)).append("\",\"after\":\"").append(hex.formatHex(h.read(0x448aa0,before.length))).append("\",\"script_before\":\"").append(hex.formatHex(script)).append("\",\"script_after\":\"").append(hex.formatHex(h.read(0x479eb2,33))).append("\"}");
        }finally{h.cpu.dispose();}
    }
    public void run() throws Exception {
        root=Path.of(getScriptArgs()[0]);warriors=new byte[6*3138];String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};for(int i=0;i<6;i++)System.arraycopy(Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Warriors/"+names[i])),0,warriors,i*3138,3138);
        for(long initial:new long[]{12,1}) {
            for(int[] input:new int[][]{{0,10,0},{1,12,1},{1,3,0},{2,828,0},{2,809,1},{2,777,0},{3,0,0},{3,50,0},{3,100,1},{3,50,2},{3,50,3},{6,0,0},{7,828,0},{7,809,0},{8,0,0}})select(input[0],input[1],input[2],4,0,initial,0,0,0,0,0,0);
            for(int selector=0;selector<6;selector++)select(4,selector,0,4,0,initial,0,0,0,0,0,0);
            for(int selector=0;selector<9;selector++)select(5,selector,0,4,0,initial,0,0,0,0,0,0);
            select(5,0,0,4,0,initial,0,0,-7,3,12,1);select(5,3,1,4,0,initial,1,2,0,0,0,0);select(5,2,0,4,2,initial,0,0,3,0,0,0);select(7,828,0,4,1,initial,0,0,0,0,0,0);
            for(int retreat=0;retreat<2;retreat++)for(int[] input:new int[][]{{118,0,20150,0,217},{8,11,663,138,53},{11,8,0,0,-1}})groupReturn(input,retreat,0,initial);
            groupReturn(new int[]{118,0,20150,0,217},0,1,initial);
        }
        String battle=Files.readString(root.resolve("evidence/fixtures/battle-entry.json"));battle=battle.substring(battle.indexOf("\"cases\":[{")+10);battle=battle.substring(0,battle.indexOf('}'));
        for(int group:new int[]{0,1})for(int variant:new int[]{0,1})for(int[] mask:new int[][]{{1,1,1,1,1,1},{0,1,0,1,0,1},{0,0,0,0,0,0}})group(battle,group,variant,mask);
        Files.writeString(root.resolve("evidence/fixtures/story-party.json"),"{\"authority\":\"Original newland52 block1458e8..14673c including native checkforitem/savevsattr/savevs; synthetic six-warrior fields exercise eligibility, signed comparisons, chance, saves and inclusive equipped-item scan. Group filter11ec2a..11ed1c executes native bodyfieldwarrior on original battle-entry fixture state, before-state deliberately reused after getup; this isolates filtering, not full group entry. Opcode107 caller141178..1412fa executes with combat/booty/loadland/script/presentation boundaries recorded, including mocked retreat and abort outcomes. No live gameplay claim.\",\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyStoryParty.java"))+"\",\"harness_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/ActionOracleHarness.java"))+"\",\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\",\"battle_entry_sha256\":\""+ActionOracleHarness.sha256(root.resolve("evidence/fixtures/battle-entry.json"))+"\",\"selection_cases\":["+selection+"],\"group_cases\":["+groups+"],\"return_cases\":["+returns+"]}\n");
        println("Story party original cases exported");
    }
}
