package com.example.openairustrealtime.core.audio

enum class CommunicationDeviceKind {
    BLUETOOTH,
    WIRED_HEADSET,
    EARPIECE,
    SPEAKER
}

interface CommunicationAudioBackend {
    var mode: Int
    var speakerphoneOn: Boolean

    fun availableDevices(): Set<CommunicationDeviceKind>
    fun currentCommunicationDevice(): CommunicationDeviceKind?
    fun setCommunicationDevice(device: CommunicationDeviceKind): Boolean
    fun clearCommunicationDevice()
}

class CommunicationAudioController(
    private val backend: CommunicationAudioBackend,
    private val communicationMode: Int
) {
    private data class Snapshot(
        val mode: Int,
        val speakerphoneOn: Boolean,
        val communicationDevice: CommunicationDeviceKind?
    )

    private var snapshot: Snapshot? = null

    @Synchronized
    fun activate(): CommunicationDeviceKind? {
        if (snapshot == null) {
            snapshot = Snapshot(
                mode = backend.mode,
                speakerphoneOn = backend.speakerphoneOn,
                communicationDevice = backend.currentCommunicationDevice()
            )
        }

        backend.mode = communicationMode
        for (target in prioritizedDevices(backend.availableDevices())) {
            backend.speakerphoneOn = target == CommunicationDeviceKind.SPEAKER
            if (backend.setCommunicationDevice(target)) {
                return target
            }
        }
        backend.speakerphoneOn = false
        return null
    }

    @Synchronized
    fun deactivate() {
        val previous = snapshot ?: return
        if (previous.communicationDevice != null && !backend.setCommunicationDevice(previous.communicationDevice)) {
            backend.clearCommunicationDevice()
        } else if (previous.communicationDevice == null) {
            backend.clearCommunicationDevice()
        }
        backend.speakerphoneOn = previous.speakerphoneOn
        backend.mode = previous.mode
        snapshot = null
    }

    companion object {
        fun preferredDevice(devices: Set<CommunicationDeviceKind>): CommunicationDeviceKind? {
            return prioritizedDevices(devices).firstOrNull()
        }

        fun prioritizedDevices(devices: Set<CommunicationDeviceKind>): List<CommunicationDeviceKind> {
            val priority = listOf(
                CommunicationDeviceKind.BLUETOOTH,
                CommunicationDeviceKind.WIRED_HEADSET,
                CommunicationDeviceKind.SPEAKER,
                CommunicationDeviceKind.EARPIECE
            )
            return priority.filter(devices::contains)
        }
    }
}
