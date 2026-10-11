import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import com.google.gson.*;

/** Original learned healing, electric ward and outdoor party protection. */
public class VerifyPlayerSupportPsi extends VerifyReachedGuardianSpell {
    JsonArray healingCalls;
    @Override void observe(ActionOracleHarness h,RunResult r,long pc)throws Exception {
        super.observe(h,r,pc);
        if(pc==0x1287c2){JsonArray a=new JsonArray();a.add(h.word(h.sp()+4));a.add(h.word(h.sp()+6));a.add(h.word(h.sp()+8));healingCalls.add(a);}
    }
    void execute(ActionOracleHarness h,RunResult result,long stop) throws Exception {
        SaneBoundary sane=new SaneBoundary(h.cpu,currentProgram);boolean done=false;
        for(int step=0;step<1500000;step++) {
            long pc=h.pc(),sp=h.sp();
            if(pc==0x1363e4&&h.word(sp+4)==0){diagnostic=true;diagnosticSpell=(h.word(0x4435c6)+1)*1000+(h.word(0x4435c8)+1)*100+h.word(0x4435ca)+1;done=true;break;}
            observe(h,result,pc);if(pc==stop){done=true;break;}
            if(pc==0x4001a0){sane.extended(h.pointer(sp+4),Math.sqrt(sane.extended(sp+8)));h.ret();continue;}
            if(pc==0x400038){h.cpu.writeRegister("D0",Math.abs(h.word(sp+4)));h.ret();continue;}
            if(pc==0x153ca4){result.rng(h);continue;}
            if(pc==0x133a18||pc==0x16474a){result.sound(h);continue;}
            if(pc==0x123d1c){int sound=h.word(sp+14);if(sound!=0){if(result.sounds.length()>0)result.sounds.append(',');result.sounds.append(sound);}h.ret();continue;}
            if(pc==0x15b2f6||pc==0x15a1f6){h.ret();continue;}
            if(pc==0x159192){h.value(0x44719a,2,0);h.cpu.writeRegister("D0",0);h.ret();continue;}
            if(pc==0x158454){showresultsSound(h,result);h.ret();continue;}
            if(pc==0x1600fc||pc==0x120c14||pc==0x105572||pc==0x10e6b6||pc==0x133d46||pc==0x13154c||pc==0x11f1a8||pc==0x11edc4||pc==0x14a02c||pc==0x13a2c4||pc==0x1205f8||pc==0x16043e||pc==0x1382a8||pc==0x11f136||pc==0x1537ee||pc==0x12382a||pc==0x160fb2||pc==0x10ff54||pc==0x12f040||pc==0x15edfa){h.ret();continue;}
            int op=h.word(pc)&65535;
            if(op==0xa85d){int bit=(int)h.pointer(sp);long address=h.pointer(sp+4);int value=h.read(address+(bit>>3),1)[0]&255;h.value(sp+8,1,(value&(0x80>>>(bit&7)))!=0?1:0);h.pop(8);h.jump(pc+2);continue;}
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

    public void run()throws Exception {
        Path root=Path.of(getScriptArgs()[0]),original=root.resolve("NewCenturions/TheNewCenturions"),scenario=original.resolve("Scenarios/The Thunder of Guns/0");
        items=Files.readAllBytes(original.resolve("Data Files/Data ID"));scenarioItems=Files.readAllBytes(scenario.resolve("Data NI"));races=Files.readAllBytes(original.resolve("Data Files/Data Race"));monsters=Files.readAllBytes(scenario.resolve("Data MD"));properties=Files.readAllBytes(original.resolve("Data Files/Data Swamp BD"));attacks=Files.readAllBytes(original.resolve("Data Files/Data S"));areas=Files.readAllBytes(original.resolve("Data Files/Data AD"));party=new byte[18828];String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};for(int i=0;i<6;i++)System.arraycopy(Files.readAllBytes(original.resolve("Warriors/"+names[i])),0,party,i*3138,3138);
        JsonArray cases=new JsonArray(),gates=new JsonArray(),attacksOut=new JsonArray(),ticks=new JsonArray(),mitigation=new JsonArray(),geometryCases=new JsonArray(),electricCases=new JsonArray();
        for(int id:new int[]{1101,1305,1103})for(long seed:new long[]{12,1})for(int power:new int[]{1,2})for(int variant=0;variant<4;variant++)support(cases,id,seed,power,variant);
        for(long seed:new long[]{12,1})for(int variant=4;variant<10;variant++)support(cases,1101,seed,1,variant);
        for(long seed:new long[]{12,1})for(int variant=4;variant<8;variant++)support(cases,1305,seed,1,variant);
        for(int id:new int[]{1101,1305,1103})for(boolean combat:new boolean[]{false,true})for(boolean sufficient:new boolean[]{false,true})admission(gates,id,combat,sufficient);
        for(int weapon:new int[]{187,9})for(long seed:new long[]{12,1})for(int buff:new int[]{0,20})attack(attacksOut,weapon,seed,buff);
        for(int buff:new int[]{0,1,20,-1})tick(ticks,buff);
        for(int buff:new int[]{0,1,20,-1})mitigate(mitigation,buff);
        geometry(geometryCases,1305,8,false);geometry(geometryCases,1305,9,false);geometry(geometryCases,1305,8,true);geometry(geometryCases,1101,1,true);
        for(long seed:new long[]{12,1})for(int ward:new int[]{0,2})electric(electricCases,seed,ward);
        JsonObject identity=new JsonObject();String[][] inputs={{"probe","tools/ghidra/VerifyPlayerSupportPsi.java"},{"bit_boundary_probe","tools/ghidra/VerifyReachedGuardianSpell.java"},{"caller_probe","tools/ghidra/VerifyEnemySpellCaller.java"},{"spell_probe","tools/ghidra/VerifyEnemySpells.java"},{"parent_probe","tools/ghidra/VerifyAllyTurns.java"},{"inventory_probe","tools/ghidra/VerifyStoryInventory.java"},{"harness","tools/ghidra/ActionOracleHarness.java"},{"sane_boundary","tools/ghidra/SaneBoundary.java"},{"audio_boundary","tools/ghidra/ResultAudioBoundary.java"},{"mapped_binary","analysis/ghidra/startup/code1-mapped.bin"},{"monsters","NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data MD"},{"items","NewCenturions/TheNewCenturions/Data Files/Data ID"},{"scenario_items","NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data NI"},{"races","NewCenturions/TheNewCenturions/Data Files/Data Race"},{"properties","NewCenturions/TheNewCenturions/Data Files/Data Swamp BD"},{"attacks","NewCenturions/TheNewCenturions/Data Files/Data S"},{"areas","NewCenturions/TheNewCenturions/Data Files/Data AD"}};for(String[] i:inputs)identity.addProperty(i[0]+"_sha256",ActionOracleHarness.sha256(root.resolve(i[1])));for(int i=0;i<6;i++)identity.addProperty("warrior"+i+"_sha256",ActionOracleHarness.sha256(original.resolve("Warriors/"+names[i])));
        JsonObject out=new JsonObject();out.addProperty("authority","Original accepted player selection charge10d2b4, combat body/point target preparation and complete cast10d4fe/resolvespell151022 for authored learned1101/1305, plus complete original outdoor global party selection10aca0 and resolve1103 with zero allied NPC count. Explicit selected Solestri and controlled wounded/negativeHP/race3/save/immunity/condition overlap and lowAP inputs. Original mode preflight10d1fe, affordability10cc02, complete attack2101eb6 with original187/9 definitions and explicit guardian weapon selection, native party-buff reduction14dede and mitigation103c14, original range119238/checkrange11235e/LOS11b090 preflight, and complete resolvedamage15075a for original weapon9 electric component prepared by130d38/130e54/14c6c2. Full records, fields, map, packet, spell memory, buff20bytes, ordered RNG/audio captured. Accepted menu selection is explicit input; presentation and SANE/BitTst boundaries acknowledged as inherited. No live original-game or campaign victory claim.");out.add("identity",identity);out.add("cases",cases);out.add("admission_cases",gates);out.add("attack_cases",attacksOut);out.add("tick_cases",ticks);out.add("mitigation_cases",mitigation);out.add("geometry_cases",geometryCases);out.add("electric_cases",electricCases);Files.writeString(root.resolve("evidence/fixtures/player-support-psi.json"),out+"\n");println("Player support psi exported: "+cases.size()+" casts,12 admission,8 attacks,4 ticks,4 mitigation,4 geometry,4 electric");
    }
    void reset(long seed){calls=new StringBuilder();resolutions=new StringBuilder();resolveReturn=-1;diagnostic=false;diagnosticSpell=0;bodyCount=1;appliedMask=new byte[220];healingCalls=new JsonArray();}
    byte[] controlledParty(){byte[] w=party.clone();for(int i=0;i<6;i++){w[i*3138+0x9df]=1;ByteBuffer.wrap(w).putShort(i*3138+0x50,(short)27).putShort(i*3138+0x52,(short)27).putShort(i*3138+0x3c,(short)5).putShort(i*3138+0x40,(short)3);}
        ByteBuffer.wrap(w).putShort(5*3138+0x58,(short)100).putShort(5*3138+0xac6,(short)10).putShort(5*3138+0xac8,(short)0);return w;}
    void initialize(ActionOracleHarness h,byte[] w,byte[] records,byte[] map,boolean combat,int caster)throws Exception {
        state(h,records,map);h.write(0x448aa0,w);h.value(0x438776,2,combat?1:0);h.value(0x44273a,1,5);h.value(0x44273f,1,caster);h.value(0x4435d2,2,caster);h.value(0x4470ba,2,caster);h.value(0x447194,2,1);h.value(0x43874a,2,records.length/288);h.value(0x4435c4,2,records.length/288);h.value(0x44257c,2,0);h.write(0x440edc,new byte[1728]);h.write(0x442562,new byte[20]);h.write(0x442bd6,new byte[1200]);h.write(0x435884,new byte[128]);h.write(0x43788a,new byte[480]);h.write(0x442580,new byte[24]);h.write(0x437a7a,new byte[24]);h.write(0x442798,new byte[24]);h.write(0x4434e0,new byte[220]);h.write(0x443722,new byte[42]);h.write(0x453768,new byte[150]);h.write(0x4536d2,new byte[150]);h.value(0x435662,1,1);h.value(0x438778,2,0);h.value(0x438760,2,1);h.value(0x443580,1,0);h.value(0x4358c2,1,0);h.value(0x447198,2,0);h.value(0x442739,1,0);h.value(0x442780,4,5);h.value(0x442784,4,5);
        for(int i=0;i<6;i++){h.value(0x44709a+i*4,2,20);h.value(0x44709c+i*4,2,20+i);h.value(0x437a6e+i*2,2,1155);ByteBuffer.wrap(map).putShort((25*90+25+i)*2,(short)i);}h.value(0x446f0a,2,40);h.value(0x446f0c,2,40);ByteBuffer.wrap(map).putShort((45*90+45)*2,(short)10);h.write(0x4085b8,map);h.cpu.writeRegister("A6",0x6ffd00);
    }
    JsonObject snap(ActionOracleHarness h,RunResult r)throws Exception {JsonObject j=new JsonObject();j.addProperty("party",hex(h.read(0x448aa0,18828)));j.addProperty("party_positions",hex(h.read(0x44709a,24)));j.addProperty("monster_positions",hex(h.read(0x446f0a,4)));j.addProperty("records",hex(h.read(0x439d3c,288)));j.addProperty("map",hex(h.read(0x4085b8,16200)));j.addProperty("fields",hex(h.read(0x442bd6,1200)));j.addProperty("used",hex(h.read(0x435884,128)));j.addProperty("memory",hex(h.read(0x43788a,480)));j.addProperty("selected",hex(h.read(0x442580,24)));j.addProperty("powers",hex(h.read(0x437a7a,24)));j.addProperty("markers",hex(h.read(0x442798,24)));j.addProperty("party_buffs",hex(h.read(0x442562,20)));j.addProperty("packet",hex(h.read(0x44374e,58)));j.addProperty("identity",hex(h.read(0x4435c6,6)));j.addProperty("mask",hex(h.read(0x4434e0,220)));j.addProperty("targets",hex(h.read(0x443722,42)));j.addProperty("count",h.word(0x4435cc));j.addProperty("unavailable",h.read(0x442739,1)[0]&255);j.addProperty("last_amount",h.word(0x4434b8));j.addProperty("seed",r.seed);j.addProperty("draws",r.draws);return j;}
    void select(ActionOracleHarness h,int id,int power)throws Exception{h.write(0x44374e,packet(id));h.value(0x4435c6,2,id/1000-1);h.value(0x4435c8,2,id%1000/100-1);h.value(0x4435ca,2,id%100-1);h.value(0x4435cc,2,power);h.value(0x4435c0,2,0);h.value(0x4435be,2,1);}
    void support(JsonArray output,int id,long seed,int power,int variant)throws Exception {
        reset(seed);println("support "+id+" seed "+seed+" power "+power+" variant "+variant);ActionOracleHarness h=frame();RunResult r=new RunResult(seed);
        try{byte[] w=controlledParty(),m=monster(13,10000),map=terrain();ByteBuffer b=ByteBuffer.wrap(w);int hp=10;
            if(id==1101){hp=variant==1?27:variant==2?-5:variant==3?-10:variant==7?0:10;if(variant==4)b.putShort(0x32,(short)3);if(variant==5)b.putShort(0x7e,(short)1);if(variant==6)b.putShort(0x22,(short)100);b.putShort(0x50,(short)hp);}
            if(id==1305){b.putShort(0x76,(short)(variant==1?-1:variant==2?98:variant==3?99:variant==4?10:0));if(variant==5)b.putShort(0x7e,(short)1);if(variant==6)b.putShort(0x22,(short)100);}
            if((id==1101&&variant==8)||(id==1305&&variant==7))b.putShort(5*3138+0x3c,(short)1).putShort(5*3138+0x40,(short)1);
            if(id==1101&&variant==9){System.arraycopy(party,3*3138,w,3*3138,3138);w[3*3138+0x9df]=1;ByteBuffer.wrap(w).putShort(3*3138+0x50,(short)10).putShort(0x50,(short)27);}
            initialize(h,w,m,map,id!=1103,5);if(id==1101&&hp<=0)h.value(0x442739,1,1);if(id==1103)h.value(0x442566,2,variant==1?100:variant==2?-1:variant==3?1:0);select(h,id,power);if(id==1101){h.value(0x44709a+20,2,19);h.value(0x44709c+20,2,variant==9?23:20);ByteBuffer.wrap(map).putShort((25*90+30)*2,(short)1155).putShort((24*90+(variant==9?28:25))*2,(short)5);h.write(0x4085b8,map);}
            JsonObject j=new JsonObject();j.addProperty("spell_id",id);j.addProperty("seed_before",seed);j.addProperty("power",power);j.addProperty("variant",variant);j.addProperty("combat",id!=1103);j.addProperty("caster",5);j.addProperty("target",variant==9?3:0);j.add("before",snap(h,r));
            h.cpu.writeRegister("SP",0x6ffe00);h.jump(0x10d2b4);execute(h,r,0x10d3c2);j.add("charged",snap(h,r));
            if(id==1103){h.cpu.writeRegister("SP",0x6ffe00);h.jump(0x10aca0);execute(h,r,0x10adb8);j.add("targeted",snap(h,r));h.jump(0x10adc8);execute(h,r,0x10adde);}
            else{if(id==1101){h.value(0x6ffd00-0x16,2,variant==9?3:0);h.jump(0x11b30a);execute(h,r,0x11b332);h.jump(0x11b33c);execute(h,r,0x11b342);}else{h.value(0x44719e,2,20);h.value(0x4471a0,2,20);h.jump(0x11b164);execute(h,r,0x11b1be);}j.add("targeted",snap(h,r));h.cpu.writeRegister("SP",0x6ffe00);h.jump(0x11af0a);execute(h,r,0x11af72);}
            j.add("after",snap(h,r));j.add("casts",JsonParser.parseString("["+calls+"]"));j.add("resolutions",JsonParser.parseString("["+resolutions+"]"));j.add("healing_calls",healingCalls);j.add("rng",JsonParser.parseString("["+r.drawRecords+"]"));j.add("sounds",JsonParser.parseString("["+r.sounds+"]"));output.add(j);
        }finally{h.cpu.dispose();}
    }
    void admission(JsonArray output,int id,boolean combat,boolean sufficient)throws Exception {
        reset(1);ActionOracleHarness h=frame();RunResult r=new RunResult(1);try{byte[] w=controlledParty();ByteBuffer.wrap(w).putShort(5*3138+0x58,(short)(packet(id)[30]-(sufficient?0:1)));initialize(h,w,monster(13,10000),terrain(),combat,5);select(h,id,1);JsonObject j=new JsonObject();j.addProperty("spell_id",id);j.addProperty("combat",combat);j.addProperty("sufficient",sufficient);j.add("before",snap(h,r));h.jump(0x10d1fe);for(int i=0;i<12;i++){if(h.pc()==0x10d216||h.pc()==0x10d246||h.pc()==0x10d25e)break;if(!h.cpu.step(monitor))throw new Exception("Mode admission");}j.addProperty("mode_allowed",h.pc()==0x10d25e);j.addProperty("mode_stop",Long.toHexString(h.pc()));h.jump(0x10cc02);execute(h,r,0x10cc36);if(!h.cpu.step(monitor))throw new Exception("Affordability");j.addProperty("affordable",h.pc()==0x10cc3a);j.add("after",snap(h,r));output.add(j);}finally{h.cpu.dispose();}
    }
    void attack(JsonArray output,int weapon,long seed,int buff)throws Exception {
        reset(seed);ActionOracleHarness h=frame();RunResult r=new RunResult(seed);try{byte[] w=controlledParty(),m=monster(29,10000);for(int i=0;i<6;i++)ByteBuffer.wrap(w).putShort(i*3138+0x50,(short)10000).putShort(i*3138+0x52,(short)10000);ByteBuffer.wrap(m).putShort(0x4e,(short)weapon).putShort(0x28,(short)weapon);initialize(h,w,m,terrain(),true,0);h.value(0x447196,2,0);h.value(0x442566,2,buff);h.enter(0x130d38,0x6ffe00,0x900000);h.value(0x6ffe04,2,weapon);execute(h,r,0x900000);JsonObject j=new JsonObject();j.addProperty("weapon",weapon);j.addProperty("buff",buff);j.addProperty("seed_before",seed);j.add("before",snap(h,r));h.enter(0x101eb6,0x6ffe00,0x900000);h.value(0x6ffe04,2,10);h.value(0x6ffe06,2,0);h.value(0x6ffe08,2,0);execute(h,r,0x900000);j.add("after",snap(h,r));j.add("rng",JsonParser.parseString("["+r.drawRecords+"]"));j.add("sounds",JsonParser.parseString("["+r.sounds+"]"));output.add(j);}finally{h.cpu.dispose();}
    }
    void tick(JsonArray output,int buff)throws Exception {reset(1);ActionOracleHarness h=frame();RunResult r=new RunResult(1);try{initialize(h,controlledParty(),monster(13,10000),terrain(),false,5);for(int i=0;i<10;i++)h.value(0x442562+i*2,2,i==2?buff:i==0?2:i%2==0?-1:1);JsonObject j=new JsonObject();j.addProperty("buff",buff);j.add("before",snap(h,r));h.jump(0x14dede);execute(h,r,0x14df0e);j.add("after",snap(h,r));output.add(j);}finally{h.cpu.dispose();}}
    void mitigate(JsonArray output,int buff)throws Exception {reset(1);ActionOracleHarness h=frame();RunResult r=new RunResult(1);try{initialize(h,controlledParty(),monster(13,10000),terrain(),true,5);h.value(0x442566,2,buff);h.value(0x4434b8,2,buff==0?10:buff==1?1:buff==20?6:5);h.cpu.writeRegister("D3",0);JsonObject j=new JsonObject();j.addProperty("buff",buff);j.add("before",snap(h,r));h.jump(0x103c14);execute(h,r,0x103c50);j.add("after",snap(h,r));j.add("sounds",JsonParser.parseString("["+r.sounds+"]"));output.add(j);}finally{h.cpu.dispose();}}
    void geometry(JsonArray output,int id,int distance,boolean blocked)throws Exception {
        reset(1);ActionOracleHarness h=frame();RunResult r=new RunResult(1);try{byte[] map=terrain();initialize(h,controlledParty(),monster(13,10000),map,true,0);h.value(0x44709a,2,20);h.value(0x44709c,2,20);if(blocked){ByteBuffer.wrap(map).putShort(((25+Math.max(1,distance/2))*90+25)*2,(short)1000);h.write(0x4085b8,map);}select(h,id,1);JsonObject j=new JsonObject();j.addProperty("spell_id",id);j.addProperty("distance",distance);j.addProperty("blocked",blocked);j.add("before",snap(h,r));h.jump(0x119238);execute(h,r,0x11925e);j.addProperty("range",h.word(0x4435bc));h.enter(0x11235e,0x6ffe00,0x900000);h.value(0x6ffe04,2,0);h.value(0x6ffe06,2,20+distance);h.value(0x6ffe08,2,20);execute(h,r,0x900000);j.addProperty("in_range",(h.cpu.readRegister("D0").intValue()&65535)!=0);h.cpu.writeRegister("SP",0x6ffe00);h.cpu.writeRegister("A6",0x6ffd00);h.value(0x44719e,2,20+distance);h.value(0x4471a0,2,20);h.jump(0x11b090);execute(h,r,blocked&&id==1305?0x11b0e0:0x11b0f0);j.addProperty("sight_allowed",h.pc()==0x11b0f0);j.add("after",snap(h,r));output.add(j);}finally{h.cpu.dispose();}
    }

    void electric(JsonArray output,long seed,int ward)throws Exception {
        reset(seed);ActionOracleHarness h=frame();RunResult r=new RunResult(seed);try{byte[] w=controlledParty();ByteBuffer.wrap(w).putShort(0x50,(short)10000).putShort(0x52,(short)10000).putShort(0x76,(short)ward);initialize(h,w,monster(13,10000),terrain(),true,5);h.enter(0x130d38,0x6ffe00,0x900000);h.value(0x6ffe04,2,9);execute(h,r,0x900000);h.enter(0x130e54,0x6ffe00,0x900000);h.value(0x6ffe04,2,0);execute(h,r,0x900000);h.enter(0x14c6c2,0x6ffe00,0x900000);h.value(0x6ffe04,2,10);h.value(0x6ffe06,2,0);execute(h,r,0x900000);h.value(0x435662,1,0);JsonObject j=new JsonObject();j.addProperty("ward",ward);j.addProperty("seed_before",seed);j.addProperty("weapon",9);j.addProperty("weapon_work",hex(h.read(0x453768,150)));j.addProperty("ammo_work",hex(h.read(0x4536d2,150)));j.add("before",snap(h,r));h.enter(0x15075a,0x6ffe00,0x900000);h.value(0x6ffe04,2,0);h.value(0x6ffe06,2,1);h.value(0x6ffe08,2,1);h.value(0x6ffe0a,2,1);h.value(0x6ffe0c,2,0);execute(h,r,0x900000);j.addProperty("damage",(short)h.cpu.readRegister("D0").intValue());j.add("after",snap(h,r));j.add("rng",JsonParser.parseString("["+r.drawRecords+"]"));j.add("sounds",JsonParser.parseString("["+r.sounds+"]"));output.add(j);}finally{h.cpu.dispose();}
    }
}
