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
pub enum Format {
    Unspecified,
    I16,
    I24,
    I32,
    Float,
}
