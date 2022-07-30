package gay.crimew.inputmethod.latin.emojisearch;

import android.content.Context;
import android.util.JsonReader;
import android.util.JsonToken;

import com.android.inputmethod.latin.utils.JniUtils;

import java.io.IOException;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

public class EmojiSearch {
    static {
        JniUtils.loadNativeLibrary();
    }

    private static EmojiSearch instance;

    public static Map<String, List<String>> data;

    private EmojiSearch(Context context) {
        data = loadData(context);
        initNative();
    }

    public static EmojiSearch getInstance() {
        return instance;
    }

    public static void init(Context context) {
        instance = new EmojiSearch(context);
    }

    private native void initNative();

    public List<String> search(String query) {
        List<String> results = new ArrayList<>();
        searchNative(query.toLowerCase(), false, results);
        return results;
    }

    public List<String> searchExact(String query) {
        List<String> results = new ArrayList<>();
        searchNative(query.toLowerCase(), true, results);
        return results;
    }

    private native void searchNative(String query, boolean exact, List<String> outArray);

    /**
     * Loads emoji data from data json (grabbed from https://github.com/muan/emojilib)
     */
    private static Map<String, List<String>> loadData(Context context) {
        Map<String, List<String>> map = new HashMap<>();
        try {
            InputStream jsonFile = context.getAssets().open("emoji-en-US.json");
            JsonReader reader = new JsonReader(new InputStreamReader(jsonFile));

            reader.beginObject();
            while (reader.hasNext()) {
                String emoji = reader.nextName();
                List<String> keywords = new ArrayList<>();
                reader.beginArray();
                while (reader.hasNext() && reader.peek() == JsonToken.STRING) {
                    keywords.add(reader.nextString());
                }
                reader.endArray();
                map.put(emoji, keywords);
            }
            reader.endObject();

        } catch (IOException e) {
            e.printStackTrace();
        }
        return map;
    }
}
