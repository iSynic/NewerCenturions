/** Native result audio before remaining presentation acknowledgement. */
public class ResultAudioBoundary {
    static void execute(ActionOracleHarness h,VerifyStoryInventory.RunResult result,ghidra.util.task.TaskMonitor monitor)throws Exception {
        int code=h.word(h.sp()+6),attacker=h.word(h.sp()+8),target=h.word(h.sp()+4);
        if(code!=0&&code!=-1)return;
        String[] names={"D0","D1","D2","D3","D4","D5","D6","D7","A0","A1","A2","A3","A4","A5","A6","SP","PC","XF","NF","ZF","VF","CF"};
        java.math.BigInteger[] saved=new java.math.BigInteger[names.length];for(int i=0;i<names.length;i++)saved[i]=h.cpu.readRegister(names[i]);
        h.write(0x6dc000,new byte[8192]);h.cpu.writeRegister("SP",0x6dd000);h.cpu.writeRegister("D3",target);h.cpu.writeRegister("D5",attacker);h.jump(code==0?0x158720:0x159042);
        boolean done=false;
        for(int step=0;step<150;step++) {
            long pc=h.pc();if(pc==0x15886e||pc==0x1588fc||pc==0x15904c){done=true;break;}
            if(pc==0x133a18){result.sound(h);continue;}
            int op=h.word(pc)&65535;if(op==0xa873||op==0xa884){h.pop(4);h.jump(pc+2);continue;}
            if(!h.cpu.step(monitor))throw new Exception("Native result audio at "+Long.toHexString(pc)+": "+h.cpu.getLastError());
        }
        if(!done)throw new Exception("Native result audio instruction budget");
        h.value(0x438784,2,0);
        for(int i=0;i<names.length;i++)h.cpu.writeRegister(names[i],saved[i]);
    }
}
