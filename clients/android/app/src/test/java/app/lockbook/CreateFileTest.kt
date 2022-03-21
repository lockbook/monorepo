package app.lockbook

import app.lockbook.core.calculateWork
import app.lockbook.core.createFile
import app.lockbook.model.CoreModel
import app.lockbook.util.*
import com.beust.klaxon.Klaxon
import com.github.michaelbull.result.Result
import com.github.michaelbull.result.unwrap
import kotlinx.serialization.decodeFromString
import org.junit.After
import org.junit.BeforeClass
import org.junit.Test

class CreateFileTest {
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
    fun createFileOk() {
        CoreModel.createAccount(config, generateAlphaString()).unwrap()

        val rootFileMetadata = CoreModel.getRoot(config).unwrap()

        CoreModel.createFile(
            config,
            rootFileMetadata.id,
            generateAlphaString(),
            FileType.Document
        ).unwrap()

        CoreModel.createFile(
            config,
            rootFileMetadata.id,
            generateAlphaString(),
            FileType.Folder
        ).unwrap()
    }

    @Test
    fun createFileContainsSlash() {
        CoreModel.createAccount(config, generateAlphaString()).unwrap()

        val rootFileMetadata = CoreModel.getRoot(config).unwrap()

        CoreModel.createFile(
            config,
            rootFileMetadata.id,
            "/",
            FileType.Document
        ).unwrapErrorType(CreateFileError.FileNameContainsSlash)

        CoreModel.createFile(
            config,
            rootFileMetadata.id,
            "/",
            FileType.Folder
        ).unwrapErrorType(CreateFileError.FileNameContainsSlash)
    }

    @Test
    fun createFileEmpty() {
        CoreModel.createAccount(config, generateAlphaString()).unwrap()

        val rootFileMetadata = CoreModel.getRoot(config).unwrap()

        CoreModel.createFile(
            config,
            rootFileMetadata.id,
            "",
            FileType.Document
        ).unwrapErrorType(CreateFileError.FileNameEmpty)

        CoreModel.createFile(
            config,
            rootFileMetadata.id,
            "",
            FileType.Folder
        ).unwrapErrorType(CreateFileError.FileNameEmpty)
    }

    @Test
    fun createFileNotAvailable() {
        CoreModel.createAccount(config, generateAlphaString()).unwrap()

        val rootFileMetadata = CoreModel.getRoot(config).unwrap()
        val fileName = generateAlphaString()

        CoreModel.createFile(
            config,
            rootFileMetadata.id,
            fileName,
            FileType.Document
        ).unwrap()

        CoreModel.createFile(
            config,
            rootFileMetadata.id,
            fileName,
            FileType.Folder
        ).unwrapErrorType(CreateFileError.FileNameNotAvailable)
    }

    @Test
    fun createFileNoAccount() {
        CoreModel.createFile(
            config,
            generateId(),
            generateAlphaString(),
            FileType.Document
        ).unwrapErrorType(CreateFileError.NoAccount)
    }

    @Test
    fun createFileUnexpectedError() {
        CoreModel.jsonParser.decodeFromString<IntermCoreResult<DecryptedFileMetadata, CreateFileError>>(
            createFile("", "", "", "")
        ).unwrapUnexpected()
    }
}
