package gay.crimew.inputmethod.latin.emojisearch;

import com.android.inputmethod.latin.utils.JniUtils;

import java.util.ArrayList;
import java.util.List;

public class EmojiSearch {
    static {
        JniUtils.loadNativeLibrary();
    }

    private static EmojiSearch instance;

    private EmojiSearch() {
    }

    public static List<String> search(String query) {
        List<String> results = new ArrayList<>();
        searchNative(query.toLowerCase(), false, results);
        return results;
    }

    public static List<String> searchExact(String query) {
        List<String> results = new ArrayList<>();
        searchNative(query.toLowerCase(), true, results);
        return results;
    }

    private static native void searchNative(String query, boolean exact, List<String> outArray);
}
