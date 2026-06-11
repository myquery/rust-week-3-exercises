use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::Deref;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct CompactSize {
    pub value: u64,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BitcoinError {
    InsufficientBytes,
    InvalidFormat,
}

impl CompactSize {
    pub fn new(value: u64) -> Self {
        Self { value }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO: Encode according to Bitcoin's CompactSize format:
        // [0x00–0xFC] => 1 byte
        // [0xFDxxxx] => 0xFD + u16 (2 bytes)
        // [0xFExxxxxxxx] => 0xFE + u32 (4 bytes)
        // [0xFFxxxxxxxxxxxxxxxx] => 0xFF + u64 (8 bytes)
        match self.value {
            0..=0xFC => vec![self.value as u8],
            0xFD..=0xFFFF => {
                let mut bytes = vec![0xFD];
                bytes.extend_from_slice(&(self.value as u16).to_le_bytes());
                bytes
            }
            0x10000..=0xFFFFFFFF => {
                let mut bytes = vec![0xFE];
                bytes.extend_from_slice(&(self.value as u32).to_le_bytes());
                bytes
            }
            _ => {
                let mut bytes = vec![0xFF];
                bytes.extend_from_slice(&self.value.to_le_bytes());
                bytes
            }
        }

    }

    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), BitcoinError> {
        // TODO: Decode CompactSize, returning value and number of bytes consumed.
        // First check if bytes is empty.
        if bytes.is_empty() {
            return Err(BitcoinError::InsufficientBytes);
        }
        // Check that enough bytes are available based on prefix.
        let prefix = bytes[0];
        match prefix {
            0x00..=0xFC => Ok((CompactSize::new(prefix as u64), 1)),
            0xFD => {
                if bytes.len() < 3 {
                    return Err(BitcoinError::InsufficientBytes);
                }
                let value = u16::from_le_bytes([bytes[1], bytes[2]]) as u64;
                Ok((CompactSize::new(value), 3))
            }
            0xFE => {
                if bytes.len() < 5 {
                    return Err(BitcoinError::InsufficientBytes);
                }
                let value = u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]) as u64;
                Ok((CompactSize::new(value), 5))
            }
            0xFF => {
                if bytes.len() < 9 {
                    return Err(BitcoinError::InsufficientBytes);
                }
                let value = u64::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8]]);
                Ok((CompactSize::new(value), 9))
            }

        }

    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Txid(pub [u8; 32]);

impl Serialize for Txid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // TODO: Serialize as a hex-encoded string (32 bytes => 64 hex characters)
        serializer.serialize_str(&hex::encode(self.0))

    }
}

impl<'de> Deserialize<'de> for Txid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // TODO: Parse hex string into 32-byte array
        // Use `hex::decode`, validate length = 32
        let s = String::deserialize(deserializer)?;
        let bytes = hex::decode(&s).map_err(serde::de::Error::custom)?;
        if bytes.len() != 32 {
            return Err(serde::de::Error::custom("Invalid Txid length"));
        }

        let mut txid_bytes = [0u8; 32];
        txid_bytes.copy_from_slice(&bytes);
        Ok(Txid(txid_bytes))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct OutPoint {
    pub txid: Txid,
    pub vout: u32,
}

impl OutPoint {
    pub fn new(txid: [u8; 32], vout: u32) -> Self {
        // TODO: Create an OutPoint from raw txid bytes and output index
        let outpoint = OutPoint {
            txid: Txid(txid),
            vout,
        };
        outpoint
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO: Serialize as: txid (32 bytes) + vout (4 bytes, little-endian)
        let mut bytes = Vec::with_capacity(36);
        bytes.extend_from_slice(&self.txid.0);
        bytes.extend_from_slice(&self.vout.to_le_bytes());
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), BitcoinError> {
        // TODO: Deserialize 36 bytes: txid[0..32], vout[32..36]
        // Return error if insufficient bytes
        if bytes.len() < 36 {
            return Err(BitcoinError::InsufficientBytes);
        }
        let mut txid_bytes = [0u8; 32];
        txid_bytes.copy_from_slice(&bytes[0..32]);
        let vout = u32::from_le_bytes([bytes[32], bytes[33], bytes[34], bytes[35]]);
        Ok((OutPoint { txid: Txid(txid_bytes), vout }, 36))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Script {
    pub bytes: Vec<u8>,
}

impl Script {
    pub fn new(bytes: Vec<u8>) -> Self {
        // TODO: Simple constructor
        Self { bytes }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO: Prefix with CompactSize (length), then raw bytes
        let length_prefix = CompactSize::new(self.bytes.len() as u64).to_bytes();
        [length_prefix, self.bytes.clone()].concat()
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), BitcoinError> {
        // TODO: Parse CompactSize prefix, then read that many bytes
        // Return error if not enough bytes
        if bytes.is_empty() {
            return Err(BitcoinError::InsufficientBytes);
        }
        let (length_prefix, prefix_size) = CompactSize::from_bytes(bytes)?;
        let total_size = prefix_size + (length_prefix.value as usize);
        if bytes.len() < total_size {
            return Err(BitcoinError::InsufficientBytes);
        }
        let script_bytes = &bytes[prefix_size..total_size];
        Ok((Self { bytes: script_bytes.to_vec() }, total_size))
    }
}

impl Deref for Script {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        // TODO: Allow &Script to be used as &[u8]
        &self.bytes

    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct TransactionInput {
    pub previous_output: OutPoint,
    pub script_sig: Script,
    pub sequence: u32,
}

impl TransactionInput {
    pub fn new(previous_output: OutPoint, script_sig: Script, sequence: u32) -> Self {
        // TODO: Basic constructor
        Self {
            previous_output,
            script_sig,
            sequence,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO: Serialize: OutPoint + Script (with CompactSize) + sequence (4 bytes LE)
        let mut bytes = self.previous_output.to_bytes();
        bytes.extend(self.script_sig.to_bytes());
        bytes.extend_from_slice(&self.sequence.to_le_bytes());
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), BitcoinError> {
        // TODO: Deserialize in order:
        // - OutPoint (36 bytes)
        // - Script (with CompactSize)
        // - Sequence (4 bytes)
        let (previous_output, outpoint_size) = OutPoint::from_bytes(bytes)?;
        let script_start = outpoint_size;
        let (script_sig, script_size) = Script::from_bytes(&bytes[script_start..])?;
        let sequence_start = script_start + script_size;
        if bytes.len() < sequence_start + 4 {
            return Err(BitcoinError::InsufficientBytes);
        }
        let sequence = u32::from_le_bytes([
            bytes[sequence_start],
            bytes[sequence_start + 1],
            bytes[sequence_start + 2],
            bytes[sequence_start + 3],
        ]);
        Ok((Self { previous_output, script_sig, sequence }, sequence_start + 4))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct BitcoinTransaction {
    pub version: u32,
    pub inputs: Vec<TransactionInput>,
    pub lock_time: u32,
}

impl BitcoinTransaction {
    pub fn new(version: u32, inputs: Vec<TransactionInput>, lock_time: u32) -> Self {
        // TODO: Construct a transaction from parts
        Self {
            version,
            inputs,
            lock_time,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO: Format:
        // - version (4 bytes LE)
        // - CompactSize (number of inputs)
        // - each input serialized
        // - lock_time (4 bytes LE)
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.version.to_le_bytes());
        let input_count = CompactSize::new(self.inputs.len() as u64);
        bytes.extend(input_count.to_bytes());
        for input in &self.inputs {
            bytes.extend(input.to_bytes());
        }
        bytes.extend_from_slice(&self.lock_time.to_le_bytes());
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), BitcoinError> {
        // TODO: Read version, CompactSize for input count
        // Parse inputs one by one
        // Read final 4 bytes for lock_time
        if bytes.len() < 8 {
            return Err(BitcoinError::InsufficientBytes);
        }
        let version = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let (input_count, input_count_size) = CompactSize::from_bytes(&bytes[4..])?;
        let mut inputs = Vec::new();
        let mut offset = 4 + input_count_size;
        for _ in 0..input_count.value {
            let (input, input_size) = TransactionInput::from_bytes(&bytes[offset..])?;
            inputs.push(input);
            offset += input_size;
        }
        let lock_time = u32::from_le_bytes([bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]]);
        Ok((Self { version, inputs, lock_time }, offset + 4))   
    }
}

impl fmt::Display for BitcoinTransaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Version: {}", self.version)?;
        for (i, input) in self.inputs.iter().enumerate() {
            writeln!(f, "Input {}:", i)?;
            writeln!(f, "  Previous Output Txid: {}", hex::encode(input.previous_output.txid.0))?;
            writeln!(f, "  Previous Output Vout: {}", input.previous_output.vout)?;
            writeln!(f, "  ScriptSig: {}", hex::encode(&input.script_sig.bytes))?;
            writeln!(f, "  Sequence: {}", input.sequence)?;
        }
        write!(f, "Lock Time: {}", self.lock_time)
    }
}
