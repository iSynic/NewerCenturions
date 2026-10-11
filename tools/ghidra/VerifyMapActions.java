import ghidra.app.script.GhidraScript;
import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.HexFormat;

public class VerifyMapActions extends GhidraScript {
    static class MapImage {
        byte[] floor=new byte[16200],objects=new byte[16200],headers=new byte[4200];
        MapImage(int marker) {
            for(int i=0;i<8100;i++){ByteBuffer.wrap(floor).putShort(i*2,(short)(100+marker));ByteBuffer.wrap(objects).putShort(i*2,(short)(200+marker));}
            for(int i=0;i<headers.length;i++)headers[i]=(byte)(i+marker);
        }
        String json(){return "{\"floor\":\""+hex(floor)+"\",\"objects\":\""+hex(objects)+"\",\"headers\":\""+hex(headers)+"\"}";}
        void load(ActionOracleHarness h){h.write(0x4085b8,floor);h.write(0x40c500,objects);h.write(0x478e42,headers);}
        void save(ActionOracleHarness h){floor=h.read(0x4085b8,16200);objects=h.read(0x40c500,16200);headers=h.read(0x478e42,4200);}
    }
    static String hex(byte[] bytes){return HexFormat.of().formatHex(bytes);}
    ActionOracleHarness frame(){ActionOracleHarness h=new ActionOracleHarness(currentProgram);h.write(0x6ff000,new byte[8192]);h.cpu.writeRegister("A6",0x700000);h.cpu.writeRegister("SP",0x700500);return h;}
    public void run()throws Exception {
        Path root=Path.of(getScriptArgs()[0]);StringBuilder cases=new StringBuilder();
        int[][] inputs={{12,0,2,3,345,0},{12,1,2,3,23000,2},{12,1,2,3,22999,2},{12,0,2,3,-1,2},{13,0,2,257,0,0},{13,1,0,17,2,4},{13,1,5,-1,4,2},{7,0,2,3,0,0},{7,1,2,3,0,0}};
        for(int n=0;n<inputs.length;n++) {
            int[] input=inputs[n];MapImage[] maps={new MapImage(0),new MapImage(23)};String before0=maps[0].json(),before1=maps[1].json();
            byte[] extra=new byte[10];for(int j=1;j<6;j++)ByteBuffer.wrap(extra).putShort((j-1)*2,(short)input[j]);
            byte[] original=new byte[42],replacement=new byte[42];for(int j=0;j<42;j++){original[j]=(byte)(j+71);replacement[j]=(byte)(j+133);}
            ActionOracleHarness h=frame();StringBuilder events=new StringBuilder();
            try {
                maps[0].load(h);h.write(0x44330a,extra);h.write(0x479eaa,original);h.write(0x6fff8a,original);h.value(0x442774,4,0);
                h.jump(input[0]==12?0x143644:input[0]==13?0x143784:0x143414);
                boolean completed=false;
                for(int steps=0;steps<5000;steps++) {
                    long pc=h.pc(),sp=h.sp();if(pc==0x14673c){completed=true;break;}
                    if(pc==0x151e24||pc==0x13107a) {
                        int map=(int)h.pointer(sp+4);if(map<0||map>1)throw new Exception("Bad modeledmap "+map);
                        if(events.length()>0)events.append(',');events.append('"').append(pc==0x151e24?"save":"load").append(map).append('"');
                        if(pc==0x151e24)maps[map].save(h);else maps[map].load(h);h.ret();continue;
                    }
                    if(pc==0x123c60){byte[] inherited=h.read(0x479eaa,42);System.arraycopy(replacement,8,inherited,8,34);h.write(0x479eaa,inherited);h.ret();continue;}
                    if(pc==0x10f186){h.ret();continue;}
                    if(!h.cpu.step(monitor))throw new Exception("Mapaction "+n+" PC "+Long.toHexString(pc)+": "+h.cpu.getLastError());
                }
                if(!completed)throw new Exception("Mapaction budget");
                if(cases.length()>0)cases.append(',');cases.append("{\"opcode\":").append(input[0]).append(",\"extra\":\"").append(hex(extra)).append("\",\"map0_before\":").append(before0).append(",\"map1_before\":").append(before1).append(",\"map0_after\":").append(maps[0].json()).append(",\"map1_after\":").append(maps[1].json()).append(",\"script_before\":\"").append(hex(original)).append("\",\"replacement\":\"").append(hex(replacement)).append("\",\"script_after\":\"").append(hex(h.read(0x479eaa,42))).append("\",\"ordered_io\":[").append(events).append("],\"rng_draws\":0}");
            }finally{h.cpu.dispose();}
        }
        StringBuilder animations=new StringBuilder();
        for(int[] input:new int[][]{{7,12,9,64,2},{0,-3,2,31,0}}) {
            ActionOracleHarness h=new ActionOracleHarness(currentProgram);
            try {
                h.enter(0x15b84c,0x700000,0x900000);for(int j=0;j<5;j++)h.value(0x700004+j*2,2,input[j]);h.value(0x442778,4,5);h.value(0x44277c,4,4);
                StringBuilder frames=new StringBuilder();int delays=0,copies=0;boolean completed=false;
                for(int steps=0;steps<10000;steps++) {
                    long pc=h.pc(),sp=h.sp();if(pc==0x900000){completed=true;break;}
                    int op=h.word(pc)&65535;
                    if(op==0xa8a8){long rect=h.pointer(sp+4);int dh=h.word(sp+2),dv=h.word(sp);for(int j=0;j<4;j++)h.value(rect+j*2,2,h.word(rect+j*2)+(j%2==0?dv:dh));h.pop(8);h.jump(pc+2);continue;}
                    if(op==0xa873){h.pop(4);h.jump(pc+2);continue;}
                    if(op==0xa8ec){copies++;h.pop(22);h.jump(pc+2);continue;}
                    if(pc==0x14a02c){if(frames.length()>0)frames.append(',');frames.append(h.word(sp+4));h.ret();continue;}
                    if(pc==0x1205f8){delays++;h.ret();continue;}
                    if(!h.cpu.step(monitor))throw new Exception("Animation PC "+Long.toHexString(pc)+": "+h.cpu.getLastError());
                }
                if(!completed)throw new Exception("Animation budget");if(animations.length()>0)animations.append(',');
                animations.append("{\"effect\":").append(input[0]).append(",\"x\":").append(input[1]).append(",\"y\":").append(input[2]).append(",\"size\":").append(input[3]).append(",\"delay\":").append(input[4]).append(",\"frames\":[").append(frames).append("],\"delay_calls\":").append(delays).append(",\"copy_calls\":").append(copies).append(",\"rect\":\"").append(hex(h.read(0x44371a,8))).append("\",\"rng_draws\":0}");
            }finally{h.cpu.dispose();}
        }
        StringBuilder secrets=new StringBuilder();
        for(int variant=0;variant<7;variant++) {
            ActionOracleHarness h=new ActionOracleHarness(currentProgram);
            try {
                byte[] floor=new byte[16200],warrior=new byte[variant>=4?3138*6:3138];int x=variant==3?0:10,y=variant==3?0:10;ByteBuffer cells=ByteBuffer.wrap(floor);
                if(variant==0)cells.putShort((x*90+y)*2,(short)3500);
                if(variant==1)cells.putShort((x*90+y)*2,(short)-3500);
                if(variant==2){cells.putShort(((x-1)*90+y-1)*2,(short)3500);cells.putShort((x*90+y-1)*2,(short)-4500);cells.putShort(((x+1)*90+y+1)*2,(short)3500);}
                if(variant==3){cells.putShort(0,(short)3500);cells.putShort(2,(short)-3500);}
                if(variant>=4)cells.putShort((x*90+y)*2,(short)(variant==6?-32768:3500));
                ByteBuffer.wrap(warrior).putShort(0xfc,(short)60);
                if(variant>=4)for(int j=0;j<6;j++){ByteBuffer.wrap(warrior).putShort(j*3138+0xfc,(short)(variant==5?32767:50+j*3));ByteBuffer.wrap(warrior).putShort(j*3138+0xb6a,(short)(variant==5?1:5));}
                h.write(0x448aa0,warrior);h.value(0x44273a,1,variant>=4?5:0);h.value(0x442764,4,x);h.value(0x442768,4,y);h.value(0x442778,4,0);h.value(0x44277c,4,0);h.value(0x442566,2,variant==2||variant>=4?0:1);h.value(0x44256c,2,0);
                h.write(0x4085b8,floor);h.enter(0x11177e,0x700000,0x900000);h.value(0x700004,2,0);long seed=variant==2?1:12;long initial=seed;int draws=0,flashes=0,chance=-999;boolean completed=false;
                for(int steps=0;steps<10000;steps++) {
                    long pc=h.pc(),sp=h.sp();if(pc==0x900000){completed=true;break;}
                    if(pc==0x111838)chance=(short)h.cpu.readRegister("D5").intValue();
                    if(pc==0x400038){h.cpu.writeRegister("D0",(short)Math.abs(h.word(sp+4)));h.ret();continue;}
                    if(pc==0x153ca4){int bound=h.word(sp+4);seed=seed*16807%2147483647;draws++;short raw=(short)seed;if(raw==Short.MIN_VALUE)raw=0;h.cpu.writeRegister("D0",Math.abs((int)raw)*bound/32768+1);h.ret();continue;}
                    if(pc==0x10f186||pc==0x400178||pc==0x20017c){h.ret();continue;}
                    if(pc==0x123d1c){flashes++;h.ret();continue;}
                    if(!h.cpu.step(monitor))throw new Exception("Secret PC "+Long.toHexString(pc)+": "+h.cpu.getLastError());
                }
                if(!completed)throw new Exception("Secret budget");if(secrets.length()>0)secrets.append(',');
                secrets.append("{\"x\":").append(x).append(",\"y\":").append(y).append(",\"chance\":").append(chance).append(",\"warriors\":\"").append(hex(warrior)).append("\",\"forced_skill\":").append(variant!=2&&variant<4).append(",\"before\":\"").append(hex(floor)).append("\",\"after\":\"").append(hex(h.read(0x4085b8,16200))).append("\",\"seed_before\":").append(initial).append(",\"seed_after\":").append(seed).append(",\"rng_draws\":").append(draws).append(",\"flashes\":").append(flashes).append(",\"center_revealed\":").append(h.cpu.readRegister("D0").intValue()==1).append("}");
            }finally{h.cpu.dispose();}
        }
        StringBuilder controls=new StringBuilder();
        for(int opcode:new int[]{24,25,26}) {
            ActionOracleHarness h=frame();byte[] script=new byte[42];for(int j=0;j<42;j++)script[j]=(byte)(j+6);script[4]=0;script[5]=55;script[6]=60;
            try {
                h.write(0x479eaa,script);h.write(0x478e42,new byte[4200]);h.value(0x447198,2,1);h.cpu.writeRegister("A4",43);h.cpu.writeRegister("A3",84);h.value(0x4427c2,2,2);h.jump(opcode==24?0x1445fc:opcode==25?0x144606:0x14462c);
                boolean completed=false;int resets=0,flashes=0,centers=0;long flashPoint=0,flashTicks=0;
                for(int steps=0;steps<1000;steps++) {
                    long pc=h.pc(),sp=h.sp();
                    if(opcode==24&&pc==0x146782||opcode!=24&&pc==0x14673c){completed=true;break;}
                    if(pc==0x1500d8){resets++;h.ret();continue;}
                    if(pc==0x123d1c){flashes++;flashPoint=h.pointer(sp+8);flashTicks=h.pointer(sp+12);h.ret();continue;}
                    if(pc==0x10f186){centers++;h.ret();continue;}
                    if(!h.cpu.step(monitor))throw new Exception("Control "+opcode+": "+h.cpu.getLastError());
                }
                if(!completed)throw new Exception("Control budget");
                if(opcode==25){
                    h.jump(0x14686c);completed=false;
                    for(int steps=0;steps<500;steps++) {
                        if(h.pc()==0x1468c2){completed=true;break;}
                        if(!h.cpu.step(monitor))throw new Exception("Persist control25: "+h.cpu.getLastError());
                    }
                    if(!completed)throw new Exception("Persist25 budget");
                }
                if(controls.length()>0)controls.append(',');controls.append("{\"opcode\":").append(opcode).append(",\"menu_active\":").append(h.word(0x447198)).append(",\"script_before\":\"").append(hex(script)).append("\",\"script_after\":\"").append(hex(h.read(0x479eaa,42))).append("\",\"persisted\":\"").append(hex(h.read(0x478e42+84,42))).append("\",\"event_origin_x\":43,\"event_origin_y\":84,\"offset_x\":").append(h.pointer(0x442778)).append(",\"offset_y\":").append(h.pointer(0x44277c)).append(",\"reset_calls\":").append(resets).append(",\"flash_calls\":").append(flashes).append(",\"center_calls\":").append(centers).append(",\"flash_point\":").append(flashPoint).append(",\"flash_ticks\":").append(flashTicks).append(",\"rng_draws\":0}");
            }finally{h.cpu.dispose();}
        }
        Files.writeString(root.resolve("evidence/fixtures/map-actions.json"),"{\"authority\":\"Original newland12/13/7 map mutation instructions execute with in-memory save/loadland boundary; original file packing is a separate probe requirement. loaddoor2 boundary copies known source bytes8..41 preservingcurrentheader. Original spelleffectoutdoors15b84c executes QuickDraw rectoffsetmodel and graphics/frame/delay recording boundaries; ploticon3/delay exported graphics/time only, noRand. Original checkforsecret11177e executes signedtile mutations and RNG ordering using previouslyverified ParkMiller Rand boundary; presentationrecorded. Control24/25/26 execute their blocks;25 then executes separate persistence suffix14686c..1468c2 with normal originlocals. No live/fullscenario proof.\",\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyMapActions.java"))+"\",\"harness_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/ActionOracleHarness.java"))+"\",\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\",\"ghidra\":\"12.1.2\",\"cases\":["+cases+"],\"animation_cases\":["+animations+"],\"secret_cases\":["+secrets+"],\"control_cases\":["+controls+"]}\n");println("Mapaction cases exported");
    }
}
