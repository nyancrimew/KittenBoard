/*
 * Copyright (C) 2011 The Android Open Source Project
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

package com.android.inputmethod.latin.spellcheck;

import android.Manifest;
import android.content.SharedPreferences;
import android.os.Bundle;
import android.preference.PreferenceScreen;
import android.preference.SwitchPreference;
import android.text.TextUtils;

import gay.crimew.inputmethod.latin.R;
import com.android.inputmethod.latin.permissions.PermissionsManager;
import com.android.inputmethod.latin.permissions.PermissionsUtil;
import com.android.inputmethod.latin.settings.SubScreenFragment;
import com.android.inputmethod.latin.settings.TwoStatePreferenceHelper;
import com.android.inputmethod.latin.utils.ApplicationUtils;

import static com.android.inputmethod.latin.permissions.PermissionsManager.get;

/**
 * Preference screen.
 */
public final class SpellCheckerSettingsFragment extends SubScreenFragment
    implements SharedPreferences.OnSharedPreferenceChangeListener,
            PermissionsManager.PermissionsResultCallback {

    private SwitchPreference mLookupContactsPreference;

    @Override
    public void onActivityCreated(final Bundle savedInstanceState) {
        super.onActivityCreated(savedInstanceState);
        addPreferencesFromResource(R.xml.spell_checker_settings);
        final PreferenceScreen preferenceScreen = getPreferenceScreen();
        preferenceScreen.setTitle(ApplicationUtils.getActivityTitleResId(
                getActivity(), SpellCheckerSettingsActivity.class));
        TwoStatePreferenceHelper.replaceCheckBoxPreferencesBySwitchPreferences(preferenceScreen);

        mLookupContactsPreference = (SwitchPreference) findPreference(
                AndroidSpellCheckerService.PREF_USE_CONTACTS_KEY);
        turnOffLookupContactsIfNoPermission();
    }

    @Override
    public void onSharedPreferenceChanged(SharedPreferences sharedPreferences, String key) {
        if (!TextUtils.equals(key, AndroidSpellCheckerService.PREF_USE_CONTACTS_KEY)) {
            return;
        }

        if (!sharedPreferences.getBoolean(key, false)) {
            // don't care if the preference is turned off.
            return;
        }

        // Check for permissions.
        if (PermissionsUtil.checkAllPermissionsGranted(
                getActivity() /* context */, Manifest.permission.READ_CONTACTS)) {
            return; // all permissions granted, no need to request permissions.
        }

        get(getActivity() /* context */).requestPermissions(this /* PermissionsResultCallback */,
                getActivity() /* activity */, Manifest.permission.READ_CONTACTS);
    }

    @Override
    public void onRequestPermissionsResult(boolean allGranted) {
        turnOffLookupContactsIfNoPermission();
    }

    private void turnOffLookupContactsIfNoPermission() {
        if (!PermissionsUtil.checkAllPermissionsGranted(
                getActivity(), Manifest.permission.READ_CONTACTS)) {
            mLookupContactsPreference.setChecked(false);
        }
    }
}
