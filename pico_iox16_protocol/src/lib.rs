#![no_std]

use core::fmt::Debug;
use crc::{CRC_16_KERMIT, Crc};
use defmt::Format;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use zerocopy::{
    I16, I32, Immutable, IntoBytes, KnownLayout, LE, TryFromBytes, U16, U32, U64, Unaligned,
};

pub const MAGIC: [u8; 2] = *b"OM";

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Format, derive_more::Display,
)]
#[repr(u16)]
pub enum Command {
    /// Check if the device is alive and responding.
    Check = 0,
    /// Get information about the device.
    InfoGet = 1,
    /// Set the current configuration of the device. Persists across reboots.
    ConfigSet = 2,
    /// Get the current configuration of the device.
    ConfigGet = 3,
    /// Set the output states. Resets after reboot.
    OutputSet = 4,
    /// Get the output states.
    OutputGet = 5,
    /// Get the current input values.
    ///
    /// Returns a 16-bit signed integer for each input avaraged over the last reads
    /// performed since the previous `InputGet` or `InputGetFull` request.
    InputGet = 6,
    /// Get the current input values along with additional statistics and reset
    /// the accumulated data.
    InputGetFull = 7,
    /// Set the input calibrations. Persists across reboots.
    ///
    /// Each input can be scaled an shifted individually. This is intended to store
    /// calibration on the device itself, but it can be used to offload minor preprocessing
    /// from the host.
    InputSetCalibrations = 8,
    /// Get the input calibrations.
    InputGetCalibrations = 9,
    /// Set the input thresholds. Persists across reboots.
    ///
    /// Each input can be configured with a high and low threshold, as well as debounce parameters.
    InputSetThresholds = 10,
    /// Get the input thresholds.
    InputGetThresholds = 11,
    /// Get the times of the last threshold crossings for each input.
    InputGetThresholdTimes = 12,
    /// Get the current states of the input thresholds.
    InputGetThresholdStates = 13,
    /// Reboot the device.
    Reboot = 14,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Request<'a> {
    Check(&'a CheckReq),
    InfoGet(&'a InfoGetReq),
    ConfigGet(&'a ConfigGetReq),
    ConfigSet(&'a ConfigSetReq),
    OutputSet(&'a OutputSetReq),
    OutputGet(&'a OutputGetReq),
    InputGet(&'a InputGetReq),
    InputGetFull(&'a InputGetFullReq),
    InputSetCalibrations(&'a InputSetCalibrationsReq),
    InputGetCalibrations(&'a InputGetCalibrationsReq),
    InputSetThresholds(&'a InputSetThresholdsReq),
    InputGetThresholds(&'a InputGetThresholdsReq),
    InputGetThresholdTimes(&'a InputGetThresholdTimesReq),
    InputGetThresholdStates(&'a InputGetThresholdStatesReq),
    Reboot(&'a RebootReq),
}
impl Request<'_> {
    pub fn command(&self) -> Command {
        match self {
            Request::Check(_) => Command::Check,
            Request::InfoGet(_) => Command::InfoGet,
            Request::ConfigGet(_) => Command::ConfigGet,
            Request::ConfigSet(_) => Command::ConfigSet,
            Request::OutputSet(_) => Command::OutputSet,
            Request::OutputGet(_) => Command::OutputGet,
            Request::InputGet(_) => Command::InputGet,
            Request::InputGetFull(_) => Command::InputGetFull,
            Request::InputSetCalibrations(_) => Command::InputSetCalibrations,
            Request::InputGetCalibrations(_) => Command::InputGetCalibrations,
            Request::InputSetThresholds(_) => Command::InputSetThresholds,
            Request::InputGetThresholds(_) => Command::InputGetThresholds,
            Request::InputGetThresholdTimes(_) => Command::InputGetThresholdTimes,
            Request::InputGetThresholdStates(_) => Command::InputGetThresholdStates,
            Request::Reboot(_) => Command::Reboot,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Response<'a> {
    Check(&'a CheckRes),
    InfoGet(&'a InfoGetRes),
    ConfigGet(&'a ConfigGetRes),
    ConfigSet(&'a ConfigSetRes),
    OutputSet(&'a OutputSetRes),
    OutputGet(&'a OutputGetRes),
    InputGet(&'a InputGetRes),
    InputGetFull(&'a InputGetFullRes),
    InputSetCalibrations(&'a InputSetCalibrationsRes),
    InputGetCalibrations(&'a InputGetCalibrationsRes),
    InputSetThresholds(&'a InputSetThresholdsRes),
    InputGetThresholds(&'a InputGetThresholdsRes),
    InputGetThresholdTimes(&'a InputGetThresholdTimesRes),
    InputGetThresholdStates(&'a InputGetThresholdStatesRes),
    Reboot(&'a RebootRes),
}
impl Response<'_> {
    pub fn command(&self) -> Command {
        match self {
            Response::Check(_) => Command::Check,
            Response::InfoGet(_) => Command::InfoGet,
            Response::ConfigGet(_) => Command::ConfigGet,
            Response::ConfigSet(_) => Command::ConfigSet,
            Response::OutputSet(_) => Command::OutputSet,
            Response::OutputGet(_) => Command::OutputGet,
            Response::InputGet(_) => Command::InputGet,
            Response::InputGetFull(_) => Command::InputGetFull,
            Response::InputSetCalibrations(_) => Command::InputSetCalibrations,
            Response::InputGetCalibrations(_) => Command::InputGetCalibrations,
            Response::InputSetThresholds(_) => Command::InputSetThresholds,
            Response::InputGetThresholds(_) => Command::InputGetThresholds,
            Response::InputGetThresholdTimes(_) => Command::InputGetThresholdTimes,
            Response::InputGetThresholdStates(_) => Command::InputGetThresholdStates,
            Response::Reboot(_) => Command::Reboot,
        }
    }
}

pub trait RequestTrait:
    Debug
    + Clone
    + Copy
    + PartialEq
    + Eq
    + IntoBytes
    + TryFromBytes
    + Unaligned
    + Immutable
    + KnownLayout
{
    const COMMAND: Command;
    const TIMEOUT_US: u32;
    type Response: Debug
        + Clone
        + Copy
        + PartialEq
        + Eq
        + IntoBytes
        + TryFromBytes
        + Unaligned
        + Immutable
        + KnownLayout;
    fn get_response(response: Response<'_>) -> Option<&Self::Response>;
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, IntoBytes, TryFromBytes, Unaligned, Immutable, KnownLayout,
)]
#[repr(C)]
pub struct CheckReq;
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, IntoBytes, TryFromBytes, Unaligned, Immutable, KnownLayout,
)]
#[repr(C)]
pub struct CheckRes;
impl RequestTrait for CheckReq {
        const COMMAND: Command = Command::Check;
        const TIMEOUT_US: u32 = 100;
    type Response = CheckRes;
    fn get_response(response: Response<'_>) -> Option<&Self::Response> {
        match response {
            Response::Check(res) => Some(res),
            _ => None,
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, TryFromBytes, IntoBytes, Unaligned, Immutable, KnownLayout,
)]
#[repr(C)]
pub struct InfoGetReq;
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, TryFromBytes, IntoBytes, Unaligned, Immutable, KnownLayout,
)]
#[repr(C)]
pub struct InfoGetRes {
    /// UTF-8 string filled with null bytes (not necessarily null-terminated)
    pub info: [u8; 32],
    /// Major version
    pub firmware_version_major: u8,
    /// Minor version
    pub firmware_version_minor: u8,
    /// Patch version
    pub firmware_version_patch: U16<LE>,
    /// Uptime in seconds
    pub uptime: U32<LE>,
}
impl RequestTrait for InfoGetReq {
    const COMMAND: Command = Command::InfoGet;
    const TIMEOUT_US: u32 = 100;
    type Response = InfoGetRes;
    fn get_response(response: Response<'_>) -> Option<&Self::Response> {
        match response {
            Response::InfoGet(res) => Some(res),
            _ => None,
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, IntoBytes, TryFromBytes, Unaligned, Immutable, KnownLayout,
)]
#[repr(C)]
pub struct OutputGroup {
    /// Duty cycle scaled by 32768 (i.e. 50% = 16384, 100% = 32768)
    pub duty_cycle: [U16<LE>; 2],
    /// Frequency in Hz
    pub frequency: U16<LE>,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, IntoBytes, TryFromBytes, Unaligned, Immutable, KnownLayout,
)]
#[repr(C)]
pub struct OutputSetReq(pub [OutputGroup; 8]);
impl Default for OutputSetReq {
    fn default() -> Self {
        Self(
            [OutputGroup {
                duty_cycle: [0.into(); 2],
                frequency: 1000.into(),
            }; 8],
        )
    }
}
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, IntoBytes, TryFromBytes, Unaligned, Immutable, KnownLayout,
)]
#[repr(C)]
pub struct OutputSetRes;
impl RequestTrait for OutputSetReq {
    const COMMAND: Command = Command::OutputSet;
    const TIMEOUT_US: u32 = 100;
    type Response = OutputSetRes;
    fn get_response(response: Response<'_>) -> Option<&Self::Response> {
        match response {
            Response::OutputSet(res) => Some(res),
            _ => None,
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, IntoBytes, TryFromBytes, Unaligned, Immutable, KnownLayout,
)]
#[repr(C)]
pub struct OutputGetReq;
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, IntoBytes, TryFromBytes, Unaligned, Immutable, KnownLayout,
)]
#[repr(C)]
pub struct OutputGetRes(pub [OutputGroup; 8]);
impl RequestTrait for OutputGetReq {
    const COMMAND: Command = Command::OutputGet;
    const TIMEOUT_US: u32 = 100;
    type Response = OutputGetRes;
    fn get_response(response: Response<'_>) -> Option<&Self::Response> {
        match response {
            Response::OutputGet(res) => Some(res),
            _ => None,
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, IntoBytes, TryFromBytes, Unaligned, Immutable, KnownLayout,
)]
#[repr(C)]
pub struct InputGetReq;
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, IntoBytes, TryFromBytes, Unaligned, Immutable, KnownLayout,
)]
#[repr(C)]
pub struct InputGetRes {
    /// The current value of each input after calibration. Average over the reads since the previous `InputGet` or `InputGetFull` request.
    ///
    /// **Note**: If no reads have been performed since the previous `InputGet` or `InputGetFull` request,
    /// the same value as in the previous `InputGetRes` will be returned.
    pub values: [I16<LE>; 16],
}
impl RequestTrait for InputGetReq {
    const COMMAND: Command = Command::InputGet;
    const TIMEOUT_US: u32 = 100;
    type Response = InputGetRes;
    fn get_response(response: Response<'_>) -> Option<&Self::Response> {
        match response {
            Response::InputGet(res) => Some(res),
            _ => None,
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, IntoBytes, TryFromBytes, Unaligned, Immutable, KnownLayout,
)]
#[repr(C)]
pub struct InputGetFullReq;
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, IntoBytes, TryFromBytes, Unaligned, Immutable, KnownLayout,
)]
#[repr(C)]
pub struct InputStat {
    /// The sum of the input values since the previous `InputGet` or `InputGetFull` request.
    pub sum: I32<LE>,
    /// The sum of the squares of the input values since the previous `InputGet` or `InputGetFull` request.
    pub sum_squares: U64<LE>,
    /// The minimum input value since the previous `InputGet` or `InputGetFull` request.
    pub min: I16<LE>,
    /// The maximum input value since the previous `InputGet` or `InputGetFull` request.
    pub max: I16<LE>,
    /// The number of input values since the previous `InputGet` or `InputGetFull` request.
    pub count: U16<LE>,
}
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, IntoBytes, TryFromBytes, Unaligned, Immutable, KnownLayout,
)]
#[repr(C)]
pub struct InputGetFullRes {
    pub stats: [InputStat; 16],
}
impl RequestTrait for InputGetFullReq {
    const COMMAND: Command = Command::InputGetFull;
    const TIMEOUT_US: u32 = 100;
    type Response = InputGetFullRes;
    fn get_response(response: Response<'_>) -> Option<&Self::Response> {
        match response {
            Response::InputGetFull(res) => Some(res),
            _ => None,
        }
    }
}

/// Performed in order the following order and with 32 bit arithmetic:
/// - Multiply the input by `multiply`
/// - Divide the result by `divide` (rounding towards zero)
/// - Add `add`
/// - Clamp the result between `min` and `max`
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, IntoBytes, TryFromBytes, Unaligned, Immutable, KnownLayout,
)]
#[repr(C)]
pub struct InputCalibration {
    /// The multiplication factor for the input value. Default is `1`.
    pub multiply: I16<LE>,
    /// The division factor for the input value. Default is `1`. Must not be `0`.
    pub divide: I16<LE>,
    /// The value to add to the input after multiplication and division. Default is `0`.
    pub add: I16<LE>,
    /// The minimum allowed value for the input. Default is `-32768`.
    pub min: I16<LE>,
    /// The maximum allowed value for the input. Default is `32767`.
    pub max: I16<LE>,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, IntoBytes, TryFromBytes, Unaligned, Immutable, KnownLayout,
)]
#[repr(C)]
pub struct InputSetCalibrationsReq(pub [InputCalibration; 16]);
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, IntoBytes, TryFromBytes, Unaligned, Immutable, KnownLayout,
)]
#[repr(C)]
pub struct InputSetCalibrationsRes;
impl RequestTrait for InputSetCalibrationsReq {
    const COMMAND: Command = Command::InputSetCalibrations;
    const TIMEOUT_US: u32 = 500000;
    type Response = InputSetCalibrationsRes;
    fn get_response(response: Response<'_>) -> Option<&Self::Response> {
        match response {
            Response::InputSetCalibrations(res) => Some(res),
            _ => None,
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, IntoBytes, TryFromBytes, Unaligned, Immutable, KnownLayout,
)]
#[repr(C)]
pub struct InputGetCalibrationsReq;
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, IntoBytes, TryFromBytes, Unaligned, Immutable, KnownLayout,
)]
#[repr(C)]
pub struct InputGetCalibrationsRes(pub [InputCalibration; 16]);
impl RequestTrait for InputGetCalibrationsReq {
    const COMMAND: Command = Command::InputGetCalibrations;
    const TIMEOUT_US: u32 = 100;
    type Response = InputGetCalibrationsRes;
    fn get_response(response: Response<'_>) -> Option<&Self::Response> {
        match response {
            Response::InputGetCalibrations(res) => Some(res),
            _ => None,
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, IntoBytes, TryFromBytes, Unaligned, Immutable, KnownLayout,
)]
#[repr(C)]
pub struct InputThreshold {
    /// The high threshold for the calibrated input. If the input value crosses from below to above this threshold, a high crossing event is recorded. Default is `32767`.
    pub threshold_high: I16<LE>,
    /// The low threshold for the calibrated input. If the input value crosses from above to below this threshold, a low crossing event is recorded. Default is `-32768`.
    pub threshold_low: I16<LE>,
    /// The debounce time in microseconds. Default is `0`, which means no debouncing.
    ///
    /// A debounced event is recorded when both `debounce_time_us` and `debounce_count` conditions are met.
    pub debounce_time_us: U32<LE>,
    /// The number of consecutive readings after the first reading that must be above
    /// the high threshold or below the low threshold before a crossing event is recorded.
    /// Default is `0`, which means no debouncing.
    ///
    /// A debounced event is recorded when both `debounce_time_us` and `debounce_count` conditions are met.
    pub debounce_count: U16<LE>,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, IntoBytes, TryFromBytes, Unaligned, Immutable, KnownLayout,
)]
#[repr(C)]
pub struct InputSetThresholdsReq(pub [InputThreshold; 16]);
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, IntoBytes, TryFromBytes, Unaligned, Immutable, KnownLayout,
)]
#[repr(C)]
pub struct InputSetThresholdsRes;
impl RequestTrait for InputSetThresholdsReq {
    const COMMAND: Command = Command::InputSetThresholds;
    const TIMEOUT_US: u32 = 500000;
    type Response = InputSetThresholdsRes;
    fn get_response(response: Response<'_>) -> Option<&Self::Response> {
        match response {
            Response::InputSetThresholds(res) => Some(res),
            _ => None,
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, IntoBytes, TryFromBytes, Unaligned, Immutable, KnownLayout,
)]
#[repr(C)]
pub struct InputGetThresholdsReq;
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, IntoBytes, TryFromBytes, Unaligned, Immutable, KnownLayout,
)]
#[repr(C)]
pub struct InputGetThresholdsRes(pub [InputThreshold; 16]);
impl RequestTrait for InputGetThresholdsReq {
    const COMMAND: Command = Command::InputGetThresholds;
    const TIMEOUT_US: u32 = 100;
    type Response = InputGetThresholdsRes;
    fn get_response(response: Response<'_>) -> Option<&Self::Response> {
        match response {
            Response::InputGetThresholds(res) => Some(res),
            _ => None,
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, IntoBytes, TryFromBytes, Unaligned, Immutable, KnownLayout,
)]
#[repr(C)]
pub struct InputGetThresholdTimesReq;
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, IntoBytes, TryFromBytes, Unaligned, Immutable, KnownLayout,
)]
#[repr(C)]
pub struct InputThresholdTimes {
    /// The time of the last low crossing event for the input in microseconds since boot. If no low crossing event has been recorded, this will be `0`.
    ///
    /// **Note**: This is the 'true' time of the crossing event, not the time when the debounce condition was met.
    /// **Note**: Timer ticks are in microseconds.
    pub last_low: U64<LE>,
    /// The time of the last high crossing event for the input in microseconds since boot. If no high crossing event has been recorded, this will be `0`.
    ///
    /// **Note**: This is the 'true' time of the crossing event, not the time when the debounce condition was met.
    /// **Note**: Timer ticks are in microseconds.
    pub last_high: U64<LE>,
}
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, IntoBytes, TryFromBytes, Unaligned, Immutable, KnownLayout,
)]
#[repr(C)]
pub struct InputGetThresholdTimesRes {
    /// Timer ticks in microseconds since boot for each input.
    pub now: U64<LE>,
    /// The times of the last threshold crossing events for each input.
    pub inputs: [InputThresholdTimes; 16],
}
impl RequestTrait for InputGetThresholdTimesReq {
    const COMMAND: Command = Command::InputGetThresholdTimes;
    const TIMEOUT_US: u32 = 100;
    type Response = InputGetThresholdTimesRes;
    fn get_response(response: Response<'_>) -> Option<&Self::Response> {
        match response {
            Response::InputGetThresholdTimes(res) => Some(res),
            _ => None,
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, IntoBytes, TryFromBytes, Unaligned, Immutable, KnownLayout,
)]
#[repr(C)]
pub struct InputGetThresholdStatesReq;
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, IntoBytes, TryFromBytes, Unaligned, Immutable, KnownLayout,
)]
#[repr(C)]
pub struct InputGetThresholdStatesRes {
    /// A bitmask indicating which inputs are above their `threshold_high` setting.
    pub above: U16<LE>,
    /// A bitmask indicating which inputs are below their `threshold_low` setting.
    pub below: U16<LE>,
}

impl RequestTrait for InputGetThresholdStatesReq {
    const COMMAND: Command = Command::InputGetThresholdStates;
    type Response = InputGetThresholdStatesRes;
    const TIMEOUT_US: u32 = 100;
    fn get_response(response: Response<'_>) -> Option<&Self::Response> {
        match response {
            Response::InputGetThresholdStates(res) => Some(res),
            _ => None,
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, IntoBytes, TryFromBytes, Unaligned, Immutable, KnownLayout,
)]
#[repr(C)]
pub struct Config {
    /// Device address. Address `0xFFFF` is reserved for unconfigured devices. Effective only after reboot.
    pub address: U16<LE>,
    /// The baudrate to use for communication with the device. Effective only after reboot.
    pub baudrate: U32<LE>,
    #[doc(hidden)]
    pub _reserved: [u8; 2],
}
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, IntoBytes, TryFromBytes, Unaligned, Immutable, KnownLayout,
)]
#[repr(C)]
pub struct ConfigGetReq;
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, IntoBytes, TryFromBytes, Unaligned, Immutable, KnownLayout,
)]
#[repr(C)]
pub struct ConfigGetRes(pub Config);
impl RequestTrait for ConfigGetReq {
    const COMMAND: Command = Command::ConfigGet;
    const TIMEOUT_US: u32 = 100;
    type Response = ConfigGetRes;
    fn get_response(response: Response<'_>) -> Option<&Self::Response> {
        match response {
            Response::ConfigGet(res) => Some(res),
            _ => None,
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, IntoBytes, TryFromBytes, Unaligned, Immutable, KnownLayout,
)]
#[repr(C)]
pub struct ConfigSetReq(pub Config);
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, IntoBytes, TryFromBytes, Unaligned, Immutable, KnownLayout,
)]
#[repr(C)]
pub struct ConfigSetRes;
impl RequestTrait for ConfigSetReq {
    const COMMAND: Command = Command::ConfigSet;
    const TIMEOUT_US: u32 = 500000;
    type Response = ConfigSetRes;
    fn get_response(response: Response<'_>) -> Option<&Self::Response> {
        match response {
            Response::ConfigSet(res) => Some(res),
            _ => None,
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, IntoBytes, TryFromBytes, Unaligned, Immutable, KnownLayout,
)]
#[repr(C)]
pub struct RebootReq;
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, IntoBytes, TryFromBytes, Unaligned, Immutable, KnownLayout,
)]
#[repr(C)]
pub struct RebootRes;
impl RequestTrait for RebootReq {
    const COMMAND: Command = Command::Reboot;
    const TIMEOUT_US: u32 = 500000;
    type Response = RebootRes;
    fn get_response(response: Response<'_>) -> Option<&Self::Response> {
        match response {
            Response::Reboot(res) => Some(res),
            _ => None,
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, TryFromBytes, IntoBytes, Unaligned, Immutable, KnownLayout,
)]
#[repr(C)]
pub struct Header {
    /// Magic bytes marking the start of a message. Must be `"OM"`.
    pub magic: [u8; 2],
    /// The length of the payload in 32-bit words. Must be equal to `!length_inverted`.
    pub length: u8,
    /// The bitwise inverse of `length`. Must be equal to `!length`.
    pub length_inverted: u8,
    /// The address of the device. For requests, this is the target address. For responses, this is the source address.
    pub address: U16<LE>,
    /// The command of the message. Valid values are defined in the [`Command`] enum.
    pub command: U16<LE>,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, TryFromBytes, IntoBytes, Unaligned, Immutable, KnownLayout,
)]
#[repr(C)]
pub struct Footer {
    /// The checksum of the message. Must be equal to the CRC-16/Kermit of the header and payload.
    pub checksum: U16<LE>,
}

pub const CHECKSUM: Crc<u16> = Crc::<u16>::new(&CRC_16_KERMIT);

#[derive(Debug, Clone, Copy, TryFromBytes, IntoBytes, Immutable)]
#[repr(C)]
pub struct Message<T> {
    pub header: Header,
    pub payload: T,
    pub footer: Footer,
}

impl<T: IntoBytes + Unaligned + Immutable> Message<T> {
    fn new_raw(address: u16, command: u16, payload: T) -> Self {
        assert!(size_of::<T>() <= u8::MAX as usize * 4);
        assert!(size_of::<T>().is_multiple_of(4));
        let header = Header {
            magic: MAGIC,
            length: u8::try_from(size_of::<T>() / 4).unwrap(),
            length_inverted: !u8::try_from(size_of::<T>() / 4).unwrap(),
            address: address.into(),
            command: command.into(),
        };
        let footer = Footer { checksum: 0.into() };
        let mut message = Message {
            header,
            payload,
            footer,
        };
        message.footer.checksum = CHECKSUM
            .checksum(&message.as_bytes()[..size_of::<Header>() + size_of::<T>()])
            .into();
        message
    }
    /// Creates a new request message with the given address, command, and payload.
    pub fn new_request(address: u16, command: Command, payload: T) -> Self {
        Self::new_raw(address, u16::from(command), payload)
    }
    /// Creates a new response message with the given address, command, and payload.
    pub fn new_response(address: u16, command: Command, payload: T) -> Self {
        Self::new_raw(address, u16::from(command), payload)
    }
}

/// Searches for the next valid message in the given byte slice and returns it along with the number of bytes processed.
/// If a valid message is found, the returned byte slice will contain the header and payload of the message, but not the footer.
/// If no valid message is found, the returned byte slice will be `None`.
/// The number of bytes processed is the number of bytes that were consumed from the input byte slice,
/// including any invalid data that was skipped over. Therefore it may consume bytes even if no valid message is found.
pub fn next_message(mut bytes: &[u8]) -> (Option<(&Header, &[u8])>, usize) {
    let mut processed = 0;
    while bytes.len() >= MAGIC.len() + 2 {
        if bytes[0..MAGIC.len()] == MAGIC && bytes[MAGIC.len()] == !bytes[MAGIC.len() + 1] {
            // valid header marker found
            let Ok((header, _)) = Header::try_ref_from_prefix(bytes) else {
                // too short
                break;
            };
            let length = header.length as usize * 4 + size_of::<Header>() + size_of::<Footer>();
            if bytes.len() < length {
                // too short
                break;
            }
            processed += length;
            let payload = &bytes[size_of::<Header>()..length - size_of::<Footer>()];
            let footer =
                Footer::try_ref_from_bytes(&bytes[length - size_of::<Footer>()..length]).unwrap();
            let checksum = CHECKSUM.checksum(&bytes[..length - size_of::<Footer>()]);
            if footer.checksum.get() == checksum {
                return (Some((header, payload)), processed);
            } else {
                // println!("checksum invalid: {:04x} != {:04x}", footer.checksum.get(), checksum);
                // Invalid checksum, continue searching
                bytes = &bytes[length..];
                continue;
            }
        }
        bytes = &bytes[1..];
        processed += 1;
    }
    (None, processed)
}

/// Parses the next message from the given byte slice and returns its address and the payload as a [`Response`]
/// along with the number of bytes processed. Skips invalid message headers and
/// messages with invalid checksums.
pub fn master_next<'a>(buffer: &'a [u8]) -> (Option<(u16, Response<'a>)>, usize) {
    let (maybe_message, processed) = next_message(buffer);
    let Some((header, payload)) = maybe_message else {
        return (None, processed);
    };
    let address = header.address.get();
    let command = header.command.get();
    match Command::try_from(command) {
        Err(_) => (None, processed),
        Ok(Command::Check) => (Some((address, Response::Check(&CheckRes))), processed),
        Ok(Command::InfoGet) => {
            let Ok(message) = InfoGetRes::try_ref_from_bytes(payload) else {
                return (None, processed);
            };
            (Some((address, Response::InfoGet(message))), processed)
        }
        Ok(Command::ConfigGet) => {
            let Ok(message) = ConfigGetRes::try_ref_from_bytes(payload) else {
                return (None, processed);
            };
            (Some((address, Response::ConfigGet(message))), processed)
        }
        Ok(Command::ConfigSet) => (
            Some((address, Response::ConfigSet(&ConfigSetRes))),
            processed,
        ),
        Ok(Command::OutputGet) => {
            let Ok(message) = OutputGetRes::try_ref_from_bytes(payload) else {
                return (None, processed);
            };
            (Some((address, Response::OutputGet(message))), processed)
        }
        Ok(Command::OutputSet) => (
            Some((address, Response::OutputSet(&OutputSetRes))),
            processed,
        ),
        Ok(Command::InputGet) => {
            let Ok(message) = InputGetRes::try_ref_from_bytes(payload) else {
                return (None, processed);
            };
            (Some((address, Response::InputGet(message))), processed)
        }
        Ok(Command::InputGetFull) => {
            let Ok(message) = InputGetFullRes::try_ref_from_bytes(payload) else {
                return (None, processed);
            };
            (Some((address, Response::InputGetFull(message))), processed)
        }
        Ok(Command::InputSetCalibrations) => (
            Some((
                address,
                Response::InputSetCalibrations(&InputSetCalibrationsRes),
            )),
            processed,
        ),
        Ok(Command::InputGetCalibrations) => {
            let Ok(message) = InputGetCalibrationsRes::try_ref_from_bytes(payload) else {
                return (None, processed);
            };
            (
                Some((address, Response::InputGetCalibrations(message))),
                processed,
            )
        }
        Ok(Command::InputSetThresholds) => (
            Some((
                address,
                Response::InputSetThresholds(&InputSetThresholdsRes),
            )),
            processed,
        ),
        Ok(Command::InputGetThresholds) => {
            let Ok(message) = InputGetThresholdsRes::try_ref_from_bytes(payload) else {
                return (None, processed);
            };
            (
                Some((address, Response::InputGetThresholds(message))),
                processed,
            )
        }
        Ok(Command::InputGetThresholdTimes) => {
            let Ok(message) = InputGetThresholdTimesRes::try_ref_from_bytes(payload) else {
                return (None, processed);
            };
            (
                Some((address, Response::InputGetThresholdTimes(message))),
                processed,
            )
        }
        Ok(Command::InputGetThresholdStates) => {
            let Ok(message) = InputGetThresholdStatesRes::try_ref_from_bytes(payload) else {
                return (None, processed);
            };
            (
                Some((address, Response::InputGetThresholdStates(message))),
                processed,
            )
        }
        Ok(Command::Reboot) => (Some((address, Response::Reboot(&RebootRes))), processed),
    }
}

/// Parses the next message with the given address from the given byte slice and returns the payload
/// as a [`Request`] along with the number of bytes processed. Skips invalid message headers,
/// messages with invalid checksums and messages with a different address.
pub fn slave_next<'a>(buffer: &'a [u8], address: u16) -> (Option<Request<'a>>, usize) {
    let (maybe_message, processed) = next_message(buffer);
    let Some((header, payload)) = maybe_message else {
        return (None, processed);
    };
    if address != header.address.into() {
        return (None, processed);
    }
    match Command::try_from(u16::from(header.command)) {
        Err(_) => (None, processed),
        Ok(Command::Check) => (Some(Request::Check(&CheckReq)), processed),
        Ok(Command::InfoGet) => (Some(Request::InfoGet(&InfoGetReq)), processed),
        Ok(Command::ConfigGet) => (Some(Request::ConfigGet(&ConfigGetReq)), processed),
        Ok(Command::ConfigSet) => {
            let Ok(message) = ConfigSetReq::try_ref_from_bytes(payload) else {
                return (None, processed);
            };
            (Some(Request::ConfigSet(message)), processed)
        }
        Ok(Command::OutputGet) => (Some(Request::OutputGet(&OutputGetReq)), processed),
        Ok(Command::OutputSet) => {
            let Ok(message) = OutputSetReq::try_ref_from_bytes(payload) else {
                return (None, processed);
            };
            (Some(Request::OutputSet(message)), processed)
        }
        Ok(Command::InputGet) => (Some(Request::InputGet(&InputGetReq)), processed),
        Ok(Command::InputGetFull) => (Some(Request::InputGetFull(&InputGetFullReq)), processed),
        Ok(Command::InputSetCalibrations) => {
            let Ok(message) = InputSetCalibrationsReq::try_ref_from_bytes(payload) else {
                return (None, processed);
            };
            (Some(Request::InputSetCalibrations(message)), processed)
        }
        Ok(Command::InputGetCalibrations) => (
            Some(Request::InputGetCalibrations(&InputGetCalibrationsReq)),
            processed,
        ),
        Ok(Command::InputSetThresholds) => {
            let Ok(message) = InputSetThresholdsReq::try_ref_from_bytes(payload) else {
                return (None, processed);
            };
            (Some(Request::InputSetThresholds(message)), processed)
        }
        Ok(Command::InputGetThresholds) => (
            Some(Request::InputGetThresholds(&InputGetThresholdsReq)),
            processed,
        ),
        Ok(Command::InputGetThresholdTimes) => (
            Some(Request::InputGetThresholdTimes(&InputGetThresholdTimesReq)),
            processed,
        ),
        Ok(Command::InputGetThresholdStates) => (
            Some(Request::InputGetThresholdStates(
                &InputGetThresholdStatesReq,
            )),
            processed,
        ),
        Ok(Command::Reboot) => (Some(Request::Reboot(&RebootReq)), processed),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_parsing() {
        let payload = OutputSetReq::default();
        let message = Message::new_request(0x1234, Command::OutputSet, payload);
        let bytes = message.as_bytes();
        let checksum = CHECKSUM.checksum(&bytes[..bytes.len() - size_of::<Footer>()]);
        assert_eq!(
            Footer::try_ref_from_bytes(&bytes[bytes.len() - size_of::<Footer>()..])
                .unwrap()
                .checksum
                .get(),
            checksum
        );
        let (maybe_message, processed) = next_message(bytes);
        assert_eq!(processed, bytes.len());
        let (header, payload_bytes) = maybe_message.expect("Failed to parse payload");
        assert_eq!(header.address.get(), 0x1234);
        assert_eq!(header.command.get(), u16::from(Command::OutputSet));
        let parsed_payload =
            OutputSetReq::try_ref_from_bytes(payload_bytes).expect("Failed to parse payload");
        assert_eq!(*parsed_payload, payload);
    }

    #[test]
    fn test_master_next() {
        let payload = InfoGetRes {
            info: *b"Test Device                     ",
            firmware_version_major: 1,
            firmware_version_minor: 0,
            firmware_version_patch: 2.into(),
            uptime: 123456.into(),
        };
        let message = Message::new_response(0x1234, Command::InfoGet, payload);
        let bytes = message.as_bytes();
        let (maybe_request, processed) = master_next(bytes);
        assert_eq!(processed, bytes.len());
        let (address, response) = maybe_request.expect("Failed to parse message");
        assert_eq!(address, 0x1234);
        match response {
            Response::InfoGet(info) => {
                assert_eq!(*info, payload);
            }
            _ => panic!("Unexpected response type"),
        }
    }

    #[test]
    fn test_slave_next() {
        let payload = OutputSetReq::default();
        let message = Message::new_request(0x1234, Command::OutputSet, payload);
        let bytes = message.as_bytes();
        let (maybe_request, processed) = slave_next(bytes, 0x1234);
        assert_eq!(processed, bytes.len());
        let request = maybe_request.expect("Failed to parse message");
        match request {
            Request::OutputSet(cmd) => {
                assert_eq!(*cmd, payload);
            }
            _ => panic!("Unexpected request type"),
        }
    }
}
