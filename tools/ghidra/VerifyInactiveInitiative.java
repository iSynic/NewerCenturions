import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.HexFormat;

public class VerifyInactiveInitiative extends VerifyStoryParty {
    String changes(byte[] before,byte[] after){StringBuilder b=new StringBuilder("[");for(int i=0;i<before.length;i++)if(before[i]!=after[i]){if(b.length()>1)b.append(',');b.append('[').append(i).append(',').append(after[i]&255).append(']');}return b.append(']').toString();}
    String digest(byte[] data)throws Exception{return hex.formatHex(java.security.MessageDigest.getInstance("SHA-256").digest(data));}
    public void run()throws Exception{
        root=Path.of(getScriptArgs()[0]);warriors=new byte[6*3138];String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};int[] speeds={12,10,9,8,8,9},enemies={8,7,6,5,9,7,9,5,8};
        for(int i=0;i<6;i++){System.arraycopy(Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Warriors/"+names[i])),0,warriors,i*3138,3138);word(warriors,i*3138+0x50,30);word(warriors,i*3138+0x90,0);word(warriors,i*3138+0x9e8,speeds[i]);word(warriors,i*3138+0x20,-1);}
        StringBuilder cases=new StringBuilder(),diagnostics=new StringBuilder();
        for(int variant=0;variant<24;variant++)for(int mode:new int[]{-1,0,1})for(int initial:new int[]{12,1}){
            if(variant==22&&mode==0)continue;
            ActionOracleHarness h=new ActionOracleHarness(currentProgram);
            try{
                byte[] input=warriors.clone();int unavailable=0;
                for(int member=0;member<6;member++){
                    int hp=30;if(variant<18&&member==variant/3)hp=new int[]{-32768,-3,0}[variant%3];else if(variant==18||variant==19){if(member%2==variant-18)hp=-3;}else if(variant==21)hp=-10;
                    word(input,member*3138+0x50,hp);if(hp<1)unavailable++;
                }
                if(variant==22)unavailable=-1;else if(variant==23)unavailable=1;
                h.write(0x448aa0,input);h.value(0x442739,1,unavailable);h.value(0x44273a,1,5);h.value(0x43874a,2,enemies.length);for(int i=0;i<enemies.length;i++)h.value(0x439d3c+i*288+4,2,enemies[i]);
                seed=initial;draws=0;h.enter(0x124d9e,0x700000,0x900000);h.value(0x700004,2,mode);try{execute(h,0x900000,100000);}catch(Exception exception){if(variant!=23||mode!=1||!exception.getMessage().equals("Story party instruction budget"))throw exception;if(diagnostics.length()>0)diagnostics.append(',');diagnostics.append("{\"variant\":23,\"mode\":1,\"unavailable\":1,\"seed_before\":").append(initial).append(",\"seed_after\":").append(seed).append(",\"draws\":").append(draws).append(",\"instruction_limit\":100000,\"pc\":").append(h.pc()).append(",\"order_partial\":\"").append(hex.formatHex(h.read(0x4470b8,220))).append("\",\"before_sha256\":\"").append(digest(input)).append("\",\"after_sha256\":\"").append(digest(h.read(0x448aa0,input.length))).append("\"}");continue;}
                if(cases.length()>0)cases.append(',');cases.append("{\"variant\":").append(variant).append(",\"mode\":").append(mode).append(",\"unavailable\":").append(unavailable).append(",\"before_changes\":").append(changes(warriors,input)).append(",\"after_changes\":").append(changes(input,h.read(0x448aa0,input.length))).append(",\"before_sha256\":\"").append(digest(input)).append("\",\"after_sha256\":\"").append(digest(h.read(0x448aa0,input.length))).append("\",\"order\":\"").append(hex.formatHex(h.read(0x4470b8,220))).append("\",\"seed_before\":").append(initial).append(",\"seed_after\":").append(seed).append(",\"draws\":").append(draws).append('}');
            }finally{h.cpu.dispose();}
        }
        Files.writeString(root.resolve("evidence/fixtures/inactive-initiative.json"),"{\"authority\":\"Original full getbattleorder124d9e..125128. Only Rand intercepted; complete six warrior records and110 signedword order slots observed. SignedHP>0 living list, signed unavailable byte, surprise modes-1/0/1, exact ordered adjacent swaps, indices20 and RNG verified. Eighteen single unavailable patterns, two alternating three-member patterns, fully living and fully unavailable parties with nine enemies; unavailable equals actual nonpositiveHP count. Extra surprise+/-1 cases use unavailable=-1 to preserve the signed-counter expanded range without out-of-list access. All six living with unavailable1 normal mode inserts lastfive compact livingmembers1..5; surprise-1 insertsall6. Surprise+1 reservesall6 slots1..6 but bounds enemyinsertion to14;9enemies cannotfit8free slots, so2cases record bounded100000instruction diagnostics and are excluded from completed cases. Normal mode with expanded range can read outside compact livinglist and is deliberately excluded; port capacity diagnostic is tested separately. Warrior status90 is zero; confusion and live UI are outside this fixture.\",\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyInactiveInitiative.java"))+"\",\"harness_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/ActionOracleHarness.java"))+"\",\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\",\"warriors\":\""+hex.formatHex(warriors)+"\",\"enemy_speeds\":"+Arrays.toString(enemies)+",\"cases\":["+cases+"],\"diagnostics\":["+diagnostics+"]}\n");println("140 completed original inactive initiative cases and two bounded surprise-capacity diagnostics exported");
    }
}
