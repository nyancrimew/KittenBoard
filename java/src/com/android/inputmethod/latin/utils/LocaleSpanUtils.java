/*
 * Copyright (C) 2014 The Android Open Source Project
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

package com.android.inputmethod.latin.utils;

import android.text.Spannable;
import android.text.Spanned;
import android.text.style.LocaleSpan;
import android.util.Log;

import com.android.inputmethod.annotations.UsedForTesting;

import java.util.ArrayList;
import java.util.Locale;

@UsedForTesting
public final class LocaleSpanUtils {
    private static final String TAG = LocaleSpanUtils.class.getSimpleName();

    /**
     * Ensures that the specified range is covered with only one {@link LocaleSpan} with the given
     * locale. If the region is already covered by one or more {@link LocaleSpan}, their ranges are
     * updated so that each character has only one locale.
     *
     * @param spannable the spannable object to be updated.
     * @param start     the start index from which {@link LocaleSpan} is attached (inclusive).
     * @param end       the end index to which {@link LocaleSpan} is attached (exclusive).
     * @param locale    the locale to be attached to the specified range.
     */
    @UsedForTesting
    public static void updateLocaleSpan(final Spannable spannable, final int start,
            final int end, final Locale locale) {
        if (end < start) {
            Log.e(TAG, "Invalid range: start=" + start + " end=" + end);
            return;
        }

        // A brief summary of our strategy;
        //   1. Enumerate all LocaleSpans between [start - 1, end + 1].
        //   2. For each LocaleSpan S:
        //      - Update the range of S so as not to cover [start, end] if S doesn't have the
        //        expected locale.
        //      - Mark S as "to be merged" if S has the expected locale.
        //   3. Merge all the LocaleSpans that are marked as "to be merged" into one LocaleSpan.
        //      If no appropriate span is found, create a new one with newLocaleSpan method.
        final int searchStart = Math.max(start - 1, 0);
        final int searchEnd = Math.min(end + 1, spannable.length());
        // LocaleSpans found in the target range. See the step 1 in the above comment.
        final LocaleSpan[] existingLocaleSpans = spannable.getSpans(searchStart, searchEnd,
                LocaleSpan.class);
        // LocaleSpans that are marked as "to be merged". See the step 2 in the above comment.
        final ArrayList<LocaleSpan> existingLocaleSpansToBeMerged = new ArrayList<>();
        boolean isStartExclusive = true;
        boolean isEndExclusive = true;
        int newStart = start;
        int newEnd = end;
        for (final LocaleSpan existingLocaleSpan : existingLocaleSpans) {
            final Locale attachedLocale = existingLocaleSpan.getLocale();
            if (!locale.equals(attachedLocale)) {
                // This LocaleSpan does not have the expected locale. Update its range if it has
                // an intersection with the range [start, end] (the first case of the step 2 in the
                // above comment).
                removeLocaleSpanFromRange(existingLocaleSpan, spannable, start, end);
                continue;
            }
            final int spanStart = spannable.getSpanStart(existingLocaleSpan);
            final int spanEnd = spannable.getSpanEnd(existingLocaleSpan);
            if (spanEnd < spanStart) {
                Log.e(TAG, "Invalid span: spanStart=" + spanStart + " spanEnd=" + spanEnd);
                continue;
            }
            if (spanEnd < start || end < spanStart) {
                // No intersection found.
                continue;
            }

            // Here existingLocaleSpan has the expected locale and an intersection with the
            // range [start, end] (the second case of the the step 2 in the above comment).
            final int spanFlag = spannable.getSpanFlags(existingLocaleSpan);
            if (spanStart < newStart) {
                newStart = spanStart;
                isStartExclusive = ((spanFlag & Spanned.SPAN_EXCLUSIVE_EXCLUSIVE) ==
                        Spanned.SPAN_EXCLUSIVE_EXCLUSIVE);
            }
            if (newEnd < spanEnd) {
                newEnd = spanEnd;
                isEndExclusive = ((spanFlag & Spanned.SPAN_EXCLUSIVE_EXCLUSIVE) ==
                        Spanned.SPAN_EXCLUSIVE_EXCLUSIVE);
            }
            existingLocaleSpansToBeMerged.add(existingLocaleSpan);
        }

        int originalLocaleSpanFlag = 0;
        LocaleSpan localeSpan;
        if (existingLocaleSpansToBeMerged.isEmpty()) {
            // If there is no LocaleSpan that is marked as to be merged, create a new one.
            localeSpan = new LocaleSpan(locale);
        } else {
            // Reuse the first LocaleSpan to avoid unnecessary object instantiation.
            localeSpan = existingLocaleSpansToBeMerged.get(0);
            originalLocaleSpanFlag = spannable.getSpanFlags(localeSpan);
            // No need to keep other instances.
            for (int i = 1; i < existingLocaleSpansToBeMerged.size(); ++i) {
                spannable.removeSpan(existingLocaleSpansToBeMerged.get(i));
            }
        }
        final int localeSpanFlag = getSpanFlag(originalLocaleSpanFlag, isStartExclusive,
                isEndExclusive);
        spannable.setSpan(localeSpan, newStart, newEnd, localeSpanFlag);
    }

    private static void removeLocaleSpanFromRange(final LocaleSpan localeSpan,
            final Spannable spannable, final int removeStart, final int removeEnd) {
        final int spanStart = spannable.getSpanStart(localeSpan);
        final int spanEnd = spannable.getSpanEnd(localeSpan);
        if (spanStart > spanEnd) {
            Log.e(TAG, "Invalid span: spanStart=" + spanStart + " spanEnd=" + spanEnd);
            return;
        }
        if (spanEnd < removeStart) {
            // spanStart < spanEnd < removeStart < removeEnd
            return;
        }
        if (removeEnd < spanStart) {
            // spanStart < removeEnd < spanStart < spanEnd
            return;
        }
        final int spanFlags = spannable.getSpanFlags(localeSpan);
        if (spanStart < removeStart) {
            if (removeEnd < spanEnd) {
                // spanStart < removeStart < removeEnd < spanEnd
                final Locale locale = localeSpan.getLocale();
                spannable.setSpan(localeSpan, spanStart, removeStart, spanFlags);
                final LocaleSpan attionalLocaleSpan = new LocaleSpan(locale);
                spannable.setSpan(attionalLocaleSpan, removeEnd, spanEnd, spanFlags);
                return;
            }
            // spanStart < removeStart < spanEnd <= removeEnd
            spannable.setSpan(localeSpan, spanStart, removeStart, spanFlags);
            return;
        }
        if (removeEnd < spanEnd) {
            // removeStart <= spanStart < removeEnd < spanEnd
            spannable.setSpan(localeSpan, removeEnd, spanEnd, spanFlags);
            return;
        }
        // removeStart <= spanStart < spanEnd < removeEnd
        spannable.removeSpan(localeSpan);
    }

    private static int getSpanFlag(final int originalFlag,
            final boolean isStartExclusive, final boolean isEndExclusive) {
        return (originalFlag & ~Spanned.SPAN_POINT_MARK_MASK) |
                getSpanPointMarkFlag(isStartExclusive, isEndExclusive);
    }

    private static int getSpanPointMarkFlag(final boolean isStartExclusive,
            final boolean isEndExclusive) {
        if (isStartExclusive) {
            return isEndExclusive ? Spanned.SPAN_EXCLUSIVE_EXCLUSIVE
                    : Spanned.SPAN_EXCLUSIVE_INCLUSIVE;
        }
        return isEndExclusive ? Spanned.SPAN_INCLUSIVE_EXCLUSIVE
                : Spanned.SPAN_INCLUSIVE_INCLUSIVE;
    }
}
