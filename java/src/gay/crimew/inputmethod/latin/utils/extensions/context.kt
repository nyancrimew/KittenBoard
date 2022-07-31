package gay.crimew.inputmethod.latin.utils.extensions

import android.content.Context
import androidx.annotation.ColorInt
import androidx.annotation.ColorRes
import androidx.core.content.ContextCompat

@ColorInt
fun Context.getColorCompat(@ColorRes id: Int) = ContextCompat.getColor(this, id)

val Context.appName
    get() = getString(applicationInfo.labelRes)