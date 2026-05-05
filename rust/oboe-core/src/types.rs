#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioApi {
    Unspecified,
    AAudio,
    OpenSLES,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Input,
    Output,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SharingMode {
    Shared,
    Exclusive,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PerformanceMode {
    None,
    PowerSaving,
    LowLatency,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputPreset {
    Unspecified,
    Generic,
    Camcorder,
    VoiceRecognition,
    VoiceCommunication,
    Unprocessed,
    VoicePerformance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Usage {
    Unspecified,
    Media,
    VoiceCommunication,
    VoiceCommunicationSignalling,
    Alarm,
    Notification,
    NotificationRingtone,
    NotificationEvent,
    AssistanceAccessibility,
    AssistanceNavigationGuidance,
    AssistanceSonification,
    Game,
    Assistant,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContentType {
    Unspecified,
    Speech,
    Music,
    Movie,
    Sonification,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Unspecified,
    I16,
    I24,
    I32,
    Float,
}
