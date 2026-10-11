import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import com.google.gson.*;

/** Original earned firearm transfer, fitting and accepted firing caller. */
public class VerifyOwnedPlayerFirearm extends VerifyEmptyRifleAdvance {
    JsonObject foundation; JsonArray nativeCalls=new JsonArray(),milestones=new JsonArray();
    void initialize(ActionOracleHarness h,RunResult r,int member) {
        setup(h,warriors(foundation),crate(foundation),member,0,r);
        orderedCalls=new JsonArray();guiSelections=new JsonArray();procedureReturns=new JsonArray();nativeCalls=new JsonArray();milestones=new JsonArray();selectAmmo=true;
        JsonArray stock=foundation.getAsJsonArray("ammunition");for(int i=0;i<200;i++)h.value(0x42dbe4+i*4,4,stock.get(i).getAsLong());
        h.write(0x442562,array(foundation.getAsJsonArray("party_buffs")));h.write(0x43788a,array(foundation.getAsJsonArray("psi_memory")));
        h.write(0x44374e,array(foundation.getAsJsonArray("attack_packet")));JsonArray ident=foundation.getAsJsonArray("attack_identity");for(int i=0;i<3;i++)h.value(0x4435c6+i*2,2,ident.get(i).getAsInt());
        h.value(0x4435cc,2,foundation.get("attack_count").getAsInt());h.value(0x45365c,2,foundation.get("loaded_item_field32").getAsInt());
        JsonObject work=foundation.getAsJsonObject("weapon_work");h.value(0x453768+14,2,work.get("sound").getAsInt());h.value(0x453768+136,2,work.get("weapon_armor_filter").getAsInt());h.value(0x4536d2+136,2,work.get("ammunition_armor_filter").getAsInt());
        h.value(0x438778,2,0);h.value(0x438760,2,0);h.value(0x438784,2,0);h.value(0x438766,2,0);h.value(0x437a6c,1,0);h.value(0x443318,2,member);h.value(0x44374c,2,0);
    }
    @Override void observe(ActionOracleHarness h,RunResult r,long pc)throws Exception {
        super.observe(h,r,pc);int argc=0;String name=null;
        if(pc==0x157586){name="canuse";argc=3;}else if(pc==0x12c874){name="installitem";argc=2;}else if(pc==0x14c6c2){name="prepattack";argc=2;}else if(pc==0x14edd0){name="doreload";argc=1;}else if(pc==0x153ca4){name="Rand";argc=1;}else if(pc==0x133a18){name="Sound";argc=1;}else if(pc==0x12382a){name="button_disabled";argc=2;}else if(pc==0x125a3a){name="getup";argc=1;}else if(pc==0x15075a){name="resolvedamage";argc=5;}else if(pc==0x100af0){name="playerattack";argc=2;}else if(pc==0x13145c){name="loadspell2";argc=3;}
        if(name!=null){JsonObject j=new JsonObject();j.addProperty("name",name);j.addProperty("address",Long.toHexString(pc));j.addProperty("return",Long.toHexString(h.pointer(h.sp())));JsonArray a=new JsonArray();for(int i=0;i<argc;i++)a.add(h.word(h.sp()+4+i*2));j.add("arguments",a);j.addProperty("draws",r.draws);nativeCalls.add(j);}
        if(pc==0x14c868||pc==0x14c8a2||pc==0x11b488||pc==0x11af72){JsonObject j=new JsonObject();j.addProperty("address",Long.toHexString(pc));j.addProperty("packet",hex(h.read(0x44374e,58)));j.addProperty("draws",r.draws);milestones.add(j);}
    }
    JsonObject ownedSnap(ActionOracleHarness h,RunResult r){JsonObject j=advanceSnap(h,r);j.addProperty("selected_weapon",h.word(0x44719a));j.addProperty("spell_semaphore",h.word(0x438760));j.addProperty("health_context",h.read(0x435662,1)[0]&255);j.addProperty("combat",h.word(0x438776));j.add("native_calls",nativeCalls.deepCopy());j.add("milestones",milestones.deepCopy());return j;}
    int ordinarySlot(ActionOracleHarness h){for(int i=0;i<1000;i++){byte[] b=h.read(0x4285f4+i*22,22);if(hex(b).equals("000200000000000c00000f0000010000000000000000"))return i;}throw new IllegalStateException("No earned ordinary item 2 remains");}
    int firearmSlot(ActionOracleHarness h,int member){for(int i=0;i<h.word(0x448aa0+member*3138+26);i++){long a=0x448aa0+member*3138+0x132+i*22;if(h.word(a)==2&&h.read(a+9,1)[0]==1&&h.word(a+12)==1)return i;}throw new IllegalStateException("No equipped ordinary firearm");}
    void selectMember(ActionOracleHarness h,int member,int slot){h.value(0x44273f,1,member);h.value(0x4435d2,2,member);h.value(0x443318,2,member);h.value(0x4470ba,2,member);h.value(0x44719c,2,slot);h.value(0x4471aa,2,0);selected(h,member,slot);}
    void closeCrate(ActionOracleHarness h,RunResult r)throws Exception{h.cpu.writeRegister("A6",0x700000);h.cpu.writeRegister("SP",0x6ffe00);h.value(0x70000a,2,1);h.jump(0x10800e);execute(h,r,0x10808c);}
    void takeEquip(ActionOracleHarness h,RunResult r,int member,JsonArray stages)throws Exception{
        int cr=ordinarySlot(h);JsonObject j=new JsonObject();j.addProperty("member",member);j.addProperty("crate_slot",cr);j.add("before",ownedSnap(h,r));h.write(0x41fc74,h.read(0x4285f4,22000));int slot=h.word(0x448aa0+member*3138+26);if(pickup(h,r,member,cr)!=1)throw new Exception("Earned pickup rejected");j.add("picked",ownedSnap(h,r));closeCrate(h,r);selectMember(h,member,slot);j.addProperty("wear_return",call(h,r,0x164a0e,member,slot,1));j.add("worn",ownedSnap(h,r));call(h,r,0x12006e,member);slot=firearmSlot(h,member);selectMember(h,member,slot);j.addProperty("inventory_slot",slot);j.add("compacted",ownedSnap(h,r));h.cpu.writeRegister("A6",0x700000);h.cpu.writeRegister("SP",0x6ffe00);h.jump(0x12bfb8);execute(h,r,0x12c004);j.add("fitted",ownedSnap(h,r));stages.add(j);
    }
    void combatControls(ActionOracleHarness h,int member,int ap,int movement){
        h.value(0x438776,2,1);h.value(0x448aa0+member*3138+0x40,2,ap);h.value(0x448aa0+member*3138+0x3c,2,movement);byte[] order=new byte[220];for(int i=0;i<110;i++)ByteBuffer.wrap(order).putShort(i*2,(short)-1);ByteBuffer.wrap(order).putShort(2,(short)member).putShort(4,(short)10);h.write(0x4470b8,order);h.value(0x447194,2,1);h.value(0x447196,2,0);h.value(0x44255c,1,1);h.value(0x44374c,2,0);h.value(0x43800c,2,0);h.value(0x43958e,2,0);h.value(0x442739,1,0);h.value(0x44273a,1,6);h.value(0x438778,2,0);h.value(0x435884,1,1);h.value(0x44374e+19,1,8);h.value(0x446f0a,2,20);h.value(0x446f0c,2,20);byte[] target=h.read(0x439d3c,288),map=terrain();ByteBuffer.wrap(target).putShort(14,(short)1);place(h,map,target,10,20,20);h.write(0x439d3c,target);h.write(0x4085b8,map);
    }
    void gates(JsonArray out)throws Exception{for(int member=0;member<6;member++){ActionOracleHarness h=frame();RunResult r=new RunResult(12);try{initialize(h,r,member);JsonObject j=new JsonObject();j.addProperty("member",member);j.add("before",ownedSnap(h,r));j.addProperty("allowed",call(h,r,0x157586,2,h.word(0x448aa0+member*3138+50),h.word(0x448aa0+member*3138+52)));j.add("after",ownedSnap(h,r));out.add(j);}finally{h.cpu.dispose();}}}
    void pickupControl(JsonArray out,int variant)throws Exception{ActionOracleHarness h=frame();RunResult r=new RunResult(12);try{initialize(h,r,2);int cr=ordinarySlot(h);if(variant==1)h.value(0x448aa0+2*3138+0x9d8,2,0);else h.value(0x448aa0+2*3138+26,2,99);h.write(0x41fc74,h.read(0x4285f4,22000));JsonObject j=new JsonObject();j.addProperty("kind",variant==1?"pickup_capacity_zero":"pickup_count_99");j.add("before",ownedSnap(h,r));j.addProperty("returned",pickup(h,r,2,cr));j.add("after",ownedSnap(h,r));out.add(j);}finally{h.cpu.dispose();}}
    void admissionControl(JsonArray out,String kind)throws Exception{ActionOracleHarness h=frame();RunResult r=new RunResult(12);try{initialize(h,r,2);JsonArray stages=new JsonArray();takeEquip(h,r,2,stages);int slot=firearmSlot(h,2);selectMember(h,2,slot);combatControls(h,2,kind.equals("fire_movement_zero")?3:0,kind.equals("fire_movement_zero")?0:5);if(kind.contains("inactive")){h.value(0x4470ba,2,3);h.value(0x448aa0+2*3138+0x40,2,3);h.value(0x4435d2,2,3);}JsonObject j=new JsonObject();j.addProperty("kind",kind);j.add("equipment",stages);j.add("before",ownedSnap(h,r));h.cpu.writeRegister("A6",0x700000);h.cpu.writeRegister("SP",0x6ffe00);
        if(kind.equals("items_wear_ap_zero")||kind.equals("items_fit_ap_zero")){h.value(0x6fffe4,4,0x6fd000);h.value(0x6fd000,4,0x6fd100);h.write(0x6fd100,new byte[32]);h.jump(kind.equals("items_wear_ap_zero")?0x12c36a:0x12beb4);execute(h,r,0x12bf30);}
        else if(kind.equals("wear_inactive"))j.addProperty("returned",call(h,r,0x164a0e,2,slot,1));
        else if(kind.equals("items_fit_inactive")){h.jump(0x12beb4);execute(h,r,0x12ba98);}
        else{h.jump(0x117e5e);execute(h,r,0x117eaa);}j.add("after",ownedSnap(h,r));out.add(j);}finally{h.cpu.dispose();}}
    void ammunitionControl(JsonArray out,String kind,long seed)throws Exception{ActionOracleHarness h=frame();RunResult r=new RunResult(seed);try{initialize(h,r,2);JsonArray stages=new JsonArray();takeEquip(h,r,2,stages);int slot=firearmSlot(h,2);selectMember(h,2,slot);long record=0x448aa0+2*3138+0x132+slot*22;if(!kind.equals("manual_full")){h.value(record+2,2,0);h.value(record+4,2,0);h.value(0x448aa0+2*3138+0x126,2,0);}if(kind.equals("manual_partial")||kind.equals("reload_partial"))h.value(0x42dbe4+4,4,2);if(kind.equals("manual_missing_201")){h.value(0x42dbe4+4,4,0);selectAmmo=false;}if(kind.equals("manual_wrong_caliber")||kind.equals("reload_wrong_caliber")){h.value(0x42dbe4+4,4,0);h.value(0x42dbe4+19*4,4,0);selectAmmo=false;}
        if(kind.startsWith("reload")){h.value(record+4,2,201);h.value(0x448aa0+2*3138+0x126,2,201);}selected(h,2,slot);JsonObject j=new JsonObject();j.addProperty("kind",kind);j.addProperty("seed_before",seed);j.add("equipment",stages);j.add("before",ownedSnap(h,r));
        if(kind.startsWith("reload"))j.addProperty("returned",call(h,r,0x14edd0,2));else if(kind.equals("prepare_empty"))j.addProperty("returned",call(h,r,0x14c6c2,2,0));else{h.cpu.writeRegister("A6",0x700000);h.cpu.writeRegister("SP",0x6ffe00);h.jump(0x12bfb8);execute(h,r,0x12c004);}j.add("after",ownedSnap(h,r));out.add(j);}finally{h.cpu.dispose();}}
    void combatFit(JsonArray out,int reserve)throws Exception{ActionOracleHarness h=frame();RunResult r=new RunResult(12);try{initialize(h,r,2);JsonArray stages=new JsonArray();takeEquip(h,r,2,stages);int slot=firearmSlot(h,2);selectMember(h,2,slot);h.value(0x448aa0+2*3138+0x134+slot*22,2,0);h.value(0x42dbe4+4,4,reserve);combatControls(h,2,2,5);JsonObject j=new JsonObject();j.addProperty("kind","combat_fit_and_advance");j.addProperty("reserve201",reserve);j.add("equipment",stages);j.add("before",ownedSnap(h,r));h.enter(0x117be2,0x6ffe00,0x116b6a);execute(h,r,0x117c46);h.jump(0x117cd4);execute(h,r,0x117cde);h.jump(0x117cf6);execute(h,r,0x117d5c);j.add("fitted",ownedSnap(h,r));execute(h,r,0x11ce76);advance(h,r,0x127090);j.add("after",ownedSnap(h,r));out.add(j);}finally{h.cpu.dispose();}}
    void composed(JsonArray out,boolean fire,long seed,int member)throws Exception{ActionOracleHarness h=frame();RunResult r=new RunResult(seed);try{initialize(h,r,2);JsonObject j=new JsonObject();j.addProperty("seed_before",seed);j.addProperty("member",member);j.add("before",ownedSnap(h,r));JsonArray stages=new JsonArray();takeEquip(h,r,2,stages);takeEquip(h,r,3,stages);j.add("equipment",stages);if(fire){int slot=firearmSlot(h,member);selectMember(h,member,slot);combatControls(h,member,3,5);h.value(0x4435cc,2,1);h.value(0x4435c0,2,0);h.value(0x4435be,2,1);j.add("combat_before",ownedSnap(h,r));call(h,r,0x14c6c2,member,0);j.add("prepared",ownedSnap(h,r));h.cpu.writeRegister("A6",0x700000);h.cpu.writeRegister("SP",0x6ffe00);h.value(0x6fffea,2,10);h.value(0x43565e,1,1);h.jump(0x11b3e2);execute(h,r,0x11b488);j.add("targeted",ownedSnap(h,r));h.jump(0x11af0a);execute(h,r,0x11af72);j.add("fired",ownedSnap(h,r));j.add("casts",JsonParser.parseString("["+calls+"]"));j.add("resolutions",JsonParser.parseString("["+resolutions+"]"));h.jump(0x116b6a);h.cpu.writeRegister("A6",0x700000);h.cpu.writeRegister("SP",0x6ffe00);advance(h,r,0x127090);j.add("advanced",ownedSnap(h,r));}j.add("after",ownedSnap(h,r));out.add(j);println("Owned firearm composition seed"+seed+" member"+member+" fire"+fire);}finally{h.cpu.dispose();}}
    public void run()throws Exception {
        Path root=Path.of(getScriptArgs()[0]),original=root.resolve("NewCenturions/TheNewCenturions"),scenario=original.resolve("Scenarios/The Thunder of Guns/0");items=Files.readAllBytes(original.resolve("Data Files/Data ID"));scenarioItems=Files.readAllBytes(scenario.resolve("Data NI"));races=Files.readAllBytes(original.resolve("Data Files/Data Race"));mos=Files.readAllBytes(original.resolve("Data Files/Data MOS"));monsters=Files.readAllBytes(scenario.resolve("Data MD"));properties=Files.readAllBytes(original.resolve("Data Files/Data Swamp BD"));attacks=Files.readAllBytes(original.resolve("Data Files/Data S"));areas=Files.readAllBytes(original.resolve("Data Files/Data AD"));
        String input="analysis/completion/archive/laboratory-resources-fixed-route/laboratory-12.json";foundation=JsonParser.parseString(Files.readString(root.resolve(input))).getAsJsonObject().getAsJsonObject("session");party=warriors(foundation);JsonObject f=new JsonObject();f.addProperty("input",input);f.addProperty("digest",ActionOracleHarness.sha256(root.resolve(input)));f.add("session",foundation);JsonObject identity=JsonParser.parseString(Files.readString(root.resolve("evidence/fixtures/empty-rifle-advance.json"))).getAsJsonObject().getAsJsonObject("identity").deepCopy();identity.addProperty("advance_probe_sha256",identity.get("probe_sha256").getAsString());identity.addProperty("probe_sha256",ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyOwnedPlayerFirearm.java")));identity.addProperty("advance_fixture_sha256",ActionOracleHarness.sha256(root.resolve("evidence/fixtures/empty-rifle-advance.json")));
        JsonArray gates=new JsonArray(),equipment=new JsonArray(),fire=new JsonArray(),admission=new JsonArray(),ammoControls=new JsonArray();gates(gates);composed(equipment,false,12,2);for(long seed:new long[]{12,1})for(int member:new int[]{2,3})composed(fire,true,seed,member);pickupControl(admission,1);pickupControl(admission,2);for(String k:new String[]{"items_wear_ap_zero","items_fit_ap_zero","wear_inactive","items_fit_inactive","fire_ap_zero","fire_movement_zero"})admissionControl(admission,k);for(String k:new String[]{"manual_full","manual_partial","manual_missing_201","manual_wrong_caliber","prepare_empty","reload_empty","reload_partial","reload_wrong_caliber"})ammunitionControl(ammoControls,k,12);combatFit(ammoControls,12);combatFit(ammoControls,2);JsonObject out=new JsonObject();out.addProperty("authority","Exact earned ordinary item2/ammunition201 full original profile, serial pickup/crate-close/wear/compact/manual compatible candidate selection, original full prepattack and accepted projectile/fire confirmation against original MD13 controlled HP10000/faction1/flat tactical placement. Controlled AP3/MOV5/order and RNG seeds12/1; actual earned party, crate, stock and serialized work projection restored; uncaptured work bytes neutral. Separate native original outerAPcompare116b6a and fullgetup125a3a..127090 continuation, no enemy action. Original SANE and QuickDraw/presentation boundaries inherited; display chooser acknowledges input; no live-original-game, next-battle survival, enhanced item, projectile219 or broad firearm claim.");out.add("identity",identity);out.add("foundation",f);out.add("gate_cases",gates);out.add("equipment_cases",equipment);out.add("fire_cases",fire);out.add("admission_cases",admission);out.add("ammunition_cases",ammoControls);Files.writeString(root.resolve("evidence/fixtures/owned-player-firearm.json"),out+"\n");println("Owned firearm exported");
    }
    void execute(ActionOracleHarness h,RunResult result,long stop) throws Exception {
        SaneBoundary sane=new SaneBoundary(h.cpu,currentProgram);boolean done=false;
        for(int step=0;step<1500000;step++) {
            long pc=h.pc(),sp=h.sp();
            if(pc==0x1363e4&&h.word(sp+4)==0){diagnostic=true;diagnosticSpell=(h.word(0x4435c6)+1)*1000+(h.word(0x4435c8)+1)*100+h.word(0x4435ca)+1;done=true;break;}
            observe(h,result,pc);if(pc==stop||secondaryStop!=0&&pc==secondaryStop){done=true;break;}
            if(pc==0x14de26){h.jump(0x14dede);continue;}
            if(pc==0x12946c){h.cpu.writeRegister("D0",1);h.ret();continue;}
            if(pc==0x14ce2a||pc==0x161b80||pc==0x15f66c||pc==0x1600fc||pc==0x13a498||pc==0x160a8e||pc==0x160cca||pc==0x160baa){h.ret();continue;}
            if(pc==0x130114){h.jump(0x13015e);continue;}
            if(pc==0x130192||pc==0x13023a||pc==0x130296){h.jump(0x1302d6);continue;}
            if(pc==0x13033a){h.jump(0x13040c);continue;}
            if(pc==0x130424){h.jump(0x130434);continue;}
            if(pc==0x13043e){h.jump(0x1305ca);continue;}
            if(pc==0x13071c){long base=h.cpu.readRegister("A6").longValue();int count=h.word(base-0x266),choice=0;JsonArray candidates=new JsonArray();for(int i=0;i<count;i++){int candidate=h.word(base-0x264+i*2);candidates.add(candidate);if(candidate==1&&selectAmmo)choice=i+2;}JsonObject selected=new JsonObject();selected.add("candidates",candidates);selected.addProperty("popup_result",choice);guiSelections.add(selected);h.cpu.writeRegister("D6",choice);h.jump(0x13075c);continue;}
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
