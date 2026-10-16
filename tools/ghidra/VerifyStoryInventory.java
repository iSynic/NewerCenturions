import java.util.HexFormat;

/** Shared byte formatting for the native Sound Manager receipt. */
public final class VerifyStoryInventory {
    private VerifyStoryInventory() {}

    static String hex(byte[] value) {
        return HexFormat.of().formatHex(value);
    }
}
