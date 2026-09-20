//! Presentation of standard endpoint fields (USB 2.0, table 9-13).
use cyme::usb::{Speed, TransferType};

/// Decode the descriptor's requested service interval, not measured timing.
pub fn interval(kind: &TransferType, speed: Option<&Speed>, raw: u8) -> String {
    if matches!(kind, TransferType::Bulk | TransferType::Control) {
        return "Not periodic".into();
    }
    let unit = match speed {
        Some(Speed::LowSpeed | Speed::FullSpeed) if matches!(kind, TransferType::Interrupt) => {
            return if raw == 0 {
                "Invalid interval".into()
            } else {
                format!("{raw} ms")
            };
        }
        Some(Speed::FullSpeed) => 1000u32,
        Some(
            Speed::HighSpeed
            | Speed::HighBandwidth
            | Speed::SuperSpeed
            | Speed::SuperSpeedPlus
            | Speed::SuperSpeedPlusX2
            | Speed::Usb40Gbps
            | Speed::Usb80Gbps,
        ) => 125,
        _ => return "Timing unavailable".into(),
    };
    if !(1..=16).contains(&raw) {
        return "Invalid interval".into();
    }
    let micros = unit * (1 << (raw - 1));
    if micros.is_multiple_of(1000) {
        format!("{} ms", micros / 1000)
    } else {
        format!("{micros} µs")
    }
}

pub fn transfer_help(kind: &TransferType) -> &'static str {
    match kind {
        TransferType::Control => "Commands and device setup, with error recovery.",
        TransferType::Bulk => "Reliable data transfer using available bus bandwidth.",
        TransferType::Interrupt => "Periodic service for small, time-sensitive transfers.",
        TransferType::Isochronous => "Reserved periodic bandwidth; failed packets are not retried.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn interval_depends_on_speed_and_transfer_type() {
        assert_eq!(
            interval(&TransferType::Interrupt, Some(&Speed::FullSpeed), 4),
            "4 ms"
        );
        assert_eq!(
            interval(&TransferType::Interrupt, Some(&Speed::HighSpeed), 4),
            "1 ms"
        );
        assert_eq!(
            interval(&TransferType::Isochronous, Some(&Speed::HighSpeed), 1),
            "125 µs"
        );
        assert_eq!(
            interval(&TransferType::Isochronous, Some(&Speed::FullSpeed), 4),
            "8 ms"
        );
        assert_eq!(
            interval(&TransferType::Bulk, Some(&Speed::HighSpeed), 0),
            "Not periodic"
        );
    }
    #[test]
    fn invalid_or_unknown_timing_is_not_invented() {
        assert_eq!(
            interval(&TransferType::Interrupt, None, 4),
            "Timing unavailable"
        );
        assert_eq!(
            interval(&TransferType::Isochronous, Some(&Speed::HighSpeed), 0),
            "Invalid interval"
        );
        assert_eq!(
            interval(&TransferType::Isochronous, Some(&Speed::HighSpeed), 17),
            "Invalid interval"
        );
    }
}
