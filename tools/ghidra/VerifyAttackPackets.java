import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;

/** Original persistent attack packet writers across direct combat actions. */
public class VerifyAttackPackets extends VerifyAllyTurns {
    StringBuilder cases=new StringBuilder();
    byte[] retained(int variant){
        int id=variant%3==0?0:variant%3==1?1713:4101;
        if(id==0)return new byte[58];
        int offset=(id/1000-1)*2842+(id%1000/100-1)*406+(id%100-1)*58;
        byte[] result=Arrays.copyOfRange(attacks,offset,offset+58);
        if(variant%3==1)result[19]=0;
        return result;
    }
    public void run()throws Exception {
        Path root=Path.of(getScriptArgs()[0]),original=root.resolve("NewCenturions/TheNewCenturions"),scenario=original.resolve("Scenarios/The Thunder of Guns/0");items=Files.readAllBytes(original.resolve("Data Files/Data ID"));scenarioItems=Files.readAllBytes(scenario.resolve("Data NI"));races=Files.readAllBytes(original.resolve("Data Files/Data Race"));monsters=Files.readAllBytes(scenario.resolve("Data MD"));properties=Files.readAllBytes(original.resolve("Data Files/Data Swamp BD"));attacks=Files.readAllBytes(original.resolve("Data Files/Data S"));areas=Files.readAllBytes(original.resolve("Data Files/Data AD"));party=new byte[3138*6];String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};for(int i=0;i<6;i++){System.arraycopy(Files.readAllBytes(original.resolve("Warriors/"+names[i])),0,party,i*3138,3138);party[i*3138+0x9df]=1;ByteBuffer.wrap(party).putShort(i*3138+0x50,(short)100).putShort(i*3138+0x52,(short)100);}

        for(long seed:new long[]{12,1,4,7})for(int id:new int[]{49,67,2,44,16})for(int member=0;member<3;member++)for(int index=0;index<2;index++)for(int variant=0;variant<2;variant++)probe("natural",seed,id,member,index,variant);
        for(long seed:new long[]{12,1,4,7})for(int member=0;member<6;member++)for(int variant=0;variant<5;variant++)probe("unarmed",seed,49,member,0,variant);
        for(long seed:new long[]{12,1,4,7})for(int id:new int[]{13,36})for(int member=0;member<3;member++)for(int index=0;index<2;index++)for(int variant=0;variant<2;variant++)probe("melee",seed,id,member,index,variant);
        for(long seed:new long[]{12,1,4,7})for(int id:new int[]{42,44,49})for(int index=0;index<3;index++)for(int variant=0;variant<2;variant++)probe("ally",seed,id,0,index,variant);
        StringBuilder identity=new StringBuilder();String[][] inputs={{"probe","tools/ghidra/VerifyAttackPackets.java"},{"parent_probe","tools/ghidra/VerifyAllyTurns.java"},{"inventory_probe","tools/ghidra/VerifyStoryInventory.java"},{"harness","tools/ghidra/ActionOracleHarness.java"},{"sane_boundary","tools/ghidra/SaneBoundary.java"},{"mapped_binary","analysis/ghidra/startup/code1-mapped.bin"},{"monsters","NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data MD"},{"items","NewCenturions/TheNewCenturions/Data Files/Data ID"},{"scenario_items","NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data NI"},{"races","NewCenturions/TheNewCenturions/Data Files/Data Race"},{"attacks","NewCenturions/TheNewCenturions/Data Files/Data S"},{"areas","NewCenturions/TheNewCenturions/Data Files/Data AD"}};
        for(String[] input:inputs){if(identity.length()>0)identity.append(',');identity.append('"').append(input[0]).append("_sha256\":\"").append(ActionOracleHarness.sha256(root.resolve(input[1]))).append('"');}
        for(int i=0;i<6;i++)identity.append(",\"warrior").append(i).append("_sha256\":\"").append(ActionOracleHarness.sha256(original.resolve("Warriors/"+names[i]))).append('"');
        Files.writeString(root.resolve("evidence/fixtures/attack-packets.json"),"{\"authority\":\"Original complete attack100af0 and attack2101eb6 capture all58 persistent44374e bytes, descriptor identity435c6/8/a and count435cc. Natural0/4/5/9/14, both loaded178 attack indices, hit/miss/fully saved, unconscious and exhausted warrior attack budget cases use blank/stale1713(type byte19 cleared)/4101 packet inputs. Whole target/attacker records and other warrior stability are checked. Native loadweapon/loadammo/resolvedamage/savevs/SANE/RNG execute; presentation and actual killbody12e4e4 boundaries intercepted. Unarmed exhausted budget returns before4103 copy; attack2 copies4103 beforehit. No live original battle claim.\",\"identity\":{"+identity+"},\"base4103\":\""+hex(Arrays.copyOfRange(attacks,3*2842+2*58,3*2842+3*58))+"\",\"cases\":["+cases+"]}\n");println("528 native persistent attack packet cases exported");
    }
    void probe(String kind,long seed,int id,int member,int index,int variant)throws Exception{
        ActionOracleHarness h=frame();RunResult result=new RunResult(seed);
        try{
            byte[] warriors=party.clone(),enemy=monster(id,10000),records=enemy,initialPacket=retained(variant);ByteBuffer w=ByteBuffer.wrap(warriors),m=ByteBuffer.wrap(enemy);
            for(int i=0;i<6;i++)w.putShort(i*3138+0x50,(short)10000).putShort(i*3138+0x52,(short)10000);
            if(kind.equals("melee"))m.putShort(0x28,(short)178).putShort(0x4e,(short)178);
            if(kind.equals("natural"))m.putShort(0x28,(short)0).putShort(0x4e,(short)0);
            if(kind.equals("unarmed")){
                w.putShort(member*3138+0xca,(short)0);
                if(variant==1)w.putShort(member*3138+0xacc,w.getShort(member*3138+0xaca));
                if(variant==2)m.putShort(0x62,(short)1000);
                if(variant==3)enemy[0xb3]=1;
                if(variant==4)m.putShort(8,(short)32767);
            }else if(variant==1&&!kind.equals("ally")){
                for(int offset=0x112;offset<=0x120;offset+=2)w.putShort(member*3138+offset,(short)100);
                w.putShort(member*3138+0x32,(short)3);
            }
            if(kind.equals("ally")){
                m.putShort(0x28,(short)0);byte[] target=monster(65,10000);if(variant==1)ByteBuffer.wrap(target).putShort(0x58,(short)1000);
                records=new byte[576];System.arraycopy(enemy,0,records,0,288);System.arraycopy(target,0,records,288,288);
            }
            byte[] beforeRecords=records.clone();state(h,records,terrain());h.write(0x448aa0,warriors);h.write(0x44374e,initialPacket);int count=variant%2==0?0:7;
            h.value(0x4435cc,2,count);h.value(0x4435c6,2,2);h.value(0x4435c8,2,4);h.value(0x4435ca,2,6);h.value(0x4435d4,2,index);
            h.value(0x446f0a,2,40+member);h.value(0x446f0c,2,40);h.value(0x446f0e,2,40+member);h.value(0x446f10,2,40);
            h.enter(kind.equals("unarmed")?0x100af0:0x101eb6,0x6ffe00,0x900000);
            h.value(0x6ffe04,2,kind.equals("unarmed")?member:10);h.value(0x6ffe06,2,kind.equals("unarmed")?10:kind.equals("ally")?11:member);h.value(0x6ffe08,2,index);execute(h,result,0x900000);
            byte[] afterParty=h.read(0x448aa0,party.length);
            for(int i=0;i<6;i++)if(kind.equals("ally")||i!=member)if(!Arrays.equals(Arrays.copyOfRange(warriors,i*3138,(i+1)*3138),Arrays.copyOfRange(afterParty,i*3138,(i+1)*3138)))throw new Exception("Other warrior changed "+kind+" member"+i);
            if(cases.length()>0)cases.append(',');cases.append("{\"kind\":\"").append(kind).append("\",\"seed_before\":").append(seed).append(",\"monster_id\":").append(id).append(",\"member\":").append(member).append(",\"attack\":").append(index).append(",\"variant\":").append(variant).append(",\"warrior_before\":\"").append(hex(Arrays.copyOfRange(warriors,member*3138,(member+1)*3138))).append("\",\"warrior_after\":\"").append(hex(Arrays.copyOfRange(afterParty,member*3138,(member+1)*3138))).append("\",\"records_before\":\"").append(hex(beforeRecords)).append("\",\"records_after\":\"").append(hex(h.read(0x439d3c,records.length))).append("\",\"packet_before\":\"").append(hex(initialPacket)).append("\",\"packet_after\":\"").append(hex(h.read(0x44374e,58))).append("\",\"identity_before\":[2,4,6],\"identity_after\":[").append(h.word(0x4435c6)).append(',').append(h.word(0x4435c8)).append(',').append(h.word(0x4435ca)).append("],\"count_before\":").append(count).append(",\"count_after\":").append(h.word(0x4435cc)).append(",\"other_warriors_unchanged\":true,\"seed_after\":").append(result.seed).append(",\"rng_draws\":").append(result.draws).append(",\"draws\":[").append(result.drawRecords).append("],\"sounds\":[").append(result.sounds).append("]}");
        }finally{h.cpu.dispose();}
    }
    @Override void execute(ActionOracleHarness h,RunResult result,long stop) throws Exception {
        SaneBoundary sane=new SaneBoundary(h.cpu,currentProgram);boolean done=false;
        for(int step=0;step<1500000;step++) {
            long pc=h.pc(),sp=h.sp();if(pc==stop){done=true;break;}
            if(pc==0x400038){h.cpu.writeRegister("D0",Math.abs(h.word(sp+4)));h.ret();continue;}
            if(pc==0x153ca4){result.rng(h);continue;}
            if(pc==0x133a18||pc==0x16474a){result.sound(h);continue;}
            if(pc==0x159192){h.value(0x44719a,2,0);h.cpu.writeRegister("D0",0);h.ret();continue;}
            if(pc==0x12e4e4||pc==0x120c14||pc==0x105572||pc==0x10e6b6||pc==0x133d46||pc==0x158454||pc==0x13154c||pc==0x11f1a8||pc==0x11edc4||pc==0x14a02c||pc==0x13a2c4||pc==0x1205f8||pc==0x16043e||pc==0x1382a8||pc==0x11f136){h.ret();continue;}
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
