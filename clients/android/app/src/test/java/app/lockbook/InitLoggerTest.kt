package app.lockbook

import app.lockbook.model.CoreModel
import app.lockbook.util.Config
import app.lockbook.util.CoreError
import org.junit.After
import org.junit.BeforeClass
import org.junit.Test

class InitLoggerTest {
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
    fun initLoggerOk() {
        CoreModel.setUpInitLogger(config.writeable_path).unwrapOk()
    }

    @Test
    fun initLoggerUnexpected() {
        CoreModel.setUpInitLogger("${config.writeable_path}/${generateAlphaString()}.txt").component2()!! as CoreError.Unexpected
    }
}
