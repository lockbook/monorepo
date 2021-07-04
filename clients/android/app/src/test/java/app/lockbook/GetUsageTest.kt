package app.lockbook

import app.lockbook.core.getUsage
import app.lockbook.model.CoreModel
import app.lockbook.util.*
import com.beust.klaxon.Klaxon
import com.github.michaelbull.result.Result
import org.junit.After
import org.junit.BeforeClass
import org.junit.Test

class GetUsageTest {
    var config = Config(createRandomPath())

    companion object {
        @BeforeClass
        @JvmStatic
        fun loadLib() {
            System.loadLibrary("lockbook_core")
        }
    }

    @After
    fun createDirectory() {
        config = Config(createRandomPath())
    }

    @Test
    fun getUsageOk() {
        assertType<Unit>(
            CoreModel.generateAccount(config, generateAlphaString()).component1()
        )

        assertType<UsageMetrics>(
            CoreModel.getUsage(config).component1()
        )
    }

    @Test
    fun getUsageNoAccount() {
        assertType<GetUsageError.NoAccount>(
            CoreModel.getUsage(config).component2()
        )
    }

    @Test
    fun getUsageUnexpectedError() {
        assertType<GetUsageError.Unexpected>(
            Klaxon().converter(getUsageConverter).parse<Result<UsageMetrics, GetUsageError>>(
                getUsage("")
            )?.component2()
        )
    }
}
