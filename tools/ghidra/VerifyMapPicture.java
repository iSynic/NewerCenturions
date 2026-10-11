import ghidra.app.script.GhidraScript;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.HexFormat;

/** Original opcode 113 signed word assignment and redraw boundary. */
public class VerifyMapPicture extends GhidraScript {
    static String hex(byte[] data){return HexFormat.of().formatHex(data);}
    static byte[] patterned(int length,int salt){byte[] data=new byte[length];for(int i=0;i<length;i++)data[i]=(byte)(i*17+salt);return data;}
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]),original=root.resolve("NewCenturions/TheNewCenturions");
        String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};
        byte[] party=new byte[6*3138];for(int i=0;i<6;i++)System.arraycopy(Files.readAllBytes(original.resolve("Warriors/"+names[i])),0,party,i*3138,3138);
        byte[][] state={party,patterned(16200,7),patterned(16200,31),patterned(4200,59),patterned(128,83),patterned(16,101),patterned(64,113)};
        long[] addresses={0x448aa0,0x4085b8,0x40c500,0x478e42,0x44258c,0x4428de,0x442740};
        String[] labels={"party","floor","objects","headers","flags","calendar","world"};
        StringBuilder cases=new StringBuilder();
        Path scripts=original.resolve("Scenarios/The Thunder of Guns/0/Data ED3");byte[] table=Files.readAllBytes(scripts);
        for(int test=0;test<26;test++) {
            int before=test<24?new int[]{0,-32768,32767}[test/8]:-1;
            int parameter=test<24?new int[]{0,1,-1,-357,357,-32768,32767,256}[test%8]:test==24?1:0;
            int scriptId=test<24?-1:test==24?85:86;
            ActionOracleHarness h=new ActionOracleHarness(currentProgram);
            try {
                for(int i=0;i<state.length;i++)h.write(addresses[i],state[i]);
                h.write(0x6ff000,new byte[8192]);h.cpu.writeRegister("A6",0x700000);h.cpu.writeRegister("SP",0x700500);
                h.value(0x6fffea,2,parameter);h.value(0x40857a,2,before);
                if(scriptId<0)h.jump(0x141af0);else {h.write(0x479eaa,Arrays.copyOfRange(table,scriptId*42,(scriptId+1)*42));h.value(0x6fffee,2,scriptId==85?0:4);h.jump(0x140336);}
                StringBuilder events=new StringBuilder();int redraws=0;boolean done=false;
                for(int step=0;step<40;step++) {
                    long pc=h.pc();if(pc==0x14673c){done=true;break;}
                    if(pc==0x10f186){redraws++;if(events.length()>0)events.append(',');events.append("{\"boundary\":\"centerpict\",\"stored_word\":").append(h.word(0x40857a)).append('}');h.ret();continue;}
                    if(pc==0x153ca4)throw new Exception("Unexpected RNG in opcode113");
                    if(!h.cpu.step(monitor))throw new Exception("Map picture at "+Long.toHexString(pc)+": "+h.cpu.getLastError());
                }
                if(!done)throw new Exception("Map picture budget");
                for(int i=0;i<state.length;i++)if(!Arrays.equals(state[i],h.read(addresses[i],state[i].length)))throw new Exception("Unexpected mutation "+labels[i]);
                if(cases.length()>0)cases.append(',');cases.append("{\"script_id\":").append(scriptId).append(",\"before\":").append(before).append(",\"parameter\":").append(parameter).append(",\"after\":").append(h.word(0x40857a)).append(",\"redraws\":").append(redraws).append(",\"ordered_boundaries\":[").append(events).append("],\"continuation\":\"14673c\",\"rng_draws\":0,\"unchanged\":true}");
            } finally {h.cpu.dispose();}
        }
        byte[] script85=Arrays.copyOfRange(table,85*42,86*42),script86=Arrays.copyOfRange(table,86*42,87*42);
        StringBuilder identity=new StringBuilder(),snapshot=new StringBuilder();String[][] inputs={{"probe","tools/ghidra/VerifyMapPicture.java"},{"harness","tools/ghidra/ActionOracleHarness.java"},{"mapped_binary","analysis/ghidra/startup/code1-mapped.bin"},{"scripts","NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data ED3"}};
        for(String[] input:inputs){if(identity.length()>0)identity.append(',');identity.append('"').append(input[0]).append("_sha256\":\"").append(ActionOracleHarness.sha256(root.resolve(input[1]))).append('"');}
        for(int i=0;i<6;i++)identity.append(",\"warrior").append(i).append("_sha256\":\"").append(ActionOracleHarness.sha256(original.resolve("Warriors/"+names[i]))).append('"');
        for(int i=0;i<state.length;i++){if(snapshot.length()>0)snapshot.append(',');snapshot.append('"').append(labels[i]).append("\":\"").append(hex(state[i])).append('"');}
        Files.writeString(root.resolve("evidence/fixtures/map-picture.json"),"{\"authority\":\"Original newland opcode113 executes141af0 signed word assignment from local-16 to40857a, calls recorded centerpict10f186 boundary after assignment and continues14673c. Redraw acknowledged without executing artwork routines. Complete six original warrior records plus controlled full floor, object, event header, flag, calendar and world bytes are checked unchanged. Native dispatch140336..14036a executes original DataED3 script85 slot0 opcode113 parameter1 and script86 slot4 opcode113 parameter0. Input-357 is separately controlled; in original script85 it belongs to opcode9 slot1. Signed boundary and prior word cases consume no RNG. No image rendering or live original-game claim.\",\"identity\":{"+identity+"},\"snapshots\":{"+snapshot+"},\"original_script85\":\""+hex(script85)+"\",\"original_script86\":\""+hex(script86)+"\",\"cases\":["+cases+"]}\n");
        println("26 original map picture word cases exported");
    }
}
