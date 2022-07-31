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

import android.inputmethodservice.ExtractEditText;
import android.inputmethodservice.InputMethodService;
import android.view.View;
import android.view.ViewParent;
import android.view.inputmethod.CursorAnchorInfo;
import android.widget.TextView;

/**
 * This class allows input methods to extract {@link CursorAnchorInfo} directly from the given
 * {@link TextView}. This is useful and even necessary to support full-screen mode where the default
 * {@link InputMethodService#onUpdateCursorAnchorInfo(CursorAnchorInfo)} event callback must be
 * ignored because it reports the character locations of the target application rather than
 * characters on {@link ExtractEditText}.
 */
public final class CursorAnchorInfoUtils {
    private CursorAnchorInfoUtils() {
        // This helper class is not instantiable.
    }

    private static boolean isPositionVisible(final View view, final float positionX,
            final float positionY) {
        final float[] position = new float[] { positionX, positionY };
        View currentView = view;

        while (currentView != null) {
            if (currentView != view) {
                // Local scroll is already taken into account in positionX/Y
                position[0] -= currentView.getScrollX();
                position[1] -= currentView.getScrollY();
            }

            if (position[0] < 0 || position[1] < 0 ||
                    position[0] > currentView.getWidth() || position[1] > currentView.getHeight()) {
                return false;
            }

            if (!currentView.getMatrix().isIdentity()) {
                currentView.getMatrix().mapPoints(position);
            }

            position[0] += currentView.getLeft();
            position[1] += currentView.getTop();

            final ViewParent parent = currentView.getParent();
            if (parent instanceof View) {
                currentView = (View) parent;
            } else {
                // We've reached the ViewRoot, stop iterating
                currentView = null;
            }
        }

        // We've been able to walk up the view hierarchy and the position was never clipped
        return true;
    }
}
