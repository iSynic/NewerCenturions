import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;

/** Original scientist turns and monster attacks against scientists. */
public class VerifyAllyTurns extends VerifyStoryInventory {
    byte[] monsters,properties,attacks,areas;
    StringBuilder turns=new StringBuilder(),hits=new StringBuilder(),contexts=new StringBuilder();
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
        for(int i=0;i<6;i++){System.arraycopy(Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Warriors/"+names[i])),0,party,i*3138,3138);party[i*3138+0x9df]=1;}
        for(long seed:new long[]{12,1})for(int points:new int[]{3,4})for(int distance:new int[]{4,12})for(int itemContext:new int[]{0,1})turn(seed,points,distance,itemContext);
        for(long seed:new long[]{12,1})for(int id:new int[]{42,43,44,45,46,47,49})for(int attack:new int[]{0,1,2,3})for(int targetHp:new int[]{2,100})hit(seed,id,attack,targetHp);
        for(int id:new int[]{0,1,-1,201,804,805,832,-832})context(id,-1);
        for(int member=0;member<6;member++)context(0,member);
        Files.writeString(root.resolve("evidence/fixtures/ally-turns.json"),"{\"authority\":\"Original combat1145c8..115732 scientist65 decision slice including complete weapon1 preparation, target faction selection and movement; controlled action points3/4, movement110 and distant enemy49. Complete original attack2 at101eb6 executes monster42..47/49 attacks0..3 against scientist65 with controlled HP2/100, including loadweapon/loadammo/prepattack/resolvedamage and ordered sounds. Original all four file item caches, Data AD and Data S supplied; combat-active context and controlled loaditem word32 zero/nonzero. Presentation-only routines intercepted, QuickDraw Rand and SANE boundary modeled. Original loaditem-work word32 zero/nonzero is an explicit input; it controls the original stale ammo-selector branch after loadweapon. Original loaditem130afe and calcw139192 work-record context cases supplied. No live battle claim.\",\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyAllyTurns.java"))+"\",\"parent_probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyStoryInventory.java"))+"\",\"harness_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/ActionOracleHarness.java"))+"\",\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\",\"monsters_sha256\":\""+ActionOracleHarness.sha256(root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data MD"))+"\",\"items_sha256\":\""+ActionOracleHarness.sha256(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data ID"))+"\",\"attacks_sha256\":\""+ActionOracleHarness.sha256(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data S"))+"\",\"areas_sha256\":\""+ActionOracleHarness.sha256(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data AD"))+"\",\"scenario_items_sha256\":\""+ActionOracleHarness.sha256(root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data NI"))+"\",\"races_sha256\":\""+ActionOracleHarness.sha256(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data Race"))+"\",\"properties_sha256\":\""+ActionOracleHarness.sha256(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data Swamp BD"))+"\",\"sane_boundary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/SaneBoundary.java"))+"\",\"properties\":\""+hex(properties)+"\",\"turn_cases\":["+turns+"],\"attack_cases\":["+hits+"],\"context_cases\":["+contexts+"]}\n");
        println("Sixteen original scientist turns and one hundred twelve monster attacks exported");
    }
    byte[] monster(int id,int hp) {
        byte[] result=Arrays.copyOfRange(monsters,id*288,(id+1)*288);
        ByteBuffer.wrap(result).putShort(0x110,(short)hp).putShort(0x112,(short)hp).putShort(0x10c,(short)-1).putShort(0x11e,(short)1155);
        return result;
    }
    void state(ActionOracleHarness h,byte[] records,byte[] map) {
        h.write(0x448aa0,new byte[12*3138]);
        common(h,party);h.write(0x439d3c,records);h.write(0x4085b8,map);h.value(0x43874a,2,2);
        h.write(0x4537fe,Arrays.copyOfRange(items,0,30000));h.write(0x46225e,Arrays.copyOfRange(items,30000,60000));h.write(0x45ad2e,Arrays.copyOfRange(items,60000,90000));h.write(0x46978e,Arrays.copyOfRange(items,90000,120000));h.write(0x443788,attacks);
        h.write(0x470cbe,Arrays.copyOfRange(scenarioItems,0,30000));
        h.write(0x43167c,Arrays.copyOfRange(properties,64,12064));h.value(0x442780,4,5);h.value(0x442784,4,5);
        h.value(0x438776,2,1);h.write(0x438210,areas);
        h.value(0x447196,2,0);h.value(0x447194,2,1);h.value(0x4470ba,2,10);h.value(0x4435c4,2,6);h.value(0x44374c,2,0);
        h.value(0x402f43,1,2);h.value(0x43550c,2,155);h.value(0x439568,2,0);h.write(0x442bd6,new byte[1200]);
        h.value(0x43565f,1,0);h.value(0x447d96,2,0);h.value(0x4435cc,2,0);
        for(int i=0;i<6;i++){h.value(0x44709a+i*4,2,40+i);h.value(0x44709c+i*4,2,40);}
    }
    void execute(ActionOracleHarness h,RunResult result,long stop) throws Exception {
        SaneBoundary sane=new SaneBoundary(h.cpu,currentProgram);boolean done=false;
        for(int step=0;step<1500000;step++) {
            long pc=h.pc(),sp=h.sp();if(pc==stop){done=true;break;}
            if(pc==0x400038){h.cpu.writeRegister("D0",Math.abs(h.word(sp+4)));h.ret();continue;}
            if(pc==0x153ca4){result.rng(h);continue;}
            if(pc==0x133a18||pc==0x16474a){result.sound(h);continue;}
            if(pc==0x159192){h.value(0x44719a,2,0);h.cpu.writeRegister("D0",0);h.ret();continue;}
            if(pc==0x120c14||pc==0x105572||pc==0x10e6b6||pc==0x133d46||pc==0x158454||pc==0x13154c||pc==0x11f1a8||pc==0x11edc4||pc==0x14a02c||pc==0x13a2c4||pc==0x1205f8||pc==0x16043e||pc==0x1382a8||pc==0x11f136){h.ret();continue;}
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
    byte[] terrain() {byte[] result=new byte[16200];ByteBuffer w=ByteBuffer.wrap(result);for(int i=0;i<8100;i++)w.putShort(i*2,(short)1155);return result;}
    void turn(long seed,int points,int distance,int itemContext) throws Exception {
        ActionOracleHarness h=frame();RunResult result=new RunResult(seed);
        try {
            byte[] ally=monster(65,2),hostile=monster(49,100),records=new byte[576];ally[0x106]=110;ally[0x109]=(byte)points;ByteBuffer.wrap(hostile).putShort(0xe,(short)1);System.arraycopy(ally,0,records,0,288);System.arraycopy(hostile,0,records,288,288);
            byte[] map=terrain();ByteBuffer.wrap(map).putShort((25*90+25)*2,(short)10).putShort(((25+distance)*90+25)*2,(short)11);
            for(int i=0;i<6;i++)ByteBuffer.wrap(map).putShort(((45+i)*90+45)*2,(short)i);
            state(h,records,map);h.value(0x45365c,2,itemContext);h.value(0x446f0a,2,20);h.value(0x446f0c,2,20);h.value(0x446f0e,2,20+distance);h.value(0x446f10,2,20);h.jump(0x1145c8);
            execute(h,result,0x115732);
            if(turns.length()>0)turns.append(',');turns.append("{\"seed_before\":").append(seed).append(",\"points\":").append(points).append(",\"item_context\":").append(itemContext).append(",\"distance\":").append(distance).append(",\"records_before\":\"").append(hex(records)).append("\",\"records_after\":\"").append(hex(h.read(0x439d3c,576))).append("\",\"warriors_before\":\"").append(hex(party)).append("\",\"warriors_after\":\"").append(hex(h.read(0x448aa0,party.length))).append("\",\"map_before\":\"").append(hex(map)).append("\",\"map_after\":\"").append(hex(h.read(0x4085b8,16200))).append("\",\"positions\":\"").append(hex(h.read(0x446f0a,8))).append("\",\"seed_after\":").append(result.seed).append(",\"draws\":[").append(result.drawRecords).append("],\"rng_draws\":").append(result.draws).append(",\"sounds\":[").append(result.sounds).append("]}");
        }finally{h.cpu.dispose();}
    }
    void hit(long seed,int id,int attack,int targetHp) throws Exception {
        ActionOracleHarness h=frame();RunResult result=new RunResult(seed);
        try {
            byte[] attacker=monster(id,100),target=monster(65,targetHp),records=new byte[576];ByteBuffer.wrap(attacker).putShort(0xe,(short)1);System.arraycopy(attacker,0,records,0,288);System.arraycopy(target,0,records,288,288);
            byte[] map=terrain();state(h,records,map);h.enter(0x101eb6,0x6ffe00,0x900000);h.value(0x6ffe04,2,10);h.value(0x6ffe06,2,11);h.value(0x6ffe08,2,attack);
            execute(h,result,0x900000);
            if(hits.length()>0)hits.append(',');hits.append("{\"seed_before\":").append(seed).append(",\"attacker_id\":").append(id).append(",\"attack\":").append(attack).append(",\"target_hp\":").append(targetHp).append(",\"records_before\":\"").append(hex(records)).append("\",\"records_after\":\"").append(hex(h.read(0x439d3c,576))).append("\",\"map_before\":\"").append(hex(map)).append("\",\"map_after\":\"").append(hex(h.read(0x4085b8,16200))).append("\",\"warriors_before\":\"").append(hex(party)).append("\",\"warriors_after\":\"").append(hex(h.read(0x448aa0,party.length))).append("\",\"positions\":\"").append(hex(h.read(0x446f0a,8))).append("\",\"packet\":\"").append(hex(h.read(0x44374e,58))).append("\",\"result\":").append((short)h.cpu.readRegister("D0").intValue()).append(",\"seed_after\":").append(result.seed).append(",\"draws\":[").append(result.drawRecords).append("],\"rng_draws\":").append(result.draws).append(",\"sounds\":[").append(result.sounds).append("]}");
        }finally{h.cpu.dispose();}
    }
    void context(int id,int member) throws Exception {
        ActionOracleHarness h=frame();RunResult result=new RunResult(12);
        try {
            state(h,new byte[576],terrain());byte[] before=new byte[150];Arrays.fill(before,(byte)0x5a);h.write(0x45363c,before);
            long entry=member<0?0x130afe:0x139192;h.enter(entry,0x6ffe00,0x900000);h.value(0x6ffe04,2,member<0?id:member);execute(h,result,0x900000);
            if(contexts.length()>0)contexts.append(',');contexts.append("{\"kind\":\"").append(member<0?"loaditem":"calcw").append("\",\"item_id\":").append(id).append(",\"member\":").append(member).append(",\"work_before\":\"").append(hex(before)).append("\",\"work_after\":\"").append(hex(h.read(0x45363c,150))).append("\",\"field32\":").append(h.word(0x45365c)).append(",\"warriors_before\":\"").append(hex(party)).append("\",\"warriors_after\":\"").append(hex(h.read(0x448aa0,party.length))).append("\",\"rng_draws\":").append(result.draws).append('}');
        }finally{h.cpu.dispose();}
    }
}
