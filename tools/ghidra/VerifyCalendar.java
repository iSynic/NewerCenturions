import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.ByteBuffer;
import java.util.HexFormat;

public class VerifyCalendar extends GhidraScript {
    int word(EmulatorHelper cpu, long address) { return ByteBuffer.wrap(cpu.readMemory(toAddr(address),2)).getShort(); }
    long pointer(EmulatorHelper cpu, long address) { return Integer.toUnsignedLong(ByteBuffer.wrap(cpu.readMemory(toAddr(address),4)).getInt()); }
    void ret(EmulatorHelper cpu) { long sp=cpu.readRegister("SP").longValue(); cpu.writeRegister("PC",pointer(cpu,sp)); cpu.writeRegister("SP",sp+4); }
    public void run() throws Exception {
        Path root=Path.of(getScriptArgs()[0]); HexFormat hex=HexFormat.of();
        byte[] initial=new byte[6*3138];
        String[] names={"Cindred","Coriand","Kevlar","PDP 11","Sifini","Solestri"};
        for(int i=0;i<6;i++) System.arraycopy(Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Warriors/"+names[i])),0,initial,i*3138,3138);
        byte[] authored=Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data TD3"));
        byte[] food=java.util.Arrays.copyOf(Files.readAllBytes(root.resolve("NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data NI")),150);
        StringBuilder cases=new StringBuilder();
        for(int variant=0;variant<6;variant++) {
            EmulatorHelper cpu=new EmulatorHelper(currentProgram);
            try {
                for(String reg:new String[]{"D0","D1","D2","D3","D4","D5","D6","D7","A0","A1","A2","A3","A4","A5","A6","XF","NF","ZF","VF","CF"}) cpu.writeRegister(reg,0);
                int hour=variant==0||variant==3?11:23, minute=variant==2?50:55, ticks=variant==2?4:1;
                byte[] input=initial.clone(), schedule=authored.clone();
                ByteBuffer.wrap(input).putShort(0x50,(short)20).putShort(0x5e,(short)1);
                if(variant==3) ByteBuffer.wrap(input).putShort(0x66,(short)-4).putShort(3138+0x50,(short)26);
                if(variant>=4) ByteBuffer.wrap(schedule).putShort(0,(short)2).putShort(2,(short)3).putShort(4,(short)(variant==4?0:100)).putShort(6,(short)117).putShort(20,(short)0);
                byte[] beforeSchedule=schedule.clone();
                cpu.writeMemory(toAddr(0x448aa0),input);
                cpu.writeMemory(toAddr(0x470cbe),food);
                cpu.writeMemory(toAddr(0x442562),new byte[20]); cpu.writeMemory(toAddr(0x440edc),new byte[6*288]);
                cpu.writeMemoryValue(toAddr(0x44273a),1,5); cpu.writeMemoryValue(toAddr(0x438776),2,0); cpu.writeMemoryValue(toAddr(0x44274a),1,0);
                cpu.writeMemoryValue(toAddr(0x442739),1,0); cpu.writeMemoryValue(toAddr(0x442750),1,0); cpu.writeMemoryValue(toAddr(0x44257c),2,0);
                cpu.writeMemoryValue(toAddr(0x4427c8),2,minute); cpu.writeMemoryValue(toAddr(0x4427ca),2,hour); cpu.writeMemoryValue(toAddr(0x4427d4),2,1); cpu.writeMemoryValue(toAddr(0x442578),2,4);
                cpu.writeRegister("PC",0x15e1a8); cpu.writeRegister("SP",0x700000); cpu.writeRegister("A6",0x700100);
                cpu.writeMemoryValue(toAddr(0x700100-0xb),1,5); cpu.writeMemoryValue(toAddr(0x700100-5),1,0); cpu.writeMemoryValue(toAddr(0x700100-0xa),2,ticks);
                long seed=1; int draws=0, cursor=0, dispatches=0; boolean stopped=false;
                for(int step=0;step<3000000;step++) {
                    long pc=cpu.getExecutionAddress().getOffset(), sp=cpu.readRegister("SP").longValue();
                    if(pc==0x15e75a) {
                        if((cpu.readRegister("D5").longValue()&255)!=0) {cpu.writeRegister("PC",0x15e8d8);continue;}
                        stopped=true; break;
                    }
                    if(pc==0x15ec56||pc==0x1600fc||pc==0x13a498||pc==0x15f66c||pc==0x136d86) {ret(cpu);continue;}
                    if(pc==0x14de26){cpu.writeRegister("PC",0x14dede);continue;}
                    if(pc==0x160a98){cpu.writeRegister("PC",0x160bb0);continue;}
                    if(pc==0x160bda){cpu.writeRegister("PC",0x160cf6);continue;}
                    if(pc==0x400070){cpu.writeRegister("A0",0x600000);ret(cpu);continue;}
                    if(pc==0x400078){cursor=(int)pointer(cpu,sp+8);cpu.writeRegister("D0",0);ret(cpu);continue;}
                    if(pc==0x400048){
                        int size=(int)pointer(cpu,sp+8); long destination=pointer(cpu,sp+4);
                        if(cursor+size<=schedule.length){cpu.writeMemory(toAddr(destination),java.util.Arrays.copyOfRange(schedule,cursor,cursor+size));cursor+=size;cpu.writeRegister("D0",1);}
                        else cpu.writeRegister("D0",0);
                        ret(cpu);continue;
                    }
                    if(pc==0x400050){int size=(int)pointer(cpu,sp+8);if(cursor+size>schedule.length)throw new Exception("Schedule write outside file");System.arraycopy(cpu.readMemory(toAddr(pointer(cpu,sp+4)),size),0,schedule,cursor,size);cursor+=size;cpu.writeRegister("D0",1);ret(cpu);continue;}
                    if(pc==0x400068){cpu.writeRegister("D0",0);ret(cpu);continue;}
                    if(pc==0x153ca4){int bound=word(cpu,sp+4);seed=seed*16807%2147483647;short raw=(short)seed;if(raw==Short.MIN_VALUE)raw=0;cpu.writeRegister("D0",Math.abs((int)raw)*bound/32768+1);draws++;ret(cpu);continue;}
                    if(pc==0x1400e4){dispatches++;cpu.writeRegister("D0",0);ret(cpu);continue;}
                    if(!cpu.step(monitor))throw new Exception("Variant "+variant+" PC "+Long.toHexString(pc)+": "+cpu.getLastError());
                }
                if(!stopped)throw new Exception("Calendar instruction budget variant "+variant+" PC "+cpu.getExecutionAddress());
                if(cases.length()>0)cases.append(',');
                cases.append("{\"variant\":").append(variant).append(",\"input\":\"").append(hex.formatHex(input)).append("\",\"output\":\"").append(hex.formatHex(cpu.readMemory(toAddr(0x448aa0),input.length))).append("\",\"schedule_before\":\"").append(hex.formatHex(beforeSchedule)).append("\",\"schedule_after\":\"").append(hex.formatHex(schedule)).append("\",\"hour_before\":").append(hour).append(",\"minute_before\":").append(minute).append(",\"ticks\":").append(ticks).append(",\"day_after\":").append(word(cpu,0x4427d4)).append(",\"hour_after\":").append(word(cpu,0x4427ca)).append(",\"minute_after\":").append(word(cpu,0x4427c8)).append(",\"fatigue_after\":").append(word(cpu,0x442578)).append(",\"seed_after\":").append(seed).append(",\"draws\":").append(draws).append(",\"dispatches\":").append(dispatches).append('}');
            } finally {cpu.dispose();}
        }
        Files.writeString(root.resolve("evidence/fixtures/calendar.json"),"{\"authority\":\"Centurions 1.3.2 timeclick hourly/noon/midnight instructions, original reduce, checkforitem, loaditem and heal; drawing omitted, file calls modeled with authored bytes and normal EOF, verified RNG boundary modeled; newland dispatch intercepted with no state effects\",\"food_definition\":\""+hex.formatHex(food)+"\",\"cases\":["+cases+"]}\n");
        println("Calendar instruction cases exported");
    }
}
