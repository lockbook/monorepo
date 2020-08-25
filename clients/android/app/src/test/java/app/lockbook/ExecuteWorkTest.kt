package app.lockbook

import app.lockbook.core.loadLockbookCore
import app.lockbook.utils.Config
import app.lockbook.utils.CoreModel
import app.lockbook.utils.FileType
import com.beust.klaxon.Klaxon
import org.junit.After
import org.junit.BeforeClass
import org.junit.Test

class ExecuteWorkTest {
    private val coreModel = CoreModel(Config(path))

    companion object {
        @BeforeClass
        @JvmStatic
        fun loadLib() {
            loadLockbookCore()
            Runtime.getRuntime().exec("mkdir $path")
        }
    }

    @After
    fun resetDirectory() {
        Runtime.getRuntime().exec("rm -rf $path/*")
    }

    @Test
    fun executeWorkOk() {
        CoreModel.generateAccount(
            Config(path),
            generateAlphaString()
        ).component1()!!
        coreModel.setParentToRoot().component1()!!
        val document = coreModel.createFile(generateAlphaString(), Klaxon().toJsonString(FileType.Document)).component1()!!
        coreModel.insertFile(document).component1()!!
        val folder = coreModel.createFile(generateAlphaString(), Klaxon().toJsonString(FileType.Folder)).component1()!!
        coreModel.insertFile(folder).component1()!!
        val syncWork = coreModel.calculateFileSyncWork().component1()!!
        for(workUnit in syncWork.work_units) {
            coreModel.executeFileSyncWork(coreModel.getAccount().component1()!!, workUnit).component1()!!
        }
    }
}