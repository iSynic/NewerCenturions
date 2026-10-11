import ghidra.app.script.GhidraScript;
import ghidra.program.model.data.*;
import ghidra.program.model.listing.*;
import ghidra.program.model.symbol.SourceType;

public class SetKnownTypes extends GhidraScript {
    private void signature(long address,String name,DataType result,DataType... arguments) throws Exception {
        var function=getFunctionAt(toAddr(address));
        if(function==null) { disassemble(toAddr(address)); function=createFunction(toAddr(address),name); }
        if(function==null) throw new Exception("No function at "+Long.toHexString(address));
        function.setName(name,SourceType.USER_DEFINED);
        function.setReturnType(result,SourceType.USER_DEFINED);
        Parameter[] params=new Parameter[arguments.length];
        for(int i=0;i<arguments.length;i++) params[i]=new ParameterImpl("arg"+i,arguments[i],currentProgram);
        function.replaceParameters(Function.FunctionUpdateType.DYNAMIC_STORAGE_ALL_PARAMS,true,SourceType.USER_DEFINED,params);
    }
    public void run() throws Exception {
        var shortType=ShortDataType.dataType;
        var longType=LongDataType.dataType;
        var pointer=new PointerDataType(VoidDataType.dataType,4,currentProgram.getDataTypeManager());
        signature(0x300308,"abs_short",shortType,shortType);
        signature(0x300ab4,"fread",longType,pointer,longType,longType,pointer);
        signature(0x300d12,"fwrite",longType,pointer,longType,longType,pointer);
        signature(0x300fca,"fclose",shortType,pointer);
        signature(0x3010f4,"fopen",pointer,pointer,pointer);
        signature(0x301468,"fseek",shortType,pointer,longType,shortType);
        signature(0x13846a,"twixt",shortType,shortType,shortType,shortType);
        signature(0x13849e,"randrange",shortType,shortType,shortType);
        signature(0x153ca4,"Rand",shortType,shortType);
        signature(0x153de6,"getmovecost",shortType,shortType);
        signature(0x130afe,"loaditem",longType,shortType);
        signature(0x13107a,"loadland",longType,longType);
        signature(0x13a7c4,"loadprofile",longType,shortType);
        println("Assembly-backed signatures applied; Toolbox traps remain opaque");
    }
}
