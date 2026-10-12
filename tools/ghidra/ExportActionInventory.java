import ghidra.app.script.GhidraScript;
import ghidra.app.decompiler.DecompInterface;
import ghidra.program.model.block.BasicBlockModel;
import ghidra.program.model.listing.Function;
import ghidra.program.model.listing.Instruction;
import ghidra.program.model.pcode.PcodeOp;
import ghidra.program.model.pcode.Varnode;
import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import com.google.gson.GsonBuilder;
import java.nio.file.Files;
import java.nio.file.Path;

/** Recover every application function and its ordered blocks for translation. */
public class ExportActionInventory extends GhidraScript {
    private JsonObject node(Varnode value) {
        JsonObject result = new JsonObject();
        result.addProperty("space", value.getAddress().getAddressSpace().getName());
        result.addProperty("offset", Long.toUnsignedString(value.getOffset(), 16));
        result.addProperty("bytes", value.getSize());
        return result;
    }

    public void run() throws Exception {
        String[] arguments = getScriptArgs();
        if (arguments.length != 1) throw new IllegalArgumentException("Expected a fresh output directory");
        Path output = Path.of(arguments[0]);
        if (Files.exists(output)) throw new IllegalArgumentException("Output directory must be fresh");
        Files.createDirectories(output);
        JsonObject inventory = new JsonObject();
        inventory.addProperty("schema_version", 1);
        inventory.addProperty("program", currentProgram.getName());
        inventory.addProperty("scope", "Mapped CODE 1 application functions; excludes runtime/math/import thunks");
        JsonArray functions = new JsonArray();
        DecompInterface decompiler = new DecompInterface();
        decompiler.openProgram(currentProgram);
        BasicBlockModel blockModel = new BasicBlockModel(currentProgram);
        try {
            for (Function function : currentProgram.getFunctionManager().getFunctions(true)) {
                monitor.checkCancelled();
                long entry = function.getEntryPoint().getOffset();
                if (entry < 0x100000L || entry >= 0x200000L) continue;
                JsonObject item = new JsonObject();
                item.addProperty("name", function.getName());
                item.addProperty("entry", function.getEntryPoint().toString());
                item.addProperty("body_bytes", function.getBody().getNumAddresses());
                JsonArray ranges = new JsonArray();
                for (var range : function.getBody().getAddressRanges()) {
                    JsonObject span = new JsonObject();
                    span.addProperty("start", range.getMinAddress().toString());
                    span.addProperty("end", range.getMaxAddress().toString());
                    ranges.add(span);
                }
                item.add("ranges", ranges);
                JsonArray calls = new JsonArray();
                for (Function called : function.getCalledFunctions(monitor)) {
                    JsonObject call = new JsonObject();
                    call.addProperty("name", called.getName());
                    call.addProperty("entry", called.getEntryPoint().toString());
                    calls.add(call);
                }
                item.add("calls", calls);
                var decoded = decompiler.decompileFunction(function, 30, monitor);
                item.addProperty("decompiled", decoded.decompileCompleted());
                if (decoded.decompileCompleted()) {
                    Files.writeString(output.resolve(function.getEntryPoint() + ".c"), decoded.getDecompiledFunction().getC());
                } else {
                    item.addProperty("decompiler_error", decoded.getErrorMessage());
                }
                JsonArray blocks = new JsonArray();
                var blockIterator = blockModel.getCodeBlocksContaining(function.getBody(), monitor);
                while (blockIterator.hasNext()) {
                    var block = blockIterator.next();
                    JsonObject atom = new JsonObject();
                    atom.addProperty("entry", block.getFirstStartAddress().toString());
                    JsonArray destinations = new JsonArray();
                    var destinationsIterator = block.getDestinations(monitor);
                    while (destinationsIterator.hasNext()) {
                        var destination = destinationsIterator.next();
                        JsonObject edge = new JsonObject();
                        edge.addProperty("target", destination.getDestinationAddress().toString());
                        edge.addProperty("flow", destination.getFlowType().toString());
                        destinations.add(edge);
                    }
                    atom.add("destinations", destinations);
                    JsonArray instructions = new JsonArray();
                    for (Instruction instruction : currentProgram.getListing().getInstructions(block, true)) {
                        if (!function.getBody().contains(instruction.getAddress())) continue;
                        JsonObject step = new JsonObject();
                        step.addProperty("address", instruction.getAddress().toString());
                        step.addProperty("instruction", instruction.toString());
                        JsonArray operations = new JsonArray();
                        for (PcodeOp operation : instruction.getPcode()) {
                            JsonObject operationItem = new JsonObject();
                            operationItem.addProperty("opcode", operation.getMnemonic());
                            if (operation.getOutput() != null) operationItem.add("output", node(operation.getOutput()));
                            JsonArray inputs = new JsonArray();
                            for (int input = 0; input < operation.getNumInputs(); input++) inputs.add(node(operation.getInput(input)));
                            operationItem.add("inputs", inputs);
                            operations.add(operationItem);
                        }
                        step.add("operations", operations);
                        instructions.add(step);
                    }
                    atom.add("instructions", instructions);
                    blocks.add(atom);
                }
                item.add("blocks", blocks);
                functions.add(item);
                println("Recovered " + function.getName() + " at " + function.getEntryPoint());
            }
        } finally {
            decompiler.dispose();
        }
        inventory.add("functions", functions);
        Files.writeString(output.resolve("actions.json"), new GsonBuilder().setPrettyPrinting().create().toJson(inventory) + "\n");
        println("Recovered " + functions.size() + " application functions with ordered blocks and p-code");
    }
}
