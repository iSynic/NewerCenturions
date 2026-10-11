import ghidra.app.script.GhidraScript;
import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.HexFormat;

public class VerifyManualEncounter extends GhidraScript {
    static String hex(byte[] raw){return HexFormat.of().formatHex(raw);}
    static void put(byte[] raw,int at,int value){ByteBuffer.wrap(raw).putShort(at,(short)value);}
    void step(ActionOracleHarness h)throws Exception{if(!h.cpu.step(monitor))throw new Exception("Manual encounter PC "+Long.toHexString(h.pc())+": "+h.cpu.getLastError());}
    static void region(byte[] raw,int id,int chance,int[] scripts,int[] chances){
        for(int i=0;i<4;i++)put(raw,id*8+i*2,i<2?0:10);
        put(raw,160+id*2,chance);
        for(int i=0;i<3;i++){put(raw,280+id*6+i*2,scripts[i]);put(raw,400+id*6+i*2,chances[i]);}
    }
    byte[] synthetic(int variant){
        byte[] raw=new byte[644];for(int i=520;i<644;i++)raw[i]=(byte)(i*3+variant);
        if(variant==1)region(raw,19,-1,new int[]{250,0,0},new int[]{-100,0,0});
        if(variant==2){region(raw,19,-1,new int[]{251,0,0},new int[]{-100,0,0});region(raw,0,-1,new int[]{250,0,0},new int[]{-100,0,0});}
        if(variant==3)region(raw,7,-1,new int[]{250,251,0},new int[]{-100,-100,-100});
        if(variant==4)region(raw,9,-1,new int[]{250,251,252},new int[]{100,100,100});
        if(variant==5)region(raw,4,-32768,new int[]{250,251,252},new int[]{-32768,-101,32767});
        if(variant==6){region(raw,8,-1,new int[]{250,251,252},new int[]{100,100,100});put(raw,8*8+4,0);}
        if(variant==7)region(raw,3,1,new int[]{250,251,252},new int[]{100,100,100});
        if(variant==8)region(raw,15,-1,new int[]{250,251,0},new int[]{33,-67,0});
        if(variant==9)for(int r=0;r<20;r++)region(raw,r,-1,new int[]{r*3,r*3+1,r*3+2},new int[]{r%2==0?100:-100,r+1,-100});
        return raw;
    }
    String selection(byte[] before,int variant,int x,int y,int seedBefore)throws Exception{
        ActionOracleHarness h=new ActionOracleHarness(currentProgram);
        try{
            h.write(0x6fc000,new byte[24576]);h.cpu.writeRegister("A6",0x700000);h.cpu.writeRegister("SP",0x6ffe00);
            h.write(0x443086,before);h.value(0x442778,4,x);h.value(0x44277c,4,y);h.value(0x442764,4,0);h.value(0x442768,4,0);h.value(0x442774,4,7);
            h.value(0x442736,1,-1);h.value(0x442737,1,1);h.value(0x43959c,1,0);h.value(0x439438,4,0x680000);h.value(0x680000,4,0x680100);h.write(0x680100,new byte[32]);h.jump(0x10a478);
            long seed=seedBefore;int draws=0,script=-999,manual=-999,saves=0;byte[] saved=null;String args=null;boolean done=false;
            for(int budget=0;budget<100000;budget++){
                long pc=h.pc(),sp=h.sp();
                if(pc==0x10a5dc){done=true;break;}
                if(pc==0x400038){h.cpu.writeRegister("D0",(short)Math.abs(h.word(sp+4)));h.ret();continue;}
                if(pc==0x153ca4){int bound=h.word(sp+4);seed=seed*16807%2147483647;draws++;short raw=(short)seed;if(raw==Short.MIN_VALUE)raw=0;h.cpu.writeRegister("D0",Math.abs((int)raw)*bound/32768+1);h.ret();continue;}
                if(pc==0x151e24){if(h.pointer(sp+4)!=7)throw new Exception("Manual save map");saved=h.read(0x443086,644);saves++;h.ret();continue;}
                if(pc==0x14ce2a){h.ret();continue;}
                if(pc==0x1400e4){script=h.word(sp+14);manual=h.read(0x43959c,1)[0];args="["+h.pointer(sp+4)+","+h.pointer(sp+8)+","+h.word(sp+12)+","+script+","+h.word(sp+16)+"]";h.cpu.writeRegister("D0",0);h.ret();continue;}
                if((h.word(pc)&65535)==0xa8ad){long rect=h.pointer(sp);int py=h.word(sp+4),px=h.word(sp+6);boolean inside=py>=h.word(rect)&&px>=h.word(rect+2)&&py<h.word(rect+4)&&px<h.word(rect+6);h.value(sp+8,1,inside?1:0);h.pop(8);h.jump(pc+2);continue;}
                step(h);
            }
            if(!done||saves!=1||script==-999)throw new Exception("Manual selection budget or missing boundaries");
            return "{\"variant\":"+variant+",\"x\":"+x+",\"y\":"+y+",\"seed_before\":"+seedBefore+",\"seed_after\":"+seed+",\"draws\":"+draws+",\"script\":"+script+",\"before\":\""+hex(before)+"\",\"after\":\""+hex(h.read(0x443086,644))+"\",\"saved_regions\":\""+hex(saved)+"\",\"last_move_after\":["+h.read(0x442736,1)[0]+","+h.read(0x442737,1)[0]+"],\"manual_during_newland\":"+manual+",\"manual_after\":"+h.read(0x43959c,1)[0]+",\"newland_args\":"+args+"}";
        }finally{h.cpu.dispose();}
    }
    String direct(byte[] scripts,byte[] typed,int script,int response)throws Exception{
        ActionOracleHarness h=new ActionOracleHarness(currentProgram);
        try{
            h.enter(0x1400e4,0x700000,0x900000);h.value(0x700004,4,0);h.value(0x700008,4,0);h.value(0x70000c,2,1);h.value(0x70000e,2,script);h.value(0x700010,2,0);
            h.value(0x442774,4,7);h.value(0x442778,4,87);h.value(0x44277c,4,87);h.value(0x442764,4,0);h.value(0x442768,4,0);h.value(0x44274c,1,0);h.value(0x43959c,1,1);h.value(0x438776,2,0);
            h.write(0x479eaa,new byte[42]);int load=-1,typedId=-1,saves=0,moves=0,draws=0,uiCalls=0;long offset=0;byte[] expected=null,atUi=null,typedAtUi=null;boolean done=false;
            for(int budget=0;budget<10000;budget++){
                long pc=h.pc(),sp=h.sp();if(pc==0x900000){done=true;break;}
                if(pc==0x123c60){load=h.word(sp+4);byte[] record=Arrays.copyOfRange(scripts,load*42,(load+1)*42);System.arraycopy(h.read(0x479eaa,8),0,record,0,8);h.write(0x479eaa,record);expected=record.clone();h.ret();continue;}
                if(pc==0x133c56||pc==0x136d86||pc==0x400068||pc==0x1500d8||pc==0x161b80){h.ret();continue;}
                if(pc==0x400070){h.cpu.writeRegister("A0",0x600000);h.ret();continue;}
                if(pc==0x400078){offset=h.pointer(sp+8);typedId=(int)(offset/516);h.ret();continue;}
                if(pc==0x400048){int amount=(int)(h.pointer(sp+8)*h.pointer(sp+12));h.write(h.pointer(sp+4),Arrays.copyOfRange(typed,(int)offset,(int)offset+amount));offset+=amount;h.cpu.writeRegister("D0",1);h.ret();continue;}
                if(pc==0x400178){h.ret();continue;}
                if(pc==0x12185a){uiCalls++;atUi=h.read(0x479eaa,42);typedAtUi=h.read(0x443396,156);h.cpu.writeRegister("D0",response);h.ret();continue;}
                if(pc==0x152dc4){if(h.word(sp+4)!=typedId||h.word(sp+6)!=2)throw new Exception("Manual typed save arguments");saves++;h.ret();continue;}
                if(pc==0x13ea82){if(h.word(sp+4)!=-1)throw new Exception("Manual typed cancel move");moves++;h.ret();continue;}
                if(pc==0x153ca4){draws++;throw new Exception("Direct mode unexpectedly draws RNG");}
                step(h);
            }
            if(!done||uiCalls!=1||load!=script||typedId!=(script==0?0:8))throw new Exception("Direct manual script budget or dispatch");
            return "{\"script\":"+script+",\"mode\":1,\"load_script\":"+load+",\"typed_id\":"+typedId+",\"ui_response\":"+response+",\"draws\":"+draws+",\"result\":"+(short)h.cpu.readRegister("D0").intValue()+",\"save_calls\":"+saves+",\"move_calls\":"+moves+",\"record_after\":\""+hex(h.read(0x479eaa,42))+"\",\"record_at_ui\":\""+hex(atUi)+"\",\"typed_record\":\""+hex(typedAtUi)+"\",\"expected_loaded_record\":\""+hex(expected)+"\"}";
        }finally{h.cpu.dispose();}
    }
    public void run()throws Exception{
        Path root=Path.of(getScriptArgs()[0]);Path base=root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0");
        byte[] authored=Files.readAllBytes(base.resolve("Data RD")),scripts=Files.readAllBytes(base.resolve("Data ED3")),typed=Files.readAllBytes(base.resolve("Data ED2"));
        StringBuilder cases=new StringBuilder(),directCases=new StringBuilder();int total=0;
        for(int variant=0;variant<10;variant++)for(int[] p:new int[][]{{5,5},{0,0},{9,9},{10,5},{5,10},{-1,5},{65541,65541},{-65531,-65531}})for(int seed:new int[]{12,1}){
            if(cases.length()>0)cases.append(',');cases.append(selection(synthetic(variant),variant,p[0],p[1],seed));total++;
        }
        for(int[] p:new int[][]{{87,87},{84,84},{88,88},{89,88},{88,89},{85,87}})for(int seed:new int[]{12,1}){if(cases.length()>0)cases.append(',');cases.append(selection(Arrays.copyOfRange(authored,7*644,8*644),107,p[0],p[1],seed));total++;}
        byte[] oneUse=synthetic(4);for(int invocation=0;invocation<3;invocation++){String value=selection(oneUse,204+invocation,5,5,12);if(cases.length()>0)cases.append(',');cases.append(value);total++;for(int slot=0;slot<3;slot++)put(oneUse,400+9*6+slot*2,0);}
        for(int script:new int[]{0,250})for(int response:new int[]{0,-1}){if(directCases.length()>0)directCases.append(',');directCases.append(direct(scripts,typed,script,response));}
        String identity="\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyManualEncounter.java"))+"\",\"harness_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/ActionOracleHarness.java"))+"\",\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\",\"regions_sha256\":\""+ActionOracleHarness.sha256(base.resolve("Data RD"))+"\",\"scripts_sha256\":\""+ActionOracleHarness.sha256(base.resolve("Data ED3"))+"\",\"typed_sha256\":\""+ActionOracleHarness.sha256(base.resolve("Data ED2"))+"\"";
        Files.writeString(root.resolve("evidence/fixtures/manual-encounter.json"),"{\"authority\":\"Original Encounter control selector buttonchoice10a478..10a5dc executes all descending region scans, three ordered slot RNG calls, last successful script replacement, positive chance clearing, save-before-dispatch, movement clearing and manual flag writes. PtInRect uses documented exclusive right/bottom semantics; QuickDraw Rand and abs-short services modeled from separately verified arithmetic. Save and presentation boundaries recorded; newland caller arguments captured. Separate original newland full mode1 entry loads actual ED3 script0/250, bypasses header chance, serves exact ED2 bytes at stdio, then supplies typed UI cancellation0 or-1 and executes complete cancel cleanup. Script0 dispatches typed0; it is not a no-op. Typed UI, save service and movement service are boundaries, not live gameplay proof.\",\"cache_identity\":{"+identity+"},\"cases\":["+cases+"],\"direct_cases\":["+directCases+"]}\n");println("Manual encounters "+total+" selection and four direct cases exported");
    }
}
