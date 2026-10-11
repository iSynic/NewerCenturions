import ghidra.app.script.GhidraScript;
import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.HexFormat;

/** Original opcode 54 scheduled-record edit with exact in-memory file services. */
public class VerifyScheduleEdit extends GhidraScript {
    static String hex(byte[] data){return HexFormat.of().formatHex(data);}
    static byte[] pattern(int length,int salt){byte[] result=new byte[length];for(int i=0;i<length;i++)result[i]=(byte)(i*19+salt);return result;}
    Path root;byte[] extras,schedule,scripts,party;StringBuilder cases=new StringBuilder();
    void probe(int extraId,int day,int[] controlled)throws Exception {
        ActionOracleHarness h=new ActionOracleHarness(currentProgram);
        try {
            byte[] extraFile=extras.clone(),table=schedule.clone();
            if(controlled!=null)for(int i=0;i<5;i++)ByteBuffer.wrap(extraFile).putShort(extraId*10+i*2,(short)controlled[i]);
            byte[] extra=Arrays.copyOfRange(extraFile,extraId*10,extraId*10+10),before=table.clone(),extraBefore=extraFile.clone();
            int scriptId=extraId==331?169:extraId==574?305:168,slot=extraId==331?4:extraId==574?6:3;
            byte[] record=Arrays.copyOfRange(scripts,scriptId*42,(scriptId+1)*42);
            byte[] floor=pattern(16200,17),objects=pattern(16200,31),headers=pattern(4200,47),flags=pattern(128,61),calendar=pattern(16,79);
            ByteBuffer.wrap(calendar).putShort(12,(short)day);
            h.write(0x448aa0,party);h.write(0x4085b8,floor);h.write(0x40c500,objects);h.write(0x478e42,headers);h.write(0x44258c,flags);h.write(0x4427c8,calendar);
            h.write(0x439d14,pattern(40,97));h.write(0x479eaa,record);h.write(0x6fd000,new byte[16384]);h.cpu.writeRegister("A6",0x700000);h.cpu.writeRegister("SP",0x6ffe00);h.value(0x6fffee,2,slot);h.jump(0x140336);
            int selected=0,extraCursor=0,scheduleCursor=0;boolean done=false,boundaryOnly=(Short.toUnsignedInt(ByteBuffer.wrap(extra).getShort())*40+40>table.length);StringBuilder effects=new StringBuilder();
            for(int step=0;step<1000;step++) {
                long pc=h.pc(),sp=h.sp();if(pc==0x14673c){done=true;break;}
                if(pc==0x136cfa||pc==0x136d86){selected=pc==0x136cfa?1:2;append(effects,"{\"kind\":\"filename\",\"file\":\""+(selected==1?"EDCD":"CTD3")+"\"}");h.ret();continue;}
                if(pc==0x400070){h.cpu.writeRegister("A0",selected==1?0x600000:0x600010);append(effects,"{\"kind\":\"open\",\"file\":\""+(selected==1?"EDCD":"CTD3")+"\"}");h.ret();continue;}
                if(pc==0x400078){long handle=h.pointer(sp+4);int offset=(int)h.pointer(sp+8);if(h.word(sp+12)!=0)throw new Exception("Unexpected seek origin");if(handle==0x600000)extraCursor=offset;else if(handle==0x600010)scheduleCursor=offset;else throw new Exception("Unknown seek handle");append(effects,"{\"kind\":\"seek\",\"file\":\""+(handle==0x600000?"EDCD":"CTD3")+"\",\"offset\":"+offset+"}");if(handle==0x600010&&boundaryOnly){done=true;break;}h.cpu.writeRegister("D0",0);h.ret();continue;}
                if(pc==0x400048||pc==0x400050){long handle=h.pointer(sp+16),destination=h.pointer(sp+4);int size=(int)h.pointer(sp+8),count=(int)h.pointer(sp+12);byte[] file=handle==0x600000?extraFile:handle==0x600010?table:null;int cursor=handle==0x600000?extraCursor:scheduleCursor;if(file==null||count!=1||cursor<0||cursor+size>file.length)throw new Exception("Unsupported file operation");
                    if(pc==0x400048)h.write(destination,Arrays.copyOfRange(file,cursor,cursor+size));else System.arraycopy(h.read(destination,size),0,file,cursor,size);
                    append(effects,"{\"kind\":\""+(pc==0x400048?"read":"write")+"\",\"file\":\""+(handle==0x600000?"EDCD":"CTD3")+"\",\"offset\":"+cursor+",\"size\":"+size+"}");if(handle==0x600000)extraCursor+=size;else scheduleCursor+=size;h.cpu.writeRegister("D0",1);h.ret();continue;}
                if(pc==0x400068){long handle=h.pointer(sp+4);append(effects,"{\"kind\":\"close\",\"file\":\""+(handle==0x600000?"EDCD":"CTD3")+"\"}");h.cpu.writeRegister("D0",0);h.ret();continue;}
                if(pc==0x153ca4)throw new Exception("Unexpected RNG");
                if(!h.cpu.step(monitor))throw new Exception("Schedule edit at "+Long.toHexString(pc)+": "+h.cpu.getLastError());
            }
            if(!done)throw new Exception("Schedule edit budget");
            if(!Arrays.equals(extraFile,extraBefore))throw new Exception("Extra table mutation");
            if(!Arrays.equals(party,h.read(0x448aa0,party.length))||!Arrays.equals(floor,h.read(0x4085b8,floor.length))||!Arrays.equals(objects,h.read(0x40c500,objects.length))||!Arrays.equals(headers,h.read(0x478e42,headers.length))||!Arrays.equals(flags,h.read(0x44258c,flags.length))||!Arrays.equals(calendar,h.read(0x4427c8,calendar.length)))throw new Exception("Unexpected world or party mutation");
            if(cases.length()>0)cases.append(',');cases.append("{\"boundary_only\":").append(boundaryOnly).append(",\"file_cursor_after\":").append(scheduleCursor).append(",\"extra_id\":").append(extraId).append(",\"controlled\":").append(controlled!=null).append(",\"day\":").append(day).append(",\"extra\":\"").append(hex(extra)).append("\",\"script\":\"").append(hex(record)).append("\",\"before\":\"").append(hex(before)).append("\",\"after\":\"").append(hex(table)).append("\",\"scratch_after\":\"").append(hex(h.read(0x439d14,40))).append("\",\"effects\":[").append(effects).append("],\"rng_draws\":0,\"unchanged_world_and_party\":true}");
        }finally{h.cpu.dispose();}
    }
    static void append(StringBuilder output,String event){if(output.length()>0)output.append(',');output.append(event);}
    public void run()throws Exception {
        root=Path.of(getScriptArgs()[0]);Path original=root.resolve("NewCenturions/TheNewCenturions"),scenario=original.resolve("Scenarios/The Thunder of Guns/0");extras=Files.readAllBytes(scenario.resolve("Data EDCD"));schedule=Files.readAllBytes(scenario.resolve("Data TD3"));scripts=Files.readAllBytes(scenario.resolve("Data ED3"));party=new byte[6*3138];String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};for(int i=0;i<6;i++)System.arraycopy(Files.readAllBytes(original.resolve("Warriors/"+names[i])),0,party,i*3138,3138);
        for(int index:new int[]{-1,-32768,12,32767})probe(330,1,new int[]{index,100,0,1,2});
        int[] days={-32768,-1,0,1,32767};for(int extra:new int[]{330,331,574})for(int day:days)probe(extra,day,null);
        for(int index:new int[]{0,1,11})for(int[] fields:new int[][]{{-1,-1,0,-1},{0,0,0,0},{32767,32767,-32768,32767},{-32768,-32768,1,-32768},{-1,-1,0,32767},{123,-1,0,-1},{-1,234,0,-1},{123,234,0,-1}})for(int day:days)probe(330,day,new int[]{index,fields[0],fields[1],fields[2],fields[3]});
        StringBuilder identity=new StringBuilder();String[][] inputs={{"probe",getScriptArgs()[2]},{"harness",root.resolve("tools/ghidra/ActionOracleHarness.java").toString()},{"mapped_binary",root.resolve("analysis/ghidra/startup/code1-mapped.bin").toString()},{"extras",scenario.resolve("Data EDCD").toString()},{"schedule",scenario.resolve("Data TD3").toString()},{"scripts",scenario.resolve("Data ED3").toString()}};for(String[] input:inputs){if(identity.length()>0)identity.append(',');identity.append('"').append(input[0]).append("_sha256\":\"").append(ActionOracleHarness.sha256(Path.of(input[1]))).append('"');}for(int i=0;i<6;i++)identity.append(",\"warrior").append(i).append("_sha256\":\"").append(ActionOracleHarness.sha256(original.resolve("Warriors/"+names[i]))).append('"');
        Files.writeString(Path.of(getScriptArgs()[1]),"{\"authority\":\"Original dispatcher140336 through opcode54 block145c82..145d74 and complete loadextracode1383e6..138458 execute. Exact virtual file services serve authored EDCD and cloned authored TD3 as temporary CTD3. Original script168slot3,169slot4,305slot6 load authored extra330/331/574 or explicitly controlled five-word overrides; full480byte table before/after, scratch40bytes and ordered file effects captured. Signed guards and word addition tested with currentdayMIN/-1/0/1/MAX, record0/1/11 and independent chance/offset2 fields. Full original party and controlled full world/map/flags/calendar remain unchanged. Four out-of-table indicesMIN/-1/12/MAX stop at the native unsigned seek boundary before unsupported read/write, independently recording MULU offset and unchanged table. No RNG. Filename builders acknowledged; no disk originals written or live original observation.\",\"identity\":{"+identity+"},\"party\":\""+hex(party)+"\",\"cases\":["+cases+"]}\n");println("139 original scheduled-record edit cases exported");
    }
}
