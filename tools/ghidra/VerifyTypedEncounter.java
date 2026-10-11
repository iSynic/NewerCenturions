import ghidra.app.script.GhidraScript;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.charset.StandardCharsets;
import java.util.Arrays;
import java.util.HexFormat;

public class VerifyTypedEncounter extends GhidraScript {
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]);
        Path source=root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data ED2");
        byte[] table=Files.readAllBytes(source),extras=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data EDCD"));
        StringBuilder passwords=new StringBuilder(),dispatch=new StringBuilder(),repeats=new StringBuilder(),questions=new StringBuilder(),controls=new StringBuilder(),saves=new StringBuilder();
        String[] texts={"wildriver 7","WILDRIVER 7","wildriver","wildriver wrong","wild rive","bad","feed me","FEED whatever","br549","BR549more","burn baby burn","burn"};
        for(int n=0;n<texts.length;n++) {
            int id=n<6?5:n<8?1:n<10?2:3;
            byte[] record=Arrays.copyOfRange(table,id*516,id*516+516),expected=new byte[256],input=new byte[256];
            int length=record[476]&255;System.arraycopy(record,477,expected,0,length);byte[] text=texts[n].getBytes(StandardCharsets.US_ASCII);System.arraycopy(text,0,input,0,text.length);
            ActionOracleHarness h=new ActionOracleHarness(currentProgram);
            try {
                frame(h);h.write(0x443396,Arrays.copyOf(record,156));h.write(0x438aa4,input);h.write(0x6fff00,expected);h.cpu.writeRegister("D6",1);h.cpu.writeRegister("D4",0);h.jump(0x121b4a);
                for(int steps=0;steps<5000;steps++) {
                    if(h.pc()==0x122b02)break;
                    if(h.pc()==0x121ba8){h.jump(0x122880);continue;}
                    if(h.pc()==0x400040){int value=h.word(h.sp()+4);h.cpu.writeRegister("D0",value>='A'&&value<='Z'?value+32:value);h.ret();continue;}
                    if(h.pc()==0x160a8e||h.pc()==0x10f186){h.ret();continue;}
                    int op=h.word(h.pc())&65535;if(op==0xaa1c||op==0xa983){h.pop(4);h.jump(h.pc()+2);continue;}
                    step(h);
                }
                if(h.pc()!=0x122b02)throw new Exception("Password budget");
                if(passwords.length()>0)passwords.append(',');passwords.append("{\"record\":\"").append(hex(record)).append("\",\"input\":\"").append(hex(text)).append("\",\"selector\":").append((short)h.cpu.readRegister("D0").intValue()).append(",\"matched\":").append(h.cpu.readRegister("D6").intValue()!=0).append('}');
            } finally {h.cpu.dispose();}
        }
        for(int count:new int[]{-1,0,1,2,3,99}) {
            ActionOracleHarness h=new ActionOracleHarness(currentProgram);
            try {frame(h);h.value(0x447198,2,2);h.value(0x442bce,2,count);h.jump(0x146782);for(int s=0;s<100;s++){if(h.pc()==0x142e72||h.pc()==0x1467a0)break;step(h);}if(h.pc()!=0x142e72&&h.pc()!=0x1467a0)throw new Exception("Repeat budget");
                if(repeats.length()>0)repeats.append(',');repeats.append("{\"before\":").append(count).append(",\"after\":").append(h.word(0x442bce)).append(",\"repeat\":").append(h.pc()==0x142e72).append('}');
            } finally {h.cpu.dispose();}
        }
        for(int n=0;n<2;n++) {
            ActionOracleHarness h=new ActionOracleHarness(currentProgram);
            try {frame(h);h.value(0x4433f7,1,n==0?1:0);h.jump(0x121ae0);int prompts=0;for(int s=0;s<100;s++){if(h.pc()==0x1218fc)break;if(h.pc()==0x127132){prompts++;h.ret();continue;}if(h.pc()==0x133a18||h.pc()==0x10f186){h.ret();continue;}if(h.pc()==0x400088){h.cpu.writeRegister("D0",0);h.ret();continue;}step(h);}if(h.pc()!=0x1218fc)throw new Exception("Empty/disabled budget");
                if(controls.length()>0)controls.append(',');controls.append("{\"name\":\"").append(n==0?"empty-input":"disabled-input").append("\",\"retry\":true,\"prompts\":").append(prompts).append('}');
            } finally {h.cpu.dispose();}
        }
        for(int id:new int[]{2,5}) {
            byte[] record=Arrays.copyOfRange(table,id*516,id*516+156);ActionOracleHarness h=new ActionOracleHarness(currentProgram);
            try {h.write(0x443396,record);h.enter(0x152dc4,0x700000,0x900000);h.value(0x700004,2,id);h.value(0x700006,2,2);long offset=-1;byte[] written=null;
                for(int s=0;s<1000;s++){long pc=h.pc(),stack=h.sp();if(pc==0x900000)break;if(pc==0x136d86||pc==0x400068){h.ret();continue;}if(pc==0x400070){h.cpu.writeRegister("A0",0x600000);h.ret();continue;}if(pc==0x400078){offset=h.pointer(stack+8);h.ret();continue;}if(pc==0x400050){if(h.pointer(stack+8)!=156||h.pointer(stack+12)!=1)throw new Exception("Typed save size");written=h.read(h.pointer(stack+4),156);h.cpu.writeRegister("D0",1);h.ret();continue;}step(h);}if(h.pc()!=0x900000||written==null)throw new Exception("Typed save budget");
                if(saves.length()>0)saves.append(',');saves.append("{\"id\":").append(id).append(",\"offset\":").append(offset).append(",\"record\":\"").append(hex(record)).append("\",\"written\":\"").append(hex(written)).append("\"}");
            } finally {h.cpu.dispose();}
        }
        String cancelCase;
        {ActionOracleHarness h=new ActionOracleHarness(currentProgram);byte[] original=pattern(42,1),working=pattern(42,71);
            try {frame(h);h.write(0x6fff5e,original);h.write(0x479eaa,working);h.value(0x4471ea,2,0);h.value(0x447198,2,2);h.value(0x4427c2,2,2);h.value(0x6fff5c,2,5);h.jump(0x142eb0);int moved=0,saved=0;
                for(int s=0;s<1000;s++){long pc=h.pc();if(pc==0x14694c)break;if(pc==0x152dc4){if(h.word(h.sp()+4)!=5||h.word(h.sp()+6)!=2)throw new Exception("Typed cancel save args");saved++;h.ret();continue;}if(pc==0x13ea82){moved++;h.ret();continue;}if(pc==0x1500d8){h.ret();continue;}step(h);}if(h.pc()!=0x14694c||saved!=1||moved!=1)throw new Exception("Typed cancel budget");
                cancelCase="{\"original\":\""+hex(original)+"\",\"working\":\""+hex(working)+"\",\"restored\":\""+hex(h.read(0x479eaa,42))+"\",\"persisted\":\""+hex(h.read(0x478e42+84,42))+"\",\"menu_active\":"+h.word(0x447198)+",\"save_id\":5,\"save_mode\":2,\"moved\":"+moved+",\"return\":"+(short)h.cpu.readRegister("D0").intValue()+"}";
            } finally {h.cpu.dispose();}
        }
        for(int[] test:new int[][]{{5,1,1},{5,4,1},{5,4,2},{2,4,1},{2,4,2},{5,2,2},{5,3,1}}) {
            int id=test[0],selected=test[1],remaining=test[2];byte[] record=Arrays.copyOfRange(table,id*516,id*516+516),script=pattern(42,7);
            ActionOracleHarness h=new ActionOracleHarness(currentProgram);
            try {frame(h);h.write(0x443396,Arrays.copyOf(record,156));h.write(0x479eaa,script);h.value(0x4471ea,2,selected);h.value(0x442bce,2,remaining);h.jump(0x142f1e);bounded(h,0x14032e);
                if(dispatch.length()>0)dispatch.append(',');dispatch.append("{\"record\":\"").append(hex(record)).append("\",\"selected\":").append(selected).append(",\"remaining\":").append(remaining).append(",\"effective_selector\":").append(h.word(0x4471ea)).append(",\"before\":\"").append(hex(script)).append("\",\"after\":\"").append(hex(h.read(0x479eaa,42))).append("\"}");
            } finally {h.cpu.dispose();}
        }
        for(int trigger:new int[]{0,1,-1})for(int mode:new int[]{0,1,2,3,4,5})for(int answer:new int[]{0,1}) {
            byte[] extra=Arrays.copyOfRange(extras,82*10,83*10),script=pattern(42,7),replacement=pattern(42,71),menu=Arrays.copyOfRange(table,5*516,5*516+156);
            put(extra,0,trigger);put(extra,2,mode);put(extra,4,mode==1?39:2);
            ActionOracleHarness h=new ActionOracleHarness(currentProgram);
            try {
                frame(h);h.write(0x44330a,extra);h.write(0x443396,menu);h.write(0x443432,menu);h.write(0x479eaa,script);h.jump(0x14299a);int moved=0,loaded=-1;
                for(int s=0;s<3000;s++) {
                    long pc=h.pc();if(pc==0x14673c||pc==0x14032e||pc==0x14694c)break;
                    if(pc==0x1383e6||pc==0x1500d8){h.ret();continue;}
                    if(pc==0x14d21a){h.cpu.writeRegister("D0",answer);h.ret();continue;}
                    if(pc==0x13ea82){moved++;h.ret();continue;}
                    if(pc==0x123c60){loaded=h.word(h.sp()+4);h.write(0x479eb2,Arrays.copyOfRange(replacement,8,42));h.ret();continue;}
                    if((h.word(pc)&65535)==0xa873){h.pop(4);h.jump(pc+2);continue;}
                    step(h);
                }
                if(h.pc()!=0x14673c&&h.pc()!=0x14032e&&h.pc()!=0x14694c)throw new Exception("Question budget");
                if(questions.length()>0)questions.append(',');questions.append("{\"extra\":\"").append(hex(extra)).append("\",\"answer\":").append(answer!=0).append(",\"moved\":").append(moved).append(",\"loaded\":").append(loaded).append(",\"cursor\":").append(h.word(0x6fffee)).append(",\"stop\":\"").append(Long.toHexString(h.pc())).append("\",\"before\":\"").append(hex(script)).append("\",\"after\":\"").append(hex(h.read(0x479eaa,42))).append("\",\"branch_record\":\"").append(hex(menu)).append("\"}");
            } finally {h.cpu.dispose();}
        }
        String identity="\"typed_sha256\":\""+ActionOracleHarness.sha256(source)+"\",\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\",\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyTypedEncounter.java"))+"\",\"harness_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/ActionOracleHarness.java"))+"\"";
        Files.writeString(root.resolve("evidence/fixtures/typed-encounter.json"),"{\"authority\":\"Bounded encounter2 expected-text comparison121b4a..121ba8 and selector/return122880..122b02, with ASCII lowercase C service modeled; expected/input strings loaded directly after original Pascal conversion boundary. Developer-command dispatcher skipped explicitly. Empty/disabled121ae0..1218fc retry with typed UI and strlen boundaries. Originalnewlandtypedbranch142f1e..142fa8 and repeat146782..1467a0 execute; cancel142eb0..14694c restores/persists original event with saveencounter/movement boundaries. Separate saveencounter152dc4 executes156bytewritesat encounter*516 with stdio interception. Opcode3question14299a..142b0e executes with answer UI, loadextracode, loaddoor2 and movement boundaries intercepted. No full encounter UI, non-ASCII conversion, or live-play claim.\",\"cache_identity\":{"+identity+"},\"rng_draws\":0,\"password_cases\":["+passwords+"],\"dispatch_cases\":["+dispatch+"],\"repeat_cases\":["+repeats+"],\"question_cases\":["+questions+"],\"control_cases\":["+controls+"],\"save_cases\":["+saves+"],\"cancel_case\":"+cancelCase+"}\n");
        println("Typed encounter cases exported");
    }
    void frame(ActionOracleHarness h){h.write(0x6ff000,new byte[8192]);h.cpu.writeRegister("A6",0x700000);h.cpu.writeRegister("SP",0x700500);}
    void bounded(ActionOracleHarness h,long stop)throws Exception{for(int s=0;s<5000;s++){if(h.pc()==stop)return;step(h);}throw new Exception("Instruction budget");}
    void step(ActionOracleHarness h)throws Exception{if(!h.cpu.step(monitor))throw new Exception("PC "+Long.toHexString(h.pc())+": "+h.cpu.getLastError());}
    static void put(byte[] raw,int offset,int value){raw[offset]=(byte)(value>>8);raw[offset+1]=(byte)value;}
    static byte[] pattern(int count,int start){byte[] raw=new byte[count];for(int i=0;i<count;i++)raw[i]=(byte)(start+i);return raw;}
    static String hex(byte[] raw){return HexFormat.of().formatHex(raw);}
}
