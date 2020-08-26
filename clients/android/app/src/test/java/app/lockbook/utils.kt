package app.lockbook

import java.util.*
// You have to build the macos jni from core first to be able to run the tests.
// Next you have to add a vm option that helps java find the library:
// -ea -Djava.library.path="//lockbook/clients/android/core/src/main/jniLibs/desktop/"

fun generateAlphaString(): String =
    (1..20).map { (('A'..'Z') + ('a'..'z')).random() }.joinToString("")

fun generateId(): String = UUID.randomUUID().toString()

const val path = "/tmp/lockbook/"
