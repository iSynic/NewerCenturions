import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;

/** Original enemy firearm decision, target loop and whole actor turn. */
public class VerifyEnemyFireCaller extends VerifyEnemySpells {
    StringBuilder calls,resolutions;long resolveReturn=-1;int bodyCount;
    public void run()throws Exception {
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
        StringBuilder cases=new StringBuilder();
        for(long seed:new long[]{12,1,2,3,4,5,6,7})for(int id:new int[]{4,3,104,105,102})for(int variant=0;variant<6;variant++)probe(seed,id,variant,cases);
        StringBuilder identity=new StringBuilder();String[][] inputs={{"probe","tools/ghidra/VerifyEnemyFireCaller.java"},{"spell_probe","tools/ghidra/VerifyEnemySpells.java"},{"parent_probe","tools/ghidra/VerifyAllyTurns.java"},{"inventory_probe","tools/ghidra/VerifyStoryInventory.java"},{"harness","tools/ghidra/ActionOracleHarness.java"},{"sane_boundary","tools/ghidra/SaneBoundary.java"},{"mapped_binary","analysis/ghidra/startup/code1-mapped.bin"},{"monsters","NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data MD"},{"items","NewCenturions/TheNewCenturions/Data Files/Data ID"},{"scenario_items","NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data NI"},{"races","NewCenturions/TheNewCenturions/Data Files/Data Race"},{"properties","NewCenturions/TheNewCenturions/Data Files/Data Swamp BD"},{"attacks","NewCenturions/TheNewCenturions/Data Files/Data S"},{"areas","NewCenturions/TheNewCenturions/Data Files/Data AD"}};
        for(String[] input:inputs){if(identity.length()>0)identity.append(',');identity.append('"').append(input[0]).append("_sha256\":\"").append(ActionOracleHarness.sha256(root.resolve(input[1]))).append('"');}
        for(int i=0;i<6;i++)identity.append(",\"warrior").append(i).append("_sha256\":\"").append(ActionOracleHarness.sha256(root.resolve("NewCenturions/TheNewCenturions/Warriors/"+names[i]))).append('"');
        Files.writeString(root.resolve("evidence/fixtures/enemy-fire-caller.json"),"{\"authority\":\"Original complete enemy actor decision1145c8..115776 executes authored weapons through original preparation, targeting, cast10d4fe, spelltargets15cb0e, resolvespell151022, resist, resolvedamage, killbody and movement. Explicit controlled HP/positions/item-context; actual cast list/count and resolve masks captured, including alive-target checks. SANE mathematical boundary and presentation/graphics/delay services modeled; no cast reconstruction or live original battle claim. Variant5 first executes original natural attack2 with zeroed conditions, observes LOADITEM reset, then runs authored ranged actor turn with its resulting work context.\",\"identity\":{"+identity+"},\"cases\":["+cases+"]}\n");println("Original complete enemy firearm callers exported");
    }
    void observe(ActionOracleHarness h,RunResult result,long pc)throws Exception{
        if(pc==0x10d4fe){if(calls.length()>0)calls.append(',');calls.append("{\"count\":").append(h.word(0x4435cc)).append(",\"targets\":\"").append(hex(h.read(0x443722,h.word(0x4435c0)*6))).append("\",\"target_count\":").append(h.word(0x4435c0)).append(",\"item_context\":").append(h.word(0x45365c)).append(",\"draws_before\":").append(result.draws).append('}');}
        if(pc==resolveReturn){resolutions.append(",\"party_after\":\"").append(hex(h.read(0x448aa0,party.length))).append("\",\"records_after\":\"").append(hex(h.read(0x439d3c,bodyCount*288))).append("\",\"count_after\":").append(h.word(0x4435cc)).append(",\"draws_after\":").append(result.draws).append('}');resolveReturn=-1;}
        if(pc==0x151022){if(resolutions.length()>0)resolutions.append(',');resolutions.append("{\"count\":").append(h.word(0x4435cc)).append(",\"target_index\":").append(h.word(0x442bd4)).append(",\"mask\":\"").append(hex(h.read(0x4434e0,220))).append("\",\"draws_before\":").append(result.draws);resolveReturn=h.pointer(h.sp());}
    }
    void execute(ActionOracleHarness h,RunResult result,long stop) throws Exception {
        SaneBoundary sane=new SaneBoundary(h.cpu,currentProgram);boolean done=false;
        for(int step=0;step<1500000;step++) {
            long pc=h.pc(),sp=h.sp();
            observe(h,result,pc);if(pc==stop){done=true;break;}
            if(pc==0x4001a0){sane.extended(h.pointer(sp+4),Math.sqrt(sane.extended(sp+8)));h.ret();continue;}
            if(pc==0x400038){h.cpu.writeRegister("D0",Math.abs(h.word(sp+4)));h.ret();continue;}
            if(pc==0x153ca4){result.rng(h);continue;}
            if(pc==0x133a18||pc==0x16474a){result.sound(h);continue;}
            if(pc==0x15b2f6||pc==0x15a1f6){h.ret();continue;}
            if(pc==0x159192){h.value(0x44719a,2,0);h.cpu.writeRegister("D0",0);h.ret();continue;}
            if(pc==0x120c14||pc==0x105572||pc==0x10e6b6||pc==0x133d46||pc==0x158454||pc==0x13154c||pc==0x11f1a8||pc==0x11edc4||pc==0x14a02c||pc==0x13a2c4||pc==0x1205f8||pc==0x16043e||pc==0x1382a8||pc==0x11f136||pc==0x1537ee||pc==0x12382a||pc==0x160fb2||pc==0x10ff54||pc==0x12f040||pc==0x15edfa){h.ret();continue;}
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

    void place(ActionOracleHarness h,byte[] map,byte[] record,int body,int x,int y){
        int size=Math.abs(ByteBuffer.wrap(record).getShort(0x10));
        int[][] cells={{0,0,0x11e,1},{-1,-1,0x118,size==3?1:0},{0,-1,0x11c,size==1||size==3?1:0},{-1,0,0x11a,size>1?1:0}};
        for(int[] cell:cells){ByteBuffer.wrap(record).putShort(cell[2],(short)(cell[3]!=0?1155:0));if(cell[3]!=0)ByteBuffer.wrap(map).putShort(((x+5+cell[0])*90+y+5+cell[1])*2,(short)(cell[2]==0x11e?body:-body));}
    }
    void probe(long seed,int id,int variant,StringBuilder cases)throws Exception{
        ActionOracleHarness h=frame();RunResult result=new RunResult(seed);calls=new StringBuilder();resolutions=new StringBuilder();resolveReturn=-1;bodyCount=variant==1?2:1;
        try{
            byte[] caster=monster(id,10000),target=monster(65,10000),warriors=party.clone(),map=terrain(),records=new byte[bodyCount*288],order=new byte[220];Arrays.fill(order,(byte)255);
            ByteBuffer.wrap(caster).putShort(0xe,(short)1).putShort(0x48,(short)100).putShort(0x42,(short)0);caster[0x106]=(byte)(variant==4?6:24);caster[0x109]=4;
            if(variant==2){ByteBuffer.wrap(warriors).putShort(0x50,(short)1);for(int n=0x2c;n<=0x2e;n+=2)ByteBuffer.wrap(warriors).putShort(n,(short)0);ByteBuffer.wrap(warriors).putShort(0x24,(short)0).putShort(0x38,(short)0);}
            for(int i=0;i<6;i++){ByteBuffer.wrap(warriors).putShort(i*3138+0x16,(short)0);if(variant==1)ByteBuffer.wrap(warriors).putShort(i*3138+0x10,(short)1);}
            state(h,records,map);h.write(0x448aa0,warriors);h.value(0x43874a,2,bodyCount);h.value(0x4435c4,2,bodyCount);h.value(0x442739,1,0);h.value(0x45365c,2,1);h.value(0x4435cc,2,7);h.value(0x446f0a,2,20);h.value(0x446f0c,2,20);
            byte[] warriorPositions=new byte[24];for(int i=0;i<6;i++){
                int x=24+i*4,y=20;if(variant==1){x=40+i;y=40;}if(variant==2||variant==3){x=i<(variant==3?2:1)?24+i:40+i;y=i<(variant==3?2:1)?20:40;}if(variant==4){x=64+i;y=20;}
                ByteBuffer.wrap(warriorPositions).putShort(i*4,(short)x).putShort(i*4+2,(short)y);h.value(0x44709a+i*4,2,x);h.value(0x44709c+i*4,2,y);h.value(0x437a6e+i*2,2,1155);ByteBuffer.wrap(map).putShort(((x+5)*90+y+5)*2,(short)i);ByteBuffer.wrap(order).putShort((i+2)*2,(short)i);
            }
            place(h,map,caster,10,20,20);if(bodyCount>1){place(h,map,target,11,24,20);System.arraycopy(target,0,records,288,288);h.value(0x446f0e,2,24);h.value(0x446f10,2,20);ByteBuffer.wrap(order).putShort(16,(short)11);}
            System.arraycopy(caster,0,records,0,288);ByteBuffer.wrap(order).putShort(2,(short)10);h.write(0x439d3c,records);h.write(0x4085b8,map);h.write(0x4470b8,order);int contextBefore=h.word(0x45365c);String prelude="null";
            if(variant==5){for(int i=0;i<6;i++)h.value(0x439d3c+0x96+i*4,1,0);h.value(0x439d3c+0x28,2,0);byte[] preludeBefore=h.read(0x439d3c,288);h.enter(0x101eb6,0x6ffe00,0x900000);h.value(0x6ffe04,2,10);h.value(0x6ffe06,2,0);h.value(0x6ffe08,2,0);execute(h,result,0x900000);prelude="{\"enemy_before\":\""+hex(preludeBefore)+"\",\"enemy_after\":\""+hex(h.read(0x439d3c,288))+"\",\"item_context_after\":"+h.word(0x45365c)+",\"count_after\":"+h.word(0x4435cc)+",\"seed_after\":"+result.seed+",\"draws\":"+result.draws+"}";h.write(0x439d3c,caster);h.cpu.writeRegister("A6",0x700000);h.cpu.writeRegister("SP",0x6ffe00);}
            byte[] beforeParty=h.read(0x448aa0,party.length);long turnSeed=result.seed;int turnDraws=result.draws,turnContext=h.word(0x45365c),turnCount=h.word(0x4435cc);result.sounds.setLength(0);h.value(0x438780,2,0);h.jump(0x1145c8);execute(h,result,0x115776);
            if(cases.length()>0)cases.append(',');cases.append("{\"monster_id\":").append(id).append(",\"variant\":").append(variant).append(",\"seed_initial\":").append(seed).append(",\"prelude\":").append(prelude).append(",\"seed_before\":").append(turnSeed).append(",\"item_context_initial\":").append(contextBefore).append(",\"item_context_before\":").append(turnContext).append(",\"attack_count_before\":").append(turnCount).append(",\"records_before\":\"").append(hex(records)).append("\",\"records_after\":\"").append(hex(h.read(0x439d3c,records.length))).append("\",\"warriors_before\":\"").append(hex(beforeParty)).append("\",\"warriors_after\":\"").append(hex(h.read(0x448aa0,party.length))).append("\",\"warrior_positions\":\"").append(hex(warriorPositions)).append("\",\"positions_after\":\"").append(hex(h.read(0x446f0a,bodyCount*4))).append("\",\"map_before\":\"").append(hex(map)).append("\",\"map_after\":\"").append(hex(h.read(0x4085b8,16200))).append("\",\"order_before\":\"").append(hex(order)).append("\",\"order_after\":\"").append(hex(h.read(0x4470b8,220))).append("\",\"item_context_after\":").append(h.word(0x45365c)).append(",\"attack_count_after\":").append(h.word(0x4435cc)).append(",\"nearest_after\":").append(h.word(0x4436b4)).append(",\"unavailable_after\":").append((int)(h.read(0x442739,1)[0]&255)).append(",\"seed_after\":").append(result.seed).append(",\"rng_draws\":").append(result.draws-turnDraws).append(",\"draws\":[").append(result.drawRecords).append("],\"sounds\":[").append(result.sounds).append("],\"casts\":[").append(calls).append("],\"resolutions\":[").append(resolutions).append("]}");println("enemycaller MD"+id+" variant"+variant+" seed"+seed);
        }finally{h.cpu.dispose();}
    }
}
