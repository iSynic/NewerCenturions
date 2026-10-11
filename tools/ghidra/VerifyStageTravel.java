import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;

public class VerifyStageTravel extends VerifyTravel {
    byte[] mission;
    String fileMode="map";
    int answer, questionCount, movie, branch;
    StringBuilder effects;
    void callBoundary(ActionOracleHarness h) throws Exception {
        long pc=h.pc(),sp=h.sp();
        if(pc==0x131182) {
            properties=Files.readAllBytes(originals.resolve("Data Files").resolve(terrainFile(h.word(sp+4))));fileMode="property";cursor=0;
        }
        if(pc==0x1312c0){h.jump(0x13137c);return;}
        if(pc==0x136d86){fileMode="map";h.ret();return;}
        if(pc==0x136cfa){if(h.pointer(sp+4)==0x4028c0)fileMode="mission";h.ret();return;}
        if(pc==0x400090||pc==0x400098||pc==0x400180){h.ret();return;}
        if(pc==0x400070){cursor=0;h.cpu.writeRegister("A0",0x600100);h.ret();return;}
        if(pc==0x400078){cursor=(int)h.pointer(sp+8);if(h.word(sp+12)!=0)throw new Exception("Nonabsolute stage seek");h.ret();return;}
        if(pc==0x400048||pc==0x400050) {
            int size=Math.toIntExact(h.pointer(sp+8)*h.pointer(sp+12));byte[] source=fileMode.equals("mission")?mission:fileMode.equals("property")?properties:files;
            if(cursor<0||cursor>source.length||(!fileMode.equals("property")&&cursor+size>source.length))throw new Exception("Stage file boundary "+fileMode+" "+cursor+"+"+size);
            int transferred=Math.min(size,source.length-cursor);long address=h.pointer(sp+4);
            if(pc==0x400048)h.write(address,Arrays.copyOfRange(source,cursor,cursor+transferred));
            else System.arraycopy(h.read(address,size),0,source,cursor,size);
            cursor+=transferred;h.cpu.writeRegister("D0",transferred/h.pointer(sp+8));h.ret();return;
        }
        if(pc==0x400068){h.ret();return;}
        if(pc==0x400038){h.cpu.writeRegister("D0",Math.abs(h.word(sp+4)));h.ret();return;}
        if(pc==0x14cff6){questionCount++;effect("question",h.word(sp+4));h.cpu.writeRegister("D0",answer);h.ret();return;}
        if(pc==0x133a18){effect("sound",h.word(sp+4));h.ret();return;}
        if(pc==0x15dc34){effect("dialogue",h.word(sp+6));h.ret();return;}
        if(pc==0x13ebde){movie=h.word(sp+4);effect("movie",movie);h.ret();return;}
        if(pc==0x123c60){branch=h.word(sp+4);effect("script",branch);h.ret();return;}
        if(pc==0x123d1c){effect("mission_name",h.word(0x435500));h.ret();return;}
        if(pc==0x13bb20||pc==0x10f186){h.ret();return;}
        int trap=h.word(pc)&65535;
        if(trap==0xa9bc){h.pop(2);h.value(h.sp(),4,0x600300);h.jump(pc+2);return;}
        if(trap==0xa873){h.pop(4);h.jump(pc+2);return;}
        if(trap==0xa8f6){h.pop(8);h.jump(pc+2);return;}
        if(trap==0xa939||trap==0xa93a){h.pop(6);h.jump(pc+2);return;}
        if(trap==0xa947){h.pop(10);h.jump(pc+2);return;}
        if(trap==0xa940){h.pop(8);h.jump(pc+2);return;}
        if(!h.cpu.step(monitor))throw new Exception("Stage/travel at "+Long.toHexString(pc)+": "+h.cpu.getLastError());
    }
    Path originals;
    void effect(String kind,int value){if(effects.length()>0)effects.append(',');effects.append("{\"kind\":\"").append(kind).append("\",\"value\":").append(value).append('}');}
    byte[] party() {
        byte[] result=new byte[3138*6];ByteBuffer b=ByteBuffer.wrap(result);
        for(int i=0;i<6;i++){b.putShort(i*3138+0x50,(short)(40+i));b.putShort(i*3138+0x52,(short)(100+i));b.putShort(i*3138+0x58,(short)(10+i));b.putShort(i*3138+0x5a,(short)(20+i));}
        return result;
    }
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]);originals=root.resolve("NewCenturions/TheNewCenturions");Path scenario=originals.resolve("Scenarios/The Thunder of Guns/0");
        mission=Files.readAllBytes(scenario.resolve("Data Mission"));byte[] headers=Files.readAllBytes(scenario.resolve("Data DD")),regions=Files.readAllBytes(scenario.resolve("Data RD")),land=Files.readAllBytes(scenario.resolve("Data LD"));
        byte[] authored=new byte[10*77744];
        for(int m=0;m<10;m++){int at=m*77744;System.arraycopy(headers,m*4200,authored,at,4200);at+=4200;System.arraycopy(regions,m*644,authored,at,644);at+=644+8100;System.arraycopy(land,m*64800,authored,at,64800);}
        StringBuilder stages=new StringBuilder(),travels=new StringBuilder();
        for(int[] input:new int[][]{{1,0},{1,1},{2,1},{3,1},{4,1},{6,1},{-5,0},{-1,0}})probe(root,authored,110,input,null,stages);
        for(int[] extra:new int[][]{{1,52,12,6000,224},{-1,-1,-1,0,0},{-1,0,0,0,0},{3,-1,39,0,0},{0,50,82,0,0}})probe(root,authored,45,null,extra,travels);
        String identity="\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyStageTravel.java"))+"\",\"travel_probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyTravel.java"))+"\",\"harness_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/ActionOracleHarness.java"))+"\",\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\",\"mission_sha256\":\""+digest(mission)+"\",\"headers_sha256\":\""+digest(headers)+"\",\"regions_sha256\":\""+digest(regions)+"\",\"land_sha256\":\""+digest(land)+"\",\"ghidra\":\"12.1.2\"";
        Files.writeString(root.resolve("evidence/fixtures/stage-travel.json"),"{\"authority\":\"Original newland110 block14192a and45 block143bd6. Original Displaymissionname133f3e, loadland13107a, saveland151e24, loadpixmap property reader and updatenpcmenu1046e0 execute. Exact original Data Mission and ten map records serve file services. Graphics, strings, question answer, centre/update drawing, sound/dialogue/movie and final loaddoor2 dispatch are recorded boundaries. Stage110 saves before question, positive asks question8, negative direct; restores only warriors i<lastindex5; flags and variables retained. No original movie playback, copied script or live route claim.\",\"cache_identity\":{"+identity+"},\"rng_draws\":0,\"stage_cases\":["+stages+"],\"travel_cases\":["+travels+"]}\n");println("Stage and cross-map cases exported");
    }
    void probe(Path root,byte[] authored,int opcode,int[] input,int[] extra,StringBuilder out) throws Exception {
        files=authored.clone();ActionOracleHarness h=new ActionOracleHarness(currentProgram);
        try {
            h.value(0x402f43,1,255);execute(h,0x13107a,0,originals);
            h.write(0x6fd000,new byte[16384]);h.cpu.writeRegister("A6",0x700000);h.cpu.writeRegister("SP",0x6ffe00);h.cpu.writeRegister("A4",50);h.cpu.writeRegister("A3",82);
            h.value(0x6fffea,2,opcode==110?input[0]:0);h.value(0x6fff58,2,opcode);h.value(0x6fffee,2,3);
            h.value(0x442764,4,50);h.value(0x442768,4,82);h.value(0x442774,4,0);h.value(0x442778,4,6);h.value(0x44277c,4,7);
            h.value(0x435500,2,2);h.value(0x435502,2,17);h.value(0x44273a,1,5);h.value(0x44257c,2,3);
            byte[] flags=new byte[128];for(int i=0;i<flags.length;i++)flags[i]=(byte)(i%7-3);h.write(0x44258c,flags);
            byte[] menus=new byte[20*288];for(int i=0;i<menus.length;i++)menus[i]=(byte)(i%19+1);h.write(0x440edc,menus);
            byte[] warriors=party();h.write(0x448aa0,warriors);h.value(0x4427c8,2,55);h.value(0x4427ca,2,12);h.value(0x4427d4,2,3);h.value(0x442578,2,4);
            h.value(0x4085b8,2,1234);byte[] mapBefore=packed(h),timeBefore=h.read(0x4427c8,16),stateBefore=h.read(0x442764,40),extraBytes=new byte[10];
            if(opcode==45){ByteBuffer b=ByteBuffer.wrap(extraBytes);for(int value:extra)b.putShort((short)value);h.write(0x44330a,extraBytes);}
            answer=opcode==110?input[1]:0;questionCount=0;movie=0;branch=0;effects=new StringBuilder();fileMode="map";cursor=0;h.jump(opcode==110?0x14192a:0x143bd6);boolean done=false;
            for(int step=0;step<100000;step++){long pc=h.pc();if(pc==0x14673c||pc==0x14032e||pc==0x14694c){done=true;break;}callBoundary(h);}
            if(!done)throw new Exception("Stage/travel budget");
            if(out.length()>0)out.append(',');out.append("{\"parameter\":").append(opcode==110?input[0]:0).append(",\"answer\":").append(answer).append(",\"extra\":\"").append(hex(extraBytes)).append("\",\"before\":\"").append(hex(warriors)).append("\",\"after\":\"").append(hex(h.read(0x448aa0,warriors.length))).append("\",\"flags_before\":\"").append(hex(flags)).append("\",\"flags_after\":\"").append(hex(h.read(0x44258c,128))).append("\",\"menu_before\":\"").append(hex(menus)).append("\",\"menu_after\":\"").append(hex(h.read(0x440edc,menus.length))).append("\",\"time_before\":\"").append(hex(timeBefore)).append("\",\"time_after\":\"").append(hex(h.read(0x4427c8,16))).append("\",\"state_before\":\"").append(hex(stateBefore)).append("\",\"state_after\":\"").append(hex(h.read(0x442764,40))).append("\",\"map_before\":\"").append(hex(mapBefore)).append("\",\"map_after\":\"").append(hex(packed(h))).append("\",\"saved_origin\":\"").append(hex(Arrays.copyOfRange(files,0,77744))).append("\",\"stage_before\":2,\"stage_after\":").append(h.word(0x435500)).append(",\"score_after\":").append(h.word(0x435502)).append(",\"party_location\":\"").append(hex(h.read(0x479eae,3))).append("\",\"menu_count_after\":").append(h.word(0x44257c)).append(",\"questions\":").append(questionCount).append(",\"movie\":").append(movie).append(",\"branch\":").append(branch).append(",\"continuation\":\"").append(h.pc()==0x14694c?"return":h.pc()==0x14032e?"jump":"continue").append("\",\"effects\":[").append(effects).append("]}");
        }finally{h.cpu.dispose();}
    }
}
