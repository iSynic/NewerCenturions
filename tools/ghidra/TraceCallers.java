import ghidra.app.script.GhidraScript;
import java.nio.file.Files;
import java.nio.file.Path;

public class TraceCallers extends GhidraScript {
    public void run() throws Exception {
        StringBuilder output = new StringBuilder();
        for (int i = 1; i < getScriptArgs().length; i++) {
            var function = getGlobalFunctions(getScriptArgs()[i]).get(0);
            for (var reference : getReferencesTo(function.getEntryPoint())) {
                var caller = getFunctionContaining(reference.getFromAddress());
                output.append(function.getName()).append('\t').append(reference.getFromAddress())
                    .append('\t').append(caller == null ? "unknown" : caller.getName()).append('\n');
            }
        }
        Files.writeString(Path.of(getScriptArgs()[0]), output.toString());
    }
}
