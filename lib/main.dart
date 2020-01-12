import 'package:client/account_helper.dart';
import 'package:client/encryption_helper.dart';
import 'package:client/network_helper.dart';
import 'package:client/persistence_helper.dart';
import 'package:flutter/cupertino.dart';

import 'new_lockbook.dart';

// Compile Time Constants for Dependency Injection
const String apiBase = "http://lockbook.app:8000";
const EncryptionHelper encryptionHelper = EncryptionHelper();
const PersistenceHelper persistenceHelper = PersistenceHelper();
const NetworkHelper networkHelper = NetworkHelper(apiBase, persistenceHelper);
const AccountHelper accountHelper =
    AccountHelper(encryptionHelper, persistenceHelper, networkHelper);

const NewLockbook newLockbook = NewLockbook(accountHelper);

void main() {
  return runApp(newLockbook);
}
