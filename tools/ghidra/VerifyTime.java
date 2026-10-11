import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.ByteBuffer;
import java.util.HexFormat;

public class VerifyTime extends GhidraScript {
    void returnFrom(EmulatorHelper cpu) throws Exception {
        long sp=cpu.readRegister("SP").longValue();
        cpu.writeRegister("PC",Integer.toUnsignedLong(ByteBuffer.wrap(cpu.readMemory(toAddr(sp),4)).getInt()));
        cpu.writeRegister("SP",sp+4);
    }
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]);
        byte[] initial=new byte[3138*6];
        String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};
        for(int index=0;index<6;index++) {
            byte[] bytes=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Warriors/"+names[index]));
            System.arraycopy(bytes,0,initial,index*3138,3138);
        }
        StringBuilder cases=new StringBuilder();
        for(int variant=0;variant<3;variant++) {
            EmulatorHelper cpu=new EmulatorHelper(currentProgram);
            try {
                for(String reg:new String[]{"D0","D1","D2","D3","D4","D5","D6","D7","A0","A1","A2","A3","A4","A5","A6","XF","NF","ZF","VF","CF"})cpu.writeRegister(reg,0);
                byte[] input=initial.clone();
                if(variant==1) { ByteBuffer.wrap(input).putShort(0x50,(short)20).putShort(0x5e,(short)3).putShort(3138+0x58,(short)20); }
                if(variant==2) { ByteBuffer.wrap(input).putShort(0x50,(short)20).putShort(0x66,(short)-4).putShort(3138+0x58,(short)32); }
                cpu.writeMemory(toAddr(0x448aa0),input);
                cpu.writeMemory(toAddr(0x442562),new byte[20]);
                cpu.writeMemory(toAddr(0x440edc),new byte[6*288]);
                cpu.writeMemoryValue(toAddr(0x44273a),1,5);cpu.writeMemoryValue(toAddr(0x438776),2,0);cpu.writeMemoryValue(toAddr(0x44274a),1,0);
                cpu.writeMemoryValue(toAddr(0x442739),1,0);cpu.writeMemoryValue(toAddr(0x442750),1,0);
                cpu.writeMemoryValue(toAddr(0x44257c),2,0);
                cpu.writeMemoryValue(toAddr(0x4427c8),2,55);cpu.writeMemoryValue(toAddr(0x4427ca),2,12);
                cpu.writeMemoryValue(toAddr(0x442578),2,variant==2?135:4);
                cpu.writeRegister("PC",0x15e1a8);cpu.writeRegister("SP",0x700000);cpu.writeRegister("A6",0x700100);
                cpu.writeMemoryValue(toAddr(0x700100-0xb),1,5);cpu.writeMemoryValue(toAddr(0x700100-5),1,0);
                boolean stopped=false;
                for(int step=0;step<100000;step++) {
                    long pc=cpu.getExecutionAddress().getOffset();
                    if(pc==0x15e3a4) { stopped=true;break; }
                    if(pc==0x15ec56||pc==0x1600fc||pc==0x13a498) {returnFrom(cpu);continue;}
                    // Omit only drawing in reduce and updatefat; execute their state writes.
                    if(pc==0x14de26){cpu.writeRegister("PC",0x14dede);continue;}
                    if(pc==0x160a98){cpu.writeRegister("PC",0x160bb0);continue;}
                    if(pc==0x160bda){cpu.writeRegister("PC",0x160cf6);continue;}
                    if(!cpu.step(monitor))throw new Exception("PC "+Long.toHexString(pc)+": "+cpu.getLastError());
                }
                if(!stopped)throw new Exception("Time instruction budget");
                if(cases.length()>0)cases.append(',');
                cases.append("{\"input\":\"").append(HexFormat.of().formatHex(input)).append("\",\"output\":\"").append(HexFormat.of().formatHex(cpu.readMemory(toAddr(0x448aa0),initial.length))).append("\",\"fatigue_before\":").append(variant==2?135:4).append(",\"fatigue_after\":").append(ByteBuffer.wrap(cpu.readMemory(toAddr(0x442578),2)).getShort()).append(",\"hour_after\":").append(ByteBuffer.wrap(cpu.readMemory(toAddr(0x4427ca),2)).getShort()).append(",\"minute_after\":").append(ByteBuffer.wrap(cpu.readMemory(toAddr(0x4427c8),2)).getShort()).append('}');
            }finally{cpu.dispose();}
        }
        Files.writeString(root.resolve("evidence/fixtures/time.json"),"{\"authority\":\"Centurions 1.3.2 timeclick hourly instruction slice, actual reduce and heal; presentation omitted\",\"cases\":["+cases+"]}\n");
        println("Three hourly instruction cases exported");
    }
}
