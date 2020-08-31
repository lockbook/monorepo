package app.lockbook

import app.lockbook.core.getChildren
import app.lockbook.utils.*
import com.beust.klaxon.Klaxon
import com.github.michaelbull.result.Result
import org.junit.After
import org.junit.BeforeClass
import org.junit.Test

class GetChildrenTest {
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
    fun getChildrenOk() {
        assertType<Unit>(
            this::getChildrenOk.name,
            CoreModel.generateAccount(config, generateAlphaString()).component1()
        )

        val rootFileMetadata = assertTypeReturn<FileMetadata>(
            this::getChildrenOk.name,
            CoreModel.getRoot(config).component1()
        )

        assertType<List<FileMetadata>>(
            this::getChildrenOk.name,
            CoreModel.getChildren(config, rootFileMetadata.id).component1()
        )
    }

    @Test
    fun getChildrenUnexpectedError() {
        val getChildrenResult: Result<List<FileMetadata>, GetChildrenError>? =
            Klaxon().converter(getChildrenConverter)
                .parse(getChildren("", ""))

        assertType<GetChildrenError.UnexpectedError>(
            this::getChildrenUnexpectedError.name,
            getChildrenResult?.component2()
        )
    }
}
