import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;

/** Required high-level spells and immediate faction masks. */
public class VerifyConditionTargets extends VerifyEnemySpells {
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
        StringBuilder selections=new StringBuilder();int selectionTotal=0;
        for(int initial:new int[]{12,1,2,3,4,5,6,7})for(int distance:new int[]{0,2,8,12})for(int psi:new int[]{0,15,350}) {
            ActionOracleHarness h=frame();RunResult result=new RunResult(initial);
            try {
                byte[] caster=monster(28,10000);ByteBuffer.wrap(caster).putShort(0xe,(short)1).putShort(0x10e,(short)psi);caster[0x106]=20;caster[0x109]=7;
                byte[] map=terrain();ByteBuffer field=ByteBuffer.wrap(map);field.putShort((25*90+25)*2,(short)10);for(int i=0;i<6;i++)field.putShort(((25+distance+i)*90+25)*2,(short)i);
                state(h,caster,map);h.value(0x43874a,2,1);h.value(0x446f0a,2,20);h.value(0x446f0c,2,20);for(int i=0;i<6;i++){h.value(0x44709a+i*4,2,20+distance+i);h.value(0x44709c+i*4,2,20);}
                h.write(0x44374e,packet(1703));h.value(0x4435c6,2,0);h.value(0x4435c8,2,6);h.value(0x4435ca,2,2);h.cpu.writeRegister("D5",0);h.jump(0x114b78);execute(h,result,0x1150d0);
                if(selections.length()>0)selections.append(',');selections.append("{\"spell_id\":1703,\"distance\":").append(distance).append(",\"psi\":").append(psi).append(",\"seed_before\":").append(initial).append(",\"packet\":\"").append(hex(packet(1703))).append("\",\"caster_before\":\"").append(hex(caster)).append("\",\"caster_after\":\"").append(hex(h.read(0x439d3c,288))).append("\",\"map\":\"").append(hex(map)).append("\",\"party\":\"").append(hex(party)).append("\",\"range\":").append(h.word(0x4435bc)).append(",\"nearest\":").append(h.word(0x4436b4)).append(",\"distances\":\"").append(hex(h.read(0x4435d6,224))).append("\",\"count\":").append(h.word(0x4435cc)).append(",\"target_count\":").append(h.word(0x4435c0)).append(",\"targets\":\"").append(hex(h.read(0x443722,42))).append("\",\"cast\":").append(h.pc()==0x1150d0).append(",\"seed_after\":").append(result.seed).append(",\"rng_draws\":").append(result.draws).append("}");selectionTotal++;
            }finally{h.cpu.dispose();}
        }
        Files.writeString(root.resolve("evidence/fixtures/condition-targets.json"),"{\"authority\":\"Original1703 common selection114b78..cast/no-cast includes zero/insufficient/350psi and near/far targets. Exact packet, caster, target records, signed distances and RNG captured; no condition resolution or live battle claim.\",\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyConditionTargets.java"))+"\",\"parent_probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyEnemySpells.java"))+"\",\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\",\"areas\":\""+hex(areas)+"\",\"properties\":\""+hex(properties)+"\",\"cases\":["+selections+"]}\n");println(selectionTotal+" original condition selection cases exported");
    }
}
