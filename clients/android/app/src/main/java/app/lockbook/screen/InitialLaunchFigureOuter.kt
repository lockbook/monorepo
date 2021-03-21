package app.lockbook.screen

import android.content.Intent
import android.content.SharedPreferences
import android.os.Bundle
import android.view.View
import androidx.appcompat.app.AlertDialog
import androidx.appcompat.app.AppCompatActivity
import androidx.biometric.BiometricManager
import androidx.biometric.BiometricManager.Authenticators.BIOMETRIC_WEAK
import androidx.biometric.BiometricPrompt
import androidx.biometric.BiometricPrompt.*
import androidx.core.content.ContextCompat
import androidx.preference.PreferenceManager
import app.lockbook.R
import app.lockbook.model.CoreModel
import app.lockbook.util.*
import app.lockbook.util.SharedPreferences.BIOMETRIC_NONE
import app.lockbook.util.SharedPreferences.BIOMETRIC_OPTION_KEY
import app.lockbook.util.SharedPreferences.BIOMETRIC_RECOMMENDED
import app.lockbook.util.SharedPreferences.BIOMETRIC_STRICT
import com.github.michaelbull.result.Err
import com.github.michaelbull.result.Ok
import com.google.android.material.snackbar.Snackbar
import kotlinx.android.synthetic.main.splash_screen.*
import kotlinx.coroutines.*
import timber.log.Timber

const val STATE_REQUIRES_CLEANING =
    "This lockbook version is incompatible with your data, please clear your data or downgrade your lockbook."

class InitialLaunchFigureOuter : AppCompatActivity() {
    private var job = Job()
    private val uiScope = CoroutineScope(Dispatchers.Main + job)

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.splash_screen)
        Timber.plant(Timber.DebugTree())

        handleOnDBState()
    }

    private fun handleOnDBState() {
        when (val getDBStateResult = CoreModel.getDBState(Config(filesDir.absolutePath))) {
            is Ok -> {
                when (getDBStateResult.value) {
                    State.Empty -> {
                        startActivity(Intent(this, WelcomeActivity::class.java))
                        finish()
                    }
                    State.ReadyToUse -> startFromExistingAccount()
                    State.MigrationRequired -> {
                        Snackbar.make(
                            splash_screen,
                            "Upgrading data...",
                            Snackbar.LENGTH_LONG
                        ).show()
                        migrate_progress_bar.visibility = View.VISIBLE
                        migrateDB()
                    }
                    State.StateRequiresClearing -> {
                        Timber.e("DB state requires cleaning!")
                        Snackbar.make(
                            splash_screen,
                            STATE_REQUIRES_CLEANING,
                            Snackbar.LENGTH_SHORT
                        ).show()
                    }
                }
            }
            is Err -> when (val error = getDBStateResult.error) {
                is GetStateError.Unexpected -> {
                    AlertDialog.Builder(this, R.style.Main_Widget_Dialog)
                        .setTitle(UNEXPECTED_ERROR)
                        .setMessage(error.error)
                        .show()
                    Timber.e("Unable to get DB State: ${error.error}")
                }
            }
        }.exhaustive
    }

    private fun migrateDB() {
        uiScope.launch {
            withContext(Dispatchers.IO) {
                when (val migrateDBResult = CoreModel.migrateDB(Config(filesDir.absolutePath))) {
                    is Ok -> {
                        withContext(Dispatchers.Main) {
                            migrate_progress_bar.visibility = View.GONE
                            Snackbar.make(
                                splash_screen,
                                "Your data has been migrated.",
                                Snackbar.LENGTH_SHORT
                            ).addCallback(object : Snackbar.Callback() {
                                override fun onDismissed(
                                    transientBottomBar: Snackbar?,
                                    event: Int
                                ) {
                                    super.onDismissed(transientBottomBar, event)
                                    startFromExistingAccount()
                                }
                            }).show()
                        }
                    }
                    is Err -> when (val error = migrateDBResult.error) {
                        is MigrationError.StateRequiresCleaning -> {
                            withContext(Dispatchers.Main) {
                                migrate_progress_bar.visibility = View.GONE
                                Snackbar.make(
                                    splash_screen,
                                    STATE_REQUIRES_CLEANING,
                                    Snackbar.LENGTH_LONG
                                ).addCallback(object : Snackbar.Callback() {
                                    override fun onDismissed(
                                        transientBottomBar: Snackbar?,
                                        event: Int
                                    ) {
                                        super.onDismissed(transientBottomBar, event)
                                        finish()
                                    }
                                }).show()
                            }
                            Timber.e("DB state requires cleaning!")
                        }
                        is MigrationError.Unexpected -> {
                            withContext(Dispatchers.Main) {
                                migrate_progress_bar.visibility = View.GONE
                                AlertDialog.Builder(
                                    this@InitialLaunchFigureOuter,
                                    R.style.Main_Widget_Dialog
                                )
                                    .setTitle(UNEXPECTED_ERROR)
                                    .setMessage(error.error)
                                    .setOnCancelListener {
                                        finish()
                                    }
                                    .show()
                            }
                            Timber.e("Unable to migrate DB: ${error.error}")
                        }
                    }
                }.exhaustive
            }
        }
    }

    private fun startFromExistingAccount() {
        val pref = PreferenceManager.getDefaultSharedPreferences(this)

        if (!isBiometricsOptionsAvailable() && pref.getString(
                BIOMETRIC_OPTION_KEY,
                BIOMETRIC_NONE
            ) != BIOMETRIC_NONE
        ) {
            pref.edit()
                .putString(BIOMETRIC_OPTION_KEY, BIOMETRIC_NONE)
                .apply()
        }
        performBiometricFlow(pref)
    }

    private fun launchListFilesActivity() {
        val intent = Intent(this, ListFilesActivity::class.java)
        intent.addFlags(Intent.FLAG_ACTIVITY_NO_ANIMATION)
        overridePendingTransition(0, 0)
        startActivity(intent)
        finish()
    }

    private fun isBiometricsOptionsAvailable(): Boolean =
        BiometricManager.from(applicationContext)
            .canAuthenticate(BIOMETRIC_WEAK) == BiometricManager.BIOMETRIC_SUCCESS

    private fun performBiometricFlow(pref: SharedPreferences) {
        when (
            val optionValue = pref.getString(
                BIOMETRIC_OPTION_KEY,
                BIOMETRIC_NONE
            )
        ) {
            BIOMETRIC_STRICT -> {
                if (BiometricManager.from(applicationContext)
                    .canAuthenticate(BIOMETRIC_WEAK) != BiometricManager.BIOMETRIC_SUCCESS
                ) {
                    Timber.e("Biometric shared preference is strict despite no biometrics.")
                    Snackbar.make(
                        splash_screen,
                        BASIC_ERROR,
                        Snackbar.LENGTH_LONG
                    ).addCallback(object : Snackbar.Callback() {
                        override fun onDismissed(transientBottomBar: Snackbar?, event: Int) {
                            super.onDismissed(transientBottomBar, event)
                            finish()
                        }
                    }).show()
                }

                val executor = ContextCompat.getMainExecutor(this)
                val biometricPrompt = BiometricPrompt(
                    this,
                    executor,
                    object : BiometricPrompt.AuthenticationCallback() {
                        override fun onAuthenticationError(
                            errorCode: Int,
                            errString: CharSequence
                        ) {
                            super.onAuthenticationError(errorCode, errString)
                            when (errorCode) {
                                ERROR_HW_UNAVAILABLE, ERROR_UNABLE_TO_PROCESS, ERROR_NO_BIOMETRICS, ERROR_HW_NOT_PRESENT -> {
                                    Timber.e("Biometric authentication error: $errString")
                                    Snackbar.make(
                                        splash_screen,
                                        BASIC_ERROR,
                                        Snackbar.LENGTH_LONG
                                    ).addCallback(object : Snackbar.Callback() {
                                        override fun onDismissed(
                                            transientBottomBar: Snackbar?,
                                            event: Int
                                        ) {
                                            super.onDismissed(transientBottomBar, event)
                                            finish()
                                        }
                                    }).show()
                                }
                                ERROR_LOCKOUT, ERROR_LOCKOUT_PERMANENT ->
                                    Snackbar.make(
                                        splash_screen,
                                        "Too many tries, try again later!",
                                        Snackbar.LENGTH_SHORT
                                    ).show()
                                else -> finish()
                            }.exhaustive
                        }

                        override fun onAuthenticationSucceeded(
                            result: BiometricPrompt.AuthenticationResult
                        ) {
                            super.onAuthenticationSucceeded(result)
                            launchListFilesActivity()
                        }
                    }
                )

                val promptInfo = PromptInfo.Builder()
                    .setTitle("Lockbook Biometric Verification")
                    .setSubtitle("Verify your identity to access Lockbook.")
                    .setAllowedAuthenticators(BIOMETRIC_WEAK)
                    .setNegativeButtonText("Cancel")
                    .build()

                biometricPrompt.authenticate(promptInfo)
            }
            BIOMETRIC_NONE, BIOMETRIC_RECOMMENDED -> launchListFilesActivity()
            else -> {
                Timber.e("Biometric shared preference does not match every supposed option: $optionValue")
                Snackbar.make(
                    splash_screen,
                    BASIC_ERROR,
                    Snackbar.LENGTH_SHORT
                ).show()
            }
        }.exhaustive
    }
}
