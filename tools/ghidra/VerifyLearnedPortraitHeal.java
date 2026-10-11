import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import com.google.gson.*;

/** Original learned healing preparation and unconscious portrait continuation. */
public class VerifyLearnedPortraitHeal extends VerifyCombatAid {
    JsonArray learnedEvents=new JsonArray();
    byte[] healName;
    @Override void initializeActual(ActionOracleHarness h,JsonObject s)throws Exception {super.initializeActual(h,s);h.value(0x442749,1,s.get("psi_disabled").getAsInt());h.value(0x442750,1,s.getAsJsonObject("health_context").get("marks_affected").getAsBoolean()?1:0);h.value(0x435662,1,0);}
    @Override void observe(ActionOracleHarness h,RunResult r,long pc)throws Exception {
        super.observe(h,r,pc);
        if(pc==0x13beea||pc==0x119144||pc==0x10b96c||pc==0x10b720||pc==0x10b724||pc==0x10c88c||pc==0x10caaa||pc==0x10cc36||pc==0x10c64e||pc==0x10d2b4||pc==0x10d2ec||pc==0x10d3b6||pc==0x119210||pc==0x11a91a||pc==0x11a9a4||pc==0x11a9ba||pc==0x11a16a){JsonObject e=new JsonObject();e.addProperty("address",Long.toHexString(pc));e.addProperty("draws",r.draws);e.addProperty("d0",h.cpu.readRegister("D0").intValue());e.addProperty("d3",h.cpu.readRegister("D3").intValue());e.addProperty("d5",h.cpu.readRegister("D5").intValue());e.addProperty("psi",h.word(0x448aa0+activeBody*3138+0x58));e.addProperty("count",h.word(0x4435cc));e.addProperty("selected_member",h.read(0x44273f,1)[0]&255);e.addProperty("learned_marker",h.read(0x435662,1)[0]&255);e.addProperty("heal_marks_affected",h.read(0x442750,1)[0]&255);if(pc==0x13beea){e.addProperty("return",Long.toHexString(h.pointer(h.sp())));JsonArray a=new JsonArray();a.add(h.word(h.sp()+4));a.add(h.word(h.sp()+6));e.add("arguments",a);}learnedEvents.add(e);}
    }
    @Override JsonObject completeSnap(ActionOracleHarness h,RunResult r)throws Exception {JsonObject j=super.completeSnap(h,r);j.add("learned_events",learnedEvents.deepCopy());j.addProperty("psi_disabled",h.read(0x442749,1)[0]&255);j.addProperty("psi_menu_mode",h.word(0x43877a));j.addProperty("resume_selection",h.read(0x437a6a,1)[0]&255);j.addProperty("learned_marker",h.read(0x435662,1)[0]&255);j.addProperty("heal_marks_affected",h.read(0x442750,1)[0]&255);return j;}
    void combatFrame(ActionOracleHarness h,RunResult r)throws Exception {h.enter(0x117be2,0x6ffe00,0x116b6a);execute(h,r,0x117c46);h.jump(0x117c56);execute(h,r,0x117c6e);h.write(0x820000,new byte[512]);h.value(0x820000,4,0x820100);h.value(0x439470,4,0x820000);h.value(0x439404,4,0x820000);h.value(0x4394d4,4,0x820000);h.jump(0x119126);}
    boolean prepare(ActionOracleHarness h,RunResult r,int power,String kind)throws Exception {
        combatFrame(h,r);secondaryStop=0x116b6a;execute(h,r,0x10b9bc);secondaryStop=0;if(h.pc()==0x116b6a)return false;
        execute(h,r,0x10b9c4);h.jump(0x10b9da);execute(h,r,0x10b9ee);h.pop(2);
        h.jump(0x10bba6);execute(h,r,0x10bbc2);h.jump(0x10bbc8);execute(h,r,0x10bc3e);
        h.value(0x4434b2,2,23);h.jump(0x10c464);execute(h,r,0x10bf56);
        int school=h.cpu.readRegister("D5").intValue();h.cpu.writeRegister("D3",0);h.cpu.writeRegister("D4",school);h.jump(0x10b700);secondaryStop=0x10b7d4;execute(h,r,0x10b728);secondaryStop=0;boolean learned=h.pc()==0x10b728;
        h.write(0x479ed8,new byte[256]);if(learned)h.write(0x479ed8,healName);
        h.value(0x4434b2,2,1);h.jump(0x10c846);execute(h,r,0x10c852);h.jump(0x10c880);secondaryStop=0x10cb60;execute(h,r,0x10c890);secondaryStop=0;if(!learned)return false;
        h.jump(0x10c918);execute(h,r,0x10c938);h.jump(0x10c9f0);execute(h,r,0x10ca32);h.jump(0x10ca78);execute(h,r,0x10cab0);
        if(power==2){h.value(0x4434b2,2,31);h.jump(0x10c5e6);secondaryStop=0x10c788;execute(h,r,0x10c622);secondaryStop=0;if(h.pc()==0x10c788){execute(h,r,0x10c792);return false;}execute(h,r,0x10c644);execute(h,r,0x10c698);}
        h.jump(0x10cb8c);secondaryStop=0x10c788;execute(h,r,0x10cc40);secondaryStop=0;if(h.pc()==0x10c788){execute(h,r,0x10c792);return false;}
        if(h.sp()!=h.cpu.readRegister("A6").longValue()-0x3c)throw new Exception("Unbalanced native castspell frame");h.jump(0x10d1d6);execute(h,r,0x119238);execute(h,r,0x1192d0);return true;
    }
    void caseRun(JsonArray out,JsonObject s,int target,int power,String kind,int foundation)throws Exception {
        long seed=s.getAsJsonObject("random").get("seed").getAsLong();begin(seed);poisonEvents=new JsonArray();aidEvents=new JsonArray();warnings=new JsonArray();medicalEvents=new JsonArray();learnedEvents=new JsonArray();choice=target;itemUse=true;ActionOracleHarness h=frame();RunResult r=new RunResult(seed);r.draws=s.getAsJsonObject("random").get("draws").getAsInt();try{initializeActual(h,s);if(kind.equals("race_three"))h.value(0x448aa0+target*3138+0x32,2,3);if(kind.equals("terminal_portrait"))h.value(0x448aa0+target*3138+0x50,2,-10);if(kind.equals("insufficient_psi"))h.value(0x448aa0+activeBody*3138+0x58,2,5);if(kind.equals("cannot_cast"))h.value(0x448aa0+activeBody*3138+0x14,2,1);if(kind.equals("not_learned"))h.value(0x448aa0+activeBody*3138+0x9f0,1,0);JsonObject j=new JsonObject();j.addProperty("kind",kind);j.addProperty("foundation",foundation);j.addProperty("actor",activeBody);j.addProperty("target",target);j.addProperty("power",power);j.add("before",completeSnap(h,r));boolean accepted=prepare(h,r,power,kind);j.addProperty("prepared",accepted);j.addProperty("preparation_stop",Long.toHexString(h.pc()));j.add("after_preparation",completeSnap(h,r));if(accepted){h.write(0x821000,new byte[512]);h.value(0x821000,4,0x821100);h.value(0x821108,2,0);h.value(0x82110a,2,330);h.value(0x82110c,2,300);h.value(0x82110e,2,380);h.value(0x439440,4,0x821000);h.value(0x4471ee,2,target*50+10);h.value(0x4471f0,2,340);if(kind.equals("confirmed_cancel")){h.value(0x44720e,2,0x61);h.jump(0x11a18e);execute(h,r,0x116b6a);}else{h.jump(0x119db6);execute(h,r,kind.equals("living_portrait")||kind.equals("terminal_portrait")?0x11a16a:0x116b6a);}if(!kind.equals("living_portrait")&&!kind.equals("terminal_portrait"))execute(h,r,0x117338);}j.add("after",completeSnap(h,r));saveTrace(j,r);out.add(j);println("Learned portrait Heal "+kind+" foundation"+foundation+" target"+target+" power"+power);}finally{h.cpu.dispose();itemUse=false;secondaryStop=0;}
    }
    public void run()throws Exception {
        Path root=Path.of(getScriptArgs()[0]),original=root.resolve("NewCenturions/TheNewCenturions"),scenario=original.resolve("Scenarios/The Thunder of Guns/0");items=Files.readAllBytes(original.resolve("Data Files/Data ID"));scenarioItems=Files.readAllBytes(scenario.resolve("Data NI"));races=Files.readAllBytes(original.resolve("Data Files/Data Race"));monsters=Files.readAllBytes(scenario.resolve("Data MD"));properties=Files.readAllBytes(original.resolve("Data Files/Data Swamp BD"));attacks=Files.readAllBytes(original.resolve("Data Files/Data S"));areas=Files.readAllBytes(original.resolve("Data Files/Data AD"));mos=Files.readAllBytes(original.resolve("Data Files/Data MOS"));byte[] strings=Files.readAllBytes(original.resolve("Data Files/Cargo Pod.rsrc"));healName=Arrays.copyOfRange(strings,6520247,6520257);if(healName[0]!=9)throw new Exception("Changed Heal name resource");JsonObject identity=JsonParser.parseString(Files.readString(root.resolve("evidence/fixtures/combat-aid.json"))).getAsJsonObject().getAsJsonObject("identity").deepCopy();identity.addProperty("combat_aid_probe_sha256",identity.get("probe_sha256").getAsString());identity.addProperty("probe_sha256",ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyLearnedPortraitHeal.java")));identity.addProperty("combat_aid_fixture_sha256",ActionOracleHarness.sha256(root.resolve("evidence/fixtures/combat-aid.json")));identity.addProperty("spell_names_container_sha256",ActionOracleHarness.sha256(original.resolve("Data Files/Cargo Pod.rsrc")));JsonArray foundations=new JsonArray(),cases=new JsonArray();for(int i=0;i<2;i++){int seed=i==0?12:1,target=i==0?1:2;Path input=root.resolve("analysis/completion/learned-portrait-heal-input-"+seed+".json");JsonObject s=JsonParser.parseString(Files.readString(input)).getAsJsonObject().getAsJsonObject("session");JsonObject f=new JsonObject();f.addProperty("input",root.relativize(input).toString().replace('\\','/'));f.addProperty("digest",ActionOracleHarness.sha256(input));f.add("session",s);foundations.add(f);for(int power:new int[]{1,2})caseRun(cases,s,target,power,"actual",i);if(i==0){for(int power:new int[]{1,2})caseRun(cases,s,target,power,"confirmed_cancel",i);caseRun(cases,s,target,1,"race_three",i);caseRun(cases,s,5,1,"living_portrait",i);caseRun(cases,s,target,1,"terminal_portrait",i);caseRun(cases,s,target,1,"insufficient_psi",i);caseRun(cases,s,target,1,"cannot_cast",i);caseRun(cases,s,target,1,"not_learned",i);}}JsonObject out=new JsonObject();out.addProperty("authority","Original ordinary combat Psi preflight11913e/fullcastspellprologue10b89e/internalCanCast10b966, nativelearnedbyte9f0 branch10b700..724 andauthoredSTR#1000entry1 nonemptyStringWidth, nativepacketload1314fc/selection/power/affordability/fullacceptedcharge10d1d6..10d3e8 thencompleteunconsciousportrait119db6/fullplacement/Getup/caches; powers1/2 on two actualearnedrouteprefixes before316/679 pluscancel0.66SANErefund andqualifiedrace/invalid/affordability/canCast/notlearnedcontrols. CompleteSessionprojection as frozen CombatAid, nativeprecedingGetupUIcontext restore, explicitmenu/windowinputanddrawing boundaries. Native sounds fordeclaredrow0/defaultpower1/column1/power2widget31 recipe; notuniversalhealclickaudio. Noactualinventory/HP/Psi/RNGgrant, nooldfixturechanges, nolivegameplayorcampaigndefeatcompletionclaim.");out.add("identity",identity);out.add("foundations",foundations);out.add("cases",cases);Files.writeString(root.resolve("evidence/fixtures/learned-portrait-heal.json"),out+"\n");println("Learned portrait Heal exported12cases");
    }
    void execute(ActionOracleHarness h,RunResult result,long stop) throws Exception {
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
            if(pc==0x400178||pc==0x1382d0){h.ret();continue;}
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
