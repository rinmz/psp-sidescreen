/// The four bytes the host sends first on the USB link.
///
/// PSPdisp took them from PSPLINK's `usbhostfs` driver, which the PSP application loads to
/// talk USB.
pub const HOSTFS_MAGIC: [u8; 4] = [0x12, 0x08, 0x2F, 0x78];

/// Length of the PSP's answer to the first write and of the host's echo of it, in bytes.
pub const HELLO_LEN: usize = 12;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn magic_matches_the_bytes_pspdisp_sends() {
        assert_eq!(HOSTFS_MAGIC, *b"\x12\x08\x2f\x78");
        assert_eq!(HELLO_LEN, 12);
    }
}
