import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.LinkedHashSet;
import com.google.gson.*;
public class VerifyEditorLevels extends VerifyLevelPromotion {
 int firstBonus(Path original,String name) throws Exception {
  byte[] w=Files.readAllBytes(original.resolve("Warriors/"+name));int id=ByteBuffer.wrap(w).getShort(0x34);
  return profiles[(id-1)*576+432]-1;
 }
 LinkedHashSet<Integer> levels(Path original,String name) throws Exception {
  LinkedHashSet<Integer> levels=new LinkedHashSet<>();
  for(int level:new int[]{3,6,9,firstBonus(original,name)})levels.add(level);
  return levels;
 }
 @Override public void run() throws Exception {
  Path root=Path.of(getScriptArgs()[0]);Path original=root.resolve("NewCenturions/TheNewCenturions");
  races=Files.readAllBytes(original.resolve("Data Files/Data Race"));profiles=Files.readAllBytes(original.resolve("Data Files/Data MOS"));registered=true;
  JsonObject fixture=new JsonObject();fixture.addProperty("authority","Original complete levelup12ed42 mode2 and updatespec161d86; information drawing skipped12ef50..12f048, controlled Done if reached. Actual CODE3 C rand executes with unsigned multiply boundary; original Race/MOS profiles supplied. Registered behavior. Full3138B records, library seed/draws and ordered source sounds. No experience or scan allocation.");
  for(String[] p:new String[][]{{"probe","tools/ghidra/VerifyEditorLevels.java"},{"parent","tools/ghidra/VerifyLevelPromotion.java"},{"binary","analysis/ghidra/startup/code1-mapped.bin"},{"harness","tools/ghidra/ActionOracleHarness.java"},{"sane","tools/ghidra/SaneBoundary.java"},{"code3","analysis/extracted/resources/106/434f4445_3.bin"}})fixture.addProperty(p[0]+"_sha256",ActionOracleHarness.sha256(root.resolve(p[1])));
  JsonArray out=new JsonArray();
  for(String name:new String[]{"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"})for(int level:levels(original,name))for(long seed:new long[]{1,123}){
   byte[] before=Files.readAllBytes(original.resolve("Warriors/"+name));ByteBuffer.wrap(before).putShort(0x3a,(short)level);
   ActionOracleHarness h=frame();RunResult result=new RunResult(12);calls=new StringBuilder();libraryDraws=0;
   try{
    h.write(0x448aa0,before);h.value(0x3ffe60,4,seed);h.cpu.writeRegister("A5",0x400000);h.value(0x442560,2,6);h.value(0x44273a,1,0);
    h.enter(0x12ed42,0x6ffe00,0x900000);h.value(0x6ffe04,2,0);h.value(0x6ffe06,2,2);execute(h,result,false);
    JsonObject c=new JsonObject();c.addProperty("case",name+"_level"+level+"_seed"+seed);c.addProperty("before",hex(before));c.addProperty("after",hex(h.read(0x448aa0,3138)));c.addProperty("seed_before",seed);c.addProperty("seed_after",h.pointer(0x3ffe60));c.addProperty("draws",libraryDraws);c.add("sounds",JsonParser.parseString("["+result.sounds+"]"));out.add(c);
   }finally{h.cpu.dispose();}
  }
  fixture.add("cases",out);Files.writeString(root.resolve("evidence/fixtures/editor-levels.json"),new GsonBuilder().setPrettyPrinting().create().toJson(fixture)+"\n");println("Editor level observations exported: "+out.size());
 }
}
