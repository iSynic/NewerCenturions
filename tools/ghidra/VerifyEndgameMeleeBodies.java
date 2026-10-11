import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;

/** Original default melee weapon damage against a friendly monster record. */
public class VerifyEndgameMeleeBodies extends VerifyMonsterMelee {
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]),original=root.resolve("NewCenturions/TheNewCenturions"),scenario=original.resolve("Scenarios/The Thunder of Guns/0");
        items=Files.readAllBytes(original.resolve("Data Files/Data ID"));scenarioItems=Files.readAllBytes(scenario.resolve("Data NI"));races=Files.readAllBytes(original.resolve("Data Files/Data Race"));monsters=Files.readAllBytes(scenario.resolve("Data MD"));properties=Files.readAllBytes(original.resolve("Data Files/Data Swamp BD"));attacks=Files.readAllBytes(original.resolve("Data Files/Data S"));areas=Files.readAllBytes(original.resolve("Data Files/Data AD"));party=new byte[3138*6];String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};
        for(int i=0;i<6;i++){System.arraycopy(Files.readAllBytes(original.resolve("Warriors/"+names[i])),0,party,i*3138,3138);party[i*3138+0x9df]=1;ByteBuffer.wrap(party).putShort(i*3138+0x50,(short)10000).putShort(i*3138+0x52,(short)10000);}
        StringBuilder cases=new StringBuilder();
        for(long seed:new long[]{12,1})for(int weapon:new int[]{194})for(int loaded=0;loaded<3;loaded++)for(int index=0;index<2;index++)for(int variant=0;variant<5;variant++)for(int actor:new int[]{0,2})probe(seed,weapon,loaded,index,variant,actor,cases);
        StringBuilder identity=new StringBuilder();String[][] inputs={{"probe","tools/ghidra/VerifyEndgameMeleeBodies.java"},{"melee_probe","tools/ghidra/VerifyMonsterMelee.java"},{"parent_probe","tools/ghidra/VerifyAllyTurns.java"},{"inventory_probe","tools/ghidra/VerifyStoryInventory.java"},{"harness","tools/ghidra/ActionOracleHarness.java"},{"sane_boundary","tools/ghidra/SaneBoundary.java"},{"mapped_binary","analysis/ghidra/startup/code1-mapped.bin"},{"monsters","NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data MD"},{"items","NewCenturions/TheNewCenturions/Data Files/Data ID"},{"scenario_items","NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data NI"},{"races","NewCenturions/TheNewCenturions/Data Files/Data Race"},{"properties","NewCenturions/TheNewCenturions/Data Files/Data Swamp BD"},{"attacks","NewCenturions/TheNewCenturions/Data Files/Data S"},{"areas","NewCenturions/TheNewCenturions/Data Files/Data AD"}};
        for(String[] input:inputs){if(identity.length()>0)identity.append(',');identity.append('"').append(input[0]).append("_sha256\":\"").append(ActionOracleHarness.sha256(root.resolve(input[1]))).append('"');}
        Files.writeString(root.resolve("evidence/fixtures/endgame-melee-bodies.json"),"{\"authority\":\"Complete original attack2101eb6 actual MD119 default DataID194 against friendly scientist65, initialcurrent zero, loaded194 or authoredgun29, attackindices0/1, retainedactor0/2 and seeds12/1. Controlled shield, armor, unconscious and forcedmiss targets. Full three monster records, whole unchanged party, persistent58byte packet, native damage/saves and RNG captured. Actual MD119 stats, naturalcondition0 and zero-reflection scientist target; complete active original human party prevents original early no-human return. Original native showresults sound branches execute; actual killbody12e4e4 and remaining presentation are intercepted, so caller owns deaths. No live game claim.\",\"identity\":{"+identity+"},\"cases\":["+cases+"]}\n");println("120 original MD119 default194 monster target cases exported");
    }
    void probe(long seed,int weapon,int loaded,int index,int variant,int actor,StringBuilder cases)throws Exception{
        ActionOracleHarness h=frame();RunResult result=new RunResult(seed);
        try{
            int id=119;
            byte[] source=monster(id,10000),target=monster(65,10000),other=monster(49,10000),records=new byte[864];ByteBuffer s=ByteBuffer.wrap(source),t=ByteBuffer.wrap(target);
            s.putShort(0x28,(short)weapon).putShort(0x4e,(short)(loaded==0?0:loaded==1?weapon:29));
            if(variant==1)t.putShort(0x56,(short)80).putShort(0x58,(short)300);
            if(variant==2)t.putShort(0x62,(short)1000);
            if(variant==3)target[0xb3]=1;
            if(variant==4)t.putShort(8,(short)32767);
            System.arraycopy(source,0,records,0,288);System.arraycopy(target,0,records,288,288);System.arraycopy(other,0,records,576,288);
            state(h,records,terrain());h.value(0x43874a,2,3);h.value(0x447196,2,actor);h.value(0x4435d4,2,index);h.value(0x446f0a,2,20);h.value(0x446f0c,2,20);h.value(0x446f0e,2,21);h.value(0x446f10,2,20);h.value(0x446f12,2,40);h.value(0x446f14,2,40);
            h.enter(0x101eb6,0x6ffe00,0x900000);h.value(0x6ffe04,2,10);h.value(0x6ffe06,2,11);h.value(0x6ffe08,2,index);execute(h,result,0x900000);
            if(!Arrays.equals(party,h.read(0x448aa0,party.length)))throw new Exception("Unexpected party writes");
            if(cases.length()>0)cases.append(',');cases.append("{\"seed_before\":").append(seed).append(",\"weapon\":").append(weapon).append(",\"loaded\":").append(loaded).append(",\"index\":").append(index).append(",\"variant\":").append(variant).append(",\"actor\":").append(actor+10).append(",\"records_before\":\"").append(hex(records)).append("\",\"records_after\":\"").append(hex(h.read(0x439d3c,records.length))).append("\",\"packet_after\":\"").append(hex(h.read(0x44374e,58))).append("\",\"seed_after\":").append(result.seed).append(",\"rng_draws\":").append(result.draws).append(",\"sounds\":[").append(result.sounds).append("]}");
        }finally{h.cpu.dispose();}
    }
    void showresultsSound(ActionOracleHarness h,RunResult result)throws Exception{
        int code=h.word(h.sp()+6),attacker=h.word(h.sp()+8);
        if(code!=0&&code!=-1)throw new Exception("Unexpected loaded melee showresults code "+code);
        String[] names={"D0","D1","D2","D3","D4","D5","D6","D7","A0","A1","A2","A3","A4","A5","A6","SP","PC","XF","NF","ZF","VF","CF"};
        java.math.BigInteger[] saved=new java.math.BigInteger[names.length];for(int i=0;i<names.length;i++)saved[i]=h.cpu.readRegister(names[i]);
        h.cpu.writeRegister("SP",0x6dd000);h.cpu.writeRegister("D5",attacker);h.jump(code==0?0x158776:0x159042);
        boolean done=false;
        for(int step=0;step<100;step++){
            long pc=h.pc();if(pc==0x15886e||pc==0x15904c){done=true;break;}
            if(pc==0x133a18){result.sound(h);continue;}
            if((h.word(pc)&65535)==0xa873){h.pop(4);h.jump(pc+2);continue;}
            if(!h.cpu.step(monitor))throw new Exception("Loaded melee native audio at "+Long.toHexString(pc)+": "+h.cpu.getLastError());
        }
        if(!done)throw new Exception("Loaded melee audio instruction budget");
        for(int i=0;i<names.length;i++)h.cpu.writeRegister(names[i],saved[i]);
    }
    @Override void execute(ActionOracleHarness h,RunResult result,long stop) throws Exception {
        SaneBoundary sane=new SaneBoundary(h.cpu,currentProgram);boolean done=false;
        for(int step=0;step<1500000;step++) {
            long pc=h.pc(),sp=h.sp();if(pc==stop){done=true;break;}
            if(pc==0x400038){h.cpu.writeRegister("D0",Math.abs(h.word(sp+4)));h.ret();continue;}
            if(pc==0x153ca4){result.rng(h);continue;}
            if(pc==0x133a18||pc==0x16474a){result.sound(h);continue;}
            if(pc==0x158454){showresultsSound(h,result);h.ret();continue;}
            if(pc==0x159192){h.value(0x44719a,2,0);h.cpu.writeRegister("D0",0);h.ret();continue;}
            if(pc==0x12e4e4||pc==0x120c14||pc==0x105572||pc==0x10e6b6||pc==0x133d46||pc==0x13154c||pc==0x11f1a8||pc==0x11edc4||pc==0x14a02c||pc==0x13a2c4||pc==0x1205f8||pc==0x16043e||pc==0x1382a8||pc==0x11f136){h.ret();continue;}
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
}
