import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;

public class VerifyHeaderDestination extends VerifyStageTravel {
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]);originals=root.resolve("NewCenturions/TheNewCenturions");Path scenario=originals.resolve("Scenarios/The Thunder of Guns/0");
        byte[] headers=Files.readAllBytes(scenario.resolve("Data DD")),regions=Files.readAllBytes(scenario.resolve("Data RD")),land=Files.readAllBytes(scenario.resolve("Data LD"));
        byte[] authored=new byte[10*77744];
        for(int m=0;m<10;m++){int at=m*77744;System.arraycopy(headers,m*4200,authored,at,4200);at+=4200;System.arraycopy(regions,m*644,authored,at,644);at+=644+8100;System.arraycopy(land,m*64800,authored,at,64800);}
        StringBuilder cases=new StringBuilder();
        int[][] inputs={
            {1,79,6,0,2,3,3,0,24,0,0,0},
            {2,1,2,0,1,79,7,0,24,0,0,0},
            {8,23,15,0,4,40,7,0,24,0,0,0},
            {1,79,6,0,1,79,6,0,24,0,0,0},
            {1,79,6,0,1,79,7,0,24,0,0,0},
            {1,79,6,0,1,79,6,1,24,0,0,0},
            {1,79,6,0,-1,3,3,0,24,0,0,0},
            {1,79,6,0,1,-128,-1,0,24,0,0,0},
            {1,79,6,255,1,79,6,255,24,0,0,0},
            {1,79,6,0,2,3,3,0,24,1,0,0},
            {1,79,6,0,2,3,3,0,24,0,1,0},
            {1,79,6,0,2,3,3,0,0,0,0,0},
            {1,79,6,0,2,3,3,0,0,0,0,-1},
            {1,79,6,0,2,3,3,0,0,0,0,1},
            {1,79,6,0,2,3,3,0,25,0,0,0},
            {1,79,6,0,1,79,6,0,25,0,0,0}
        };
        for(int[] t:inputs) {
            files=authored.clone();ActionOracleHarness h=new ActionOracleHarness(currentProgram);
            try {
                h.value(0x402f43,1,255);execute(h,0x13107a,t[0],originals);
                h.write(0x6fd000,new byte[16384]);h.cpu.writeRegister("A6",0x700000);h.cpu.writeRegister("SP",0x6ffe00);h.cpu.writeRegister("A4",t[1]);h.cpu.writeRegister("A3",t[2]);
                h.value(0x700010,2,t[11]);h.value(0x6ffee6,2,t[10]);h.value(0x442774,4,t[0]);h.value(0x442778,4,t[1]);h.value(0x44277c,4,t[2]);h.value(0x442764,4,0);h.value(0x442768,4,0);h.value(0x44274c,1,t[3]);h.value(0x438776,2,t[9]);h.value(0x4427c2,2,7);
                byte[] record=Arrays.copyOfRange(headers,1*4200+7*42,1*4200+8*42);record[4]=(byte)t[4];record[5]=(byte)t[5];record[6]=(byte)t[6];record[40]=(byte)t[7];record[7]=100;h.write(0x479eaa,record);h.write(0x478e42+7*42,record);
                // Exercise one mutation in each of the six independently persisted regions.
                for(int i=0;i<REGIONS.length;i++){byte[] raw=h.read(REGIONS[i],SIZES[i]);raw[SIZES[i]-1]^=(byte)(0x31+i);h.write(REGIONS[i],raw);}
                byte[] packedBefore=packed(h);byte[] headersBefore=h.read(0x478e42,4200);StringBuilder calls=new StringBuilder();boolean actual=t[4]>=0&&t[4]<10&&t[5]>=0&&t[5]<90&&t[6]>=0&&t[6]<90;
                effects=new StringBuilder();fileMode="map";cursor=0;h.jump(t[8]==24?0x1445fc:t[8]==25?0x144606:0x14674a);boolean complete=false;boolean prepared=t[8]!=25;
                for(int s=0;s<100000;s++) {
                    long pc=h.pc();if(pc==0x14694c||pc==0x140282){complete=true;break;}
                    if(pc==0x14673c&&!prepared){prepared=true;h.jump(0x14674a);continue;}
                    if(pc==0x151e24||pc==0x13107a){int m=(int)h.pointer(h.sp()+4);if(calls.length()>0)calls.append(',');calls.append('"').append(pc==0x151e24?"save":"load").append(m).append('"');if(!actual){h.ret();continue;}if(pc==0x13107a)h.value(0x402f43,1,255);}
                    if(pc==0x1205f8||pc==0x16043e){h.ret();continue;}
                    if((h.word(pc)&65535)==0xa032){h.jump(pc+2);continue;}
                    if(pc==0x1468e4&&!actual){h.jump(0x146928);continue;}
                    callBoundary(h);
                }
                if(!complete)throw new Exception("Header destination budget");
                if(cases.length()>0)cases.append(',');cases.append("{\"input\":").append(Arrays.toString(t)).append(",\"record_before\":\"").append(hex(record)).append("\",\"record_after\":\"").append(hex(h.read(0x479eaa,42))).append("\",\"persisted\":\"").append(hex(h.read(0x478e42+7*42,42))).append("\",\"location_after\":[").append((int)h.pointer(0x442774)).append(',').append((int)h.pointer(0x442778)).append(',').append((int)h.pointer(0x44277c)).append(',').append(h.read(0x44274c,1)[0]&255).append("],\"calls\":[").append(calls).append("],\"actual_file_services\":").append(actual).append(",\"origin_before_sha256\":\"").append(digest(packedBefore)).append("\",\"origin_saved_sha256\":\"").append(digest(Arrays.copyOfRange(files,t[0]*77744,(t[0]+1)*77744))).append("\",\"loaded_sha256\":\"").append(digest(packed(h))).append("\",\"header_chance_after\":").append(h.read(0x478e42+7*42+7,1)[0]).append(",\"stop\":\"").append(Long.toHexString(h.pc())).append("\"}");
            }finally{h.cpu.dispose();}
        }
        String identity="\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyHeaderDestination.java"))+"\",\"stage_probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyStageTravel.java"))+"\",\"travel_probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyTravel.java"))+"\",\"harness_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/ActionOracleHarness.java"))+"\",\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\"";
        Files.writeString(root.resolve("evidence/fixtures/header-destination.json"),"{\"authority\":\"Original newland full exhaustion14674a or opcode24/25 reaches finish146782..14694c. Map/x/y sign extension, asymmetric level comparison, reload and persistence instructions execute. Valid locations execute original saveland/loadland/propertyreader through inherited modeled file services and graphics boundaries; invalid coordinates/maps intercept save/load and omit tile lookup. Each persisted region mutated independently before travel. Opcode25 executes first then bounded finish; no action-body, live gameplay or invalid-map safety claim.\",\"cache_identity\":{"+identity+"},\"rng_draws\":0,\"cases\":["+cases+"]}\n");println("Header destination cases exported");
    }
}
