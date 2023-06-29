use core::convert::TryInto;
use crate::errors::Result;
use base_x;
use crate::errors::RippleBinaryCodecError::DecodeError;
use cryptoxide::hashing;
use alloc::vec::Vec;
use alloc::string::ToString;

const CHECKSUM_LENGTH: usize = 4;
const ALPHABET: &str = "rpshnaf39wBUDNEGHJKLM4PQRST7VWXYZ2bcdeCg65jkm8oFqi1tuvAxyz";

struct Address;

trait Settings {
    const PAYLOAD_LEN: usize;
    const PREFIX: &'static [u8] = &[];

    fn prefix(&self) -> &'static [u8] {
        Self::PREFIX
    }

    fn prefix_len(&self) -> usize {
        Self::PREFIX.len()
    }

    fn payload_len(&self) -> usize {
        Self::PAYLOAD_LEN
    }
}

impl Settings for Address {
    const PAYLOAD_LEN: usize = 20;
    const PREFIX: &'static [u8] = &[0x00];
}

fn decode_with_xrp_alphabet(s: &str) -> Result<Vec<u8>> {
    Ok(base_x::decode(ALPHABET, s)?)
}

fn verify_checksum_length(bytes: &[u8]) -> Result<()> {
    let len = bytes.len();

    if len < CHECKSUM_LENGTH + 1 {
        return Err(DecodeError(format!("invalid checksum length {:?}", len)));
    }

    Ok(())
}

fn verify_prefix(prefix: &[u8], bytes: &[u8]) -> Result<()> {
    if bytes.starts_with(prefix) {
        return Ok(());
    }

    Err(DecodeError("verify prefix failed".to_string()))
}

fn get_checked_bytes(mut bytes_with_checksum: Vec<u8>) -> Result<Vec<u8>> {
    verify_checksum_length(&bytes_with_checksum)?;

    //Split bytes with checksum to checked bytes and checksum
    let checksum = bytes_with_checksum.split_off(bytes_with_checksum.len() - CHECKSUM_LENGTH);
    let bytes = bytes_with_checksum;

    verify_checksum(&bytes, &checksum)?;

    Ok(bytes)
}

fn verify_payload_len(bytes: &[u8], prefix_len: usize, expected_len: usize) -> Result<()> {
    if bytes[prefix_len..bytes.len() - CHECKSUM_LENGTH].len() == expected_len {
        return Ok(());
    }

    Err(DecodeError("verify payload length failed".to_string()))
}

fn get_payload(bytes: Vec<u8>, settings: impl Settings) -> Result<Vec<u8>> {
    verify_payload_len(&bytes, settings.prefix_len(), settings.payload_len())?;
    verify_prefix(settings.prefix(), &bytes)?;
    let checked_bytes = get_checked_bytes(bytes)?;
    Ok(checked_bytes[settings.prefix_len()..].to_vec())
}

fn calc_checksum(bytes: &[u8]) -> Vec<u8> {
    sha256_digest(&sha256_digest(bytes))[..CHECKSUM_LENGTH].to_vec()
}

fn sha256_digest(data: &[u8]) -> Vec<u8> {
    hashing::sha256(&data).to_vec()
}

fn verify_checksum(input: &[u8], checksum: &[u8]) -> Result<()> {
    if calc_checksum(input) == checksum {
        Ok(())
    } else {
        Err(DecodeError("varify checksum failed".to_string()))
    }
}

pub fn decode_account_id(account_id: &str) -> Result<[u8; Address::PAYLOAD_LEN]> {
    let decoded_bytes = decode_with_xrp_alphabet(account_id)?;
    let payload = get_payload(decoded_bytes, Address)?;
    payload.try_into().map_err(|_e| DecodeError(format!("decode_account_id failed {:?}", account_id)))
}