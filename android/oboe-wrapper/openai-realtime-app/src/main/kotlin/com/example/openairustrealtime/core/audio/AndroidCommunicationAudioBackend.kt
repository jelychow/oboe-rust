package com.example.openairustrealtime.core.audio

import android.content.Context
import android.media.AudioDeviceInfo
import android.media.AudioManager
import android.os.Build

class AndroidCommunicationAudioBackend(context: Context) : CommunicationAudioBackend {
    private val audioManager = context.applicationContext
        .getSystemService(Context.AUDIO_SERVICE) as AudioManager

    override var mode: Int
        get() = audioManager.mode
        set(value) {
            audioManager.mode = value
        }

    override var speakerphoneOn: Boolean
        @Suppress("DEPRECATION")
        get() = audioManager.isSpeakerphoneOn
        set(value) {
            @Suppress("DEPRECATION")
            audioManager.isSpeakerphoneOn = value
        }

    override fun availableDevices(): Set<CommunicationDeviceKind> {
        val audioDevices = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
            audioManager.availableCommunicationDevices
        } else {
            audioManager.getDevices(AudioManager.GET_DEVICES_OUTPUTS).asList()
        }
        return audioDevices.mapNotNull(::deviceKind).toCollection(linkedSetOf())
    }

    override fun currentCommunicationDevice(): CommunicationDeviceKind? {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.S) {
            return null
        }
        return audioManager.communicationDevice?.let(::deviceKind)
    }

    override fun setCommunicationDevice(device: CommunicationDeviceKind): Boolean {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.S) {
            return when (device) {
                CommunicationDeviceKind.BLUETOOTH -> false
                CommunicationDeviceKind.WIRED_HEADSET,
                CommunicationDeviceKind.EARPIECE,
                CommunicationDeviceKind.SPEAKER -> device in availableDevices()
            }
        }
        val target = audioManager.availableCommunicationDevices.firstOrNull { deviceKind(it) == device }
        return target?.let(audioManager::setCommunicationDevice) ?: false
    }

    override fun clearCommunicationDevice() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
            audioManager.clearCommunicationDevice()
        }
    }

    private fun deviceKind(device: AudioDeviceInfo): CommunicationDeviceKind? {
        return when (device.type) {
            AudioDeviceInfo.TYPE_BLE_HEADSET,
            AudioDeviceInfo.TYPE_BLUETOOTH_SCO,
            AudioDeviceInfo.TYPE_BLUETOOTH_A2DP -> CommunicationDeviceKind.BLUETOOTH
            AudioDeviceInfo.TYPE_WIRED_HEADSET,
            AudioDeviceInfo.TYPE_WIRED_HEADPHONES,
            AudioDeviceInfo.TYPE_USB_HEADSET,
            AudioDeviceInfo.TYPE_USB_DEVICE -> CommunicationDeviceKind.WIRED_HEADSET
            AudioDeviceInfo.TYPE_BUILTIN_EARPIECE -> CommunicationDeviceKind.EARPIECE
            AudioDeviceInfo.TYPE_BUILTIN_SPEAKER -> CommunicationDeviceKind.SPEAKER
            else -> null
        }
    }
}
