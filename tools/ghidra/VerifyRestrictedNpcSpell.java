import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.HexFormat;
import java.security.MessageDigest;
import com.google.gson.*;

/** Original restricted spell resolution using the supplied item cache alias. */
public class VerifyRestrictedNpcSpell extends VerifySamplesEnemySpell {
    static final long BIT_CODE=0x800000;
    static final byte[] BIT_BYTES=HexFormat.of().parseHex("6120225f0131180056d744174ed06112225f01f118004ed06108225f01b118004ed0225f205f221f3001e68146404ed1");
    JsonArray bitCalls=new JsonArray();
    String sha(byte[] b)throws Exception{return HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(b));}
    void nativeBit(ActionOracleHarness h,long returnPc)throws Exception {
        long inputSp=h.sp();int index=(int)h.pointer(inputSp);long pointer=h.pointer(inputSp+4);
        h.write(BIT_CODE,BIT_BYTES);h.value(inputSp-4,4,returnPc);h.cpu.writeRegister("SP",inputSp-4);h.jump(BIT_CODE);
        JsonObject j=new JsonObject();j.addProperty("trap_return",Long.toHexString(returnPc));j.addProperty("index",index);j.addProperty("pointer",Long.toHexString(pointer));j.addProperty("source_bytes",hex(BIT_BYTES));JsonArray trace=new JsonArray();
        for(int step=0;step<40;step++){
            long pc=h.pc();if(pc==returnPc){j.addProperty("result",h.read(inputSp+8,1)[0]&255);j.addProperty("stack_after",Long.toHexString(h.sp()));j.add("trace",trace);bitCalls.add(j);if(h.sp()!=inputSp+8)throw new Exception("BitTst original stack cleanup");return;}
            JsonObject t=new JsonObject();t.addProperty("pc",Long.toHexString(pc));t.addProperty("opcode",Integer.toHexString(h.word(pc)&65535));t.addProperty("d0",h.cpu.readRegister("D0").intValue());t.addProperty("d1",h.cpu.readRegister("D1").intValue());t.addProperty("a1",h.cpu.readRegister("A1").intValue());t.addProperty("sp",Long.toHexString(h.sp()));
            if(pc==BIT_CODE+4){long effective=h.cpu.readRegister("A1").longValue()+h.cpu.readRegister("D1").intValue();j.addProperty("effective_address",Long.toHexString(effective));j.addProperty("effective_byte",h.read(effective,1)[0]&255);j.addProperty("byte_displacement",h.cpu.readRegister("D1").intValue());j.addProperty("machine_bit",h.cpu.readRegister("D0").intValue()&7);}
            trace.add(t);if(!h.cpu.step(monitor))throw new Exception("Historical BitTst "+Long.toHexString(pc)+": "+h.cpu.getLastError());
        }throw new Exception("Historical BitTst instruction budget");
    }
    JsonArray bitControls()throws Exception {
        JsonArray out=new JsonArray();int[] indices={-9,-8,-1,0,2,7,8,15},offsets={-2,-1,-1,0,0,0,1,1},masks={1,128,1,128,32,1,128,1};
        for(int k=0;k<indices.length;k++)for(boolean set:new boolean[]{false,true}){ActionOracleHarness h=frame();try{byte[] input=new byte[4];input[2+offsets[k]]=(byte)(set?masks[k]:255^masks[k]);h.write(0x810000,input);h.cpu.writeRegister("SP",0x6ffe00);h.value(0x6ffe00,4,indices[k]);h.value(0x6ffe04,4,0x810002);h.value(0x6ffe08,1,0x55);bitCalls=new JsonArray();nativeBit(h,0x900000);JsonObject j=bitCalls.get(0).getAsJsonObject();j.addProperty("input_bytes",hex(input));j.addProperty("expected_offset",offsets[k]);j.addProperty("expected_mask",masks[k]);j.addProperty("expected_result",set?1:0);if(j.get("result").getAsInt()!=(set?1:0)||j.get("byte_displacement").getAsInt()!=offsets[k])throw new Exception("Independent BitTst control");out.add(j);}finally{h.cpu.dispose();}}
        return out;
    }
    @Override void observe(ActionOracleHarness h,RunResult r,long pc)throws Exception {
        super.observe(h,r,pc);
        if(pc==0x150130||pc==0x15017e||pc==0x150588||pc==0x1505b2||pc==0x1505d0||pc==0x1505fe||pc==0x150748){JsonObject e=new JsonObject();e.addProperty("address",Long.toHexString(pc));e.addProperty("body",(short)h.cpu.readRegister("D4").intValue());e.addProperty("d0",h.cpu.readRegister("D0").intValue());e.addProperty("draws",r.draws);e.addProperty("packet_type",h.read(0x443761,1)[0]&255);e.addProperty("packet_flags",h.read(0x443757,1)[0]&255);if(pc==0x1505b2)e.addProperty("tested_signed_byte",(int)h.read(h.cpu.readRegister("A0").longValue()+0x8b,1)[0]);nativeCalls.add(e);}
    }
    @Override JsonObject completeSnap(ActionOracleHarness h,RunResult r)throws Exception {JsonObject j=super.completeSnap(h,r);j.addProperty("npc18_alias_word",h.word(0x456776));j.addProperty("first_item_cache",hex(h.read(0x4537fe,30000)));j.addProperty("first_item_cache_digest",sha(h.read(0x4537fe,30000)));j.addProperty("restriction_preceding_byte",h.read(0x44377f,1)[0]&255);return j;}
    @Override void initializeActual(ActionOracleHarness h,JsonObject s)throws Exception {super.initializeActual(h,s);h.value(0x45365c,2,s.get("loaded_item_field32").getAsInt());h.value(0x442739,1,s.getAsJsonObject("combat").get("unavailable").getAsInt());}
    void npcProbe(JsonArray out,JsonObject s,long seed,boolean turn,int preceding)throws Exception {
        reset(seed);nativeCalls=new JsonArray();bitCalls=new JsonArray();ActionOracleHarness h=frame();RunResult r=new RunResult(seed);try{initializeActual(h,s);r.draws=s.getAsJsonObject("random").get("draws").getAsInt();JsonObject j=new JsonObject();j.addProperty("complete_turn",turn);j.addProperty("seed_before",seed);j.addProperty("preceding_byte_control",preceding);if(!turn){select(h,2207,3);h.value(0x4434e0+18*2,2,1);h.value(0x438760,2,1);if(preceding>=0)h.value(0x44377f,1,preceding);}j.add("before",completeSnap(h,r));if(turn){if(h.sp()>h.cpu.readRegister("A6").longValue()-0x3c)throw new Exception("Invalid combat caller stack");h.jump(0x1145c8);execute(h,r,0x115776);}else{h.enter(0x151022,0x6ffe00,0x900000);execute(h,r,0x900000);}if(diagnostic)throw new Exception("Unproved diagnostic "+diagnosticSpell);j.add("after",completeSnap(h,r));j.add("casts",JsonParser.parseString("["+calls+"]"));j.add("resolutions",JsonParser.parseString("["+resolutions+"]"));j.add("native_calls",nativeCalls);j.add("bit_calls",bitCalls);j.add("rng",JsonParser.parseString("["+r.drawRecords+"]"));j.add("sounds",JsonParser.parseString("["+r.sounds+"]"));out.add(j);println("Restricted NPC turn="+turn+" seed="+seed+" preceding="+preceding);}finally{h.cpu.dispose();}
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
            if(pc==0x159192){displaySpell(h,result);continue;}
            if(pc==0x158454){showresultsSound(h,result);h.ret();continue;}
            if(pc==0x1600fc||pc==0x120c14||pc==0x105572||pc==0x10e6b6||pc==0x133d46||pc==0x13154c||pc==0x11f1a8||pc==0x11edc4||pc==0x14a02c||pc==0x13a2c4||pc==0x1205f8||pc==0x16043e||pc==0x1382a8||pc==0x11f136||pc==0x1537ee||pc==0x12382a||pc==0x160fb2||pc==0x10ff54||pc==0x12f040||pc==0x15edfa){h.ret();continue;}
            int op=h.word(pc)&65535;
            if(op==0xa85d){nativeBit(h,pc+2);continue;}
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
        Path root=Path.of(getScriptArgs()[0]),original=root.resolve("NewCenturions/TheNewCenturions"),scenario=original.resolve("Scenarios/The Thunder of Guns/0");items=Files.readAllBytes(original.resolve("Data Files/Data ID"));scenarioItems=Files.readAllBytes(scenario.resolve("Data NI"));races=Files.readAllBytes(original.resolve("Data Files/Data Race"));monsters=Files.readAllBytes(scenario.resolve("Data MD"));properties=Files.readAllBytes(original.resolve("Data Files/Data Swamp BD"));attacks=Files.readAllBytes(original.resolve("Data Files/Data S"));areas=Files.readAllBytes(original.resolve("Data Files/Data AD"));
        String input="analysis/completion/archive/earned-armor-fixed-route/samples-failure-1.json";JsonObject s=JsonParser.parseString(Files.readString(root.resolve(input))).getAsJsonObject().getAsJsonObject("session"),identity=JsonParser.parseString(Files.readString(root.resolve("evidence/fixtures/samples-enemy-spell.json"))).getAsJsonObject().getAsJsonObject("identity").deepCopy();identity.addProperty("samples_probe_sha256",identity.get("probe_sha256").getAsString());identity.addProperty("probe_sha256",ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyRestrictedNpcSpell.java")));identity.addProperty("samples_fixture_sha256",ActionOracleHarness.sha256(root.resolve("evidence/fixtures/samples-enemy-spell.json")));JsonObject foundation=new JsonObject();foundation.addProperty("input",input);foundation.addProperty("digest",ActionOracleHarness.sha256(root.resolve(input)));foundation.add("session",s);JsonArray controls=bitControls(),cases=new JsonArray();npcProbe(cases,s,s.getAsJsonObject("random").get("seed").getAsLong(),true,-1);npcProbe(cases,s,s.getAsJsonObject("random").get("seed").getAsLong(),false,-1);npcProbe(cases,s,12,false,-1);npcProbe(cases,s,12,false,1);
        JsonObject out=new JsonObject();out.addProperty("authority","Four original restricted2207 Samples NPC18 cases: complete actual body21 decision1145c8..115776 and three full151022 resolution controls, two seeds and explicit precedingpacketbyte49=1. Exact actualDataID firstcache30000 loadedunchanged; nativebody18alias456776 equalsoriginalitem81word2=0. HistoricalAppleQuickDraw1984 BitTst source hand-encoded and executed inprivate emulator memory, including16independent signed/positiveindex controls; notliveClassicOS/ROM equivalence claim. Fullparty/14NPCs/map/order/work150/fields/packet/mask/targets/RNG/audio captured. Foundationarchived andembedded. Unserializedworkbytes andpresentationfeedback neutral; correctoriginalcallerframeA6=700000/SP6ffe00. No getup/campaigncompletion/broadNPCalias claim.");out.add("identity",identity);out.add("foundation",foundation);JsonObject boundary=new JsonObject();boundary.addProperty("source_url","https://github.com/ComputerHistoryMuseum/quickdraw/blob/37332b4bb448199e27a9a9a082a1f664c15b63a1/102658076_quickdraw_acc/QuickDraw%20Source/Util.a#L74-L116");boundary.addProperty("source_digest",ActionOracleHarness.sha256(root.resolve("analysis/completion/quickdraw-original-util.a")));boundary.addProperty("assembled_bytes",hex(BIT_BYTES));boundary.add("controls",controls);out.add("historical_bit_boundary",boundary);out.add("cases",cases);Files.writeString(root.resolve("evidence/fixtures/restricted-npc-spell.json"),out+"\n");println("Restricted NPC exported:4actions16BitTstcontrols");
    }
}
