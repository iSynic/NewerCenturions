import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import com.google.gson.*;

/** Original current NPC turns with the actual active terrain property writer. */
public class VerifyCurrentTerrainRestrictedNpcSpell extends VerifyCurrentRestrictedNpcSpell {
    Path root;
    byte[] originalRegions;
    int selectedTerrain;
    JsonObject terrainWriter;
    JsonArray terrainWriters=new JsonArray();

    @Override public void run()throws Exception {
        if(getScriptArgs().length!=2)throw new Exception("A coherent frontier directory is required");
        root=Path.of(getScriptArgs()[0]);Path frontiers=root.resolve(getScriptArgs()[1]);
        Path policy=root.resolve("evidence/current-terrain-restriction-policy.md");
        if(!ActionOracleHarness.sha256(policy).equals("ed2738d497e237ce75cf42602228c980985b244c141144abc31267f77f38735e"))throw new Exception("Actual terrain input policy changed");
        Path original=root.resolve("NewCenturions/TheNewCenturions"),scenario=original.resolve("Scenarios/The Thunder of Guns/0");
        items=Files.readAllBytes(original.resolve("Data Files/Data ID"));scenarioItems=Files.readAllBytes(scenario.resolve("Data NI"));races=Files.readAllBytes(original.resolve("Data Files/Data Race"));mos=Files.readAllBytes(original.resolve("Data Files/Data MOS"));monsters=Files.readAllBytes(scenario.resolve("Data MD"));properties=Files.readAllBytes(original.resolve("Data Files/Data Biologic BD"));attacks=Files.readAllBytes(original.resolve("Data Files/Data S"));areas=Files.readAllBytes(original.resolve("Data Files/Data AD"));originalRegions=Files.readAllBytes(scenario.resolve("Data RD"));
        if(!sha(properties).equals("3b46f1256c925601947c26b3fd551b9659b650c98c20ce0ac496b1ca478fe0ff")||!sha(originalRegions).equals("7d61b79f4739b078a9ecc958dfbc6a9d2d1c4f0293cc3962e7289d9c2e1bf59d"))throw new Exception("Actual original terrain resources changed");
        Path previous=root.resolve("evidence/fixtures/current-restricted-npc-spell.json"),previousProbe=root.resolve("tools/ghidra/VerifyCurrentRestrictedNpcSpell.java"),controlsPath=root.resolve("analysis/completion/current-restricted-backup-controls.json");
        if(!ActionOracleHarness.sha256(previous).equals("31558d646423bed5495952e7c08029485aebfffd393601fa6cf141dbd62fdba1")||!ActionOracleHarness.sha256(previousProbe).equals("12b8791937b86d53972fe625a64d1b08328fd7dbabc59dba1ceebb7dbb46139e")||!ActionOracleHarness.sha256(controlsPath).equals("4c1dcc5e1dd0b831b38a9a54a130beebb7f5e70906c15632290b2d423b05d538"))throw new Exception("Frozen previous controls changed");
        JsonArray foundations=new JsonArray(),cases=new JsonArray();
        for(int seed:new int[]{12,1}){
            JsonObject foundation=currentFoundation(root,frontiers,seed);JsonObject session=foundation.getAsJsonObject("session");
            int map=session.getAsJsonObject("exploration").get("map").getAsInt();byte[] representedRegion=bytes(session.getAsJsonObject("exploration").getAsJsonArray("region_state"));
            if(map!=2||representedRegion.length!=644||originalRegions[map*644+0x208]!=9||representedRegion[0x208]!=9||session.get("bandage_ready").getAsInt()!=1)throw new Exception("Contradictory represented active terrain or Aid input");
            foundation.addProperty("terrain",9);foundation.addProperty("original_region_record",hex(Arrays.copyOfRange(originalRegions,map*644,(map+1)*644)));foundation.addProperty("property_resource","NewCenturions/TheNewCenturions/Data Files/Data Biologic BD");foundation.addProperty("property_digest",sha(properties));foundation.addProperty("property_work_digest",sha(Arrays.copyOfRange(properties,64,12064)));foundations.add(foundation);
        }
        for(int index=0;index<foundations.size();index++){currentCase(cases,foundations.get(index).getAsJsonObject(),index);cases.get(index).getAsJsonObject().add("native_terrain_writer",terrainWriter);}
        JsonObject identity=JsonParser.parseString(Files.readString(previous)).getAsJsonObject().getAsJsonObject("identity").deepCopy();
        identity.addProperty("previous_restricted_probe_sha256",identity.get("probe_sha256").getAsString());identity.addProperty("previous_restricted_fixture_sha256",ActionOracleHarness.sha256(previous));identity.addProperty("probe_sha256",ActionOracleHarness.sha256(root.resolve("tools/ghidra/VerifyCurrentTerrainRestrictedNpcSpell.java")));identity.addProperty("swamp_control_properties_sha256",identity.get("properties_sha256").getAsString());identity.addProperty("properties_sha256",sha(properties));identity.addProperty("regions_sha256",sha(originalRegions));identity.addProperty("input_policy_digest",ActionOracleHarness.sha256(policy));
        JsonObject out=new JsonObject();out.addProperty("authority","Two original current NPC decisions and complete native Getup/cleanup through11578a on unchanged complete coherent foundations. Actual map2 Data RD terrain9 selects original Biologic BD; original loadpixmap filename dispatch and property reader execute with declared file/graphics boundaries and full64-byte header/12000 actual property bytes. Represented bandage_ready1 projected to actual native WORD438786 before execution. Original startup editor backup and other previous work projections retained. No prescribed casts/targets/results, old Swamp family overwrite, live original-game or campaign completion claim.");out.add("identity",identity);out.add("foundations",foundations);out.add("cases",cases);out.add("writer_controls",JsonParser.parseString(Files.readString(controlsPath)));JsonObject shared=new JsonObject();shareBuffers(out,shared);out.add("shared_rule_buffers",shared);Files.writeString(root.resolve("evidence/fixtures/current-terrain-restricted-npc-spell.json"),out+"\n");println("Actual terrain current NPC decisions with full Getup exported:2");
    }

    @Override void initializeActual(ActionOracleHarness h,JsonObject session)throws Exception {
        super.initializeActual(h,session);
        byte[] region=bytes(session.getAsJsonObject("exploration").getAsJsonArray("region_state")),headers=bytes(session.getAsJsonObject("exploration").getAsJsonArray("event_headers"));
        if(region.length!=644||headers.length!=4200||region[0x208]!=9)throw new Exception("Active map header ownership mismatch");
        h.write(0x443086,region);h.write(0x478e42,headers);selectedTerrain=(int)region[0x208];
        if(h.word(0x438786)!=0)throw new Exception("Inherited Aid projection changed unexpectedly");
        h.value(0x438786,2,session.get("bandage_ready").getAsInt());
        establishTerrainProperties(h);
    }

    void establishTerrainProperties(ActionOracleHarness h)throws Exception {
        JsonObject receipt=new JsonObject();receipt.addProperty("entry","131182");receipt.addProperty("selected_terrain",selectedTerrain);receipt.addProperty("region_terrain_address","44328e");receipt.addProperty("region_terrain_byte",(int)h.read(0x44328e,1)[0]);receipt.addProperty("resource_sha256",sha(properties));receipt.addProperty("header64",hex(Arrays.copyOfRange(properties,0,64)));receipt.addProperty("before_cached_terrain",(int)h.read(0x402f43,1)[0]);receipt.addProperty("before_property_work",hex(h.read(0x43167c,16000)));
        String[] registers={"D0","D1","D2","D3","D4","D5","D6","D7","A0","A1","A2","A3","A4","A5","A6","SP","XF","NF","ZF","VF","CF"};long[] saved=new long[registers.length];for(int i=0;i<registers.length;i++)saved[i]=h.cpu.readRegister(registers[i]).longValue();
        JsonArray boundaries=new JsonArray();h.enter(0x131182,0x6dd000,0x900000);h.value(h.sp()+4,2,selectedTerrain);int cursor=0;boolean done=false;
        for(int steps=0;steps<20000;steps++){
            long pc=h.pc(),sp=h.sp();if(pc==0x900000){done=true;break;}
            if(pc==0x1312c0){JsonObject event=new JsonObject();event.addProperty("address","1312c0");event.addProperty("boundary","PICT construction omitted; native property reader retained");boundaries.add(event);h.jump(0x13137c);continue;}
            if(pc==0x400090||pc==0x400098){JsonObject event=new JsonObject();event.addProperty("address",Long.toHexString(pc));event.addProperty("boundary","Native terrain filename concatenation external C library");event.addProperty("first_argument",Long.toHexString(h.pointer(sp+4)));event.addProperty("second_argument",Long.toHexString(h.pointer(sp+8)));boundaries.add(event);h.ret();continue;}
            if(pc==0x400070){JsonObject event=new JsonObject();event.addProperty("address",Long.toHexString(pc));event.addProperty("boundary","Open selected original Biologic property file");boundaries.add(event);cursor=0;h.cpu.writeRegister("A0",0x600100);h.ret();continue;}
            if(pc==0x400048){int requested=Math.toIntExact(h.pointer(sp+8)*h.pointer(sp+12)),transferred=Math.min(requested,properties.length-cursor);long address=h.pointer(sp+4);if(transferred<0||cursor<0)throw new Exception("Terrain file reader outside resource");h.write(address,Arrays.copyOfRange(properties,cursor,cursor+transferred));JsonObject event=new JsonObject();event.addProperty("address",Long.toHexString(pc));event.addProperty("destination",Long.toHexString(address));event.addProperty("file_offset",cursor);event.addProperty("requested_bytes",requested);event.addProperty("transferred_bytes",transferred);event.addProperty("bytes_sha256",sha(Arrays.copyOfRange(properties,cursor,cursor+transferred)));boundaries.add(event);cursor+=transferred;h.cpu.writeRegister("D0",transferred/h.pointer(sp+8));h.ret();continue;}
            if(pc==0x400068){JsonObject event=new JsonObject();event.addProperty("address",Long.toHexString(pc));event.addProperty("boundary","Close original property file");boundaries.add(event);h.ret();continue;}
            if(!h.cpu.step(monitor))throw new Exception("Native terrain writer at"+Long.toHexString(pc)+":"+h.cpu.getLastError());
        }
        if(!done||h.sp()!=0x6dd004||h.cpu.readRegister("A6").longValue()!=saved[14]||cursor!=12064)throw new Exception("Native terrain writer return ABI");
        if(!Arrays.equals(h.read(0x43167c,12000),Arrays.copyOfRange(properties,64,12064))||h.read(0x402f43,1)[0]!=selectedTerrain)throw new Exception("Native terrain writer whole property mismatch");
        receipt.addProperty("return_word",(short)h.cpu.readRegister("D0").intValue());receipt.addProperty("return_stack",h.sp());receipt.addProperty("return_frame",h.cpu.readRegister("A6").longValue());receipt.addProperty("after_cached_terrain",(int)h.read(0x402f43,1)[0]);receipt.addProperty("after_property_work",hex(h.read(0x43167c,16000)));receipt.addProperty("after_header60",hex(h.read(0x40857c,60)));receipt.addProperty("after_default_tile",h.word(0x435508+selectedTerrain*2));receipt.addProperty("after_clock_word",h.word(0x435530+selectedTerrain*2));receipt.add("boundaries",boundaries);
        for(int i=0;i<registers.length;i++)h.cpu.writeRegister(registers[i],saved[i]);receipt.addProperty("entry_context_restored",true);terrainWriter=receipt;
    }

    @Override JsonObject completeSnap(ActionOracleHarness h,RunResult r)throws Exception {
        JsonObject out=super.completeSnap(h,r);out.addProperty("active_terrain",(int)h.read(0x402f43,1)[0]);out.addProperty("active_region",hex(h.read(0x443086,644)));out.addProperty("active_event_headers",hex(h.read(0x478e42,4200)));out.addProperty("terrain_property_work",hex(h.read(0x43167c,12000)));out.addProperty("terrain_property_header60",hex(h.read(0x40857c,60)));out.addProperty("terrain_default_tile",h.word(0x435508+selectedTerrain*2));out.addProperty("terrain_clock_word",h.word(0x435530+selectedTerrain*2));out.addProperty("aid_gate",h.word(0x438786));return out;
    }
}
