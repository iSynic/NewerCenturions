import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.HexFormat;

/** Placement and restoration cases execute the original ring and footprint instructions. */
public class VerifyLargePlacement extends VerifyBattleEntry {
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]);HexFormat hex=HexFormat.of();StringBuilder cases=new StringBuilder();
        byte[] definitions=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data MD"));
        byte[] properties=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data Swamp BD"));
        for(int bodySize:new int[]{-3,-2,-1,0,1,2,3})for(int layout=0;layout<3;layout++)for(long initial:new long[]{12,1}) {
            ActionOracleHarness h=new ActionOracleHarness(currentProgram);
            try {
                byte[] monster=java.util.Arrays.copyOfRange(definitions,66*288,67*288);ByteBuffer.wrap(monster).putShort(0x10,(short)bodySize);
                byte[] map=new byte[16200];ByteBuffer mw=ByteBuffer.wrap(map);for(int i=0;i<8100;i++)mw.putShort(i*2,(short)1155);
                if(layout==1)for(int x=45;x<=47;x++)for(int y=45;y<=47;y++)mw.putShort((x*90+y)*2,(short)1061);
                if(layout==2)for(int x=45;x<=47;x++)for(int y=45;y<=47;y++)mw.putShort((x*90+y)*2,(short)0);
                h.write(0x4085b8,map);h.write(0x439d3c,monster);h.write(0x43167c,java.util.Arrays.copyOfRange(properties,64,12064));
                h.value(0x442780,4,5);h.value(0x442784,4,5);h.value(0x43874a,2,0);h.value(0x4434aa,2,0);h.value(0x4434ac,2,0);h.value(0x439582,2,0);
                long frame=0x701000;h.write(frame-4096,new byte[8192]);h.cpu.writeRegister("A6",frame);h.cpu.writeRegister("SP",0x700000);h.cpu.writeRegister("D5",40);h.cpu.writeRegister("D6",40);
                h.value(frame-8,2,5);h.value(frame-6,2,5);h.value(frame-4,2,85);h.value(frame-2,2,85);h.value(frame-0x124,2,-1);
                seed=initial;draws=0;h.jump(0x11e21c);execute(h,0x11e558,100000,new SaneBoundary(h.cpu,currentProgram));
                byte[] after=h.read(0x439d3c,288),tiles=h.read(0x4085b8,16200);int x=h.word(0x446f0a),y=h.word(0x446f0c);
                h.enter(0x105366,0x700000,0x7ffff0);h.value(0x700004,2,10);execute(h,0x7ffff0,10000,new SaneBoundary(h.cpu,currentProgram));
                if(cases.length()>0)cases.append(',');cases.append("{\"size\":").append(bodySize).append(",\"layout\":").append(layout).append(",\"seed_before\":").append(initial).append(",\"seed_after\":").append(seed).append(",\"draws\":").append(draws).append(",\"x\":").append(x).append(",\"y\":").append(y);
                String[] names={"monster","map","after","tiles","restored"};byte[][] buffers={monster,map,after,tiles,h.read(0x4085b8,16200)};
                for(int i=0;i<names.length;i++)cases.append(",\"").append(names[i]).append("\":\"").append(hex.formatHex(buffers[i])).append('"');cases.append('}');
            } finally {h.cpu.dispose();}
        }
        Files.writeString(root.resolve("evidence/fixtures/large-placement.json"),"{\"authority\":\"Original combatsetup placement ring 11e21c..11e558 including placemonster, rectangle and bodyfield rejection, then complete bodyfield restore. MD66 with controlled signed sizes-3..3, flat Swamp terrain and controlled blocked/occupied cells; Rand and rectangle services modeled. Does not prove route entry or live original combat.\",\"probe_sha256\":\""+ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyLargePlacement.java"))+"\",\"mapped_binary_sha256\":\""+ActionOracleHarness.sha256(root.resolve("analysis/ghidra/startup/code1-mapped.bin"))+"\",\"properties\":\""+hex.formatHex(properties)+"\",\"cases\":["+cases+"]}\n");
        println("Forty-two original four-size placement cases exported");
    }
}
