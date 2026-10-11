import java.nio.file.Files;
import java.nio.file.Path;
import com.google.gson.*;

/** Whole occupied-cell reload with two explicit ammunition quantity controls. */
public class VerifyEarnedCombatReloadBounds extends VerifyEarnedCombatReload {
    int reserveControl;
    boolean warningRunning;
    byte[] warningStrings;
    JsonArray boundsEvents=new JsonArray(),warningPhases=new JsonArray(),reserveScan=new JsonArray();
    @Override void initializeActual(ActionOracleHarness h,JsonObject s)throws Exception {
        super.initializeActual(h,s);h.value(0x42dbe4+80*4,4,reserveControl);
    }
    @Override JsonObject completeSnap(ActionOracleHarness h,RunResult r)throws Exception {
        JsonObject j=super.completeSnap(h,r);j.addProperty("involuntary_movement",h.word(0x438764));j.addProperty("inventory_reserve_scan",h.word(0x4434ac));j.addProperty("warning_input_word",h.word(0x4471a2));j.addProperty("warning_event_record",hex(h.read(0x44720a,16)));j.addProperty("warning_text",hex(h.read(0x479ed8,256)));return j;
    }
    @Override void observe(ActionOracleHarness h,RunResult r,long pc)throws Exception {
        super.observe(h,r,pc);
        if(pc==0x14ea90){int index=h.word(0x4434ac);JsonObject e=new JsonObject();e.addProperty("index",index);e.addProperty("reserve",h.pointer(0x42dbe4+index*4));e.addProperty("ammo_work",hex(h.read(0x4536d2,150)));reserveScan.add(e);}
        if(pc==0x14e700||pc==0x14e706||pc==0x14e784||pc==0x14e786||pc==0x14e8a4||pc==0x14e95e||pc==0x14e9c2||pc==0x14ea42||pc==0x14eab6||pc==0x14edb4||pc==0x14ee22||pc==0x14ee36||pc==0x14ee58||pc==0x14ee76||pc==0x14ee7c||pc==0x14ee82||pc==0x16474a||pc==0x16495a||pc==0x164960||pc==0x164986||pc==0x164988||pc==0x1649aa||pc==0x1649b0||pc==0x1649b6||pc==0x164a04){JsonObject e=new JsonObject();e.addProperty("address",Long.toHexString(pc));e.addProperty("sp",h.sp());e.addProperty("d0",(short)h.cpu.readRegister("D0").intValue());e.addProperty("d3",(short)h.cpu.readRegister("D3").intValue());e.addProperty("d5",(short)h.cpu.readRegister("D5").intValue());e.addProperty("d6",(short)h.cpu.readRegister("D6").intValue());e.add("snapshot",completeSnap(h,r));boundsEvents.add(e);}
    }
    @Override void stringBoundary(ActionOracleHarness h){
        long sp=h.sp();if(h.word(sp+6)==3&&h.word(sp+4)==25){int offset=2;for(int i=1;i<25;i++)offset+=1+(warningStrings[offset]&255);int length=warningStrings[offset]&255;byte[] message=java.util.Arrays.copyOfRange(warningStrings,offset,offset+length+1);String text=new String(message,1,length,java.nio.charset.StandardCharsets.ISO_8859_1);if(!text.equals("This weapon is out of ammo. Reload or change weapons."))throw new IllegalStateException("Warning resource index25");h.write(h.pointer(sp+8),message);JsonObject e=new JsonObject();e.addProperty("address","400178");e.addProperty("resource",3);e.addProperty("index",25);e.addProperty("pascal_text",hex(message));e.addProperty("sp_before",sp);e.addProperty("return",h.pointer(sp));h.ret();h.pop(8);e.addProperty("sp_after",h.sp());medicalEvents.add(e);}else super.stringBoundary(h);
    }
    void nativeWarning(ActionOracleHarness h,RunResult r)throws Exception {
        long sp=h.sp(),returnPc=h.pointer(sp),d3=h.cpu.readRegister("D3").longValue(),a2=h.cpu.readRegister("A2").longValue();int number=h.word(sp+4);if(number!=25)throw new Exception("Uncontracted reload warning "+number);JsonObject e=new JsonObject();e.addProperty("address","16474a");e.addProperty("argument",number);e.addProperty("entry_sp",sp);e.addProperty("return_address",returnPc);e.addProperty("d3_before",d3);e.addProperty("a2_before",a2);e.add("before",completeSnap(h,r));warnings.add(number);warningRunning=true;try{if(!h.cpu.step(monitor))throw new Exception("Original warning LINK");execute(h,r,returnPc);}finally{warningRunning=false;}if(h.sp()!=sp+4||(short)h.cpu.readRegister("D0").intValue()!=0||h.cpu.readRegister("D3").longValue()!=d3||h.cpu.readRegister("A2").longValue()!=a2)throw new Exception("Original warning return ABI");e.addProperty("d3_after",h.cpu.readRegister("D3").longValue());e.addProperty("a2_after",h.cpu.readRegister("A2").longValue());e.addProperty("d0_return_word",(short)h.cpu.readRegister("D0").intValue());e.add("after",completeSnap(h,r));warningPhases.add(e);
    }
    @Override public void run()throws Exception {
        Path root=Path.of(getScriptArgs()[0]),original=root.resolve("NewCenturions/TheNewCenturions"),scenario=original.resolve("Scenarios/The Thunder of Guns/0");
        Path stringsPath=root.resolve("analysis/extracted/resources/74/53545223_3.bin");if(!ActionOracleHarness.sha256(stringsPath).equals("b11e47595c18787e109b64b05483f22f408fe07200a2ba977513ed2f826e2a41"))throw new Exception("Warning STR#3 identity");warningStrings=Files.readAllBytes(stringsPath);
        items=Files.readAllBytes(original.resolve("Data Files/Data ID"));scenarioItems=Files.readAllBytes(scenario.resolve("Data NI"));races=Files.readAllBytes(original.resolve("Data Files/Data Race"));mos=Files.readAllBytes(original.resolve("Data Files/Data MOS"));monsters=Files.readAllBytes(scenario.resolve("Data MD"));properties=Files.readAllBytes(original.resolve("Data Files/Data Swamp BD"));attacks=Files.readAllBytes(original.resolve("Data Files/Data S"));areas=Files.readAllBytes(original.resolve("Data Files/Data AD"));
        JsonObject identity=JsonParser.parseString(Files.readString(root.resolve("evidence/fixtures/earned-combat-reload.json"))).getAsJsonObject().getAsJsonObject("identity").deepCopy();identity.addProperty("earned_combat_reload_probe_sha256",identity.get("probe_sha256").getAsString());identity.addProperty("earned_combat_reload_fixture_sha256",ActionOracleHarness.sha256(root.resolve("evidence/fixtures/earned-combat-reload.json")));identity.addProperty("probe_sha256",ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyEarnedCombatReloadBounds.java")));
        identity.addProperty("warning_strings",root.relativize(stringsPath).toString().replace('\\','/'));identity.addProperty("warning_strings_digest",ActionOracleHarness.sha256(stringsPath));
        JsonObject foundation=foundation(root,12);JsonArray foundations=new JsonArray(),cases=new JsonArray();foundations.add(foundation);
        for(int stock:new int[]{1,0}){reserveControl=stock;boundsEvents=new JsonArray();warningPhases=new JsonArray();reserveScan=new JsonArray();warningRunning=false;reloadCase(cases,foundation,0,true);JsonObject c=cases.get(cases.size()-1).getAsJsonObject();c.addProperty("id",stock==1?"normalized_seed12_last_stock_control":"normalized_seed12_empty_stock_control");JsonObject control=new JsonObject();control.addProperty("address","42dd24");control.addProperty("ammunition_id",280);control.addProperty("index",80);control.addProperty("width_bits",32);control.addProperty("actual_quantity",3);control.addProperty("controlled_quantity",stock);control.addProperty("all_other_whole_input_bytes_unchanged",true);c.add("quantity_control",control);c.add("bounds_events",boundsEvents);c.add("warning_phases",warningPhases);c.add("reserve_scan",reserveScan);}
        JsonObject out=new JsonObject();out.addProperty("authority","Additive whole original normalized diagonal occupied-cell caller with explicit reserve LONG index80 controls 1 and 0. Both retain the actual seed12 earned input otherwise unchanged; controls are not independently earned replay states. No live original-game claim.");out.add("identity",identity);out.add("foundations",foundations);out.add("cases",cases);Files.writeString(root.resolve("evidence/fixtures/earned-combat-reload-bounds.json"),out+"\n");println("Earned combat reload bounds exported2");
    }
    void execute(ActionOracleHarness h,RunResult result,long stop) throws Exception {
        SaneBoundary sane=new SaneBoundary(h.cpu,currentProgram);boolean done=false;
        for(int step=0;step<1500000;step++) {
            long pc=h.pc(),sp=h.sp();
            if(pc==0x1363e4&&h.word(sp+4)==0){diagnostic=true;diagnosticSpell=(h.word(0x4435c6)+1)*1000+(h.word(0x4435c8)+1)*100+h.word(0x4435ca)+1;done=true;break;}
            observe(h,result,pc);if(pc==0x10ed92){h.jump(0x10f168);continue;}if(pc==0x10e844){h.jump(0x10ec34);continue;}if(pc==stop||(secondaryStop!=0&&pc==secondaryStop)){done=true;break;}
            if(pc==0x15fa20&&!characterInfo){characterRuleInfo(h,result);continue;}
            if(pc==0x14d21a&&!confirming){confirmedDialog(h,result);continue;}
            if(pc==0x11f1a8&&!updating){updateRuleWork(h,result);continue;}
            if(pc==0x13ba6c||pc==0x13bac6){profileFile=pc==0x13ba6c?0:1;profileCursor=0;h.value(0x4434a6,4,0x600000+profileFile);h.ret();continue;}
            if(pc==0x400078){profileCursor=(int)h.pointer(sp+8);h.cpu.writeRegister("D0",0);h.ret();continue;}
            if(pc==0x400048){int amount=(int)(h.pointer(sp+8)*h.pointer(sp+12));byte[] source=profileFile==0?races:mos;h.write(h.pointer(sp+4),java.util.Arrays.copyOfRange(source,profileCursor,profileCursor+amount));profileCursor+=amount;h.cpu.writeRegister("D0",h.pointer(sp+12));h.ret();continue;}
            if(pc==0x400068){h.cpu.writeRegister("D0",0);h.ret();continue;}
            if(pc==0x400090||pc==0x400098||pc==0x400168||pc==0x400180){h.cpu.writeRegister("D0",0);h.ret();continue;}
            if(pc==0x400178){stringBoundary(h);continue;}
            if(pc==0x1382d0){h.ret();continue;}
            if(pc==0x1620ea){JsonObject e=new JsonObject();e.addProperty("boundary","updatespell drawing omitted; native checkrange and cached distance retained");e.addProperty("draws",result.draws);e.addProperty("native_distance",(short)h.cpu.readRegister("D3").intValue());medicalEvents.add(e);h.jump(0x1621e8);continue;}
            if(pc==0x13e13c||pc==0x13b56c||pc==0x15f4b8){h.ret();continue;}
            if(pc==0x1251b8){JsonObject e=new JsonObject();e.addProperty("boundary","portrait event poll and geometry");e.addProperty("selected_input",choice);e.addProperty("include_dead",h.word(h.cpu.readRegister("A6").longValue()+12));aidEvents.add(e);if(choice<0){h.value(0x44720e,2,0x61);h.jump(0x125222);}else{h.value(0x4434b2,2,choice);h.jump(0x125300);}continue;}
            if(pc==0x14ce2a||pc==0x149fc6||pc==0x153720||pc==0x160d08||pc==0x13a510||pc==0x1377f4){h.ret();continue;}
            if(pc==0x16474a&&!warningRunning){nativeWarning(h,result);continue;}
            if(pc==0x14de26){h.jump(0x14dede);continue;}
            if(pc==0x161b80||pc==0x15f66c||pc==0x13a498||pc==0x160a8e||pc==0x160cca||pc==0x160baa){h.ret();continue;}
            if(pc==0x4001a0){sane.extended(h.pointer(sp+4),Math.sqrt(sane.extended(sp+8)));h.ret();continue;}
            if(pc==0x400038){h.cpu.writeRegister("D0",Math.abs(h.word(sp+4)));h.ret();continue;}
            if(pc==0x153ca4){result.rng(h);continue;}
            if(pc==0x133a18){result.sound(h);continue;}
            if(pc==0x123d1c){int sound=h.word(sp+14);if(sound!=0){if(result.sounds.length()>0)result.sounds.append(',');result.sounds.append(sound);}h.ret();continue;}
            if(pc==0x15b2f6||pc==0x15a1f6){h.ret();continue;}
            if(pc==0x159192){displaySpell(h,result);continue;}
            if(pc==0x158454){long caller=h.pointer(sp);for(int i=0;h.pc()!=0x15847a;i++){if(i>12||!h.cpu.step(monitor))throw new Exception("Native result prefix failed");}long savedStack=h.sp();h.cpu.writeRegister("SP",sp);showresultsSound(h,result);h.cpu.writeRegister("SP",savedStack);JsonObject e=new JsonObject();e.addProperty("address","158474");e.addProperty("aid_gate",h.word(0x438786));e.addProperty("return",Long.toHexString(caller));medicalEvents.add(e);h.jump(0x15916c);execute(h,result,0x159178);h.jump(0x15917a);execute(h,result,caller);continue;}
            if(pc==0x1600fc||pc==0x120c14||pc==0x105572||pc==0x133d46||pc==0x13154c||pc==0x11edc4||pc==0x14a02c||pc==0x1205f8||pc==0x16043e||pc==0x1382a8||pc==0x11f136||pc==0x1537ee||pc==0x160fb2||pc==0x10ff54||pc==0x12f040||pc==0x15edfa){h.ret();continue;}
            if(pc==0x12382a&&h.word(0x439596)!=0)throw new Exception("Unsupported tutorial-enabled UI preference");
            int op=h.word(pc)&65535;
            if(op==0xa874){h.value(h.pointer(sp),4,0x830000);h.pop(4);h.jump(pc+2);continue;}
            if(op==0xaa19||op==0xaa1a){h.write(h.pointer(sp),new byte[6]);h.pop(4);h.jump(pc+2);continue;}
            if(op==0xaa14||op==0xaa15||op==0xaa0b||op==0xa915||op==0xa981){h.pop(4);h.jump(pc+2);continue;}
            if(op==0xa98b){h.pop(16);h.jump(pc+2);continue;}
            if(op==0xa97c){h.pop(10);h.value(h.sp(),4,0x830500);h.jump(pc+2);continue;}
            if(op==0xa91b){h.pop(10);h.jump(pc+2);continue;}
            if(op==0xa860){if(!warningRunning||h.pointer(sp)!=0||h.pointer(sp+4)!=0||h.pointer(sp+8)!=0x44720a||h.word(sp+12)!=-1)throw new Exception("Uncontracted warning WaitNextEvent");byte[] event=new byte[16];java.nio.ByteBuffer.wrap(event).putShort((short)3).putInt(0x3120);h.write(h.pointer(sp+8),event);JsonObject e=new JsonObject();e.addProperty("address",Long.toHexString(pc));e.addProperty("boundary","WaitNextEvent ordinary keydown dismissal");e.addProperty("event_record",hex(event));e.addProperty("sp_before",sp);h.pop(14);h.value(h.sp(),1,1);e.addProperty("sp_after",h.sp());nativeCalls.add(e);h.jump(pc+2);continue;}
            if(op==0xa975){h.value(sp,4,0x830000);h.jump(pc+2);continue;}
            if(op==0xa91f){h.value(h.pointer(sp),4,0);h.pop(4);h.jump(pc+2);continue;}
            if(op==0xa032){JsonObject event=new JsonObject();event.addProperty("address",Long.toHexString(pc));event.addProperty("boundary","FlushEvents");event.addProperty("d0",h.cpu.readRegister("D0").intValue());event.addProperty("draws",result.draws);nativeCalls.add(event);h.jump(pc+2);continue;}
            if(op==0xa85d){nativeBit(h,pc+2);continue;}
            if(op==0xa9eb){
                // Apple Numerics Manual FSQRTX uses one extended destination address.
                if((h.word(sp)&65535)==0x12){long destination=h.pointer(sp+2);sane.extended(destination,Math.sqrt(sane.extended(destination)));h.pop(6);h.jump(pc+2);}else sane.execute();
                continue;
            }
            if(op==0xaa1b){h.pop(2);h.value(h.sp(),4,0x810000);h.jump(pc+2);continue;}
            if(op==0xaa26){h.pop(4);h.jump(pc+2);continue;}
            if(op==0xa93c){h.pop(4);h.jump(pc+2);continue;}
            if(op==0xa935){h.pop(6);h.jump(pc+2);continue;}
            if(op==0xa937){h.jump(pc+2);continue;}
            if(op==0xa983||op==0xa914||op==0xa873||op==0xaa1c||op==0xa862||op==0xa863||op==0xa89b||op==0xa893||op==0xa891||op==0xa884){h.pop(4);h.jump(pc+2);continue;}
            if(op==0xa887||op==0xa888||op==0xa889||op==0xa88a){h.pop(2);h.jump(pc+2);continue;}
            if(op==0xaa25||op==0xa8a3){h.pop(4);h.jump(pc+2);continue;}
            if(op==0xaa1f||op==0xa8a8){h.pop(8);h.jump(pc+2);continue;}
            if(op==0xa89c){h.pop(2);h.jump(pc+2);continue;}
            if(op==0xa8b7){h.pop(4);h.jump(pc+2);continue;}
            if(op==0xa8ec){h.pop(22);h.jump(pc+2);continue;}
            if(op==0xa8a9){int vertical=h.word(sp),horizontal=h.word(sp+2);long rect=h.pointer(sp+4);h.value(rect,2,h.word(rect)+vertical);h.value(rect+2,2,h.word(rect+2)+horizontal);h.value(rect+4,2,h.word(rect+4)-vertical);h.value(rect+6,2,h.word(rect+6)-horizontal);h.pop(8);h.jump(pc+2);continue;}
            if(op==0xa8aa){long output=h.pointer(sp),b=h.pointer(sp+4),a=h.pointer(sp+8);int top=Math.max(h.word(a),h.word(b)),left=Math.max(h.word(a+2),h.word(b+2)),bottom=Math.min(h.word(a+4),h.word(b+4)),right=Math.min(h.word(a+6),h.word(b+6));boolean empty=top>=bottom||left>=right;h.value(output,2,empty?0:top);h.value(output+2,2,empty?0:left);h.value(output+4,2,empty?0:bottom);h.value(output+6,2,empty?0:right);h.value(sp+12,1,empty?0:1);h.pop(12);h.jump(pc+2);continue;}
            if(op==0xa8a7){long rectangle=h.pointer(sp+8);h.write(rectangle,h.read(sp+4,4));h.write(rectangle+4,h.read(sp,4));h.pop(12);h.jump(pc+2);continue;}
            if(op==0xaa1e){h.value(sp+2,4,0);h.pop(2);h.jump(pc+2);continue;}
            if(op==0xa8ad){long rect=h.pointer(sp);int vertical=h.word(sp+4),horizontal=h.word(sp+6);boolean inside=vertical>=h.word(rect)&&vertical<h.word(rect+4)&&horizontal>=h.word(rect+2)&&horizontal<h.word(rect+6);JsonObject e=new JsonObject();e.addProperty("address",Long.toHexString(pc));e.addProperty("vertical",vertical);e.addProperty("horizontal",horizontal);e.addProperty("rectangle",hex(h.read(rect,8)));e.addProperty("inside",inside);e.addProperty("draws",result.draws);collisionTests.add(e);h.value(sp+8,1,inside?1:0);h.pop(8);h.jump(pc+2);continue;}
            if(op==0xa9b4){h.jump(pc+2);continue;}
            if(op==0xa974){h.cpu.writeRegister("SP",sp-2);h.value(sp-2,2,0);h.jump(pc+2);continue;}
            if(!h.cpu.step(monitor))throw new Exception("Ally action at "+Long.toHexString(pc)+": "+h.cpu.getLastError());
        }
        if(!done)throw new Exception("Ally action instruction budget at "+Long.toHexString(h.pc()));
    }

}
