package app.lockbook.ui

import android.content.Context
import android.util.AttributeSet
import android.widget.ProgressBar
import android.widget.TextView
import androidx.preference.Preference
import androidx.preference.PreferenceViewHolder
import app.lockbook.R
import app.lockbook.model.CoreModel
import app.lockbook.screen.SettingsActivity
import app.lockbook.screen.SettingsFragment
import app.lockbook.util.*
import com.github.michaelbull.result.Err
import com.github.michaelbull.result.andThen
import com.github.michaelbull.result.map
import kotlinx.coroutines.*

class UsageBarPreference(context: Context, attributeSet: AttributeSet?) : Preference(context, attributeSet) {
    private var job = Job()
    private val uiScope = CoroutineScope(Dispatchers.Main + job)

    private val alertModel by lazy {
        ((context as SettingsActivity).supportFragmentManager.fragments[0] as SettingsFragment).alertModel
    }

    init {
        layoutResource = R.layout.preference_usage_bar
    }

    override fun onBindViewHolder(holder: PreferenceViewHolder) {
        super.onBindViewHolder(holder)

        setUpUsagePreference(holder)
    }

    private fun setUpUsagePreference(holder: PreferenceViewHolder) {
        uiScope.launch {
            withContext(Dispatchers.IO) {
                val usageInfo = holder.itemView.findViewById<TextView>(R.id.usage_info)

                val getUsageResult = CoreModel.getUsage().andThen { usage ->

                    CoreModel.getUncompressedUsage().map { uncompressedUsage ->
                        withContext(Dispatchers.Main) {
                            val resources = holder.itemView.resources

                            val usageBar = holder.itemView.findViewById<ProgressBar>(R.id.usage_bar)
                            usageBar.max = usage.dataCap.exact
                            usageBar.progress = usage.serverUsage.exact

                            usageInfo.text = spannable {
                                resources.getString(R.string.settings_usage_current)
                                    .bold() + " " + usage.serverUsage.readable + "\n" + resources.getString(
                                    R.string.settings_usage_data_cap
                                )
                                    .bold() + " " + usage.dataCap.readable + "\n" + resources.getString(
                                    R.string.settings_usage_uncompressed_usage
                                ).bold() + " " + uncompressedUsage.readable
                            }
                        }
                    }
                }

                if (getUsageResult is Err) {
                    val lbError = getUsageResult.error.toLbError(context.resources)
                    alertModel.notifyError(lbError)
                    withContext(Dispatchers.Main) {
                        usageInfo.text = if (lbError.kind == LbErrorKind.User) {
                            lbError.msg
                        } else {
                            getString(context.resources, R.string.basic_error)
                        }
                    }
                }
            }
        }
    }
}
