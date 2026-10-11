import ghidra.app.script.GhidraScript;
import com.google.gson.*;
import java.nio.file.*;
import java.nio.charset.Charset;
import java.util.*;

/** Original menu installation and finite menu dispatch with explicit Toolbox replies. */
public class VerifyMenuBoundary extends GhidraScript {
    Path root; JsonArray services; JsonArray music; Map<Integer,JsonObject> menus; List<Integer> bar; Map<Integer,List<Integer>> bars;
    static String hex(byte[] b){return HexFormat.of().formatHex(b);}
    long handle(int id){return 0x900000L+id*0x100L;}
    int id(long handle)throws Exception {int id=(int)((handle-0x900000)/0x100);if(handle!=handle(id)||!menus.containsKey(id))throw new Exception("Unknown menu handle");return id;}
    byte[] resource(String type,int id)throws Exception{return Files.readAllBytes(root.resolve("analysis/extracted/resources/74/"+type+"_"+id+".bin"));}
    JsonObject menu(int resourceId)throws Exception {
        byte[] b=resource("4d454e55",resourceId);JsonObject m=new JsonObject();int id=((b[0]&255)<<8)|(b[1]&255),p=14,n=b[p++]&255;
        m.addProperty("resource_id",resourceId);m.addProperty("menu_id",id);m.addProperty("title_hex",hex(Arrays.copyOfRange(b,p,p+n)));p+=n;
        long enabled=0;for(int i=10;i<14;i++)enabled=(enabled<<8)|(b[i]&255);m.addProperty("enable_flags",enabled);
        JsonArray items=new JsonArray();while((n=b[p++]&255)!=0){JsonObject item=new JsonObject();item.addProperty("text_hex",hex(Arrays.copyOfRange(b,p,p+n)));p+=n;item.addProperty("icon",b[p++]&255);item.addProperty("key",b[p++]&255);item.addProperty("mark",b[p++]&255);item.addProperty("style",b[p++]&255);items.add(item);}if(p!=b.length)throw new Exception("Resource suffix");m.add("items",items);return m;
    }
    void getBar(int resourceId)throws Exception {byte[] b=resource("4d424152",resourceId);List<Integer> ids=new ArrayList<>();int count=((b[0]&255)<<8)|(b[1]&255);for(int i=0;i<count;i++){int resourceIdMenu=((b[2+2*i]&255)<<8)|(b[3+2*i]&255);JsonObject m=menu(resourceIdMenu);int actual=m.get("menu_id").getAsInt();menus.put(actual,m);ids.add(actual);}bars.put(resourceId,ids);}
    void install(int resourceId)throws Exception {if(!bars.containsKey(resourceId))throw new Exception("Unknown menu bar handle");bar=new ArrayList<>(bars.get(resourceId));}
    byte[] pascal(ActionOracleHarness h,long ptr){int n=h.read(ptr,1)[0]&255;return h.read(ptr+1,n);}
    byte[] cstring(ActionOracleHarness h,long ptr)throws Exception {for(int n=0;n<256;n++)if(h.read(ptr+n,1)[0]==0)return n==0?new byte[0]:h.read(ptr,n);throw new Exception("Bounded string");}
    JsonObject state(ActionOracleHarness h){JsonObject s=new JsonObject();JsonObject all=new JsonObject();for(int key:new TreeSet<>(menus.keySet()))all.add(Integer.toString(key),menus.get(key).deepCopy());s.add("menus",all);s.add("visible_bar",new Gson().toJsonTree(bar));
        for(long a:new long[]{0x4427be,0x4427ba,0x4386fe,0x43872a,0x438706,0x43870a,0x43870e,0x438712,0x43871e,0x438722,0x438702,0x4386a4,0x4386a0,0x43871a,0x438716,0x438742})s.addProperty(Long.toHexString(a),h.pointer(a));
        for(long a:new long[]{0x438746,0x438748,0x43956a,0x439568,0x439594,0x447218})s.addProperty(Long.toHexString(a),h.word(a));s.addProperty("playlist13",hex(h.read(0x44270e,13)));s.addProperty("character_key_byte",h.read(0x44273c,1)[0]&255);s.addProperty("sp",h.sp());s.addProperty("a6",h.cpu.readRegister("A6").longValue());return s;}
    void execute(ActionOracleHarness h,long stop,int selection)throws Exception {
        for(int steps=0;steps<20000;steps++){
            long pc=h.pc(),sp=h.sp();if(pc==stop)return;
            if(pc==0x133d46){SubmitMusicBoundary.execute(h,monitor,1,music,()->{throw new Exception("Unexpected menu RNG");});continue;}
            if(pc==0x14c01c){JsonObject e=new JsonObject();e.addProperty("service","savepref request only");e.addProperty("address",Long.toHexString(pc));e.addProperty("entry_sp",sp);e.addProperty("return_address",h.pointer(sp));e.add("state",state(h));services.add(e);h.ret();continue;}
            if(pc==0x400090){byte[] str=cstring(h,h.pointer(sp+8));for(byte b:str)if(b=='%')throw new Exception("Unqualified formatted string");h.write(h.pointer(sp+4),Arrays.copyOf(str,str.length+1));JsonObject e=new JsonObject();e.addProperty("service","literal sprintf platform copy");e.addProperty("text_hex",hex(str));e.addProperty("destination",h.pointer(sp+4));e.addProperty("entry_sp",sp);services.add(e);h.ret();continue;}
            int op=h.word(pc)&65535;if(op>=0xa000&&op<=0xafff){JsonObject e=new JsonObject();e.addProperty("address",Long.toHexString(pc));e.addProperty("trap",Integer.toHexString(op));e.addProperty("sp_before",sp);
                if(op==0xa9c0){int r=h.word(sp);getBar(r);h.value(sp+2,4,0xa00000+r*256);h.pop(2);e.addProperty("resource_id",r);}
                else if(op==0xa93c){long b=h.pointer(sp);int r=(int)((b-0xa00000)/256);if(b!=0xa00000+r*256)throw new Exception("Bar handle");install(r);h.pop(4);e.addProperty("resource_id",r);}
                else if(op==0xa949||op==0xa9bf){int r=h.word(sp);if(op==0xa9bf){JsonObject m=menu(r);menus.put(m.get("menu_id").getAsInt(),m);}long reply=menus.containsKey(r)?handle(r):0;h.value(sp+2,4,reply);h.pop(2);e.addProperty("menu_id",r);e.addProperty("returned_handle",reply);}
                else if(op==0xa935){int before=h.word(sp);int m=id(h.pointer(sp+2));if(before!=-1)throw new Exception("Only hierarchical insert qualified");h.pop(6);e.addProperty("menu_id",m);e.addProperty("before_id",before);}
                else if(op==0xa947){byte[] text=pascal(h,h.pointer(sp));int item=h.word(sp+4),m=id(h.pointer(sp+6));menus.get(m).getAsJsonArray("items").get(item-1).getAsJsonObject().addProperty("text_hex",hex(text));h.pop(10);e.addProperty("menu_id",m);e.addProperty("item",item);e.addProperty("text_hex",hex(text));}
                else if(op==0xa933){byte[] text=pascal(h,h.pointer(sp));int m=id(h.pointer(sp+4));if(text.length!=12)throw new Exception("Only twelve-dash placeholder append qualified");for(byte b:text)if(b!=45)throw new Exception("Only twelve-dash placeholder append qualified");JsonObject item=new JsonObject();item.addProperty("text_hex",hex(text));for(String k:new String[]{"icon","key","mark","style"})item.addProperty(k,0);menus.get(m).getAsJsonArray("items").add(item);h.pop(8);e.addProperty("menu_id",m);e.addProperty("text_hex",hex(text));}
                else if(op==0xa93a||op==0xa939){int item=h.word(sp),m=id(h.pointer(sp+2));long flags=menus.get(m).get("enable_flags").getAsLong(),bit=1L<<item;menus.get(m).addProperty("enable_flags",op==0xa93a?flags&~bit:flags|bit);h.pop(6);e.addProperty("menu_id",m);e.addProperty("item",item);}
                else if(op==0xa944){int mark=h.word(sp),item=h.word(sp+2),m=id(h.pointer(sp+4));menus.get(m).getAsJsonArray("items").get(item-1).getAsJsonObject().addProperty("mark",mark);h.pop(8);e.addProperty("menu_id",m);e.addProperty("item",item);e.addProperty("mark",mark);}
                else if(op==0xa938){e.addProperty("hilite",h.word(sp));h.pop(2);}
                else if(op==0xa937){}
                else if(op==0xa860){if(h.word(sp+12)!=-1||h.pointer(sp+8)!=0x44720a)throw new Exception("Playlist WaitNextEvent ABI");h.value(sp+14,1,0);h.pop(14);e.addProperty("modifiers",h.word(0x447218));}
                else if(op==0xa858){long left=h.pointer(sp),right=h.pointer(sp+4),result=left&right;h.value(sp+8,4,result);h.pop(8);e.addProperty("left",left);e.addProperty("right",right);e.addProperty("result",result);}
                else if(op==0xa93e){int key=h.word(sp),choice=0;for(int m:bar){JsonObject mm=menus.get(m);long flags=mm.get("enable_flags").getAsLong();if((flags&1)==0)continue;JsonArray items=mm.getAsJsonArray("items");for(int i=0;i<items.size();i++){int shortcut=items.get(i).getAsJsonObject().get("key").getAsInt();if(shortcut!=0&&shortcut!=27&&Character.toUpperCase((char)shortcut)==Character.toUpperCase((char)key)&&(flags&(1L<<(i+1)))!=0)choice=(m<<16)|(i+1);}}if(choice!=selection)throw new Exception("MenuKey resource selection mismatch");h.value(sp+2,4,Integer.toUnsignedLong(choice));h.pop(2);e.addProperty("key",key);e.addProperty("selection",Integer.toUnsignedLong(choice));}
                else throw new Exception("Unqualified menu Toolbox "+Integer.toHexString(op)+" at "+Long.toHexString(pc));
                e.addProperty("sp_after",h.sp());services.add(e);h.jump(pc+2);continue;
            }
            if(pc>=0x400000&&pc<0x401000||pc==0x153ca4)throw new Exception("Unqualified external/menu RNG "+Long.toHexString(pc));if(!h.cpu.step(monitor))throw new Exception("Native menu instruction "+Long.toHexString(pc)+":"+h.cpu.getLastError());
        }throw new Exception("Menu native instruction budget");
    }
    void runCase(JsonArray cases,String name,long entry,long stop,int selection,int old,int editor)throws Exception {
        menus=new HashMap<>();bars=new HashMap<>();bar=new ArrayList<>();getBar(128);install(128);for(int m:new int[]{134,135})menus.put(m,menu(m));services=new JsonArray();music=new JsonArray();ActionOracleHarness h=new ActionOracleHarness(currentProgram);
        try {h.write(0x3fe210,Files.readAllBytes(root.resolve("analysis/ghidra/startup/a5-mapped.bin")));h.value(0x4046ea,2,editor);if(entry!=0x135354){h.enter(0x135354,0x6ffe00,0x910000);h.cpu.writeRegister("A6",0x700000);execute(h,0x1354dc,0);services=new JsonArray();music=new JsonArray();}h.value(0x43956a,2,6);h.value(0x439568,2,25);h.value(0x439594,2,old);h.value(0x447218,2,name.equals("command_key_load")?0x100:0x200);h.write(0x44270e,new byte[13]);if(name.equals("playlist_off"))h.value(0x44270e+2,1,1);h.value(0x438742,4,Integer.toUnsignedLong(selection));h.value(0x44720c,4,0x6c);h.enter(entry,0x6ffe00,0x910000);h.cpu.writeRegister("A6",0x700000);
            JsonObject c=new JsonObject();c.addProperty("case",name);c.addProperty("entry",Long.toHexString(entry));c.addProperty("stop",Long.toHexString(stop));c.addProperty("selected_menu_item",Integer.toUnsignedLong(selection));c.addProperty("editor_control",editor);c.add("before",state(h));execute(h,stop,selection);c.add("after",state(h));c.add("services",services);c.add("music_native",music);cases.add(c);println("Native menu boundary "+name);
        }finally{h.cpu.dispose();}
    }
    @Override public void run()throws Exception {
        root=Path.of(getScriptArgs()[0]);JsonArray cases=new JsonArray();
        runCase(cases,"install_demo",0x135354,0x1354dc,0,0,0);runCase(cases,"install_registered",0x135354,0x1354dc,0,0,1);
        runCase(cases,"draw_bar",0x13283e,0x132864,0,0,0);
        for(int speed:new int[]{1,6,11})runCase(cases,"speed_"+speed,0x1279bc,0x910000,(134<<16)|speed,0,0);
        runCase(cases,"playlist_on",0x1279bc,0x910000,(145<<16)|12,0,0);runCase(cases,"playlist_off",0x1279bc,0x910000,(145<<16)|12,0,0);
        runCase(cases,"music_stop",0x1279bc,0x910000,(145<<16)|1,0,0);runCase(cases,"music_resume_busy",0x1279bc,0x910000,(145<<16)|1,1,0);
        runCase(cases,"command_key_load",0x1329a2,0x12f0cc,(129<<16)|1,0,0);
        JsonObject out=new JsonObject();out.addProperty("authority","Finite original menu installation prefixes and full speed, Shift-playlist, stop/resume-busy dispatch. Original mapped A5 baseline, including empty version buffer, is an explicit control rather than complete ToolBoxInit reconstruction. Explicit Toolbox resource/menu/event replies; savepref is a captured request without original file writes; keyboard/MenuKey result derives from the original enabled resource shortcut. No live menu/hardware, complete MenuInit, mission/combat enabling or gameplay claim.");JsonObject identity=new JsonObject();for(String p:new String[]{"analysis/completion/ui-menu/VerifyMenuBoundary.java","tools/ghidra/ActionOracleHarness.java","tools/ghidra/SubmitMusicBoundary.java","analysis/ghidra/startup/code1-mapped.bin","analysis/ghidra/startup/a5-mapped.bin"})identity.addProperty(p,ActionOracleHarness.sha256(root.resolve(p)));out.add("identity",identity);JsonObject cache=new JsonObject();cache.addProperty("probe_sha256",identity.get("analysis/completion/ui-menu/VerifyMenuBoundary.java").getAsString());cache.addProperty("harness_sha256",identity.get("tools/ghidra/ActionOracleHarness.java").getAsString());cache.addProperty("music_boundary_sha256",identity.get("tools/ghidra/SubmitMusicBoundary.java").getAsString());cache.addProperty("mapped_binary_sha256",identity.get("analysis/ghidra/startup/code1-mapped.bin").getAsString());cache.addProperty("ghidra","12.1.2");out.add("cache_identity",cache);out.add("cases",cases);Files.writeString(root.resolve("analysis/completion/ui-menu/menu-boundary.json"),out+"\n");
    }
}
