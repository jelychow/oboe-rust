package com.example.openairustrealtime.core.audio

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class CommunicationAudioControllerTest {
    @Test
    fun activate_prefersSpeakerOverEarpiece_forLoudRealtimePlayback_andRestoresPreviousState() {
        val backend = FakeCommunicationAudioBackend(
            initialMode = 7,
            initialSpeakerphoneOn = true,
            initialCommunicationDevice = CommunicationDeviceKind.SPEAKER,
            devices = linkedSetOf(
                CommunicationDeviceKind.SPEAKER,
                CommunicationDeviceKind.EARPIECE
            )
        )
        val controller = CommunicationAudioController(
            backend = backend,
            communicationMode = 3
        )

        val selected = controller.activate()

        assertEquals(CommunicationDeviceKind.SPEAKER, selected)
        assertEquals(3, backend.mode)
        assertTrue(backend.speakerphoneOn)
        assertEquals(listOf(CommunicationDeviceKind.SPEAKER), backend.selectedDevices)

        controller.deactivate()

        assertEquals(7, backend.mode)
        assertTrue(backend.speakerphoneOn)
        assertEquals(CommunicationDeviceKind.SPEAKER, backend.currentCommunicationDevice())
        assertEquals(0, backend.clearCommunicationDeviceCalls)
    }

    @Test
    fun activate_fallsBackToWiredHeadsetWhenBluetoothRoutingFails() {
        val backend = FakeCommunicationAudioBackend(
            devices = linkedSetOf(
                CommunicationDeviceKind.SPEAKER,
                CommunicationDeviceKind.WIRED_HEADSET,
                CommunicationDeviceKind.BLUETOOTH
            ),
            routeableDevices = setOf(
                CommunicationDeviceKind.WIRED_HEADSET,
                CommunicationDeviceKind.SPEAKER
            )
        )
        val controller = CommunicationAudioController(
            backend = backend,
            communicationMode = 3
        )

        val selected = controller.activate()

        assertEquals(CommunicationDeviceKind.WIRED_HEADSET, selected)
        assertEquals(
            listOf(CommunicationDeviceKind.BLUETOOTH, CommunicationDeviceKind.WIRED_HEADSET),
            backend.selectedDevices
        )
    }

    @Test
    fun activate_fallsBackToSpeakerWhenNoPrivateRouteExists() {
        val backend = FakeCommunicationAudioBackend(
            devices = linkedSetOf(CommunicationDeviceKind.SPEAKER)
        )
        val controller = CommunicationAudioController(
            backend = backend,
            communicationMode = 3
        )

        val selected = controller.activate()

        assertEquals(CommunicationDeviceKind.SPEAKER, selected)
        assertTrue(backend.speakerphoneOn)
        assertEquals(listOf(CommunicationDeviceKind.SPEAKER), backend.selectedDevices)
    }

    @Test
    fun activate_handlesMissingCommunicationDeviceGracefully() {
        val backend = FakeCommunicationAudioBackend(devices = linkedSetOf())
        val controller = CommunicationAudioController(
            backend = backend,
            communicationMode = 3
        )

        val selected = controller.activate()

        assertNull(selected)
        assertEquals(3, backend.mode)
        assertFalse(backend.speakerphoneOn)
        assertTrue(backend.selectedDevices.isEmpty())
    }

    @Test
    fun deactivate_restoresExactPreviousCommunicationMode() {
        val backend = FakeCommunicationAudioBackend(
            initialMode = 3,
            devices = linkedSetOf(CommunicationDeviceKind.EARPIECE)
        )
        val controller = CommunicationAudioController(
            backend = backend,
            communicationMode = 3
        )

        controller.activate()
        controller.deactivate()

        assertEquals(3, backend.mode)
    }

    @Test
    fun deactivate_clearsRouteWhenPreviousDeviceCannotBeRestored() {
        val backend = FakeCommunicationAudioBackend(
            initialCommunicationDevice = CommunicationDeviceKind.BLUETOOTH,
            devices = linkedSetOf(
                CommunicationDeviceKind.BLUETOOTH,
                CommunicationDeviceKind.EARPIECE
            ),
            routeableDevices = setOf(CommunicationDeviceKind.EARPIECE)
        )
        val controller = CommunicationAudioController(
            backend = backend,
            communicationMode = 3
        )

        controller.activate()
        controller.deactivate()

        assertNull(backend.currentCommunicationDevice())
        assertEquals(1, backend.clearCommunicationDeviceCalls)
    }

    private class FakeCommunicationAudioBackend(
        initialMode: Int = 0,
        initialSpeakerphoneOn: Boolean = false,
        initialCommunicationDevice: CommunicationDeviceKind? = null,
        private val devices: LinkedHashSet<CommunicationDeviceKind> = linkedSetOf(),
        private val routeableDevices: Set<CommunicationDeviceKind> = devices
    ) : CommunicationAudioBackend {
        override var mode: Int = initialMode
        override var speakerphoneOn: Boolean = initialSpeakerphoneOn
        private var communicationDevice: CommunicationDeviceKind? = initialCommunicationDevice
        val selectedDevices = mutableListOf<CommunicationDeviceKind>()
        var clearCommunicationDeviceCalls = 0

        override fun availableDevices(): Set<CommunicationDeviceKind> = devices

        override fun currentCommunicationDevice(): CommunicationDeviceKind? = communicationDevice

        override fun setCommunicationDevice(device: CommunicationDeviceKind): Boolean {
            selectedDevices += device
            if (device !in routeableDevices) {
                return false
            }
            communicationDevice = device
            return true
        }

        override fun clearCommunicationDevice() {
            clearCommunicationDeviceCalls += 1
            communicationDevice = null
        }
    }
}
