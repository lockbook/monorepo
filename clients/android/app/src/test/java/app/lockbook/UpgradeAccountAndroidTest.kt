package app.lockbook

import app.lockbook.model.CoreModel
import app.lockbook.util.Config
import app.lockbook.util.UpgradeAccountAndroid
import org.junit.Before
import org.junit.BeforeClass
import org.junit.Test

class UpgradeAccountAndroidTest {

    companion object {
        @BeforeClass
        @JvmStatic
        fun loadLib() {
            System.loadLibrary("lockbook_core")
        }
    }

    @Before
    fun initCore() {
        CoreModel.init(Config(false, createRandomPath()))
    }

    @Test
    fun upgradeAccountAndroidInvalidPurchaseToken() {
        CoreModel.createAccount(generateAlphaString()).unwrapOk()

        CoreModel.upgradeAccountAndroid("", "").unwrapErrorType(UpgradeAccountAndroid.InvalidPurchaseToken)
    }
}
