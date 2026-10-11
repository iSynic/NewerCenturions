import ghidra.app.script.GhidraScript;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.HexFormat;

public class VerifyWarriorChoice extends GhidraScript {
    static String hex(byte[] raw){return HexFormat.of().formatHex(raw);}
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]);StringBuilder cases=new StringBuilder();
        for(int selected=0;selected<6;selected++) {
            ActionOracleHarness h=new ActionOracleHarness(currentProgram);
            try {
                h.write(0x6ff000,new byte[8192]);h.cpu.writeRegister("A6",0x700000);h.cpu.writeRegister("SP",0x6ffe00);h.value(0x6fffea,2,1);h.value(0x6fffee,2,3);h.value(0x6fffe9,1,99);
                byte[] before=new byte[16];Arrays.fill(before,(byte)0x73);h.write(0x4434e0,before);h.jump(0x14383c);int count=-1,presentation=-1,dead=-1,calls=0;byte[] mask=new byte[16];mask[selected*2+1]=1;
                for(int s=0;s<200;s++) {
                    long pc=h.pc();if(pc==0x14673c)break;
                    if(pc==0x400038){h.cpu.writeRegister("D0",Math.abs(h.word(h.sp()+4)));h.ret();continue;}
                    if(pc==0x12512a){calls++;count=h.word(h.sp()+4);presentation=h.word(h.sp()+6);dead=h.word(h.sp()+8);h.write(0x4434e0,mask);h.cpu.writeRegister("D0",selected+1);h.ret();continue;}
                    if(!h.cpu.step(monitor))throw new Exception("Choice caller PC "+Long.toHexString(pc)+": "+h.cpu.getLastError());
                }
                if(h.pc()!=0x14673c)throw new Exception("Choice caller budget");
                if(cases.length()>0)cases.append(',');cases.append("{\"parameter\":1,\"selected\":").append(selected).append(",\"count_minus_one\":").append(count).append(",\"presentation_mode\":").append(presentation).append(",\"include_dead\":").append(dead).append(",\"local_mode\":").append(h.read(0x6fffe9,1)[0]&255).append(",\"calls\":").append(calls).append(",\"cursor\":").append(h.word(0x6fffee)).append(",\"mask\":\"").append(hex(h.read(0x4434e0,16))).append("\"}");
            }finally{h.cpu.dispose();}
        }
        StringBuilder asm=new StringBuilder();var iterator=currentProgram.getListing().getInstructions(toAddr(0x12512a),true);while(iterator.hasNext()){var ins=iterator.next();if(ins.getAddress().getOffset()>0x125500)break;asm.append(ins.getAddress()).append('\t').append(ins).append('\n');}Files.writeString(root.resolve("analysis/ghidra/exports/getchoice.asm.tsv"),asm.toString());
        StringBuilder selections=new StringBuilder();
        for(int selected=0;selected<6;selected++)for(int stamina:new int[]{-32768,-1,0,1,32767}) {
            ActionOracleHarness h=new ActionOracleHarness(currentProgram);
            try {
                h.write(0x6ff000,new byte[8192]);h.cpu.writeRegister("A6",0x700000);h.cpu.writeRegister("SP",0x700500);h.value(0x70000c,2,0);h.value(0x4434b2,2,selected);h.value(0x448aa0+selected*3138+0x50,2,stamina);h.write(0x4434e0,new byte[16]);h.jump(0x125300);
                for(int s=0;s<200;s++){if(h.pc()==0x12532e||h.pc()==0x1251dc)break;if(h.pc()==0x133a18){h.ret();continue;}if(!h.cpu.step(monitor))throw new Exception("Alive branch PC "+Long.toHexString(h.pc()));}
                boolean accepted=h.pc()==0x12532e;if(!accepted&&h.pc()!=0x1251dc)throw new Exception("Alive branch budget");
                if(accepted){h.cpu.writeRegister("D5",1);h.cpu.writeRegister("SP",0x7004f2);h.jump(0x125348);for(int s=0;s<200;s++){if(h.pc()==0x125396)break;if(h.pc()==0x133a18){h.ret();continue;}int op=h.word(h.pc())&65535;if(op==0xaa1b){h.pop(2);h.value(h.sp(),4,0x600000);h.jump(h.pc()+2);continue;}if(op==0xaa1c||op==0xaa26){h.pop(4);h.jump(h.pc()+2);continue;}if(!h.cpu.step(monitor))throw new Exception("Mask write PC "+Long.toHexString(h.pc()));}if(h.pc()!=0x125396)throw new Exception("Mask write budget");}
                if(selections.length()>0)selections.append(',');selections.append("{\"selected\":").append(selected).append(",\"stamina\":").append(stamina).append(",\"accepted\":").append(accepted).append(",\"mask\":\"").append(hex(h.read(0x4434e0,16))).append("\"}");
            }finally{h.cpu.dispose();}
        }
        String prefix;
        {ActionOracleHarness h=new ActionOracleHarness(currentProgram);try{h.enter(0x12512a,0x700000,0x900000);h.value(0x700004,2,0);h.value(0x700006,2,0);h.value(0x700008,2,0);h.value(0x44273a,1,5);h.value(0x442739,1,0);byte[] before=new byte[16];Arrays.fill(before,(byte)0x71);h.write(0x4434e0,before);for(int s=0;s<200;s++){if(h.pc()==0x1251b8)break;if(!h.cpu.step(monitor))throw new Exception("Choice prefix PC "+Long.toHexString(h.pc()));}if(h.pc()!=0x1251b8)throw new Exception("Choice prefix budget");prefix="{\"before\":\""+hex(before)+"\",\"after\":\""+hex(h.read(0x4434e0,16))+"\",\"remaining\":"+h.cpu.readRegister("D5").intValue()+"}";}finally{h.cpu.dispose();}}
        String identity="\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyWarriorChoice.java"))+"\",\"harness_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/ActionOracleHarness.java"))+"\",\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\"";
        Files.writeString(root.resolve("evidence/fixtures/warrior-choice.json"),"{\"authority\":\"Original newland14 parameter1 block14383c..14386e executes, with C abs and getchoice selectedmask boundary. Original getchoice prefix12512a..1251b8 clears all8selectionwords, getchoice alive branch125300..12532e tests signedstamina>=1 forincludeDead0, accepted mask suffix125348..125396 setschosenword1with sound/Stringgraphics boundaries. SelectionclickUI, reselect, cancel and finalgetchoice return excluded.\",\"cache_identity\":{"+identity+"},\"rng_draws\":0,\"cases\":["+cases+"],\"selection_cases\":["+selections+"],\"prefix\":"+prefix+"}\n");println("Warrior choice caller and selection cases exported");
    }
}
