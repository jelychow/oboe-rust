#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SupportLevel {
    Supported,
    Partial,
    Unsupported,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Capability {
    pub name: &'static str,
    pub support: SupportLevel,
    pub note: &'static str,
}

pub const CAPABILITIES: &[Capability] = &[
    Capability {
        name: "aaudio_blocking_io",
        support: SupportLevel::Supported,
        note: "AAudio open/start/stop/close plus blocking f32 read and write are available.",
    },
    Capability {
        name: "opensles_output",
        support: SupportLevel::Partial,
        note: "OpenSL ES output enqueue is available; input recording is not implemented.",
    },
    Capability {
        name: "stream_callbacks",
        support: SupportLevel::Partial,
        note: "Rust-native callback contracts and AAudio callback binding are available; Java realtime callback dispatch remains unsupported.",
    },
    Capability {
        name: "latency_and_xrun_diagnostics",
        support: SupportLevel::Partial,
        note: "AAudio timestamp, xrun, frame counters, burst size, and buffer size tuning APIs are exposed.",
    },
    Capability {
        name: "advanced_builder_fields",
        support: SupportLevel::Unsupported,
        note: "Usage, content type, input preset, session id, device id, capture policy, privacy, spatialization, package name, attribution tag, and conversion policy are not available.",
    },
];

pub fn capability(name: &str) -> Option<&'static Capability> {
    CAPABILITIES
        .iter()
        .find(|capability| capability.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracks_supported_aaudio_blocking_io() {
        let capability = capability("aaudio_blocking_io").unwrap();
        assert_eq!(capability.support, SupportLevel::Supported);
        assert!(capability.note.contains("blocking f32 read and write"));
    }

    #[test]
    fn tracks_partial_callback_dispatch() {
        let capability = capability("stream_callbacks").unwrap();
        assert_eq!(capability.support, SupportLevel::Partial);
        assert!(capability.note.contains("AAudio callback"));
    }

    #[test]
    fn all_capability_names_are_unique() {
        for (index, capability) in CAPABILITIES.iter().enumerate() {
            assert!(
                CAPABILITIES[index + 1..]
                    .iter()
                    .all(|other| other.name != capability.name),
                "duplicate capability name: {}",
                capability.name
            );
        }
    }
}
