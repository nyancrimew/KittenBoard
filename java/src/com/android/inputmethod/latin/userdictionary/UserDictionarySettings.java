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

package com.android.inputmethod.latin.userdictionary;

import android.app.ListFragment;
import android.content.ContentResolver;
import android.content.Context;
import android.content.Intent;
import android.database.Cursor;
import android.os.Bundle;
import android.provider.UserDictionary;
import android.text.TextUtils;
import android.view.LayoutInflater;
import android.view.Menu;
import android.view.MenuInflater;
import android.view.MenuItem;
import android.view.View;
import android.view.ViewGroup;
import android.widget.AlphabetIndexer;
import android.widget.ListAdapter;
import android.widget.ListView;
import android.widget.SectionIndexer;
import android.widget.SimpleCursorAdapter;
import android.widget.TextView;

import java.util.Locale;

import gay.crimew.inputmethod.latin.R;

// Caveat: This class is basically taken from
// packages/apps/Settings/src/com/android/settings/inputmethod/UserDictionarySettings.java
// in order to deal with some devices that have issues with the user dictionary handling

public class UserDictionarySettings extends ListFragment {
    private static final String[] QUERY_PROJECTION =
            {UserDictionary.Words._ID, UserDictionary.Words.WORD, UserDictionary.Words.SHORTCUT};

    // The index of the shortcut in the above array.
    private static final int INDEX_SHORTCUT = 2;

    private static final String[] ADAPTER_FROM = {
            UserDictionary.Words.WORD, UserDictionary.Words.SHORTCUT
    };

    private static final int[] ADAPTER_TO = {
            android.R.id.text1, android.R.id.text2
    };

    // Either the locale is empty (means the word is applicable to all locales)
    // or the word equals our current locale
    private static final String QUERY_SELECTION =
            UserDictionary.Words.LOCALE + "=?";
    private static final String QUERY_SELECTION_ALL_LOCALES =
            UserDictionary.Words.LOCALE + " is null";

    private static final String DELETE_SELECTION_WITH_SHORTCUT = UserDictionary.Words.WORD
            + "=? AND " + UserDictionary.Words.SHORTCUT + "=?";
    private static final String DELETE_SELECTION_WITHOUT_SHORTCUT = UserDictionary.Words.WORD
            + "=? AND " + UserDictionary.Words.SHORTCUT + " is null OR "
            + UserDictionary.Words.SHORTCUT + "=''";
    private static final String DELETE_SELECTION_SHORTCUT_UNSUPPORTED =
            UserDictionary.Words.WORD + "=?";

    private static final int OPTIONS_MENU_ADD = Menu.FIRST;

    private Cursor mCursor;

    protected String mLocale;

    @Override
    public void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        getActivity().getActionBar().setTitle(R.string.edit_personal_dictionary);
    }

    @Override
    public View onCreateView(
            LayoutInflater inflater, ViewGroup container, Bundle savedInstanceState) {
        return inflater.inflate(
                R.layout.user_dictionary_preference_list_fragment, container, false);
    }

    @Override
    public void onActivityCreated(Bundle savedInstanceState) {
        super.onActivityCreated(savedInstanceState);

        final Intent intent = getActivity().getIntent();
        final String localeFromIntent =
                null == intent ? null : intent.getStringExtra("locale");

        final Bundle arguments = getArguments();
        final String localeFromArguments =
                null == arguments ? null : arguments.getString("locale");

        final String locale;
        if (null != localeFromArguments) {
            locale = localeFromArguments;
        } else if (null != localeFromIntent) {
            locale = localeFromIntent;
        } else {
            locale = null;
        }

        mLocale = locale;
        // WARNING: The following cursor is never closed! TODO: don't put that in a member, and
        // make sure all cursors are correctly closed. Also, this comes from a call to
        // Activity#managedQuery, which has been deprecated for a long time (and which FORBIDS
        // closing the cursor, so take care when resolving this TODO). We should either use a
        // regular query and close the cursor, or switch to a LoaderManager and a CursorLoader.
        mCursor = createCursor(locale);
        TextView emptyView = (TextView) getView().findViewById(android.R.id.empty);
        emptyView.setText(R.string.user_dict_settings_empty_text);

        final ListView listView = getListView();
        listView.setAdapter(createAdapter());
        listView.setFastScrollEnabled(true);
        listView.setEmptyView(emptyView);

        setHasOptionsMenu(true);
        // Show the language as a subtitle of the action bar
        getActivity().getActionBar().setSubtitle(
                UserDictionarySettingsUtils.getLocaleDisplayName(getActivity(), mLocale));
    }

    @Override
    public void onResume() {
        super.onResume();
        ListAdapter adapter = getListView().getAdapter();
        if (adapter instanceof MyAdapter) {
            // The list view is forced refreshed here. This allows the changes done 
            // in UserDictionaryAddWordFragment (update/delete/insert) to be seen when 
            // user goes back to this view. 
            MyAdapter listAdapter = (MyAdapter) adapter;
            listAdapter.notifyDataSetChanged();
        }
    }

    @SuppressWarnings("deprecation")
    private Cursor createCursor(final String locale) {
        // Locale can be any of:
        // - The string representation of a locale, as returned by Locale#toString()
        // - The empty string. This means we want a cursor returning words valid for all locales.
        // - null. This means we want a cursor for the current locale, whatever this is.
        // Note that this contrasts with the data inside the database, where NULL means "all
        // locales" and there should never be an empty string. The confusion is called by the
        // historical use of null for "all locales".
        // TODO: it should be easy to make this more readable by making the special values
        // human-readable, like "all_locales" and "current_locales" strings, provided they
        // can be guaranteed not to match locales that may exist.
        if ("".equals(locale)) {
            // Case-insensitive sort
            return getActivity().managedQuery(UserDictionary.Words.CONTENT_URI, QUERY_PROJECTION,
                    QUERY_SELECTION_ALL_LOCALES, null,
                    "UPPER(" + UserDictionary.Words.WORD + ")");
        }
        final String queryLocale = null != locale ? locale : Locale.getDefault().toString();
        return getActivity().managedQuery(UserDictionary.Words.CONTENT_URI, QUERY_PROJECTION,
                QUERY_SELECTION, new String[] { queryLocale },
                "UPPER(" + UserDictionary.Words.WORD + ")");
    }

    private ListAdapter createAdapter() {
        return new MyAdapter(getActivity(), R.layout.user_dictionary_item, mCursor,
                ADAPTER_FROM, ADAPTER_TO);
    }

    @Override
    public void onListItemClick(ListView l, View v, int position, long id) {
        final String word = getWord(position);
        final String shortcut = getShortcut(position);
        if (word != null) {
            showAddOrEditDialog(word, shortcut);
        }
    }

    @Override
    public void onCreateOptionsMenu(Menu menu, MenuInflater inflater) {
        MenuItem actionItem =
                menu.add(0, OPTIONS_MENU_ADD, 0, R.string.user_dict_settings_add_menu_title)
                .setIcon(R.drawable.ic_menu_add);
        actionItem.setShowAsAction(
                MenuItem.SHOW_AS_ACTION_IF_ROOM | MenuItem.SHOW_AS_ACTION_WITH_TEXT);
    }

    @Override
    public boolean onOptionsItemSelected(MenuItem item) {
        if (item.getItemId() == OPTIONS_MENU_ADD) {
            showAddOrEditDialog(null, null);
            return true;
        }
        return false;
    }

    /**
     * Add or edit a word. If editingWord is null, it's an add; otherwise, it's an edit.
     * @param editingWord the word to edit, or null if it's an add.
     * @param editingShortcut the shortcut for this entry, or null if none.
     */
    private void showAddOrEditDialog(final String editingWord, final String editingShortcut) {
        final Bundle args = new Bundle();
        args.putInt(UserDictionaryAddWordContents.EXTRA_MODE, null == editingWord
                ? UserDictionaryAddWordContents.MODE_INSERT
                : UserDictionaryAddWordContents.MODE_EDIT);
        args.putString(UserDictionaryAddWordContents.EXTRA_WORD, editingWord);
        args.putString(UserDictionaryAddWordContents.EXTRA_SHORTCUT, editingShortcut);
        args.putString(UserDictionaryAddWordContents.EXTRA_LOCALE, mLocale);
        android.preference.PreferenceActivity pa =
                (android.preference.PreferenceActivity)getActivity();
        pa.startPreferencePanel(UserDictionaryAddWordFragment.class.getName(),
                args, R.string.user_dict_settings_add_dialog_title, null, null, 0);
    }

    private String getWord(final int position) {
        if (null == mCursor) return null;
        mCursor.moveToPosition(position);
        // Handle a possible race-condition
        if (mCursor.isAfterLast()) return null;

        return mCursor.getString(
                mCursor.getColumnIndexOrThrow(UserDictionary.Words.WORD));
    }

    private String getShortcut(final int position) {
        if (null == mCursor) return null;
        mCursor.moveToPosition(position);
        // Handle a possible race-condition
        if (mCursor.isAfterLast()) return null;

        return mCursor.getString(
                mCursor.getColumnIndexOrThrow(UserDictionary.Words.SHORTCUT));
    }

    public static void deleteWord(final String word, final String shortcut,
            final ContentResolver resolver) {
        if (TextUtils.isEmpty(shortcut)) {
            resolver.delete(
                    UserDictionary.Words.CONTENT_URI, DELETE_SELECTION_WITHOUT_SHORTCUT,
                    new String[]{word});
        } else {
            resolver.delete(
                    UserDictionary.Words.CONTENT_URI, DELETE_SELECTION_WITH_SHORTCUT,
                    new String[]{word, shortcut});
        }
    }

    private static class MyAdapter extends SimpleCursorAdapter implements SectionIndexer {
        private AlphabetIndexer mIndexer;

        private ViewBinder mViewBinder = (v, c, columnIndex) -> {
            if (columnIndex == INDEX_SHORTCUT) {
                final String shortcut = c.getString(INDEX_SHORTCUT);
                if (TextUtils.isEmpty(shortcut)) {
                    v.setVisibility(View.GONE);
                } else {
                    ((TextView) v).setText(shortcut);
                    v.setVisibility(View.VISIBLE);
                }
                v.invalidate();
                return true;
            }

            return false;
        };

        public MyAdapter(final Context context, final int layout, final Cursor c,
                final String[] from, final int[] to) {
            super(context, layout, c, from, to, 0 /* flags */);

            if (null != c) {
                final String alphabet = context.getString(R.string.user_dict_fast_scroll_alphabet);
                final int wordColIndex = c.getColumnIndexOrThrow(UserDictionary.Words.WORD);
                mIndexer = new AlphabetIndexer(c, wordColIndex, alphabet);
            }
            setViewBinder(mViewBinder);
        }

        @Override
        public int getPositionForSection(final int section) {
            return null == mIndexer ? 0 : mIndexer.getPositionForSection(section);
        }

        @Override
        public int getSectionForPosition(final int position) {
            return null == mIndexer ? 0 : mIndexer.getSectionForPosition(position);
        }

        @Override
        public Object[] getSections() {
            return null == mIndexer ? null : mIndexer.getSections();
        }
    }
}

