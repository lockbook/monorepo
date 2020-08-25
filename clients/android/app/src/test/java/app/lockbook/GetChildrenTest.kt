package app.lockbook

import app.lockbook.core.loadLockbookCore
import app.lockbook.utils.Config
import app.lockbook.utils.CoreModel
import org.junit.After
import org.junit.BeforeClass
import org.junit.Test

class GetChildrenTest {

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
    fun getChildrenOk() {
        CoreModel.generateAccount(
            Config(path),
            generateAlphaString()
        ).component1()!!
        coreModel.setParentToRoot().component1()!!
        coreModel.getChildrenOfParent().component1()!!
        coreModel.getParentOfParent().component1()!!
    }
}