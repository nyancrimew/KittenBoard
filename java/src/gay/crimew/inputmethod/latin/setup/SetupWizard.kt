package gay.crimew.inputmethod.latin.setup

import android.content.Context
import android.content.Intent
import android.graphics.PorterDuff
import android.os.Bundle
import android.provider.Settings
import android.util.AttributeSet
import android.view.View
import android.view.inputmethod.InputMethodManager
import android.widget.ProgressBar
import androidx.fragment.app.Fragment
import com.android.inputmethod.latin.settings.SettingsActivity
import com.android.inputmethod.latin.utils.UncachedInputMethodManagerUtils
import com.github.appintro.AppIntro
import com.github.appintro.AppIntroBaseFragment
import com.github.appintro.AppIntroFragment
import com.github.appintro.SlidePolicy
import com.github.appintro.indicator.IndicatorController
import com.github.appintro.model.SliderPage
import gay.crimew.inputmethod.latin.R
import gay.crimew.inputmethod.latin.utils.extensions.appName
import gay.crimew.inputmethod.latin.utils.extensions.getColorCompat

// TODO: make a lot prettier
class SetupWizard : AppIntro() {

    lateinit var imm: InputMethodManager

    val position
        get() = (indicatorController as PositionProgressIndicatorController).position

    private var needObserveSystem = false

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        imm = getSystemService(Context.INPUT_METHOD_SERVICE) as InputMethodManager;

        if (UncachedInputMethodManagerUtils.isThisImeEnabled(
                this,
                imm
            ) && UncachedInputMethodManagerUtils.isThisImeCurrent(this, imm)
        ) {
            val intent = Intent()
            intent.setClass(this, SettingsActivity::class.java)
            intent.flags = (Intent.FLAG_ACTIVITY_RESET_TASK_IF_NEEDED
                    or Intent.FLAG_ACTIVITY_CLEAR_TOP)
            intent.putExtra(
                SettingsActivity.EXTRA_ENTRY_KEY,
                SettingsActivity.EXTRA_ENTRY_VALUE_APP_ICON
            )
            startActivity(intent)
            finish()
            return
        }

        isSkipButtonEnabled = false
        isColorTransitionsEnabled = true
        isWizardMode = true
        showStatusBar(true)
        setStatusBarColorRes(R.color.kitten_brand)
        indicatorController = PositionProgressIndicatorController(this)
        setNextArrowColor(getColorCompat(R.color.kitten_brand_tertiary_muted))
        setBackArrowColor(getColorCompat(R.color.kitten_brand_tertiary_muted))
        setColorDoneText(getColorCompat(R.color.kitten_brand_tertiary_muted))
        setIndicatorColor(
            getColorCompat(R.color.kitten_brand_tertiary),
            getColorCompat(R.color.kitten_brand_tertiary_muted)
        )

        addSlide(
            AppIntroFragment.createInstance(
                title = getString(R.string.setup_steps_title, appName),
                description = getString(R.string.setup_lets_get_started),
                imageDrawable = R.drawable.ic_launcher_big,
                backgroundColorRes = R.color.kitten_brand_secondary,
                titleColorRes = R.color.kitten_brand_tertiary_muted,
                descriptionColorRes = R.color.kitten_brand_tertiary_muted
            )
        )
        addSlide(
            EnableKittenBoardStep.createInstance(
                imm,
                SliderPage(
                    title = getString(R.string.setup_step1_title, appName),
                    description = if (UncachedInputMethodManagerUtils.isThisImeEnabled(this, imm))
                        getString(R.string.setup_step1_finished_instruction, appName)
                    else
                        getString(R.string.setup_step1_instruction, appName),
                    // TODO: better/cleaner screenshot
                    imageDrawable = R.drawable.setup_manage_keyboards,
                    backgroundColorRes = R.color.kitten_brand,
                    titleColorRes = R.color.kitten_brand_dark,
                    descriptionColorRes = R.color.kitten_brand_dark,
                )
            ) {
                val intent = Intent()
                intent.action = Settings.ACTION_INPUT_METHOD_SETTINGS
                intent.addCategory(Intent.CATEGORY_DEFAULT)
                startActivity(intent)
                needObserveSystem = true
            }
        )
        addSlide(
            SetKittenBoardStep.createInstance(
                imm, SliderPage(
                    title = getString(R.string.setup_step2_title, appName),
                    description = getString(R.string.setup_step2_instruction, appName),
                    // TODO: better/cleaner screenshot
                    imageDrawable = R.drawable.setup_pick_ime,
                    backgroundColorRes = R.color.kitten_brand_secondary,
                    titleColorRes = R.color.kitten_brand_tertiary_muted,
                    descriptionColorRes = R.color.kitten_brand_tertiary_muted
                )
            ) {
                imm.showInputMethodPicker()
                needObserveSystem = true
            }
        )
        addSlide(
            AppIntroFragment.createInstance(
                title = getString(R.string.setup_step3_title),
                description = getString(R.string.setup_step3_instruction, appName),
                imageDrawable = R.drawable.ic_launcher_big,
                backgroundColorRes = R.color.kitten_brand,
                titleColorRes = R.color.kitten_brand_dark,
                descriptionColorRes = R.color.kitten_brand_dark,
            )
        )
    }

    override fun onDonePressed(currentFragment: Fragment?) {
        super.onDonePressed(currentFragment)
        finish()
    }

    override fun onSlideChanged(oldFragment: Fragment?, newFragment: Fragment?) {
        super.onSlideChanged(oldFragment, newFragment)
        if (oldFragment == null && position == 0) {
            // kinda hacky since AppIntro for some reason doesn't let you go to a specific page
            if (UncachedInputMethodManagerUtils.isThisImeEnabled(this, imm)) {
                goToNextSlide()
                goToNextSlide()
                if (UncachedInputMethodManagerUtils.isThisImeCurrent(this, imm)) {
                    goToNextSlide()
                }
            }
        }
    }

    override fun onWindowFocusChanged(hasFocus: Boolean) {
        super.onWindowFocusChanged(hasFocus)
        if (hasFocus && needObserveSystem) {
            needObserveSystem = false
            when (position) {
                1 -> {
                    if (UncachedInputMethodManagerUtils.isThisImeEnabled(this, imm)) {
                        goToNextSlide()
                    }
                }
                2 -> {
                    if (!UncachedInputMethodManagerUtils.isThisImeEnabled(this, imm)) {
                        goToPreviousSlide()
                    }
                    if (UncachedInputMethodManagerUtils.isThisImeCurrent(this, imm)) {
                        goToNextSlide()
                    }
                }
            }
        }
    }

    class EnableKittenBoardStep(val imm: InputMethodManager, private val action: () -> Unit) :
        AppIntroBaseFragment(), SlidePolicy {
        override val layoutId: Int get() = R.layout.appintro_fragment_intro
        override val isPolicyRespected: Boolean
            get() = UncachedInputMethodManagerUtils.isThisImeEnabled(requireContext(), imm)

        override fun onUserIllegallyRequestedNextPage() {
            action()
        }

        companion object {
            fun createInstance(
                imm: InputMethodManager,
                sliderPage: SliderPage,
                action: () -> Unit
            ): EnableKittenBoardStep {
                val slide = EnableKittenBoardStep(imm, action)
                slide.arguments = sliderPage.toBundle()
                return slide
            }
        }
    }

    class SetKittenBoardStep(val imm: InputMethodManager, private val action: () -> Unit) :
        AppIntroBaseFragment(), SlidePolicy {
        override val layoutId: Int get() = R.layout.appintro_fragment_intro
        override val isPolicyRespected: Boolean
            get() = UncachedInputMethodManagerUtils.isThisImeCurrent(requireContext(), imm)

        override fun onUserIllegallyRequestedNextPage() {
            action()
        }

        companion object {
            fun createInstance(
                imm: InputMethodManager,
                sliderPage: SliderPage,
                action: () -> Unit
            ): SetKittenBoardStep {
                val slide = SetKittenBoardStep(imm, action)
                slide.arguments = sliderPage.toBundle()
                return slide
            }
        }
    }

    class PositionProgressIndicatorController @JvmOverloads constructor(
        context: Context,
        attrs: AttributeSet? = null,
        defStyleAttr: Int = android.R.attr.progressBarStyleHorizontal
    ) : IndicatorController, ProgressBar(context, attrs, defStyleAttr) {

        var position: Int = 0
            private set

        override var selectedIndicatorColor = 1
            set(value) {
                field = value
                progressDrawable.setColorFilter(value, PorterDuff.Mode.SRC_IN)
            }

        override var unselectedIndicatorColor = 1
            set(value) {
                field = value
                indeterminateDrawable.setColorFilter(value, PorterDuff.Mode.SRC_IN)
            }

        override fun newInstance(context: Context) = this

        override fun initialize(slideCount: Int) {
            this.max = slideCount
            if (isRtl) {
                this.scaleX = -1F
            }
            if (slideCount == 1) {
                this.visibility = View.INVISIBLE
            }
            selectPosition(0)
        }

        override fun selectPosition(index: Int) {
            position = index
            this.progress = if (isRtl) {
                max - index
            } else {
                index + 1
            }
        }

        private val isRtl: Boolean get() = context.resources.configuration.layoutDirection == View.LAYOUT_DIRECTION_RTL
    }
}