package app.lockbook.model

import android.net.Uri
import androidx.lifecycle.LiveData
import androidx.lifecycle.MutableLiveData
import androidx.lifecycle.ViewModel
import app.lockbook.util.SingleMutableLiveData

class WorkspaceViewModel: ViewModel() {

    var isSyncing = false

    // for workspace fragment
    val _openFile = SingleMutableLiveData<Pair<String, Boolean>>()
    val openFile: LiveData<Pair<String, Boolean>>
        get() = _openFile

    val _closeDocument = SingleMutableLiveData<String>()
    val closeDocument: LiveData<String>
        get() = _closeDocument

    val _sync = SingleMutableLiveData<Unit>()
    val sync: LiveData<Unit>
        get() = _sync
    val _showTabs = SingleMutableLiveData<Boolean>()
    val showTabs: LiveData<Boolean>
        get() = _showTabs

    // for everyone else
    val _msg = MutableLiveData<String>()
    val msg: LiveData<String>
        get() = _msg

    val _selectedFile = MutableLiveData<String>()
    val selectedFile: LiveData<String>
        get() = _selectedFile

    val _docCreated = MutableLiveData<String>()
    val docCreated: LiveData<String>
        get() = _docCreated

    val _refreshFiles = SingleMutableLiveData<Unit>()
    val refreshFiles: LiveData<Unit>
        get() = _refreshFiles

    val _newFolderBtnPressed = SingleMutableLiveData<Unit>()
    val newFolderBtnPressed: LiveData<Unit>
        get() = _newFolderBtnPressed

    val _tabTitleClicked = SingleMutableLiveData<Unit>()
    val tabTitleClicked: LiveData<Unit>
        get() = _tabTitleClicked

    val _syncCompleted = SingleMutableLiveData<Unit>()
    val syncCompleted: LiveData<Unit>
        get() = _syncCompleted

    val _currentTab = MutableLiveData<WorkspaceTab>()
    val currentTab: LiveData<WorkspaceTab>
        get() = _currentTab

    val _shouldShowTabs = SingleMutableLiveData<Unit>()
    val shouldShowTabs: LiveData<Unit>
        get() = _shouldShowTabs
}

enum class WorkspaceTab(val value: Int) {
    Welcome(0),
    Loading(1),
    Image(2),
    Markdown(3),
    PlainText(4),
    Pdf(5),
    Svg(6);

    companion object {
        fun fromInt(value: Int): WorkspaceTab? {
            return values().find { it.value == value }
        }
    }

    fun viewWrapperId(): Int {
        return when (this) {
            Welcome, Pdf, Loading, Image -> 1
            Svg -> 2
            PlainText, Markdown -> 3
        }
    }

    fun isTextEdit(): Boolean {
        return this == Markdown || this == PlainText
    }

    fun isSvg(): Boolean {
        return this == Svg
    }
}
