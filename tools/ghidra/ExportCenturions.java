import ghidra.app.script.GhidraScript;
import ghidra.app.decompiler.DecompInterface;
import ghidra.program.model.listing.Function;
import ghidra.program.model.listing.Instruction;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Set;
import java.util.Arrays;
import java.util.stream.Collectors;

public class ExportCenturions extends GhidraScript {
    public void run() throws Exception {
        Path output = Path.of(getScriptArgs()[0]);
        Files.createDirectories(output);
        StringBuilder catalog = new StringBuilder("name\taddress\tbody_bytes\n");
        Set<String> wanted = getScriptArgs().length > 1 ? Arrays.stream(getScriptArgs()).skip(1).flatMap(value -> Arrays.stream(value.split(","))).collect(Collectors.toSet()) : Set.of("segment_loader", "FUN_0010010e", "FUN_00100212", "FUN_00100286", "FUN_001002b2", "FUN_00100312", "main", "loadsave", "savegame", "loadgame",
            "loadcharacter", "savecharacter", "setupnewgame", "newgame", "loaditem", "loadwarrior",
            "addtocrate", "addtobooty", "updatetier", "checkrange", "getrangetobody", "checkforcharge",
            "buildmonster", "attack", "movement", "combat", "castspell", "random", "randomnum",
            "startgame", "loadland", "loadmap", "fileprep", "checkforenemy", "checkforsecret",
            "Rand", "twixt", "randrange", "getmovecost", "moveparty", "getbattleorder",
            "load", "save", "partyselect", "newland", "collide", "loadspell", "loadspell2",
            "calcdamage", "resolvedamage", "canuse", "doreload", "combatsetup", "calcstats");
        DecompInterface decompiler = new DecompInterface();
        decompiler.openProgram(currentProgram);
        for (Function function : currentProgram.getFunctionManager().getFunctions(true)) {
            String name = function.getName();
            catalog.append(name).append('\t').append(function.getEntryPoint()).append('\t')
                .append(function.getBody().getNumAddresses()).append('\n');
            if (!wanted.contains(name)) continue;
            var result = decompiler.decompileFunction(function, 30, monitor);
            if (result.decompileCompleted()) {
                Files.writeString(output.resolve(name + ".c"), result.getDecompiledFunction().getC());
            } else {
                Files.writeString(output.resolve(name + ".error.txt"), result.getErrorMessage());
            }
            StringBuilder instructions = new StringBuilder();
            for (Instruction ins : currentProgram.getListing().getInstructions(function.getBody(), true)) {
                instructions.append(ins.getAddress()).append("\t").append(ins).append("\n");
            }
            Files.writeString(output.resolve(name + ".asm.tsv"), instructions.toString());
        }
        Files.writeString(output.resolve("functions.tsv"), catalog.toString());
        decompiler.dispose();
        println("Function catalog and selected decompilations exported");
    }
}
