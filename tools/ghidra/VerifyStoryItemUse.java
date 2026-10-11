import ghidra.app.util.PseudoDisassembler;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.ByteBuffer;
import java.util.Arrays;
import java.util.HexFormat;

public class VerifyStoryItemUse extends VerifyStoryInventory {
    void export(Path root) throws Exception {
        PseudoDisassembler disassembler=new PseudoDisassembler(currentProgram);
        StringBuilder out=new StringBuilder();
        for(long pc=0x129fe8;pc<0x12c466;) {
            try {
                var instruction=disassembler.disassemble(toAddr(pc));
                if(instruction==null){pc+=2;continue;}
                out.append(String.format("%08x\t%s\n",pc,instruction));pc+=instruction.getLength();
            } catch(Exception error){pc+=2;}
        }
        Files.writeString(root.resolve("analysis/ghidra/exports/items-tail.asm.tsv"),out);
        println("Items tail original instruction listing exported");
    }
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]);export(root);
        Path scenario=root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0");
        items=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data ID"));scenarioItems=Files.readAllBytes(scenario.resolve("Data NI"));races=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data Race"));party=new byte[3138*6];String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};for(int i=0;i<6;i++)System.arraycopy(Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Warriors/"+names[i])),0,party,i*3138,3138);
        byte[] authored=Arrays.copyOfRange(scenarioItems,32*150,33*150);StringBuilder cases=new StringBuilder();HexFormat hex=HexFormat.of();
        for(int member=0;member<6;member++)for(int variant=0;variant<12;variant++) {
            byte[] definition=authored.clone();int quantity=variant==1||variant==5||variant>=9?2:variant==2?0:variant==3?-1:1;
            if(variant==4)ByteBuffer.wrap(definition).putShort(0x26,(short)0);if(variant==8)ByteBuffer.wrap(definition).putShort(0x10,(short)17);if(variant>=9)ByteBuffer.wrap(definition).putShort(0x32,(short)(variant==9?0:variant==10?32767:32668));
            byte[] before=add(party,member,832,quantity,false);int slot=ByteBuffer.wrap(before).getShort(member*3138+0x1a)-1;
            ByteBuffer b=ByteBuffer.wrap(before);if(variant==6)b.putShort(member*3138+0x50,(short)0);if(variant==7)b.putShort(member*3138+0x50,(short)-10);if(variant==8)b.putShort(member*3138+0x9d6,(short)5);if(variant==5)before[member*3138+0x13b+slot*22]=1;
            ActionOracleHarness h=frame();RunResult result=new RunResult(12);SaneBoundary sane=new SaneBoundary(h.cpu,currentProgram);
            try {
                common(h,before);h.write(0x600000,new byte[512]);h.value(0x44273f,1,member);h.value(0x438776,2,0);h.value(0x447198,2,0);h.value(0x4471aa,2,slot);h.value(0x44719c,2,0);h.value(0x439404,4,0x600000);h.value(0x6fffd2,4,0x600000);h.value(0x600000,4,0x600100);h.value(0x600020,4,0x600100);h.cpu.writeRegister("A2",0x600010);h.cpu.writeRegister("A3",0x600020);h.cpu.writeRegister("D6",832);h.cpu.writeRegister("D7",0);h.jump(0x12a9e8);boolean done=false;StringBuilder warnings=new StringBuilder(),effects=new StringBuilder();
                for(int step=0;step<500000;step++) {
                    long pc=h.pc();if(pc==0x12c41a){done=true;break;}
                    if(pc==0x130afe){int id=Math.abs(h.word(h.sp()+4));if(id==832){h.write(0x45363c,definition);h.ret();}else loadItem(h);continue;}
                    if(pc==0x133a18){if(effects.length()>0)effects.append(',');effects.append("[\"sound\",").append(h.word(h.sp()+4)).append(']');result.sound(h);continue;}
                    if(pc==0x16474a){if(effects.length()>0)effects.append(',');effects.append("[\"warning\",").append(h.word(h.sp()+4)).append(']');if(warnings.length()>0)warnings.append(',');warnings.append(h.word(h.sp()+4));h.ret();continue;}
                    if(pc==0x153ca4){result.rng(h);continue;}
                    if(pc==0x400038){h.cpu.writeRegister("D0",Math.abs(h.word(h.sp()+4)));h.ret();continue;}
                    if(pc==0x14ce2a||pc==0x1205f8||pc==0x160d08||pc==0x15fa20||pc==0x14a02c){h.ret();continue;}
                    if((h.word(pc)&65535)==0xa9eb){sane.execute();continue;}
                    if(!h.cpu.step(monitor))throw new Exception("Item use member"+member+" variant"+variant+" PC "+Long.toHexString(pc)+": "+h.cpu.getLastError());
                }
                if(!done)throw new Exception("Item use budget");int returnCode=h.word(0x44719a),script=-1;String scriptArgs="";
                h.write(0x6fc000,new byte[24576]);h.cpu.writeRegister("A6",0x700000);h.cpu.writeRegister("SP",0x6ffe00);h.value(0x6ffffa,2,returnCode);h.jump(0x10a9d4);done=false;
                for(int step=0;step<1000;step++) {
                    long pc=h.pc();if(pc==0x1400e4){script=h.word(h.sp()+14);scriptArgs=hex.formatHex(h.read(h.sp()+4,14));if(effects.length()>0)effects.append(',');effects.append("[\"script\",").append(script).append(']');done=true;break;}
                    if(pc==0x10ae78){done=true;break;}
                    if(pc==0x13bb20){h.ret();continue;}
                    if(pc==0x400038){h.cpu.writeRegister("D0",Math.abs(h.word(h.sp()+4)));h.ret();continue;}
                    if(!h.cpu.step(monitor))throw new Exception("Outdoor item caller PC "+Long.toHexString(pc)+": "+h.cpu.getLastError());
                }
                if(!done)throw new Exception("Outdoor item caller budget");
                if(cases.length()>0)cases.append(',');cases.append("{\"member\":").append(member).append(",\"slot\":").append(slot).append(",\"variant\":").append(variant).append(",\"definition\":\"").append(hex.formatHex(definition)).append("\",\"before\":\"").append(hex.formatHex(before)).append("\",\"after\":\"").append(hex.formatHex(h.read(0x448aa0,party.length))).append("\",\"return_code\":").append(returnCode).append(",\"script\":").append(script).append(",\"script_args\":\"").append(scriptArgs).append("\",\"effects\":[").append(effects).append("],\"sounds\":[").append(result.sounds).append("],\"warnings\":[").append(warnings).append("],\"seed_before\":12,\"seed_after\":").append(result.seed).append(",\"draws\":").append(result.draws).append('}');
            }finally{h.cpu.dispose();}
        }
        StringBuilder movements=new StringBuilder();for(int i=0;i<10;i++){if(i>0)movements.append(',');movements.append(races[i*408+197]);}
        Files.writeString(root.resolve("evidence/fixtures/item-use.json"),"{\"authority\":\"Original items12a9e8..12c41a selected class23 Use button slice, including native dropitem121152/movecalc and signed quantity/weight arithmetic; composed with original outdoor buttonchoice decoder10a9d4..10aa0c to first newland call. UI selection and definition loading supplied at boundaries; pressbutton/delay/list rendering/update intercepted; abs_short service modeled. Actual scenario NI832 is used, with controlled quantity/consumption/baseweight/script fields and dead or equipped selected-member variants. newland body not executed here. No live UI or route claim.\",\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyStoryItemUse.java"))+"\",\"base_probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyStoryInventory.java"))+"\",\"harness_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/ActionOracleHarness.java"))+"\",\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\",\"scenario_items_sha256\":\""+ActionOracleHarness.sha256(scenario.resolve("Data NI"))+"\",\"items_sha256\":\""+ActionOracleHarness.sha256(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data ID"))+"\",\"races_sha256\":\""+ActionOracleHarness.sha256(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data Race"))+"\",\"race_movements\":["+movements+"],\"cases\":["+cases+"]}\n");
        println("Seventy-two original class23 item-use cases exported");
    }
}
