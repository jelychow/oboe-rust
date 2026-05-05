---
name: Bug report
about: Create a report to help improve Rust-native Oboe
title: ''
labels: bug
assignees: ''
---

Android version(s):
Android device(s):
Rust crate/version or commit:
App or wrapper used for testing:

**Short description**
(Please report one bug per issue.)

**Steps to reproduce**

**Expected behavior**

**Actual behavior**

**Device**

Please list which devices have this bug. If device specific, connect the device and share:

```sh
adb shell getprop ro.product.brand
adb shell getprop ro.product.manufacturer
adb shell getprop ro.product.model
adb shell getprop ro.product.device
adb shell getprop ro.product.cpu.abi
adb shell getprop ro.build.description
adb shell getprop ro.hardware
adb shell getprop ro.hardware.chipname
adb shell getprop ro.arch
adb shell getprop | grep aaudio
```

**Additional context**

Attach audio captures or logs when they are needed to reproduce the issue.
