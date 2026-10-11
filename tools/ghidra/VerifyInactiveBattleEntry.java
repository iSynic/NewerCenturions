import ghidra.app.script.GhidraScript;
import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.HexFormat;

/** Original formation, inactive removal and first turn on supplied underground terrain. */
public class VerifyInactiveBattleEntry extends GhidraScript {
    long seed, draws, offset;
    int direction;
    byte[] monsters;
    void execute(ActionOracleHarness h, long stop, int budget, SaneBoundary sane) throws Exception {
        for(int step=0;step<budget;step++) {
            long pc=h.pc(),sp=h.sp();
            if(pc==stop)return;
            if(pc==0x153ca4) {
                int bound=h.word(sp+4); seed=seed*16807%2147483647;draws++;
                short raw=(short)seed;if(raw==Short.MIN_VALUE)raw=0;
                int result=Math.abs((int)raw)*bound/32768+1;
                if(bound==360)direction=result;
                h.cpu.writeRegister("D0",result);h.ret();continue;
            }
            if(pc==0x136cfa||pc==0x160fb2||pc==0x11f1a8||pc==0x16043e||pc==0x10e6b6){h.ret();continue;}
            if(pc==0x12946c){h.cpu.writeRegister("D0",1);h.ret();continue;}
            if(pc==0x400070){h.cpu.writeRegister("A0",0x600000);h.ret();continue;}
            if(pc==0x400078){offset=h.pointer(sp+8);if(h.word(sp+12)!=0)throw new Exception("Relative file seek");h.cpu.writeRegister("D0",0);h.ret();continue;}
            if(pc==0x400048){int size=(int)h.pointer(sp+8),count=(int)h.pointer(sp+12);if(size!=288||count!=1||offset+size>monsters.length)throw new Exception("Monster read boundary");h.write(h.pointer(sp+4),java.util.Arrays.copyOfRange(monsters,(int)offset,(int)offset+size));offset+=size;h.cpu.writeRegister("D0",1);h.ret();continue;}
            if(pc==0x400038){h.cpu.writeRegister("D0",Math.abs(h.word(sp+4)));h.ret();continue;}
            if(pc==0x400190||pc==0x400198){sane.extended(h.pointer(sp+4),pc==0x400190?Math.cos(sane.extended(sp+8)):Math.sin(sane.extended(sp+8)));h.ret();continue;}
            int opcode=h.word(pc)&65535;
            if(opcode==0xa9eb){sane.execute();continue;}
            if(opcode==0xa8ad){long r=h.pointer(sp);int y=h.word(sp+4),x=h.word(sp+6);boolean inside=y>=h.word(r)&&x>=h.word(r+2)&&y<h.word(r+4)&&x<h.word(r+6);h.value(sp+8,1,inside?1:0);h.pop(8);h.jump(pc+2);continue;}
            if(!h.cpu.step(monitor))throw new Exception("PC "+Long.toHexString(pc)+": "+h.cpu.getLastError());
        }
        throw new Exception("Instruction budget before "+Long.toHexString(stop));
    }
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]);HexFormat hex=HexFormat.of();
        Path scenario=root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0");
        byte[] land=Files.readAllBytes(scenario.resolve("Data LD"));
        byte[] properties=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data Sub BD"));
        byte[] battles=Files.readAllBytes(scenario.resolve("Data BD"));
        monsters=Files.readAllBytes(scenario.resolve("Data MD"));
        byte[] items=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data ID"));
        byte[] warriors=new byte[6*3138];String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};
        for(int i=0;i<6;i++)System.arraycopy(Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Warriors/"+names[i])),0,warriors,i*3138,3138);
        StringBuilder cases=new StringBuilder();
        for(int variant=0;variant<20;variant++)for(long initial:new long[]{12,1}) {
            int battleId=25;
            byte[] input=warriors.clone();
            int member=variant<18?variant/3:-1;
            int hp=variant<18?new int[]{-32768,-3,0}[variant%3]:0;
            if(member>=0){ByteBuffer.wrap(input).putShort(member*3138+0x50,(short)hp);ByteBuffer.wrap(input).putShort(member*3138+0x20,(short)-1);}
            else for(int i=0;i<6;i++)if((i%2)==(variant-18)){ByteBuffer.wrap(input).putShort(i*3138+0x50,(short)(i==0?-3:-10));ByteBuffer.wrap(input).putShort(i*3138+0x20,(short)-1);}
            int unavailable=0;for(int i=0;i<6;i++)if(ByteBuffer.wrap(input).getShort(i*3138+0x50)<1)unavailable++;
            ActionOracleHarness h=new ActionOracleHarness(currentProgram);
            try {
                int x=76,y=9;
                seed=initial;draws=0;direction=0;
                h.write(0x4085b8,java.util.Arrays.copyOfRange(land,64800,81000));
                h.write(0x414390,java.util.Arrays.copyOfRange(land,97200,129600));
                h.write(0x43167c,java.util.Arrays.copyOfRange(properties,64,12064));
                h.value(0x410446,2,0);h.value(0x402f43,1,4);
                h.value(0x435510,2,ByteBuffer.wrap(properties).getShort(0));h.value(0x435538,2,ByteBuffer.wrap(properties).getShort(2));
                h.value(0x442778,4,0);h.value(0x44277c,4,0);h.value(0x442764,4,x);h.value(0x442768,4,y);
                SaneBoundary sane=new SaneBoundary(h.cpu,currentProgram);h.enter(0x11ce8e,0x700000,0x7ffff0);execute(h,0x7ffff0,4000000,sane);
                int shiftX=h.word(0x4427a8),shiftY=h.word(0x4427aa);
                h.write(0x448aa0,input);h.write(0x43979c,java.util.Arrays.copyOfRange(battles,battleId*1372,(battleId+1)*1372));
                h.write(0x439d3c,new byte[28800]);h.write(0x446f0a,new byte[400]);h.write(0x44709a,new byte[24]);
                h.write(0x435664,new byte[]{12,14,12,14,10,12,10,10,14,16,0,12});
                int[] bases={0x4537fe,0x46225e,0x45ad2e,0x46978e};for(int band=0;band<4;band++)h.write(bases[band],java.util.Arrays.copyOfRange(items,band*30000,(band+1)*30000));
                h.value(0x442780,4,shiftX+5);h.value(0x442784,4,shiftY+5);h.value(0x442739,1,unavailable);h.value(0x44273a,1,5);
                for(long address:new long[]{0x43874a,0x4435c4,0x43877c,0x439582,0x442576,0x408578,0x44257c,0x44374c,0x43800c,0x4435d2})h.value(address,2,0);
                long frame=0x701000;h.write(frame-4096,new byte[8192]);h.cpu.writeRegister("A6",frame);h.cpu.writeRegister("SP",0x700000);
                h.value(frame-8,2,5);h.value(frame-6,2,5);h.value(frame-4,2,85);h.value(frame-2,2,85);h.value(frame+10,2,0);h.value(frame+12,2,0);
                h.value(frame-0x127,1,1);h.value(frame-0x121,1,-1);
                h.jump(0x11d5b0);execute(h,0x11ed1c,2500000,sane);
                h.write(frame-4096,new byte[8192]);h.cpu.writeRegister("A6",frame);h.cpu.writeRegister("SP",0x700000);h.jump(0x125ca0);execute(h,0x127090,100000,sane);
                int count=h.word(0x43874a);
                if(cases.length()>0)cases.append(',');cases.append("{\"variant\":").append(variant).append(",\"member\":").append(member).append(",\"hp\":").append(hp).append(",\"unavailable\":").append(unavailable).append(",\"warriors_before\":\"").append(hex.formatHex(input)).append("\",\"battle\":").append(battleId).append(",\"map\":1,\"x\":").append(x).append(",\"y\":").append(y).append(",\"seed_before\":").append(initial).append(",\"seed_after\":").append(seed).append(",\"draws\":").append(draws).append(",\"direction\":").append(direction).append(",\"shift\":[").append(shiftX).append(',').append(shiftY).append("],\"edge\":").append(h.word(0x410446)).append(",\"count\":").append(count).append(",\"cursor\":").append(h.word(0x447194));
                long[] addresses={0x4085b8,0x448aa0,0x439d3c,0x44709a,0x437a6e,0x446f0a,0x4470b8};int[] sizes={16200,warriors.length,count*288,24,12,count*4,220};String[] fields={"tiles","warriors","enemies","squad","under","positions","order"};
                for(int i=0;i<fields.length;i++)cases.append(",\"").append(fields[i]).append("\":\"").append(hex.formatHex(h.read(addresses[i],sizes[i]))).append('"');cases.append('}');println("Inactive entry variant "+variant+" seed "+initial+" complete");
            }finally{h.cpu.dispose();}
        }
        String identity="\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\",\"land_sha256\":\""+ActionOracleHarness.sha256(scenario.resolve("Data LD"))+"\",\"battles_sha256\":\""+ActionOracleHarness.sha256(scenario.resolve("Data BD"))+"\",\"monsters_sha256\":\""+ActionOracleHarness.sha256(scenario.resolve("Data MD"))+"\",\"properties_sha256\":\""+ActionOracleHarness.sha256(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data Sub BD"))+"\"";
        Files.writeString(root.resolve("evidence/fixtures/inactive-battle-entry.json"),"{\"authority\":\"Original combatmap full function, combatsetup11d5b0..11ed1c and getup125ca0..127090 composed on authored map1 Data Sub BD terrain. BD25 at76,9 is a controlled route coordinate. Six slot positions each receive signed stamina -32768/-3/0, then two alternating three-member unavailable patterns, for two seeds. Unavailable counter442739 equals the signed nonpositive stamina count. Inactive warrior initiative20 is -1 as set by the recovered prior defeat lifecycle; other warrior bytes use supplied originals. Formation and initiative run before stamina<1 removal, allowing dead and unconscious input. Original bodyfield and removal execute. First-turn initialization executes for the surviving first body. File, Rand, rectangle, presentation and bounded SANE/trigonometric services modeled. Does not prove route trigger or live original combat.\",\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyInactiveBattleEntry.java"))+"\","+identity+",\"warriors_before\":\""+hex.formatHex(warriors)+"\",\"cases\":["+cases+"]}\n");
        println("Forty original inactive battle entry cases exported");
    }
}
