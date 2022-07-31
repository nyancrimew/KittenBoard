/*
 * Copyright (C) 2013 The Android Open Source Project
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package com.android.inputmethod.keyboard.emoji;

import android.content.SharedPreferences;
import android.text.TextUtils;
import android.util.Log;

import com.android.inputmethod.keyboard.Key;
import com.android.inputmethod.keyboard.Keyboard;
import com.android.inputmethod.latin.settings.Settings;
import com.android.inputmethod.latin.utils.JsonUtils;

import java.util.ArrayDeque;
import java.util.ArrayList;
import java.util.Collection;
import java.util.Collections;
import java.util.List;

/**
 * This is a Keyboard class where you can add keys dynamically shown in a grid layout
 */
final class DynamicGridKeyboard extends Keyboard {
    private static final String TAG = DynamicGridKeyboard.class.getSimpleName();
    private static final int TEMPLATE_KEY_CODE_0 = 0x30;
    private static final int TEMPLATE_KEY_CODE_1 = 0x31;
    private final Object mLock = new Object();

    private final SharedPreferences mPrefs;
    private final int mHorizontalStep;
    private final int mVerticalStep;
    private final int mColumnsNum;
    private final int mMaxKeyCount;
    private final boolean mIsRecents;
    private final ArrayDeque<GridKey> mGridKeys = new ArrayDeque<>();
    private final ArrayDeque<Key> mPendingKeys = new ArrayDeque<>();

    private List<Key> mCachedGridKeys;

    public DynamicGridKeyboard(final SharedPreferences prefs, final Keyboard templateKeyboard,
            final int maxKeyCount, final int categoryId) {
        super(templateKeyboard);
        final Key key0 = getTemplateKey(TEMPLATE_KEY_CODE_0);
        final Key key1 = getTemplateKey(TEMPLATE_KEY_CODE_1);
        mHorizontalStep = Math.abs(key1.getX() - key0.getX());
        mVerticalStep = key0.getHeight() + mVerticalGap;
        mColumnsNum = mBaseWidth / mHorizontalStep;
        mMaxKeyCount = maxKeyCount;
        mIsRecents = categoryId == EmojiCategory.ID_RECENTS;
        mPrefs = prefs;
    }

    private Key getTemplateKey(final int code) {
        for (final Key key : super.getSortedKeys()) {
            if (key.getCode() == code) {
                return key;
            }
        }
        throw new RuntimeException("Can't find template key: code=" + code);
    }

    public void addPendingKey(final Key usedKey) {
        synchronized (mLock) {
            mPendingKeys.addLast(usedKey);
        }
    }

    public void flushPendingRecentKeys() {
        synchronized (mLock) {
            while (!mPendingKeys.isEmpty()) {
                addKey(mPendingKeys.pollFirst(), true);
            }
            saveRecentKeys();
        }
    }

    public void addKeyFirst(final Key usedKey) {
        addKey(usedKey, true);
        if (mIsRecents) {
            saveRecentKeys();
        }
    }

    public void addKeyLast(final Key usedKey) {
        addKey(usedKey, false);
    }

    private void addKey(final Key usedKey, final boolean addFirst) {
        if (usedKey == null) {
            return;
        }
        synchronized (mLock) {
            mCachedGridKeys = null;
            final GridKey key = new GridKey(usedKey);
            while (mGridKeys.remove(key)) {
                // Remove duplicate keys.
            }
            if (addFirst) {
                mGridKeys.addFirst(key);
            } else {
                mGridKeys.addLast(key);
            }
            while (mGridKeys.size() > mMaxKeyCount) {
                mGridKeys.removeLast();
            }
            int index = 0;
            for (final GridKey gridKey : mGridKeys) {
                final int keyX0 = getKeyX0(index);
                final int keyY0 = getKeyY0(index);
                final int keyX1 = getKeyX1(index);
                final int keyY1 = getKeyY1(index);
                gridKey.updateCoordinates(keyX0, keyY0, keyX1, keyY1);
                index++;
            }
        }
    }

    private void saveRecentKeys() {
        final ArrayList<Object> keys = new ArrayList<>();
        for (final Key key : mGridKeys) {
            if (key.getOutputText() != null) {
                keys.add(key.getOutputText());
            } else {
                keys.add(key.getCode());
            }
        }
        final String jsonStr = JsonUtils.listToJsonStr(keys);
        Settings.writeEmojiRecentKeys(mPrefs, jsonStr);
    }

    private static Key getKeyByCode(final Collection<DynamicGridKeyboard> keyboards,
            final int code) {
        for (final DynamicGridKeyboard keyboard : keyboards) {
            for (final Key key : keyboard.getSortedKeys()) {
                if (key.getCode() == code) {
                    return key;
                }
            }
        }
        return null;
    }

    private static Key getKeyByOutputText(final Collection<DynamicGridKeyboard> keyboards,
            final String outputText) {
        for (final DynamicGridKeyboard keyboard : keyboards) {
            for (final Key key : keyboard.getSortedKeys()) {
                if (outputText.equals(key.getOutputText())) {
                    return key;
                }
            }
        }
        return null;
    }

    public void loadRecentKeys(final Collection<DynamicGridKeyboard> keyboards) {
        final String str = Settings.readEmojiRecentKeys(mPrefs);
        final List<Object> keys = JsonUtils.jsonStrToList(str);
        for (final Object o : keys) {
            final Key key;
            if (o instanceof Integer) {
                final int code = (Integer)o;
                key = getKeyByCode(keyboards, code);
            } else if (o instanceof String) {
                final String outputText = (String)o;
                key = getKeyByOutputText(keyboards, outputText);
            } else {
                Log.w(TAG, "Invalid object: " + o);
                continue;
            }
            addKeyLast(key);
        }
    }

    private int getKeyX0(final int index) {
        final int column = index % mColumnsNum;
        return column * mHorizontalStep;
    }

    private int getKeyX1(final int index) {
        final int column = index % mColumnsNum + 1;
        return column * mHorizontalStep;
    }

    private int getKeyY0(final int index) {
        final int row = index / mColumnsNum;
        return row * mVerticalStep + mVerticalGap / 2;
    }

    private int getKeyY1(final int index) {
        final int row = index / mColumnsNum + 1;
        return row * mVerticalStep + mVerticalGap / 2;
    }

    @Override
    public List<Key> getSortedKeys() {
        synchronized (mLock) {
            if (mCachedGridKeys != null) {
                return mCachedGridKeys;
            }
            final ArrayList<Key> cachedKeys = new ArrayList<>(mGridKeys);
            mCachedGridKeys = Collections.unmodifiableList(cachedKeys);
            return mCachedGridKeys;
        }
    }

    @Override
    public List<Key> getNearestKeys(final int x, final int y) {
        // TODO: Calculate the nearest key index in mGridKeys from x and y.
        return getSortedKeys();
    }

    static final class GridKey extends Key {
        private int mCurrentX;
        private int mCurrentY;

        public GridKey(final Key originalKey) {
            super(originalKey);
        }

        public void updateCoordinates(final int x0, final int y0, final int x1, final int y1) {
            mCurrentX = x0;
            mCurrentY = y0;
            getHitBox().set(x0, y0, x1, y1);
        }

        @Override
        public int getX() {
            return mCurrentX;
        }

        @Override
        public int getY() {
            return mCurrentY;
        }

        @Override
        public boolean equals(final Object o) {
            if (!(o instanceof Key)) return false;
            final Key key = (Key)o;
            if (getCode() != key.getCode()) return false;
            if (!TextUtils.equals(getLabel(), key.getLabel())) return false;
            return TextUtils.equals(getOutputText(), key.getOutputText());
        }

        @Override
        public String toString() {
            return "GridKey: " + super.toString();
        }
    }
}
