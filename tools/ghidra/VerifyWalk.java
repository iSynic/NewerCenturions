import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.ByteBuffer;
import java.util.HexFormat;

public class VerifyWalk extends GhidraScript {
    int word(EmulatorHelper cpu,long a){return ByteBuffer.wrap(cpu.readMemory(toAddr(a),2)).getShort();}
    long pointer(EmulatorHelper cpu,long a){return Integer.toUnsignedLong(ByteBuffer.wrap(cpu.readMemory(toAddr(a),4)).getInt());}
    void ret(EmulatorHelper cpu){long sp=cpu.readRegister("SP").longValue();cpu.writeRegister("PC",pointer(cpu,sp));cpu.writeRegister("SP",sp+4);}
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]);HexFormat hex=HexFormat.of();
        byte[] initial=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Warriors/Cindred"));
        byte[] properties=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Data Files/Data Swamp BD"));
        StringBuilder cases=new StringBuilder();
        for(int dx=-1;dx<=1;dx++)for(int dy=-1;dy<=1;dy++)if(dx!=0||dy!=0)for(int tile:new int[]{1155,1206,1000})for(int points:new int[]{0,1,30}) {
            EmulatorHelper cpu=new EmulatorHelper(currentProgram);
            try {
                for(String reg:new String[]{"D0","D1","D2","D3","D4","D5","D6","D7","A0","A1","A2","A3","A4","A5","A6","XF","NF","ZF","VF","CF"})cpu.writeRegister(reg,0);
                byte[] input=initial.clone();ByteBuffer.wrap(input).putShort(0x3c,(short)points);input[0x9df]=1;
                byte[] map=new byte[16200];ByteBuffer mapWords=ByteBuffer.wrap(map);for(int i=0;i<8100;i++)mapWords.putShort(i*2,(short)1155);
                mapWords.putShort((25*90+25)*2,(short)0);mapWords.putShort(((25+dx)*90+25+dy)*2,(short)tile);
                cpu.writeMemory(toAddr(0x4085b8),map);cpu.writeMemory(toAddr(0x448aa0),input);
                cpu.writeMemory(toAddr(0x43167c),java.util.Arrays.copyOfRange(properties,64,12064));
                cpu.writeMemory(toAddr(0x442bd6),new byte[1200]);cpu.writeMemoryValue(toAddr(0x4435d2),2,0);cpu.writeMemoryValue(toAddr(0x438764),2,0);
                cpu.writeMemoryValue(toAddr(0x442780),4,5);cpu.writeMemoryValue(toAddr(0x442784),4,5);
                cpu.writeMemoryValue(toAddr(0x44709a),2,20);cpu.writeMemoryValue(toAddr(0x44709c),2,20);cpu.writeMemoryValue(toAddr(0x437a6e),2,1155);
                cpu.writeMemoryValue(toAddr(0x442736),1,dx);cpu.writeMemoryValue(toAddr(0x442737),1,dy);cpu.writeMemoryValue(toAddr(0x43565f),1,0);
                long frame=0x701000;cpu.writeMemory(toAddr(frame-512),new byte[1024]);cpu.writeRegister("PC",0x11c0b2);cpu.writeRegister("SP",0x700000);cpu.writeRegister("A6",frame);
                boolean stopped=false;StringBuilder sounds=new StringBuilder();
                for(int step=0;step<20000;step++) {
                    long pc=cpu.getExecutionAddress().getOffset(),sp=cpu.readRegister("SP").longValue();
                    if(pc==0x11cd42||pc==0x11cb6c){stopped=true;break;}
                    if(pc==0x400038){cpu.writeRegister("D0",Math.abs(word(cpu,sp+4)));ret(cpu);continue;}
                    if(pc==0x110e96){if(word(cpu,sp+4)!=2)throw new Exception("Unexpected opportunity mode");cpu.writeRegister("D0",0);ret(cpu);continue;}
                    if(pc==0x133a18){if(sounds.length()>0)sounds.append(',');sounds.append(word(cpu,sp+4));ret(cpu);continue;}
                    if(pc==0x16043e||pc==0x10e6b6){ret(cpu);continue;}
                    if(!cpu.step(monitor))throw new Exception("PC "+Long.toHexString(pc)+": "+cpu.getLastError());
                }
                if(!stopped)throw new Exception("Walk instruction budget");
                if(cases.length()>0)cases.append(',');cases.append("{\"dx\":").append(dx).append(",\"dy\":").append(dy).append(",\"tile\":").append(tile).append(",\"points\":").append(points).append(",\"input\":\"").append(hex.formatHex(input)).append("\",\"after\":\"").append(hex.formatHex(cpu.readMemory(toAddr(0x448aa0),3138))).append("\",\"tiles\":\"").append(hex.formatHex(cpu.readMemory(toAddr(0x4085b8),16200))).append("\",\"x\":").append(word(cpu,0x44709a)).append(",\"y\":").append(word(cpu,0x44709c)).append(",\"under\":").append(word(cpu,0x437a6e)).append(",\"sounds\":[").append(sounds).append("]}");
            } finally {cpu.dispose();}
        }
        Files.writeString(root.resolve("evidence/fixtures/walk.json"),"{\"authority\":\"combatchoice walking instructions through original collide; no adjacent bodies or persistent spell fields; presentation intercepted\",\"properties\":\""+hex.formatHex(properties)+"\",\"cases\":["+cases+"]}\n");
        println("72 tactical walking cases exported");
    }
}
