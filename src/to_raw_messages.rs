// Copyright 2015 Sam Doshi (sam@metal-fish.co.uk)
//
// Licensed under the MIT License <LICENSE or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use heapless::Vec;

use constants::*;
use types::{U7, Channel};
use raw_message::{RawMessage};
use RawMessage::*;
use message::{Message};
use Message::*;
use utils::{mask7, status_byte, u14_to_msb_lsb};

/// Error returned when the requested output capacity is too small.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct CapacityError;

/// Fixed-capacity storage for raw MIDI messages.
pub type RawMessages<const N: usize> = Vec<RawMessage, N>;

/// Convert `self` to a fixed-capacity collection of `RawMessage`s.
///
/// A `RawMessages` value represents ordered MIDI data that must be sent as a contiguous
/// block, this is useful for representing `Message::SysEx` and `Message::NRPN14`,
/// note that midi clock messages are allowed to interrupt sysex messages as part of the spec.
pub trait ToRawMessages {
    /// Returns `CapacityError` when `N` cannot hold the converted message.
    fn to_raw_messages<const N: usize>(&self) -> Result<RawMessages<N>, CapacityError>;
}

impl ToRawMessages for RawMessage {
    fn to_raw_messages<const N: usize>(&self) -> Result<RawMessages<N>, CapacityError> {
        raw_messages(&[*self])
    }
}

impl ToRawMessages for Message {
    fn to_raw_messages<const N: usize>(&self) -> Result<RawMessages<N>, CapacityError> {
        match self {
            // System realtime
            &Start => raw_messages(&[Status(START)]),
            &TimingClock => raw_messages(&[Status(TIMING_CLOCK)]),
            &Continue => raw_messages(&[Status(CONTINUE)]),
            &Stop => raw_messages(&[Status(STOP)]),
            &ActiveSensing => raw_messages(&[Status(ACTIVE_SENSING)]),
            &SystemReset => raw_messages(&[Status(SYSTEM_RESET)]),

            // Channel mode
            &AllSoundOff(ch) => ControlChange(ch, 120, 0).to_raw_messages::<N>(),
            &ResetAllControllers(ch) => ControlChange(ch, 121, 0).to_raw_messages::<N>(),
            &LocalControlOff(ch) => ControlChange(ch, 122, 0).to_raw_messages::<N>(),
            &LocalControlOn(ch) => ControlChange(ch, 122, 127).to_raw_messages::<N>(),
            &AllNotesOff(ch) => ControlChange(ch, 123, 0).to_raw_messages::<N>(),

            // Channel voice
            &ProgramChange(ch, no) => {
                let sb = status_byte(PROGRAM_CHANGE, ch);
                raw_messages(&[StatusData(sb, mask7(no))])
            },
            &ControlChange(ch, no, val) => {
                raw_messages(&[cc(ch, mask7(no), mask7(val))])
            },
            &RPN7(ch, rpn, val) => {
                let (rpn_msb, rpn_lsb) = u14_to_msb_lsb(rpn);
                raw_messages(&[
                    cc(ch, CC_RPN_MSB, rpn_msb),
                    cc(ch, CC_RPN_LSB, rpn_lsb),
                    cc(ch, CC_DATA_ENTRY_MSB, mask7(val))
                ])
            },
            &RPN14(ch, rpn, val) => {
                let (rpn_msb, rpn_lsb) = u14_to_msb_lsb(rpn);
                let (val_msb, val_lsb) = u14_to_msb_lsb(val);
                raw_messages(&[
                    cc(ch, CC_RPN_MSB, rpn_msb),
                    cc(ch, CC_RPN_LSB, rpn_lsb),
                    cc(ch, CC_DATA_ENTRY_MSB, val_msb),
                    cc(ch, CC_DATA_ENTRY_LSB, val_lsb)
                ])
            },
            &NRPN7(ch, nrpn, val) => {
                let (nrpn_msb, nrpn_lsb) = u14_to_msb_lsb(nrpn);
                raw_messages(&[
                    cc(ch, CC_NRPN_MSB, nrpn_msb),
                    cc(ch, CC_NRPN_LSB, nrpn_lsb),
                    cc(ch, CC_DATA_ENTRY_MSB, mask7(val))
                ])
            },
            &NRPN14(ch, nrpn, val) => {
                let (nrpn_msb, nrpn_lsb) = u14_to_msb_lsb(nrpn);
                let (val_msb, val_lsb) = u14_to_msb_lsb(val);
                raw_messages(&[
                    cc(ch, CC_NRPN_MSB, nrpn_msb),
                    cc(ch, CC_NRPN_LSB, nrpn_lsb),
                    cc(ch, CC_DATA_ENTRY_MSB, val_msb),
                    cc(ch, CC_DATA_ENTRY_LSB, val_lsb)
                ])
            },
            &SysEx(manufacturer, ref data) => {
                let mut output = Vec::new();
                push(&mut output, Raw(SYSEX))?;
                for byte in manufacturer.to_u7s() {
                    push(&mut output, Raw(byte))?;
                }
                for byte in data {
                    push(&mut output, Raw(mask7(*byte)))?;
                }
                push(&mut output, Raw(SYSEX_EOX))?;
                Ok(output)
            },
            &NoteOff(ch, no, vel) => {
                let sb = status_byte(NOTE_OFF, ch);
                raw_messages(&[StatusDataData(sb, mask7(no), mask7(vel))])
            },
            &NoteOn(ch, no, vel) => {
                let sb = status_byte(NOTE_ON, ch);
                raw_messages(&[StatusDataData(sb, mask7(no), mask7(vel))])
            },
            &PitchBend(ch, bend) => {
                let sb = status_byte(PITCH_BEND, ch);
                let (msb, lsb) = u14_to_msb_lsb(bend);
                raw_messages(&[StatusDataData(sb, lsb, msb)])
            }
            &PolyphonicPressure(ch, no, vel) => {
                let sb = status_byte(POLYPHONIC_PRESSURE, ch);
                raw_messages(&[StatusDataData(sb, mask7(no), mask7(vel))])
            },
            &ChannelPressure(ch, vel) => {
                let sb = status_byte(CHANNEL_PRESSURE, ch);
                raw_messages(&[StatusData(sb, mask7(vel))])
            }
        }
    }
}

fn raw_messages<const N: usize>(messages: &[RawMessage]) -> Result<RawMessages<N>, CapacityError> {
    Vec::from_slice(messages).map_err(|_| CapacityError)
}

fn push<const N: usize>(messages: &mut RawMessages<N>, message: RawMessage)
    -> Result<(), CapacityError>
{
    messages.push(message).map_err(|_| CapacityError)
}

// we need to generate a lot of CC messages...
fn cc(ch: Channel, cc_no: U7, val: U7) -> RawMessage {
    let sb = status_byte(CONTROL_CHANGE, ch);
    StatusDataData(sb, cc_no, val)
}

#[cfg(test)]
mod test {
    use super::{CapacityError, ToRawMessages};
    use message::{MAX_SYSEX_DATA_LEN, SysExData};
    use message::Message::*;
    use raw_message::RawMessage::*;
    use manufacturer::Manufacturer::*;
    use types::Channel::*;

    #[test]
    fn test_message_to_raw_messages() {
        // Where possible these numbers have been pasted in from
        // http://www.midi.org/techspecs/midimessages.php

        // Start
        assert_eq!(Start.to_raw_messages::<16>().unwrap().as_slice(), &[Status(0b11111010)]);

        // TimingClock
        assert_eq!(TimingClock.to_raw_messages::<16>().unwrap().as_slice(), &[Status(0b11111000)]);

        // Continue
        assert_eq!(Continue.to_raw_messages::<16>().unwrap().as_slice(), &[Status(0b11111011)]);

        // Stop
        assert_eq!(Stop.to_raw_messages::<16>().unwrap().as_slice(), &[Status(0b11111100)]);

        // ActiveSensing
        assert_eq!(ActiveSensing.to_raw_messages::<16>().unwrap().as_slice(), &[Status(0b11111110)]);

        // SystemReset
        assert_eq!(SystemReset.to_raw_messages::<16>().unwrap().as_slice(), &[Status(0b11111111)]);

        // AllSoundOff
        assert_eq!(AllSoundOff(Ch1).to_raw_messages::<16>().unwrap().as_slice(), &[StatusDataData(176, 120, 0)]);

        // ResetAllControllers
        assert_eq!(ResetAllControllers(Ch1).to_raw_messages::<16>().unwrap().as_slice(), &[StatusDataData(176, 121, 0)]);

        // LocalControlOff
        assert_eq!(LocalControlOff(Ch1).to_raw_messages::<16>().unwrap().as_slice(), &[StatusDataData(176, 122, 0)]);

        // LocalControlOn
        assert_eq!(LocalControlOn(Ch1).to_raw_messages::<16>().unwrap().as_slice(), &[StatusDataData(176, 122, 127)]);

        // AllNotesOff
        assert_eq!(AllNotesOff(Ch1).to_raw_messages::<16>().unwrap().as_slice(), &[StatusDataData(176, 123, 0)]);

        // ProgramChange
        assert_eq!(ProgramChange(Ch1, 0).to_raw_messages::<16>().unwrap().as_slice(), &[StatusData(192, 0)]);
        assert_eq!(ProgramChange(Ch1, 127).to_raw_messages::<16>().unwrap().as_slice(), &[StatusData(192, 127)]);
        assert_eq!(ProgramChange(Ch1, 128).to_raw_messages::<16>().unwrap().as_slice(), &[StatusData(192, 0)]);

        // ControlChange
        assert_eq!(ControlChange(Ch1, 0, 0).to_raw_messages::<16>().unwrap().as_slice(), &[StatusDataData(176, 0, 0)]);
        assert_eq!(ControlChange(Ch1, 0, 127).to_raw_messages::<16>().unwrap().as_slice(), &[StatusDataData(176, 0, 127)]);
        assert_eq!(ControlChange(Ch1, 0, 128).to_raw_messages::<16>().unwrap().as_slice(), &[StatusDataData(176, 0, 0)]);
        assert_eq!(ControlChange(Ch1, 127, 0).to_raw_messages::<16>().unwrap().as_slice(), &[StatusDataData(176, 127, 0)]);
        assert_eq!(ControlChange(Ch1, 128, 0).to_raw_messages::<16>().unwrap().as_slice(), &[StatusDataData(176, 0, 0)]);

        // RPN7
        assert_eq!(RPN7(Ch1, 1000, 0).to_raw_messages::<16>().unwrap().as_slice(), &[StatusDataData(176, 101, 7),
                                                              StatusDataData(176, 100, 104),
                                                              StatusDataData(176, 6, 0)]);

        // RPN14
        assert_eq!(RPN14(Ch1, 1000, 1001).to_raw_messages::<16>().unwrap().as_slice(), &[StatusDataData(176, 101, 7),
                                                                  StatusDataData(176, 100, 104),
                                                                  StatusDataData(176, 6, 7),
                                                                  StatusDataData(176, 38, 105)]);

        // NRPN7
        assert_eq!(NRPN7(Ch1, 1000, 0).to_raw_messages::<16>().unwrap().as_slice(), &[StatusDataData(176, 99, 7),
                                                               StatusDataData(176, 98, 104),
                                                               StatusDataData(176, 6, 0)]);

        // NRPN14
        assert_eq!(NRPN14(Ch1, 1000, 1001).to_raw_messages::<16>().unwrap().as_slice(), &[StatusDataData(176, 99, 7),
                                                                   StatusDataData(176, 98, 104),
                                                                   StatusDataData(176, 6, 7),
                                                                   StatusDataData(176, 38, 105)]);

        // SysEx
        assert_eq!(SysEx(OneByte(100), sysex(&[1, 2, 3, 4])).to_raw_messages::<16>().unwrap().as_slice(),
                   &[Raw(0b11110000),
                        Raw(100),
                        Raw(1), Raw(2), Raw(3), Raw(4),
                        Raw(0b11110111)]);

        assert_eq!(SysEx(OneByte(128), sysex(&[1, 2, 3, 4, 128])).to_raw_messages::<16>().unwrap().as_slice(),
                   &[Raw(0b11110000),
                        Raw(0),
                        Raw(1), Raw(2), Raw(3), Raw(4), Raw(0),
                        Raw(0b11110111)]);

        assert_eq!(SysEx(ThreeByte(100, 101, 128), sysex(&[1, 2, 3, 4])).to_raw_messages::<16>().unwrap().as_slice(),
                   &[Raw(0b11110000),
                        Raw(100), Raw(101), Raw(0),
                        Raw(1), Raw(2), Raw(3), Raw(4),
                        Raw(0b11110111)]);

        // NoteOff
        assert_eq!(NoteOff(Ch1, 0, 0).to_raw_messages::<16>().unwrap().as_slice(), &[StatusDataData(128, 0, 0)]);
        assert_eq!(NoteOff(Ch2, 127, 127).to_raw_messages::<16>().unwrap().as_slice(), &[StatusDataData(129, 127, 127)]);
        assert_eq!(NoteOff(Ch3, 128, 128).to_raw_messages::<16>().unwrap().as_slice(), &[StatusDataData(130, 0, 0)]);

        // NoteOn
        assert_eq!(NoteOn(Ch4, 0, 0).to_raw_messages::<16>().unwrap().as_slice(), &[StatusDataData(147, 0, 0)]);
        assert_eq!(NoteOn(Ch5, 127, 127).to_raw_messages::<16>().unwrap().as_slice(), &[StatusDataData(148, 127, 127)]);
        assert_eq!(NoteOn(Ch6, 128, 128).to_raw_messages::<16>().unwrap().as_slice(), &[StatusDataData(149, 0, 0)]);

        // PitchBend
        assert_eq!(PitchBend(Ch7, 0).to_raw_messages::<16>().unwrap().as_slice(), &[StatusDataData(230, 0, 0)]);
        assert_eq!(PitchBend(Ch8, 1000).to_raw_messages::<16>().unwrap().as_slice(), &[StatusDataData(231, 104, 7)]);
        assert_eq!(PitchBend(Ch9, 45000).to_raw_messages::<16>().unwrap().as_slice(), &[StatusDataData(232, 72, 95)]);
        assert_eq!(PitchBend(Ch10, 12232).to_raw_messages::<16>().unwrap().as_slice(), &[StatusDataData(233, 72, 95)]);

        // PolyphonicPressure
        assert_eq!(PolyphonicPressure(Ch11, 0, 0).to_raw_messages::<16>().unwrap().as_slice(),
                   &[StatusDataData(170, 0, 0)]);
        assert_eq!(PolyphonicPressure(Ch12, 127, 127).to_raw_messages::<16>().unwrap().as_slice(),
                   &[StatusDataData(171, 127, 127)]);
        assert_eq!(PolyphonicPressure(Ch13, 128, 128).to_raw_messages::<16>().unwrap().as_slice(),
                   &[StatusDataData(172, 0, 0)]);

        // ChannelPressure
        assert_eq!(ChannelPressure(Ch14, 0).to_raw_messages::<16>().unwrap().as_slice(), &[StatusData(221, 0)]);
        assert_eq!(ChannelPressure(Ch15, 127).to_raw_messages::<16>().unwrap().as_slice(), &[StatusData(222, 127)]);
        assert_eq!(ChannelPressure(Ch16, 128).to_raw_messages::<16>().unwrap().as_slice(), &[StatusData(223, 0)]);
    }

    #[test]
    fn test_capacity_errors() {
        assert_eq!(RPN14(Ch1, 1000, 1001).to_raw_messages::<3>(), Err(CapacityError));
        assert!(SysExData::from_slice(&[0; MAX_SYSEX_DATA_LEN + 1]).is_err());
    }

    fn sysex(data: &[u8]) -> SysExData {
        SysExData::from_slice(data).unwrap()
    }
}
