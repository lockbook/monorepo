package app.lockbook.loggedin.newfilefolder

import android.app.Activity
import android.os.Bundle
import androidx.databinding.DataBindingUtil
import app.lockbook.R
import app.lockbook.databinding.ActivityNewFileFolderBinding
import app.lockbook.loggedin.mainscreen.FileFolderModel
import app.lockbook.utils.FileType
import com.beust.klaxon.Klaxon
import kotlinx.android.synthetic.main.activity_new_file_folder.*
import kotlinx.android.synthetic.main.activity_new_file_folder.name_text
import kotlinx.android.synthetic.main.activity_popup_info.*
import kotlinx.coroutines.*

class NewFileFolderActivity : Activity() {

    private var job = Job()
    private val uiScope = CoroutineScope(Dispatchers.Main + job)

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val binding: ActivityNewFileFolderBinding = DataBindingUtil.setContentView(
            this,
            R.layout.activity_new_file_folder
        )

        binding.newFileFolderActivity = this
    }

    fun createFileFolder() {
        uiScope.launch {
            withContext(Dispatchers.IO) {
                val json = Klaxon()
                val fileType = if (file_radio_button.isChecked) {
                    json.toJsonString(FileType.Document)
                } else {
                    json.toJsonString(FileType.Folder)
                }

                intent.putExtra("fileType", fileType)
                intent.putExtra("name", new_name_text.text.toString())

                withContext(Dispatchers.Main) {
                    finish()
                }
            }
        }
    }
}