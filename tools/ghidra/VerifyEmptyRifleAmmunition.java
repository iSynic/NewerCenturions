import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import com.google.gson.*;

/** Original empty rifle wear, selected ammunition loading and native refit. */
public class VerifyEmptyRifleAmmunition extends VerifyRequiredPlayerGear {
    JsonObject[] inputs=new JsonObject[2];JsonArray orderedCalls=new JsonArray(),guiSelections=new JsonArray(),procedureReturns=new JsonArray();boolean selectAmmo=true;
    @Override void observe(ActionOracleHarness h,RunResult r,long pc)throws Exception {
        super.observe(h,r,pc);if(pc==0x130ae6)procedureReturns.add((short)h.cpu.readRegister("D0").intValue());String name=null;int count=1;
        if(pc==0x164a0e){name="wear";count=3;}else if(pc==0x14ef76){name="removeitem";count=4;}else if(pc==0x1300ec){name="loadammoproc";count=4;}else if(pc==0x14e5e4){name="refit";count=4;}else if(pc==0x130afe)name="loaditem";else if(pc==0x130d38)name="loadweapon";else if(pc==0x130e54)name="loadammo";else if(pc==0x139192)name="calcw";else if(pc==0x136994)name="movecalc";else if(pc==0x12006e)name="compactitems";
        if(name!=null){JsonObject j=new JsonObject();j.addProperty("function",name);JsonArray a=new JsonArray();for(int i=0;i<count;i++)a.add(h.word(h.sp()+4+i*2));j.add("arguments",a);orderedCalls.add(j);}
    }
    JsonObject snap(ActionOracleHarness h,RunResult r){JsonObject j=snapshot(h,r);j.addProperty("identity",hex(h.read(0x4435c6,6)));j.addProperty("attack_count",h.word(0x4435cc));j.addProperty("item_context",h.word(0x45365c));j.addProperty("active_member",h.word(0x4435d2));j.addProperty("ammo_changed",h.read(0x437a6c,1)[0]&255);j.addProperty("fields",hex(h.read(0x442bd6,1200)));j.addProperty("used",hex(h.read(0x435884,128)));j.addProperty("map",hex(h.read(0x4085b8,16200)));j.addProperty("party_buffs",hex(h.read(0x442562,20)));j.add("ordered_calls",orderedCalls.deepCopy());j.add("gui_selections",guiSelections.deepCopy());j.add("procedure_returns",procedureReturns.deepCopy());return j;}
    public void run()throws Exception {
        Path root=Path.of(getScriptArgs()[0]),original=root.resolve("NewCenturions/TheNewCenturions"),scenario=original.resolve("Scenarios/The Thunder of Guns/0");
        items=Files.readAllBytes(original.resolve("Data Files/Data ID"));scenarioItems=Files.readAllBytes(scenario.resolve("Data NI"));races=Files.readAllBytes(original.resolve("Data Files/Data Race"));mos=Files.readAllBytes(original.resolve("Data Files/Data MOS"));monsters=Files.readAllBytes(scenario.resolve("Data MD"));properties=Files.readAllBytes(original.resolve("Data Files/Data Swamp BD"));attacks=Files.readAllBytes(original.resolve("Data Files/Data S"));areas=Files.readAllBytes(original.resolve("Data Files/Data AD"));party=new byte[18828];String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};for(int i=0;i<6;i++)System.arraycopy(Files.readAllBytes(original.resolve("Warriors/"+names[i])),0,party,i*3138,3138);
        JsonArray foundations=new JsonArray();JsonObject prior=Files.exists(root.resolve("evidence/fixtures/empty-rifle-ammunition.json"))?JsonParser.parseString(Files.readString(root.resolve("evidence/fixtures/empty-rifle-ammunition.json"))).getAsJsonObject():null;String[] paths={"analysis/completion/empty-rifle-laboratory-12-input.json","analysis/completion/empty-rifle-laboratory-1-input.json"};for(int i=0;i<2;i++){JsonObject j;if(Files.exists(root.resolve(paths[i]))){j=new JsonObject();j.addProperty("input",paths[i]);j.addProperty("digest",ActionOracleHarness.sha256(root.resolve(paths[i])));inputs[i]=JsonParser.parseString(Files.readString(root.resolve(paths[i]))).getAsJsonObject().getAsJsonObject("session");j.add("session",inputs[i]);}else{j=prior.getAsJsonArray("foundations").get(i).getAsJsonObject();inputs[i]=j.getAsJsonObject("session");}foundations.add(j);}
        JsonArray wear=new JsonArray(),manual=new JsonArray(),refit=new JsonArray();for(long seed:new long[]{12,1})for(int combat:new int[]{0,1})probe(wear,"wear",0,seed,combat,12);for(int source:new int[]{0,1})for(long seed:new long[]{12,1})for(int combat:new int[]{0,1})for(int reserve:new int[]{12,2,0})probe(manual,"manual",source,seed,combat,reserve);for(long seed:new long[]{12,1})for(int combat:new int[]{0,1})for(int reserve:new int[]{12,2,0})probe(refit,"refit",0,seed,combat,reserve);
        JsonObject identity=new JsonObject();String[][] definitions={{"probe","tools/ghidra/VerifyEmptyRifleAmmunition.java"},{"gear_probe","tools/ghidra/VerifyRequiredPlayerGear.java"},{"caller_probe","tools/ghidra/VerifyEnemySpellCaller.java"},{"spell_probe","tools/ghidra/VerifyEnemySpells.java"},{"parent_probe","tools/ghidra/VerifyAllyTurns.java"},{"inventory_probe","tools/ghidra/VerifyStoryInventory.java"},{"harness","tools/ghidra/ActionOracleHarness.java"},{"sane_boundary","tools/ghidra/SaneBoundary.java"},{"audio_boundary","tools/ghidra/ResultAudioBoundary.java"},{"mapped_binary","analysis/ghidra/startup/code1-mapped.bin"},{"monsters","NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data MD"},{"items","NewCenturions/TheNewCenturions/Data Files/Data ID"},{"scenario_items","NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data NI"},{"races","NewCenturions/TheNewCenturions/Data Files/Data Race"},{"mos","NewCenturions/TheNewCenturions/Data Files/Data MOS"},{"properties","NewCenturions/TheNewCenturions/Data Files/Data Swamp BD"},{"attacks","NewCenturions/TheNewCenturions/Data Files/Data S"},{"areas","NewCenturions/TheNewCenturions/Data Files/Data AD"}};for(String[] d:definitions)identity.addProperty(d[0]+"_sha256",ActionOracleHarness.sha256(root.resolve(d[1])));for(int i=0;i<6;i++)identity.addProperty("warrior"+i+"_sha256",ActionOracleHarness.sha256(original.resolve("Warriors/"+names[i])));
        JsonObject out=new JsonObject();out.addProperty("authority","Original complete wear164a0e of actual empty38 record from freshlaboratoryseed12 and original selected ammunition Inventory12bfb8..12c004/combat117d00..117d5c callers invoking complete loadammoproc1300ec with original compatible-stock scan and accepted GUI203selection input or cancellation when203stockzero. Fullnative mutations/return incl cachedwarrior126/itemammo/loaded/reserve/AP and ordered RNG/audio. Separate complete refit14e5e4 empty-ammo fallback controls, not assumed equivalent to selected manual loading. Actual two route foundations embedded, six completewarriors/crate/items/reserves/work/packet/fields/map captured; controlled AP5 and native pickup/compact source0, positiveHPdummyMD13/terrain supplied outside explored gameplay. GUIlabel/menu-creation/display skipped at explicit boundaries, candidate scan and ruleeffects original; original profile files and inherited SANE/BitTst/presentation boundaries. No live-game or campaignsurvival claim.");out.add("identity",identity);out.add("foundations",foundations);out.add("wear_cases",wear);out.add("manual_cases",manual);out.add("refit_cases",refit);Files.writeString(root.resolve("evidence/fixtures/empty-rifle-ammunition.json"),out+"\n");println("Empty rifle ammunition exported:4wear24manual12refit");
    }
    void probe(JsonArray output,String kind,int source,long seed,int combat,int reserve)throws Exception {
        ActionOracleHarness h=frame();RunResult r=new RunResult(seed);try{byte[] p=warriors(inputs[source]),c=crate(inputs[source]);setup(h,p,c,0,0,r);orderedCalls=new JsonArray();guiSelections=new JsonArray();procedureReturns=new JsonArray();selectAmmo=reserve>0;h.write(0x442562,new byte[20]);h.write(0x4435c6,new byte[6]);h.write(0x442798,new byte[24]);h.value(0x4435cc,2,0);h.value(0x437a6c,1,0);h.value(0x448aa0+0x40,2,5);JsonArray stock=inputs[source].getAsJsonArray("ammunition");for(int i=0;i<200;i++)h.value(0x42dbe4+i*4,4,stock.get(i).getAsLong());h.value(0x42dbe4+12,4,reserve);
            JsonObject j=new JsonObject();j.addProperty("kind",kind);j.addProperty("source",source);j.addProperty("seed_before",seed);j.addProperty("combat",combat);j.addProperty("reserve203",reserve);j.add("before",snap(h,r));
            if(source==0){int cr=-1;for(int i=0;i<1000;i++)if(ByteBuffer.wrap(c).getShort(i*22)==38){cr=i;break;}if(cr<0)throw new Exception("No empty rifle");h.write(0x41fc74,c);if(pickup(h,r,0,cr)==0)throw new Exception("Empty rifle pickup failed");h.cpu.writeRegister("A6",0x700000);h.value(0x70000a,2,1);h.jump(0x10800e);execute(h,r,0x10808c);j.add("picked",snap(h,r));int slot=locate(h,0,38);selected(h,0,slot);h.value(0x438776,2,combat);j.addProperty("wear_return",call(h,r,0x164a0e,0,slot,1));j.add("worn",snap(h,r));call(h,r,0x12006e,0);j.add("compacted",snap(h,r));}else h.value(0x438776,2,combat);
            int slot=locate(h,0,38);j.addProperty("inventory_slot",slot);selected(h,0,slot);j.add("operation_before",snap(h,r));
            if(kind.equals("manual")){h.cpu.writeRegister("SP",0x6ffe00);h.cpu.writeRegister("A6",0x700000);h.value(0x44719c,2,slot);h.value(0x4471aa,2,0);if(combat==0){h.jump(0x12bfb8);execute(h,r,0x12c004);}else{h.jump(0x117cd4);execute(h,r,0x117cde);h.jump(0x117d00);execute(h,r,0x117d5c);}j.addProperty("returned_d0",(short)h.cpu.readRegister("D0").intValue());}
            if(kind.equals("refit"))j.addProperty("refit_return",call(h,r,0x14e5e4,0,0,1,1));
            j.add("after",snap(h,r));output.add(j);println("Empty rifle "+kind+" source"+source+" seed"+seed+" combat"+combat+" reserve"+reserve);
        }finally{h.cpu.dispose();}
    }
    void execute(ActionOracleHarness h,RunResult result,long stop) throws Exception {
        SaneBoundary sane=new SaneBoundary(h.cpu,currentProgram);boolean done=false;
        for(int step=0;step<1500000;step++) {
            long pc=h.pc(),sp=h.sp();
            if(pc==0x1363e4&&h.word(sp+4)==0){diagnostic=true;diagnosticSpell=(h.word(0x4435c6)+1)*1000+(h.word(0x4435c8)+1)*100+h.word(0x4435ca)+1;done=true;break;}
            observe(h,result,pc);if(pc==stop||secondaryStop!=0&&pc==secondaryStop){done=true;break;}
            if(pc==0x130114){h.jump(0x13015e);continue;}
            if(pc==0x130192||pc==0x13023a||pc==0x130296){h.jump(0x1302d6);continue;}
            if(pc==0x13033a){h.jump(0x13040c);continue;}
            if(pc==0x130424){h.jump(0x130434);continue;}
            if(pc==0x13043e){h.jump(0x1305ca);continue;}
            if(pc==0x13071c){long base=h.cpu.readRegister("A6").longValue();int count=h.word(base-0x266),choice=0;JsonArray candidates=new JsonArray();for(int i=0;i<count;i++){int candidate=h.word(base-0x264+i*2);candidates.add(candidate);if(candidate==3&&selectAmmo)choice=i+2;}JsonObject selected=new JsonObject();selected.add("candidates",candidates);selected.addProperty("popup_result",choice);guiSelections.add(selected);h.cpu.writeRegister("D6",choice);h.jump(0x13075c);continue;}
            if(pc==0x130a8a){h.jump(0x130aaa);continue;}
            if(pc==0x153720){h.ret();continue;}
            if(pc==0x13ba6c||pc==0x13bac6){profileFile=pc==0x13ba6c?0:1;profileCursor=0;h.value(0x4434a6,4,0x600000+profileFile);h.ret();continue;}
            if(pc==0x400078){profileCursor=(int)h.pointer(sp+8);h.cpu.writeRegister("D0",0);h.ret();continue;}
            if(pc==0x400048){int amount=(int)(h.pointer(sp+8)*h.pointer(sp+12));byte[] source=profileFile==0?races:mos;h.write(h.pointer(sp+4),Arrays.copyOfRange(source,profileCursor,profileCursor+amount));profileCursor+=amount;h.cpu.writeRegister("D0",h.pointer(sp+12));h.ret();continue;}
            if(pc==0x400068){h.cpu.writeRegister("D0",0);h.ret();continue;}
            if(pc==0x400090||pc==0x400098||pc==0x400168||pc==0x400180){h.cpu.writeRegister("D0",0);h.ret();continue;}
            if(pc==0x16474a){warnings.add(h.word(sp+4));h.ret();continue;}
            if(pc==0x13b56c||pc==0x15f4b8||pc==0x160d08){h.ret();continue;}

            if(pc==0x4001a0){sane.extended(h.pointer(sp+4),Math.sqrt(sane.extended(sp+8)));h.ret();continue;}
            if(pc==0x400038){h.cpu.writeRegister("D0",Math.abs(h.word(sp+4)));h.ret();continue;}
            if(pc==0x153ca4){rngSites.add(Long.toHexString(h.pointer(sp)));result.rng(h);continue;}
            if(pc==0x133a18||pc==0x16474a){result.sound(h);continue;}
            if(pc==0x123d1c){int sound=h.word(sp+14);if(sound!=0){if(result.sounds.length()>0)result.sounds.append(',');result.sounds.append(sound);}h.ret();continue;}
            if(pc==0x15b2f6||pc==0x15a1f6){h.ret();continue;}
            if(pc==0x159192){h.value(0x44719a,2,0);h.cpu.writeRegister("D0",0);h.ret();continue;}
            if(pc==0x158454){showresultsSound(h,result);h.ret();continue;}
            if(pc==0x120c14||pc==0x105572||pc==0x10e6b6||pc==0x133d46||pc==0x13154c||pc==0x11f1a8||pc==0x11edc4||pc==0x14a02c||pc==0x13a2c4||pc==0x1205f8||pc==0x16043e||pc==0x1382a8||pc==0x11f136||pc==0x1537ee||pc==0x12382a||pc==0x160fb2||pc==0x10ff54||pc==0x12f040||pc==0x15edfa){h.ret();continue;}
            int op=h.word(pc)&65535;
            if(op==0xa85d){long bit=h.pointer(sp),address=h.pointer(sp+4);int value=h.read(address+(bit>>>3),1)[0]&255;h.value(sp+8,1,(value&(0x80>>>(bit&7)))!=0?1:0);h.pop(8);h.jump(pc+2);continue;}
            if(op==0xa858){h.pop(8);h.value(h.sp(),4,0);h.jump(pc+2);continue;}
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
