import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;

/** Required high-level spells and immediate faction masks. */
public class VerifyAdditionalSpells extends VerifyEnemySpells {
    void execute(ActionOracleHarness h,RunResult result,long stop) throws Exception {
        SaneBoundary sane=new SaneBoundary(h.cpu,currentProgram);boolean done=false;
        for(int step=0;step<1500000;step++) {
            long pc=h.pc(),sp=h.sp();if(pc==stop||(stop==0x1150d0&&(pc==0x11510a||(pc==0x1363e4&&h.word(sp+4)==0)))){done=true;break;}
            if(pc==0x400038){h.cpu.writeRegister("D0",Math.abs(h.word(sp+4)));h.ret();continue;}
            if(pc==0x153ca4){result.rng(h);continue;}
            if(pc==0x133a18||pc==0x16474a){result.sound(h);continue;}
            if(pc==0x15b2f6||pc==0x15a1f6){h.ret();continue;}
            if(pc==0x159192){h.value(0x44719a,2,0);h.cpu.writeRegister("D0",0);h.ret();continue;}
            if(pc==0x120c14||pc==0x105572||pc==0x10e6b6||pc==0x133d46||pc==0x158454||pc==0x13154c||pc==0x11f1a8||pc==0x11edc4||pc==0x14a02c||pc==0x13a2c4||pc==0x1205f8||pc==0x16043e||pc==0x1382a8||pc==0x11f136){h.ret();continue;}
            if(pc==0x123d1c){int sound=h.word(sp+14);if(sound!=0){if(result.sounds.length()>0)result.sounds.append(',');result.sounds.append(sound);}h.ret();continue;}
            int op=h.word(pc)&65535;
            if(op==0xa9eb){
                // Apple Numerics Manual FSQRTX uses one extended destination address.
                if((h.word(sp)&65535)==0x12){long destination=h.pointer(sp+2);sane.extended(destination,Math.sqrt(sane.extended(destination)));h.pop(6);h.jump(pc+2);}else sane.execute();
                continue;
            }
            if(op==0xa873||op==0xaa1c||op==0xa862||op==0xa863||op==0xa89b||op==0xa893||op==0xa891||op==0xa884){h.pop(4);h.jump(pc+2);continue;}
            if(op==0xa89c){h.pop(2);h.jump(pc+2);continue;}
            if(op==0xa8b7){h.pop(4);h.jump(pc+2);continue;}
            if(op==0xa8ec){h.pop(22);h.jump(pc+2);continue;}
            if(op==0xa8a9){int vertical=h.word(sp),horizontal=h.word(sp+2);long rect=h.pointer(sp+4);h.value(rect,2,h.word(rect)+vertical);h.value(rect+2,2,h.word(rect+2)+horizontal);h.value(rect+4,2,h.word(rect+4)-vertical);h.value(rect+6,2,h.word(rect+6)-horizontal);h.pop(8);h.jump(pc+2);continue;}
            if(op==0xa8aa){long output=h.pointer(sp),b=h.pointer(sp+4),a=h.pointer(sp+8);int top=Math.max(h.word(a),h.word(b)),left=Math.max(h.word(a+2),h.word(b+2)),bottom=Math.min(h.word(a+4),h.word(b+4)),right=Math.min(h.word(a+6),h.word(b+6));boolean empty=top>=bottom||left>=right;h.value(output,2,empty?0:top);h.value(output+2,2,empty?0:left);h.value(output+4,2,empty?0:bottom);h.value(output+6,2,empty?0:right);h.value(sp+12,1,empty?0:1);h.pop(12);h.jump(pc+2);continue;}
            if(op==0xa8a7){long rectangle=h.pointer(sp+8);h.write(rectangle,h.read(sp+4,4));h.write(rectangle+4,h.read(sp,4));h.pop(12);h.jump(pc+2);continue;}
            if(op==0xaa1e){h.value(sp+2,4,0);h.pop(2);h.jump(pc+2);continue;}
            if(op==0xa8ad){long rect=h.pointer(sp);int x=h.word(sp+4),y=h.word(sp+6);h.value(sp+8,1,x>=h.word(rect)&&x<h.word(rect+4)&&y>=h.word(rect+2)&&y<h.word(rect+6)?1:0);h.pop(8);h.jump(pc+2);continue;}
            if(op==0xa9b4){h.jump(pc+2);continue;}
            if(op==0xa974){h.cpu.writeRegister("SP",sp-2);h.value(sp-2,2,0);h.jump(pc+2);continue;}
            if(!h.cpu.step(monitor))throw new Exception("Ally action at "+Long.toHexString(pc)+": "+h.cpu.getLastError());
        }
        if(!done)throw new Exception("Ally action instruction budget");
    }
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]);
        items=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data ID"));
        races=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data Race"));
        monsters=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data MD"));
        properties=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data Swamp BD"));
        attacks=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data S"));
        areas=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data AD"));
        scenarioItems=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data NI"));
        party=new byte[6*3138];String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};
        for(int i=0;i<6;i++){System.arraycopy(Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Warriors/"+names[i])),0,party,i*3138,3138);party[i*3138+0x9df]=1;ByteBuffer.wrap(party).putShort(i*3138+0x50,(short)10000).putShort(i*3138+0x52,(short)10000);}
        StringBuilder cases=new StringBuilder();int total=0;
        for(int id:new int[]{2503,2602})for(int initial:new int[]{12,1,2,3,4,5,6,7})for(int count:new int[]{0,1,3,7})for(int variant=0;variant<3;variant++) {
            ActionOracleHarness h=frame();RunResult result=new RunResult(initial);
            try {
                byte[] records=new byte[864];int[] ids={13,65,94};
                for(int i=0;i<3;i++){
                    byte[] record=monster(ids[i],10000);
                    if(variant==1)record[0xc2]=1;
                    if(variant==2)ByteBuffer.wrap(record).putShort(0xa,(short)100);
                    System.arraycopy(record,0,records,i*288,288);
                }
                byte[] mask=new byte[220];for(int i=0;i<6;i++)ByteBuffer.wrap(mask).putShort(i*2,(short)1);for(int i=0;i<3;i++)ByteBuffer.wrap(mask).putShort((i+10)*2,(short)1);
                state(h,records,terrain());h.value(0x43874a,2,3);h.write(0x448aa0,party);h.write(0x44374e,packet(id));h.value(0x4435c6,2,id/1000-1);h.value(0x4435c8,2,id%1000/100-1);h.value(0x4435ca,2,id%100-1);h.value(0x4435cc,2,count);
                h.write(0x453768,new byte[150]);h.write(0x4536d2,new byte[150]);h.write(0x4434e0,mask);h.value(0x438760,2,1);h.value(0x443580,1,0);h.value(0x4358c2,1,0);h.write(0x435884,new byte[128]);
                h.enter(0x151022,0x6ffe00,0x900000);
                try { execute(h,result,0x900000); }
                catch(Exception error) { println("Damage case "+id+" seed "+initial+" count "+count+" variant "+variant+" stack "+hex(h.read(h.sp(),128)));throw error; }
                if(cases.length()>0)cases.append(',');cases.append("{\"spell_id\":").append(id).append(",\"count\":").append(count).append(",\"variant\":").append(variant).append(",\"seed_before\":").append(initial).append(",\"packet\":\"").append(hex(packet(id))).append("\",\"mask\":\"").append(hex(mask)).append("\",\"warriors_before\":\"").append(hex(party)).append("\",\"warriors_after\":\"").append(hex(h.read(0x448aa0,party.length))).append("\",\"enemies_before\":\"").append(hex(records)).append("\",\"enemies_after\":\"").append(hex(h.read(0x439d3c,records.length))).append("\",\"seed_after\":").append(result.seed).append(",\"rng_draws\":").append(result.draws).append(",\"sounds\":[").append(result.sounds).append("]}");total++;
            }finally{h.cpu.dispose();}
        }
        StringBuilder masks=new StringBuilder();int maskTotal=0;
        for(int kind:new int[]{9,10,12})for(int side:new int[]{0,1})for(int count:new int[]{0,1,7}) {
            ActionOracleHarness h=frame();RunResult result=new RunResult(12);
            try {
                byte[] input=party.clone(),records=new byte[864],p=packet(2503);p[1]=(byte)kind;
                for(int i=0;i<6;i++){ByteBuffer.wrap(input).putShort(i*3138+0x10,(short)(i%2));if(i==1)input[i*3138+0x9df]=0;if(i==2)ByteBuffer.wrap(input).putShort(i*3138+0x50,(short)0);}
                for(int i=0;i<3;i++){byte[] m=monster(28,i==2?0:10000);ByteBuffer.wrap(m).putShort(0xe,(short)(i==0?side:1-side));m[0x104]=7;if(i==1)m[0xb0]=(byte)255;System.arraycopy(m,0,records,i*288,288);}
                state(h,records,terrain());h.value(0x43874a,2,3);h.write(0x448aa0,input);h.write(0x44374e,p);h.value(0x4435cc,2,count);h.jump(0x1148b8);execute(h,result,0x151022);
                if(masks.length()>0)masks.append(',');masks.append("{\"kind\":").append(kind).append(",\"side\":").append(side).append(",\"count_before\":").append(count).append(",\"count_after\":").append(h.word(0x4435cc)).append(",\"packet\":\"").append(hex(p)).append("\",\"warriors\":\"").append(hex(input)).append("\",\"enemies_before\":\"").append(hex(records)).append("\",\"enemies_after\":\"").append(hex(h.read(0x439d3c,records.length))).append("\",\"mask\":\"").append(hex(h.read(0x4434e0,220))).append("\",\"rng_draws\":").append(result.draws).append("}");maskTotal++;
            }finally{h.cpu.dispose();}
        }
        StringBuilder selections=new StringBuilder();int selectionTotal=0;
        for(int initial:new int[]{12,1,2,3,4,5,6,7})for(int distance:new int[]{0,2,8,12})for(int psi:new int[]{0,15,350}) {
            ActionOracleHarness h=frame();RunResult result=new RunResult(initial);
            try {
                byte[] caster=monster(28,10000);ByteBuffer.wrap(caster).putShort(0xe,(short)1).putShort(0x10e,(short)psi);caster[0x106]=20;caster[0x109]=7;
                byte[] map=terrain();ByteBuffer field=ByteBuffer.wrap(map);field.putShort((25*90+25)*2,(short)10);for(int i=0;i<6;i++)field.putShort(((25+distance+i)*90+25)*2,(short)i);
                state(h,caster,map);h.value(0x43874a,2,1);h.value(0x446f0a,2,20);h.value(0x446f0c,2,20);for(int i=0;i<6;i++){h.value(0x44709a+i*4,2,20+distance+i);h.value(0x44709c+i*4,2,20);}
                h.write(0x44374e,packet(2602));h.value(0x4435c6,2,1);h.value(0x4435c8,2,5);h.value(0x4435ca,2,1);h.cpu.writeRegister("D5",0);h.jump(0x114b78);execute(h,result,0x1150d0);
                if(selections.length()>0)selections.append(',');selections.append("{\"spell_id\":2602,\"distance\":").append(distance).append(",\"psi\":").append(psi).append(",\"seed_before\":").append(initial).append(",\"packet\":\"").append(hex(packet(2602))).append("\",\"caster_before\":\"").append(hex(caster)).append("\",\"caster_after\":\"").append(hex(h.read(0x439d3c,288))).append("\",\"map\":\"").append(hex(map)).append("\",\"party\":\"").append(hex(party)).append("\",\"range\":").append(h.word(0x4435bc)).append(",\"nearest\":").append(h.word(0x4436b4)).append(",\"distances\":\"").append(hex(h.read(0x4435d6,224))).append("\",\"count\":").append(h.word(0x4435cc)).append(",\"target_count\":").append(h.word(0x4435c0)).append(",\"targets\":\"").append(hex(h.read(0x443722,42))).append("\",\"cast\":").append(h.pc()==0x1150d0).append(",\"seed_after\":").append(result.seed).append(",\"rng_draws\":").append(result.draws).append("}");selectionTotal++;
            }finally{h.cpu.dispose();}
        }
        Files.writeString(root.resolve("evidence/fixtures/additional-spells.json"),"{\"authority\":\"Original complete resolvespell151022 for required2503/2602 packets, full party plus MD13/65/94 bodies, counts0/1/3/7, native resist and damage. Original combat1148b8..first resolvespell151022 captures global targeting9/10/12, controlled mixed factions, inactive/dead warrior, dead/charmed monster and attack count preservation. Original2602 common selection114b78..cast/no-cast includes zero/insufficient/350psi and near/far targets. No cast geometry, deaths or live battle claim.\",\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyAdditionalSpells.java"))+"\",\"parent_probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyEnemySpells.java"))+"\",\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\",\"areas\":\""+hex(areas)+"\",\"properties\":\""+hex(properties)+"\",\"cases\":["+cases+"],\"mask_cases\":["+masks+"],\"selection_cases\":["+selections+"]}\n");println(total+" original additional damage, "+maskTotal+" immediate mask and "+selectionTotal+" selection cases exported");
    }
}
