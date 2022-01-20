//! Conversion between Minecraft versions and protocol version numbers.

macro_rules! protocol_versions {
    (
        $(
            $version:expr => $protocol_version:expr,
        )+
    ) => {
        const fn get_protocol_version_internal(version_string: &str) -> Option<i32> {
            match version_string.as_bytes() {
                $(
                $version => Some($protocol_version),
                )+

                _ => None
            }
        }
    };
}

protocol_versions! {
    b"1.18.1" => 757,
    b"1.18"   => 757,
    b"1.17.1" => 756,
    b"1.17"   => 755,
    b"1.16.5" => 754,
    b"1.16.4" => 754,
    b"1.16.3" => 753,
    b"1.16.2" => 751,
    b"1.16.1" => 736,
    b"1.16"   => 735,
    b"1.15.2" => 578,
    b"1.15.1" => 575,
    b"1.15"   => 573,
    b"1.14.4" => 498,
    b"1.14.3" => 490,
    b"1.14.2" => 485,
    b"1.14.1" => 480,
    b"1.14"   => 477,
    b"1.13.2" => 404,
    b"1.13.1" => 401,
    b"1.13"   => 393,
    b"1.12.2" => 340,
    b"1.12.1" => 338,
    b"1.12"   => 335,
    b"1.11.2" => 316,
    b"1.11.1" => 316,
    b"1.11"   => 315,
    b"1.10.2" => 210,
    b"1.10.1" => 210,
    b"1.10"   => 210,
    b"1.9.4"  => 110,
    b"1.9.3"  => 110,
    b"1.9.2"  => 109,
    b"1.9.1"  => 108,
    b"1.9"    => 107,
    b"1.8.9"  => 47,
    b"1.8.8"  => 47,
    b"1.8.7"  => 47,
    b"1.8.6"  => 47,
    b"1.8.5"  => 47,
    b"1.8.4"  => 47,
    b"1.8.3"  => 47,
    b"1.8.2"  => 47,
    b"1.8.1"  => 47,
    b"1.8"    => 47,
    b"1.7.10" => 5,
    b"1.7.9"  => 5,
    b"1.7.8"  => 5,
    b"1.7.7"  => 5,
    b"1.7.6"  => 5,
    b"1.7.5"  => 4,
    b"1.7.4"  => 4,
    b"1.7.2"  => 4,
}

pub const fn get_protocol_version(version_string: &str) -> Option<i32> {
    get_protocol_version_internal(version_string)
}

#[cfg(test)]
#[test]
fn test() {
    assert_eq!(get_protocol_version("1.14.4"), Some(498));
    assert_eq!(get_protocol_version("foo"), None);
}
