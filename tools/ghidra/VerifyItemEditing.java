import com.google.gson.GsonBuilder;
import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
public class VerifyItemEditing extends VerifyStoryInventory {
 @Override public void run() throws Exception {
  Path root=Path.of(getScriptArgs()[0]); byte[] original=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Warriors/Cindred"));
  byte[] definitions=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data ID"));
  byte[] races=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data Race"));
  byte[] mos=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data MOS"));
  JsonObject fixture=new JsonObject(); fixture.addProperty("authority","Original edititems1343de..134a34 mutation, pin and equipped remove/rewear instructions; source parent sound captured, original definitions/profile bytes supplied through read boundaries. Display update excluded. Full3138B records, original firearm, equipped/unequipped, empty end slot and signed-width controls. SANE and BitTst are explicit bounded service models, not native ROM execution. No port code.");
  for(String[] p:new String[][]{{"probe","tools/ghidra/VerifyItemEditing.java"},{"binary","analysis/ghidra/startup/code1-mapped.bin"},{"harness","tools/ghidra/ActionOracleHarness.java"},{"parent","tools/ghidra/VerifyStoryInventory.java"},{"sane","tools/ghidra/SaneBoundary.java"},{"definitions","NewCenturions/TheNewCenturions/Data Files/Data ID"},{"races","NewCenturions/TheNewCenturions/Data Files/Data Race"},{"mos","NewCenturions/TheNewCenturions/Data Files/Data MOS"}})fixture.addProperty(p[0]+"_sha256",ActionOracleHarness.sha256(root.resolve(p[1])));
  JsonArray cases=new JsonArray();
  for(int variant=0;variant<4;variant++)for(int control=40;control<=63;control++){
   if(variant==3 && control!=52 && control!=63)continue;int slot=variant==3?3:0;
   byte[] before=original.clone(); if(variant!=1)before[0x132+9]=0;
   if(variant==2){for(int p:new int[]{13,15,16,11,17,18,19,14,21,20})before[0x132+p]=127;ByteBuffer.wrap(before).putShort(0x132+6,(short)32767);}
   ActionOracleHarness h=frame(); StringBuilder sounds=new StringBuilder(); int file=0,cursor=0;
   try{
    h.write(0x448aa0,before);h.write(0x41fc48,Arrays.copyOfRange(before,0x132+slot*22,0x148+slot*22));h.value(0x438776,2,0);h.value(0x4471ec,2,1);h.value(0x44273a,1,0);
    for(int i=0;i<10;i++)h.value(0x435664+i,1,races[i*408+197]);
    h.cpu.writeRegister("D3",0);h.cpu.writeRegister("D4",slot);h.cpu.writeRegister("D6",0);h.value(0x4434b2,2,control);h.jump(0x1343de);boolean finished=false;SaneBoundary sane=new SaneBoundary(h.cpu,currentProgram);
    for(int step=0;step<100000;step++){
     long pc=h.pc(),sp=h.sp();if(pc==0x134a34){finished=true;break;}
     if(pc==0x133a18){if(sounds.length()>0)sounds.append(',');sounds.append(h.word(sp+4));h.ret();continue;}
     if(pc==0x130afe){int id=Math.abs(h.word(sp+4));h.write(0x45363c,Arrays.copyOfRange(definitions,id*150,(id+1)*150));h.ret();continue;}
     if(pc==0x13ba6c||pc==0x13bac6){file=pc==0x13ba6c?0:1;cursor=0;h.value(0x4434a6,4,0x600000+file);h.ret();continue;}
     if(pc==0x400078){cursor=(int)h.pointer(sp+8);h.cpu.writeRegister("D0",0);h.ret();continue;}
     if(pc==0x400048){int amount=(int)(h.pointer(sp+8)*h.pointer(sp+12));h.write(h.pointer(sp+4),Arrays.copyOfRange(file==0?races:mos,cursor,cursor+amount));cursor+=amount;h.cpu.writeRegister("D0",h.pointer(sp+12));h.ret();continue;}
     if(pc==0x400068){h.cpu.writeRegister("D0",0);h.ret();continue;}
     if((h.word(pc)&65535)==0xa85d){int bit=(int)h.pointer(sp);long address=h.pointer(sp+4)+(bit>>3);boolean set=(h.read(address,1)[0]&(128>>(bit&7)))!=0;h.value(sp+8,1,set?1:0);h.pop(8);h.jump(pc+2);continue;}
     if((h.word(pc)&65535)==0xa9eb){sane.execute();continue;}
     if(!h.cpu.step(monitor))throw new Exception("Function "+getFunctionContaining(toAddr(pc))+" opcode "+Integer.toHexString(h.word(pc)&65535)+" Item edit variant"+variant+" control"+control+" at "+Long.toHexString(pc)+": "+h.cpu.getLastError());
    }
    if(!finished)throw new Exception("Item edit instruction budget");JsonObject c=new JsonObject();c.addProperty("case","variant"+variant+"_control"+control);c.addProperty("variant",variant);if(slot!=0)c.addProperty("slot",slot);c.addProperty("control",control);c.addProperty("before",hex(before));c.addProperty("after",hex(h.read(0x448aa0,3138)));c.addProperty("item_before",hex(Arrays.copyOfRange(before,0x132+slot*22,0x148+slot*22)));c.addProperty("item_after",hex(h.read(0x448bd2+slot*22,22)));c.add("sounds",com.google.gson.JsonParser.parseString("["+sounds+"]"));cases.add(c);
   }finally{h.cpu.dispose();}
  }
  fixture.add("cases",cases);Files.writeString(root.resolve("evidence/fixtures/item-editing.json"),new GsonBuilder().setPrettyPrinting().create().toJson(fixture)+"\n");println("Original item editing observations exported: "+cases.size());
 }
}
