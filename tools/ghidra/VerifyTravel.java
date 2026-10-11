import ghidra.app.script.GhidraScript;
import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.MessageDigest;
import java.util.Arrays;
import java.util.HexFormat;

public class VerifyTravel extends GhidraScript {
    static final long[] REGIONS = {0x478e42,0x443086,0x4358e6,0x4085b8,0x40c500,0x414390};
    static final int[] SIZES = {4200,644,8100,16200,16200,32400};
    static String hex(byte[] bytes) { return HexFormat.of().formatHex(bytes); }
    static String digest(byte[] bytes) throws Exception { return hex(MessageDigest.getInstance("SHA-256").digest(bytes)); }
    byte[] files, properties;
    int cursor;
    boolean propertyFile;
    String terrainFile(int terrain) throws Exception {
        return switch(terrain) { case 1 -> "Data P BD"; case 2 -> "Data Swamp BD"; case 3 -> "Data Snow BD"; case 4 -> "Data Sub BD"; case 5 -> "Data Desert BD"; case 6 -> "Data Lava BD"; case 7 -> "Data Castle BD"; case 8 -> "Data Techno BD"; case 9 -> "Data Biologic BD"; default -> throw new Exception("Terrain unavailable: "+terrain); };
    }
    void execute(ActionOracleHarness h, long entry, int map, Path originals) throws Exception {
        h.enter(entry,0x700000,0x900000);h.value(0x700004,4,map);
        propertyFile=false;cursor=0;boolean complete=false;
        for(int steps=0;steps<20000;steps++) {
            long pc=h.pc(),sp=h.sp();
            if(pc==0x900000){complete=true;break;}
            if(pc==0x131182) {
                int terrain=h.word(sp+4);properties=Files.readAllBytes(originals.resolve("Data Files").resolve(terrainFile(terrain)));
                propertyFile=true;cursor=0;
            }
            // Window construction and PICT copying are excluded; the native property reader follows.
            if(pc==0x1312c0){h.jump(0x13137c);continue;}
            if(pc==0x136d86||pc==0x136cfa||pc==0x400090||pc==0x400098){h.ret();continue;}
            if(pc==0x400070){h.cpu.writeRegister("A0",0x600100);h.ret();continue;}
            if(pc==0x400078){cursor=(int)h.pointer(sp+8);if(h.word(sp+12)!=0)throw new Exception("Nonabsolute seek");h.ret();continue;}
            if(pc==0x400048||pc==0x400050) {
                int size=Math.toIntExact(h.pointer(sp+8)*h.pointer(sp+12));
                byte[] file=propertyFile?properties:files;
                if(cursor<0||cursor>file.length||(!propertyFile&&cursor+size>file.length))throw new Exception("File boundary "+cursor+"+"+size);
                int transferred=Math.min(size,file.length-cursor);
                long address=h.pointer(sp+4);
                if(pc==0x400048)h.write(address,Arrays.copyOfRange(file,cursor,cursor+transferred));
                else System.arraycopy(h.read(address,size),0,file,cursor,size);
                cursor+=transferred;h.cpu.writeRegister("D0",transferred / h.pointer(sp+8));h.ret();continue;
            }
            if(pc==0x400068){h.ret();continue;}
            if(!h.cpu.step(monitor))throw new Exception("Travel PC "+Long.toHexString(pc)+": "+h.cpu.getLastError());
        }
        if(!complete)throw new Exception("Travel instruction budget");
    }
    byte[] packed(ActionOracleHarness h) {
        byte[] result=new byte[77744];int at=0;
        for(int i=0;i<REGIONS.length;i++){System.arraycopy(h.read(REGIONS[i],SIZES[i]),0,result,at,SIZES[i]);at+=SIZES[i];}
        return result;
    }
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]),originals=root.resolve("NewCenturions/TheNewCenturions"),scenario=originals.resolve("Scenarios/The Thunder of Guns/0");
        byte[] headers=Files.readAllBytes(scenario.resolve("Data DD")),regions=Files.readAllBytes(scenario.resolve("Data RD")),land=Files.readAllBytes(scenario.resolve("Data LD"));
        byte[] authored=new byte[10*77744];
        for(int map=0;map<10;map++) {
            int at=map*77744;
            System.arraycopy(headers,map*4200,authored,at,4200);at+=4200;
            System.arraycopy(regions,map*644,authored,at,644);at+=644+8100;
            System.arraycopy(land,map*64800,authored,at,64800);
        }
        StringBuilder cases=new StringBuilder();
        for(int map:new int[]{0,1,2,3,5,9}) {
            files=authored.clone();ActionOracleHarness h=new ActionOracleHarness(currentProgram);
            try {
                h.value(0x402f43,1,255);execute(h,0x13107a,map,originals);
                byte[] before=packed(h);
                if(!Arrays.equals(before,Arrays.copyOfRange(authored,map*77744,(map+1)*77744)))throw new Exception("Original load mismatch");
                byte[] originalProperties=h.read(0x43167c,16000);
                int terrain=h.read(0x44328e,1)[0]&255;
                int defaultTile=h.word(0x435508+terrain*2),clock=h.word(0x435530+terrain*2);
                for(int i=0;i<REGIONS.length;i++) {
                    byte[] changed=h.read(REGIONS[i],SIZES[i]);
                    // Distinct mutations in every persisted region, preserving the terrain selector.
                    changed[SIZES[i]-1]^=(byte)(0x31+i);h.write(REGIONS[i],changed);
                }
                byte[] changed=packed(h);execute(h,0x151e24,map,originals);
                if(!Arrays.equals(changed,Arrays.copyOfRange(files,map*77744,(map+1)*77744)))throw new Exception("Original save mismatch");
                for(int i=0;i<REGIONS.length;i++)h.write(REGIONS[i],new byte[SIZES[i]]);
                h.value(0x402f43,1,255);execute(h,0x13107a,map,originals);
                if(!Arrays.equals(changed,packed(h)))throw new Exception("Original revisit mismatch");
                if(cases.length()>0)cases.append(',');
                cases.append("{\"map\":").append(map).append(",\"terrain\":").append(terrain).append(",\"loaded_record\":\"").append(hex(before)).append("\",\"saved_record\":\"").append(hex(changed)).append("\",\"saved_file_sha256\":\"").append(digest(files)).append("\",\"properties\":\"").append(hex(originalProperties)).append("\",\"default_tile\":").append(defaultTile).append(",\"clock_word\":").append(clock).append(",\"rng_draws\":0}");
            } finally {h.cpu.dispose();}
        }
        String identity="\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\",\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyTravel.java"))+"\",\"harness_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/ActionOracleHarness.java"))+"\",\"land_sha256\":\""+digest(land)+"\",\"headers_sha256\":\""+digest(headers)+"\",\"regions_sha256\":\""+digest(regions)+"\",\"ghidra\":\"12.1.2\"";
        Files.writeString(root.resolve("evidence/fixtures/travel.json"),"{\"authority\":\"Original loadland 0x13107a, saveland 0x151e24 and loadpixmap property reader 0x13137c..0x131446. File services modeled as byte reads/writes and absolute seeks; filename/PICT services excluded. All six map regions survive original save/reload. No live gameplay claim.\",\"cache_identity\":{"+identity+"},\"cases\":["+cases+"]}\n");
        println("Six original map load/save/revisit and property-reader cases verified");
    }
}
