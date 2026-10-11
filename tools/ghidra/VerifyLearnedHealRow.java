import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import com.google.gson.*;

/** Original learned-healing row changes from qualified remembered UI tables. */
public class VerifyLearnedHealRow extends VerifyLearnedHealMemory {
    JsonArray rowEvents=new JsonArray(),stringCalls=new JsonArray();
    long listStack; int listD3,listD4;
    @Override void observe(ActionOracleHarness h,RunResult r,long pc)throws Exception {
        super.observe(h,r,pc);
        if(pc==0x10b6ba){listStack=h.sp();listD3=h.cpu.readRegister("D3").intValue();listD4=h.cpu.readRegister("D4").intValue();}
        if(pc==0x10b802&&(h.sp()!=listStack||h.cpu.readRegister("D3").intValue()!=listD3||h.cpu.readRegister("D4").intValue()!=listD4))throw new Exception("Native showspelllist frame/register restoration differs");
        if(pc==0x10bc3e||pc==0x10c464||pc==0x10c4a6||pc==0x10c558||pc==0x10c56c||pc==0x10c572||pc==0x10b6ba||pc==0x10b802||pc==0x10c5ae||pc==0x10c5b6||pc==0x10c846||pc==0x10c9f0||pc==0x10ca1a||pc==0x10ca20||pc==0x10ca22||pc==0x10ca2e||pc==0x10caaa){JsonObject e=new JsonObject();e.addProperty("address",Long.toHexString(pc));e.addProperty("row",h.word(0x4435c8));e.addProperty("column",h.word(0x4435ca));e.addProperty("power",h.word(0x4435cc));e.addProperty("input",h.word(0x4434b2));e.addProperty("row_changed",h.read(h.cpu.readRegister("A6").longValue()-0x10,1)[0]&255);e.addProperty("draws",r.draws);rowEvents.add(e);}
    }
    @Override JsonObject completeSnap(ActionOracleHarness h,RunResult r)throws Exception {JsonObject j=super.completeSnap(h,r);j.add("row_events",rowEvents.deepCopy());j.add("string_calls",stringCalls.deepCopy());j.addProperty("remembered_row",h.read(0x442580+activeBody*2,1)[0]&255);j.addProperty("remembered_column",h.read(0x442581+activeBody*2,1)[0]&255);j.addProperty("remembered_power",h.word(0x437a7a+activeBody*2));return j;}
    @Override boolean prepare(ActionOracleHarness h,RunResult r,int power,String kind)throws Exception {
        combatFrame(h,r);secondaryStop=0x116b6a;execute(h,r,0x10b9bc);secondaryStop=0;if(h.pc()==0x116b6a)return false;
        execute(h,r,0x10b9c4);h.jump(0x10b9da);execute(h,r,0x10b9ee);h.pop(2);
        h.jump(0x10bba6);execute(h,r,0x10bbc2);h.jump(0x10bbc8);execute(h,r,0x10bc3e);
        h.value(0x4434b2,2,23);h.jump(0x10c464);execute(h,r,0x10c846);
        int school=h.cpu.readRegister("D5").intValue();h.cpu.writeRegister("D3",0);h.cpu.writeRegister("D4",school);h.jump(0x10b700);secondaryStop=0x10b7d4;execute(h,r,0x10b728);secondaryStop=0;boolean learned=h.pc()==0x10b728;
        h.write(0x479ed8,new byte[256]);if(learned)h.write(0x479ed8,healName);
        h.value(0x4434b2,2,1);h.jump(0x10c846);execute(h,r,0x10c852);h.jump(0x10c880);secondaryStop=0x10cb60;execute(h,r,0x10c890);secondaryStop=0;if(!learned)return false;
        h.jump(0x10c918);execute(h,r,0x10c938);h.jump(0x10c9f0);execute(h,r,0x10ca32);h.jump(0x10ca78);execute(h,r,0x10cab0);
        h.value(0x4434b2,2,29+power);h.jump(0x10c5ba);secondaryStop=0x10bf56;execute(h,r,0x10c5e6);secondaryStop=0;
        if(h.pc()==0x10c5e6){secondaryStop=0x10c788;execute(h,r,0x10c622);secondaryStop=0;if(h.pc()==0x10c788){execute(h,r,0x10c792);return false;}execute(h,r,0x10c644);execute(h,r,0x10c698);}
        h.jump(0x10cb8c);secondaryStop=0x10c788;execute(h,r,0x10cc40);secondaryStop=0;if(h.pc()==0x10c788){execute(h,r,0x10c792);return false;}
        if(h.sp()!=h.cpu.readRegister("A6").longValue()-0x3c)throw new Exception("Unbalanced native castspell frame");h.jump(0x10d1d6);execute(h,r,0x119238);execute(h,r,0x1192d0);return true;
    }


    void rowCase(JsonArray out,JsonObject s,int power)throws Exception {
        long seed=s.getAsJsonObject("random").get("seed").getAsLong();begin(seed);poisonEvents=new JsonArray();aidEvents=new JsonArray();warnings=new JsonArray();medicalEvents=new JsonArray();learnedEvents=new JsonArray();menuEvents=new JsonArray();rowEvents=new JsonArray();stringCalls=new JsonArray();choice=1;itemUse=true;ActionOracleHarness h=frame();RunResult r=new RunResult(seed);r.draws=s.getAsJsonObject("random").get("draws").getAsInt();
        try {initializeActual(h,s);for(int i=0;i<4;i++)if(h.word(0x43788a+80*activeBody+72+2*i)!=(new int[]{0,2,4,1})[i])throw new Exception("Prior accepted descriptor history differs");h.value(0x442580+activeBody*2,1,2);h.value(0x442581+activeBody*2,1,4);h.value(0x437a7a+activeBody*2,2,1);JsonObject c=new JsonObject();c.addProperty("power",power);c.add("before",completeSnap(h,r));if(!prepare(h,r,power,"qualified_row"))throw new Exception("Qualified Heal selection not admitted");c.add("prepared",completeSnap(h,r));portraitInputs(h,1);h.jump(0x119db6);execute(h,r,0x116b6a);execute(h,r,0x117338);c.add("after",completeSnap(h,r));saveTrace(c,r);out.add(c);println("Learned healing row2 column4 power1 then Heal power"+power);}finally {h.cpu.dispose();itemUse=false;secondaryStop=0;}
    }
    public void run()throws Exception {
        Path root=Path.of(getScriptArgs()[0]),original=root.resolve("NewCenturions/TheNewCenturions"),scenario=original.resolve("Scenarios/The Thunder of Guns/0");items=Files.readAllBytes(original.resolve("Data Files/Data ID"));scenarioItems=Files.readAllBytes(scenario.resolve("Data NI"));races=Files.readAllBytes(original.resolve("Data Files/Data Race"));monsters=Files.readAllBytes(scenario.resolve("Data MD"));properties=Files.readAllBytes(original.resolve("Data Files/Data Swamp BD"));attacks=Files.readAllBytes(original.resolve("Data Files/Data S"));areas=Files.readAllBytes(original.resolve("Data Files/Data AD"));mos=Files.readAllBytes(original.resolve("Data Files/Data MOS"));healName=Arrays.copyOfRange(Files.readAllBytes(original.resolve("Data Files/Cargo Pod.rsrc")),6520247,6520257);if(healName[0]!=9)throw new Exception("Changed Heal name resource");
        Path input=root.resolve("analysis/completion/learned-portrait-heal-input-12.json");JsonObject s=JsonParser.parseString(Files.readString(input)).getAsJsonObject().getAsJsonObject("session");JsonObject identity=JsonParser.parseString(Files.readString(root.resolve("evidence/fixtures/learned-heal-memory.json"))).getAsJsonObject().getAsJsonObject("identity").deepCopy();identity.addProperty("learned_memory_probe_sha256",identity.get("probe_sha256").getAsString());identity.addProperty("probe_sha256",ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyLearnedHealRow.java")));identity.addProperty("learned_memory_fixture_sha256",ActionOracleHarness.sha256(root.resolve("evidence/fixtures/learned-heal-memory.json")));JsonArray cases=new JsonArray();for(int power:new int[]{1,2})rowCase(cases,s,power);JsonObject f=new JsonObject();f.addProperty("input",root.relativize(input).toString().replace('\\','/'));f.addProperty("digest",ActionOracleHarness.sha256(input));f.add("session",s);JsonObject out=new JsonObject();out.addProperty("authority","Two qualified originallearnedHeal1101 menus onactualearnedseed12prefix before316: onlyunsavedUIrememberedrow2/column4/power1 explicitlyinitialized, correspondingtoactualprioraccepted1305 descriptorhistory[0,2,4,1] butnotclaimedserializedSessionUItruth. Fullnative rowchange23 10c464..10c5b6/10129/localflag1/column-1/row0/autochoice1/fullshowspelllistactuallearnedslotsandtemporarypacketloads, explicitcolumn1 skips141throughnativeflag thenwidget30/31 equality/change/nativeacceptedcharge/fullportrait/Getup. NoHP/Psi/inventory/RNGgrant, nooldfrozenprobe/fixtureedit, boundedUItrap/string/rendering contracts retained, nolivecampaign/fullmenuclaim.");out.add("identity",identity);out.add("foundation",f);JsonObject control=new JsonObject();control.addProperty("actor",5);control.addProperty("row",2);control.addProperty("column",4);control.addProperty("power",1);control.addProperty("qualification","Explicit unsaved native UI table control. Actual descriptor history independently retained; it does not reconstruct unknown save UI history.");out.add("remembered_control",control);out.add("cases",cases);Files.writeString(root.resolve("evidence/fixtures/learned-heal-row.json"),out+"\n");println("Learned healing row exported2cases");
    }
    @Override void execute(ActionOracleHarness h,RunResult result,long stop) throws Exception {
        SaneBoundary sane=new SaneBoundary(h.cpu,currentProgram);boolean done=false;
        for(int step=0;step<1500000;step++) {
            long pc=h.pc(),sp=h.sp();
            if(pc==0x1363e4&&h.word(sp+4)==0){diagnostic=true;diagnosticSpell=(h.word(0x4435c6)+1)*1000+(h.word(0x4435c8)+1)*100+h.word(0x4435ca)+1;done=true;break;}
            observe(h,result,pc);if(pc==stop||(secondaryStop!=0&&pc==secondaryStop)){done=true;break;}
            if(pc==0x14d21a&&!confirming){confirmedDialog(h,result);continue;}
            if(pc==0x11f1a8&&!updating){updateRuleWork(h,result);continue;}
            if(pc==0x13ba6c||pc==0x13bac6){profileFile=pc==0x13ba6c?0:1;profileCursor=0;h.value(0x4434a6,4,0x600000+profileFile);h.ret();continue;}
            if(pc==0x400078){profileCursor=(int)h.pointer(sp+8);h.cpu.writeRegister("D0",0);h.ret();continue;}
            if(pc==0x400048){int amount=(int)(h.pointer(sp+8)*h.pointer(sp+12));byte[] source=profileFile==0?races:mos;h.write(h.pointer(sp+4),java.util.Arrays.copyOfRange(source,profileCursor,profileCursor+amount));profileCursor+=amount;h.cpu.writeRegister("D0",h.pointer(sp+12));h.ret();continue;}
            if(pc==0x400068){h.cpu.writeRegister("D0",0);h.ret();continue;}
            if(pc==0x400090||pc==0x400098||pc==0x400168||pc==0x400180){h.cpu.writeRegister("D0",0);h.ret();continue;}
            if(pc==0x400178){JsonObject e=new JsonObject();e.addProperty("boundary","original CODE2 017c GetIndString; epilogue01ba..01c2 consumes8 arguments");e.addProperty("index",h.word(sp+4));e.addProperty("resource",h.word(sp+6));e.addProperty("destination",Long.toHexString(h.pointer(sp+8)));e.addProperty("sp_before",Long.toHexString(sp));h.ret();h.pop(8);e.addProperty("sp_after",Long.toHexString(h.sp()));stringCalls.add(e);continue;}
            if(pc==0x1382d0){h.ret();continue;}
            if(pc==0x13bb20||pc==0x13e13c||pc==0x153d56||pc==0x153daa||pc==0x15947c||pc==0x10b814||pc==0x13b56c||pc==0x15f4b8){h.ret();continue;}
            if(pc==0x1251b8){JsonObject e=new JsonObject();e.addProperty("boundary","portrait event poll and geometry");e.addProperty("selected_input",choice);e.addProperty("include_dead",h.word(h.cpu.readRegister("A6").longValue()+12));aidEvents.add(e);if(choice<0){h.value(0x44720e,2,0x61);h.jump(0x125222);}else{h.value(0x4434b2,2,choice);h.jump(0x125300);}continue;}
            if(pc==0x14ce2a||pc==0x149fc6||pc==0x153720||pc==0x160d08||pc==0x15fa20||pc==0x13a510||pc==0x1377f4){h.ret();continue;}
            if(pc==0x16474a){warnings.add(h.word(sp+4));h.ret();continue;}
            if(pc==0x14de26){h.jump(0x14dede);continue;}
            if(pc==0x12946c){h.cpu.writeRegister("D0",1);h.ret();continue;}
            if(pc==0x161b80||pc==0x15f66c||pc==0x13a498||pc==0x160a8e||pc==0x160cca||pc==0x160baa){h.ret();continue;}
            if(pc==0x4001a0){sane.extended(h.pointer(sp+4),Math.sqrt(sane.extended(sp+8)));h.ret();continue;}
            if(pc==0x400038){h.cpu.writeRegister("D0",Math.abs(h.word(sp+4)));h.ret();continue;}
            if(pc==0x153ca4){result.rng(h);continue;}
            if(pc==0x133a18||pc==0x16474a){result.sound(h);continue;}
            if(pc==0x123d1c){int sound=h.word(sp+14);if(sound!=0){if(result.sounds.length()>0)result.sounds.append(',');result.sounds.append(sound);}h.ret();continue;}
            if(pc==0x15b2f6||pc==0x15a1f6){h.ret();continue;}
            if(pc==0x159192){displaySpell(h,result);continue;}
            if(pc==0x158454){long caller=h.pointer(sp);for(int i=0;h.pc()!=0x15847a;i++){if(i>12||!h.cpu.step(monitor))throw new Exception("Native result prefix failed");}long savedStack=h.sp();h.cpu.writeRegister("SP",sp);showresultsSound(h,result);h.cpu.writeRegister("SP",savedStack);JsonObject e=new JsonObject();e.addProperty("address","158474");e.addProperty("aid_gate",h.word(0x438786));e.addProperty("return",Long.toHexString(caller));medicalEvents.add(e);h.jump(0x15916c);execute(h,result,0x159178);h.jump(0x15917a);execute(h,result,caller);continue;}
            if(pc==0x1600fc||pc==0x120c14||pc==0x105572||pc==0x10e6b6||pc==0x133d46||pc==0x13154c||pc==0x11edc4||pc==0x14a02c||pc==0x13a2c4||pc==0x1205f8||pc==0x16043e||pc==0x1382a8||pc==0x11f136||pc==0x1537ee||pc==0x160fb2||pc==0x10ff54||pc==0x12f040||pc==0x15edfa){h.ret();continue;}
            if(pc==0x12382a&&h.word(0x439596)!=0)throw new Exception("Unsupported tutorial-enabled UI preference");
            int op=h.word(pc)&65535;
            if(op==0xa98d){h.value(h.pointer(sp),2,0);h.value(h.pointer(sp+4),4,0x820000);h.value(h.pointer(sp+8),2,0);h.pop(18);h.jump(pc+2);continue;}
            if(op==0xa8f6){h.pop(8);h.jump(pc+2);continue;}
            if(op==0xaa14){h.pop(4);h.jump(pc+2);continue;}
            if(op==0xa88c){long text=h.pointer(sp);h.value(sp+4,2,h.read(text,1)[0]&255);h.pop(4);h.jump(pc+2);continue;}
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
            if(op==0xa8ad){long rect=h.pointer(sp);int x=h.word(sp+4),y=h.word(sp+6);h.value(sp+8,1,x>=h.word(rect)&&x<h.word(rect+4)&&y>=h.word(rect+2)&&y<h.word(rect+6)?1:0);h.pop(8);h.jump(pc+2);continue;}
            if(op==0xa9b4){h.jump(pc+2);continue;}
            if(op==0xa974){h.cpu.writeRegister("SP",sp-2);h.value(sp-2,2,0);h.jump(pc+2);continue;}
            if(!h.cpu.step(monitor))throw new Exception("Ally action at "+Long.toHexString(pc)+": "+h.cpu.getLastError());
        }
        if(!done)throw new Exception("Ally action instruction budget at "+Long.toHexString(h.pc()));
    }
}
