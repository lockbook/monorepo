
package app.lockbook

import android.app.Application
import android.content.Context
import android.content.res.Resources
import androidx.annotation.StringRes
import androidx.appcompat.app.AppCompatDelegate
import androidx.lifecycle.*
import androidx.preference.PreferenceManager
import androidx.work.*
import app.lockbook.App.Companion.PERIODIC_SYNC_TAG
import app.lockbook.App.Companion.config
import app.lockbook.model.CoreModel
import app.lockbook.util.*
import com.github.michaelbull.result.Err
import com.github.michaelbull.result.Ok
import timber.log.Timber
import java.util.concurrent.TimeUnit

class App : Application() {
    override fun onCreate() {
        super.onCreate()
        loadLockbookCore()
        ProcessLifecycleOwner.get().lifecycle
            .addObserver(ForegroundBackgroundObserver(this))

        config = Config(this.filesDir.absolutePath)

        AppCompatDelegate.setDefaultNightMode(AppCompatDelegate.MODE_NIGHT_FOLLOW_SYSTEM)
    }

    companion object {
        lateinit var config: Config
            private set

        const val PERIODIC_SYNC_TAG = "periodic_sync"
    }

    private fun loadLockbookCore() {
        System.loadLibrary("lockbook_core")
        CoreModel.setUpInitLogger(filesDir.absolutePath)
    }
}

class ForegroundBackgroundObserver(val context: Context) : DefaultLifecycleObserver {
    override fun onStart(owner: LifecycleOwner) {
        doIfLoggedIn {
            WorkManager.getInstance(context)
                .cancelAllWorkByTag(PERIODIC_SYNC_TAG)
        }
    }

    override fun onStop(owner: LifecycleOwner) {
        doIfLoggedIn {
            val work = PeriodicWorkRequestBuilder<SyncWork>(
                PreferenceManager.getDefaultSharedPreferences(context)
                    .getInt(getString(context.resources, R.string.background_sync_period_key), 30).toLong(),
                TimeUnit.MINUTES
            )
                .setConstraints(Constraints.NONE)
                .addTag(PERIODIC_SYNC_TAG)
                .build()

            WorkManager.getInstance(context)
                .enqueueUniquePeriodicWork(
                    PERIODIC_SYNC_TAG,
                    ExistingPeriodicWorkPolicy.REPLACE,
                    work
                )
        }
    }

    private fun doIfLoggedIn(onSuccess: () -> Unit) {
        when (val getDbStateResult = CoreModel.getDBState(config)) {
            is Ok -> if (getDbStateResult.value == State.ReadyToUse) {
                onSuccess()
            }
            is Err -> Timber.e("Error: ${getDbStateResult.error.toLbError(context.resources)}")
        }
    }
}

class SyncWork(appContext: Context, workerParams: WorkerParameters) :
    Worker(appContext, workerParams) {
    override fun doWork(): Result {
        val syncResult =
            CoreModel.syncAll(Config(applicationContext.filesDir.absolutePath), null)
        return if (syncResult is Err) {
            val msg = when (val error = syncResult.error) {
                is CoreError.UiError -> when (error.content) {
                    SyncAllError.ClientUpdateRequired -> "Client update required."
                    SyncAllError.CouldNotReachServer -> "Could not reach server."
                    SyncAllError.NoAccount -> "No account."
                }
                is CoreError.Unexpected -> {
                    "Unable to sync all files: ${error.content}"
                }
            }.exhaustive

            Timber.e(msg)

            Result.failure()
        } else {
            Result.success()
        }
    }
}

fun AndroidViewModel.getContext(): Context {
    return this.getApplication<Application>()
}

fun AndroidViewModel.getRes(): Resources {
    return this.getApplication<Application>().resources
}

fun AndroidViewModel.getString(
    @StringRes stringRes: Int,
    vararg formatArgs: Any = emptyArray()
): String {
    return getString(this.getRes(), stringRes, *formatArgs)
}
