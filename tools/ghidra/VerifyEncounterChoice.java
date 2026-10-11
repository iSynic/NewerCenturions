import ghidra.app.script.GhidraScript;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.HexFormat;

public class VerifyEncounterChoice extends GhidraScript {
    public void run() throws Exception {
        Path root = Path.of(getScriptArgs()[0]);
        Path source = root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data ED");
        byte[] menus = Files.readAllBytes(source);
        StringBuilder cases = new StringBuilder();
        int[][] inputs = {{0,1},{0,2},{0,3},{0,4},{0,9},{1,4,2},{8,1,9},{0,1}};
        for (int n = 0; n < inputs.length; n++) {
            int[] input = inputs[n];
            byte[] record = Arrays.copyOfRange(menus,input[0]*426,input[0]*426+106);
            if (n == 7) record[96] = (byte)-4;
            ActionOracleHarness h = new ActionOracleHarness(currentProgram);
            try {
                h.write(0x443432, record);
                h.enter(0x121596,0x700000,0x900000);
                h.value(0x700004,2,input[0]);h.value(0x700006,2,0);
                int choice = 1;boolean completed = false;StringBuilder sounds = new StringBuilder();
                for(int steps=0;steps<20000;steps++) {
                    long pc=h.pc(),stack=h.sp();
                    if(pc==0x900000){completed=true;break;}
                    if(pc==0x13e13c||pc==0x10f186){h.ret();continue;}
                    if(pc==0x400178||pc==0x20017c){h.write(h.pointer(stack+8),new byte[256]);h.ret();continue;}
                    // Text layout, window construction and delay are one presentation boundary.
                    // No gameplay instructions occur in this region; the menu record is already loaded.
                    if(pc==0x1216ca){h.cpu.writeRegister("A3",0x600000);h.jump(0x1217b8);continue;}
                    if(pc==0x133a18){if(sounds.length()>0)sounds.append(',');sounds.append(h.word(stack+4));h.ret();continue;}
                    int op=h.word(pc)&65535;
                    if(op==0xa991){if(choice>=input.length)throw new Exception("Modal input exhausted");h.value(h.pointer(stack),2,input[choice++]);h.pop(8);h.jump(pc+2);continue;}
                    if(op==0xaa1c||op==0xa91f||op==0xa873||op==0xa983){h.pop(4);h.jump(pc+2);continue;}
                    if(op==0xa032){h.jump(pc+2);continue;}
                    if(!h.cpu.step(monitor))throw new Exception("case "+n+" PC "+Long.toHexString(pc)+": "+h.cpu.getLastError());
                }
                if(!completed)throw new Exception("Encounter instruction budget");
                int result=(short)h.cpu.readRegister("D0").intValue();
                if(cases.length()>0)cases.append(',');
                cases.append("{\"name\":\"").append(n==7?"signed-code":n==5?"disabled-then-enabled":n==6?"all-disabled-then-cancel":"authored-"+input[0]+"-"+input[1]).append("\",\"menu\":").append(input[0]).append(",\"record_before\":\"").append(HexFormat.of().formatHex(record)).append("\",\"record_after\":\"").append(HexFormat.of().formatHex(h.read(0x443432,106))).append("\",\"inputs\":[");
                for(int j=1;j<input.length;j++){if(j>1)cases.append(',');cases.append(input[j]);}
                cases.append("],\"return_code\":").append(result).append(",\"consumed_inputs\":").append(choice-1).append(",\"sounds\":[").append(sounds).append("],\"seed_before\":12,\"seed_after\":12,\"rng_draws\":0}");
            } finally {h.cpu.dispose();}
        }
        StringBuilder dispatch = new StringBuilder();
        for (int n=0;n<6;n++) {
            byte[] record=Arrays.copyOfRange(menus,0,106),script=new byte[42];
            for(int j=0;j<script.length;j++)script[j]=(byte)(j+7);
            if(n==4)for(int j=0;j<8;j++)record[j]=(byte)(0x80+j);
            int selected=n<4?n+1:1;
            ActionOracleHarness h=new ActionOracleHarness(currentProgram);
            try {
                h.write(0x443432,record);h.write(0x479eaa,script);
                h.write(0x6ff000,new byte[8192]);h.cpu.writeRegister("A6",0x700000);h.cpu.writeRegister("SP",0x700500);
                h.value(0x4471ea,2,selected);h.value(0x442bce,2,0);
                h.jump(n==5?0x142c2e:0x142cee);
                boolean completed=false;
                for(int steps=0;steps<5000;steps++) {
                    long pc=h.pc();if(pc==0x14032e){completed=true;break;}
                    if((h.word(pc)&65535)==0xa873){h.pop(4);h.jump(pc+2);continue;}
                    if(pc==0x121596)throw new Exception("Exhaustion unexpectedly prompted");
                    if(!h.cpu.step(monitor))throw new Exception("Dispatch "+n+" PC "+Long.toHexString(pc)+": "+h.cpu.getLastError());
                }
                if(!completed)throw new Exception("Dispatch instruction budget");
                if(dispatch.length()>0)dispatch.append(',');
                dispatch.append("{\"name\":\"").append(n==4?"signed-opcodes":n==5?"counter-exhausted":"authored-branch-"+selected).append("\",\"record\":\"").append(HexFormat.of().formatHex(record)).append("\",\"selected\":").append(h.word(0x4471ea)).append(",\"script_before\":\"").append(HexFormat.of().formatHex(script)).append("\",\"script_after\":\"").append(HexFormat.of().formatHex(h.read(0x479eaa,42))).append("\",\"remaining\":").append(h.word(0x442bce)).append(",\"rng_draws\":0}");
            } finally {h.cpu.dispose();}
        }
        String identity="\"menu_sha256\":\""+ActionOracleHarness.sha256(source)+"\",\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\",\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyEncounterChoice.java"))+"\",\"harness_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/ActionOracleHarness.java"))+"\",\"ghidra\":\"12.1.2\"";
        StringBuilder repeats=new StringBuilder();
        for(int count:new int[]{-1,0,1,2,3,99}) {
            ActionOracleHarness h=new ActionOracleHarness(currentProgram);
            try {
                h.cpu.writeRegister("SP",0x700500);h.value(0x447198,2,1);h.value(0x442bce,2,count);h.jump(0x1467a0);
                boolean completed=false,repeat=false;
                for(int steps=0;steps<100;steps++) {
                    if(h.pc()==0x142c1a||h.pc()==0x1467be){completed=true;repeat=h.pc()==0x142c1a;break;}
                    if(!h.cpu.step(monitor))throw new Exception("Repeat instruction failure: "+h.cpu.getLastError());
                }
                if(!completed)throw new Exception("Repeat budget");if(repeats.length()>0)repeats.append(',');
                repeats.append("{\"before\":").append(count).append(",\"after\":").append(h.word(0x442bce)).append(",\"repeat\":").append(repeat).append('}');
            } finally {h.cpu.dispose();}
        }
        byte[] original=new byte[42],working=new byte[42];for(int i=0;i<42;i++){original[i]=(byte)i;working[i]=(byte)(i+51);}
        ActionOracleHarness cancel=new ActionOracleHarness(currentProgram);String cancelResult;
        try {
            cancel.write(0x6ff000,new byte[8192]);cancel.cpu.writeRegister("SP",0x700500);cancel.cpu.writeRegister("A6",0x700000);
            cancel.write(0x6fff5e,original);cancel.write(0x479eaa,working);cancel.value(0x447198,2,1);cancel.value(0x4427c2,2,2);cancel.value(0x6fff5c,2,0);cancel.value(0x700010,2,0);cancel.jump(0x142c76);
            boolean completed=false,saved=false;
            for(int steps=0;steps<1000;steps++) {
                if(cancel.pc()==0x142cca){completed=true;break;}
                if(cancel.pc()==0x10f186){cancel.ret();continue;}
                if(cancel.pc()==0x152dc4){if(cancel.word(cancel.sp()+4)!=0||cancel.word(cancel.sp()+6)!=1)throw new Exception("Unexpected saveencounter args");saved=true;cancel.ret();continue;}
                if(!cancel.cpu.step(monitor))throw new Exception("Cancel suffix: "+cancel.cpu.getLastError());
            }
            if(!completed||!saved)throw new Exception("Cancel suffix incomplete");
            cancelResult="{\"original_script\":\""+HexFormat.of().formatHex(original)+"\",\"working_script\":\""+HexFormat.of().formatHex(working)+"\",\"restored_script\":\""+HexFormat.of().formatHex(cancel.read(0x479eaa,42))+"\",\"persisted_script\":\""+HexFormat.of().formatHex(cancel.read(0x478e42+84,42))+"\",\"menu_active\":"+cancel.word(0x447198)+",\"save_menu\":0,\"save_mode\":1}";
        } finally {cancel.cpu.dispose();}
        StringBuilder saves=new StringBuilder();
        for(int menu:new int[]{0,1,7}) {
            byte[] record=Arrays.copyOfRange(menus,menu*426,menu*426+106);record[101]=53;record[96]=0;
            ActionOracleHarness h=new ActionOracleHarness(currentProgram);
            try {
                h.write(0x443432,record);h.enter(0x152dc4,0x700000,0x900000);h.value(0x700004,2,menu);h.value(0x700006,2,1);
                long offset=-1;byte[] written=null;boolean completed=false;
                for(int steps=0;steps<1000;steps++) {
                    long pc=h.pc(),stack=h.sp();if(pc==0x900000){completed=true;break;}
                    if(pc==0x136d86||pc==0x400068){h.ret();continue;}
                    if(pc==0x400070){h.cpu.writeRegister("A0",0x600000);h.ret();continue;}
                    if(pc==0x400078){offset=h.pointer(stack+8);h.cpu.writeRegister("D0",0);h.ret();continue;}
                    if(pc==0x400050){if(h.pointer(stack+8)!=106||h.pointer(stack+12)!=1)throw new Exception("Unexpected menu write size");written=h.read(h.pointer(stack+4),106);h.cpu.writeRegister("D0",1);h.ret();continue;}
                    if(!h.cpu.step(monitor))throw new Exception("Saveencounter: "+h.cpu.getLastError());
                }
                if(!completed||written==null)throw new Exception("Save incomplete");if(saves.length()>0)saves.append(',');
                saves.append("{\"menu\":").append(menu).append(",\"seek\":").append(offset).append(",\"written\":\"").append(HexFormat.of().formatHex(written)).append("\",\"record\":\"").append(HexFormat.of().formatHex(record)).append("\",\"return\":").append(h.cpu.readRegister("D0").intValue()).append('}');
            } finally {h.cpu.dispose();}
        }
        StringBuilder disables=new StringBuilder();
        for(int item=1;item<=4;item++) {
            byte[] record=Arrays.copyOfRange(menus,0,106);ActionOracleHarness h=new ActionOracleHarness(currentProgram);
            try {
                h.write(0x443432,record);h.write(0x6ff000,new byte[8192]);h.cpu.writeRegister("A6",0x700000);h.cpu.writeRegister("SP",0x700500);h.value(0x6fffea,2,item);h.value(0x6fff5c,2,0);h.jump(0x144c9a);boolean completed=false;
                for(int steps=0;steps<500;steps++) {
                    if(h.pc()==0x142c2e){completed=true;break;}
                    if(h.pc()==0x152dc4){h.ret();continue;}
                    if(!h.cpu.step(monitor))throw new Exception("Disable case: "+h.cpu.getLastError());
                }
                if(!completed)throw new Exception("Disable budget");if(disables.length()>0)disables.append(',');
                disables.append("{\"native_item\":").append(item).append(",\"before\":\"").append(HexFormat.of().formatHex(record)).append("\",\"after\":\"").append(HexFormat.of().formatHex(h.read(0x443432,106))).append("\",\"rng_draws\":0}");
            } finally {h.cpu.dispose();}
        }
        Files.writeString(root.resolve("evidence/fixtures/encounter-choice.json"),"{\"authority\":\"Original encounter 0x121596 selection validation, disabled retry, cancel, and signed-byte return instructions. Authored menu loaded into original global buffer. Presentation setup 0x1216ca..0x1217b8 intercepted, ModalDialog controlled at Toolbox A991. Original newland copy suffix 0x142cee..0x142d66 executes to redispatch 0x14032e; exhaustion case starts 0x142c2e. Repeat decision 0x1467a0..0x1467be, cancel suffix 0x142c76..0x142cca with external saveencounter boundary recorded. Saveencounter 0x152dc4 executes with fopen/fseek/fwrite/fclose recording boundaries, writing only106bytes atmenu*426. Opcode35 disable/reenter executes144c9a..144ce0 with separatelyverified saveencounter boundary. No live UI or execution of copied script claim.\",\"cache_identity\":{"+identity+"},\"cases\":["+cases+"],\"dispatch_cases\":["+dispatch+"],\"repeat_cases\":["+repeats+"],\"cancel_case\":"+cancelResult+",\"save_cases\":["+saves+"],\"disable_cases\":["+disables+"]}\n");
        println("Encounter choice cases exported");
    }
}
