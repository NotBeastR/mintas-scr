use crate::bytecode::BytecodeProgram;
use crate::errors::{MintasError, MintasResult, SourceLocation};
use aes::Aes256;
use cbc::{Decryptor, Encryptor};
use cbc::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use rand::Rng;
use std::fs;
use std::io::{Read, Write};
use sha2::{Sha256, Digest};

type Aes256CbcEnc = Encryptor<Aes256>;
type Aes256CbcDec = Decryptor<Aes256>;

const MAGIC: &[u8; 8] = b"MINTAS\0\0";
const VERSION: u32 = 1;
const DEFAULT_AES_KEY: &[u8; 32] = b"MINTAS_ENCRYPTION_KEY_V1_2026!!!"; // 32 bytes for fallback

/// Derby a 32-byte key from a user string using SHA-256
fn derive_key(secret: Option<&str>) -> [u8; 32] {
    if let Some(s) = secret {
        let mut hasher = Sha256::new();
        hasher.update(s.as_bytes());
        let result = hasher.finalize();
        let mut key = [0u8; 32];
        key.copy_from_slice(&result);
        key
    } else {
        let mut key = [0u8; 32];
        key.copy_from_slice(DEFAULT_AES_KEY);
        key
    }
}

/// Serialize and encrypt bytecode to .ms file
pub fn save_encrypted_bytecode(program: &BytecodeProgram, path: &str, secret: Option<&str>) -> MintasResult<()> {
    // Serialize bytecode to JSON (or use bincode for binary)
    let json = serde_json::to_string(program)
        .map_err(|e| MintasError::RuntimeError {
            message: format!("Failed to serialize bytecode: {}", e),
            location: SourceLocation::new(0, 0),
        })?;
    
    let plaintext = json.as_bytes();
    
    // Generate random IV (16 bytes for AES)
    let mut rng = rand::thread_rng();
    let iv: [u8; 16] = rng.gen();
    
    // Pad plaintext to multiple of 16 bytes (AES block size)
    let mut padded = plaintext.to_vec();
    let padding_len = 16 - (padded.len() % 16);
    padded.extend(vec![padding_len as u8; padding_len]);
    
    // Derive key
    let key = derive_key(secret);

    // Encrypt
    let cipher = Aes256CbcEnc::new(&key.into(), &iv.into());
    let ciphertext = cipher.encrypt_padded_mut::<cbc::cipher::block_padding::Pkcs7>(&mut padded, plaintext.len())
        .map_err(|e| MintasError::RuntimeError {
            message: format!("Encryption failed: {}", e),
            location: SourceLocation::new(0, 0),
        })?;
    
    // Write .ms file
    let mut file = fs::File::create(path)
        .map_err(|e| MintasError::RuntimeError {
            message: format!("Failed to create file: {}", e),
            location: SourceLocation::new(0, 0),
        })?;
    
    // Header
    file.write_all(MAGIC)?;
    file.write_all(&VERSION.to_le_bytes())?;
    file.write_all(&[0, 0, 0, 0])?; // Flags (reserved)
    
    // IV and ciphertext
    file.write_all(&iv)?;
    file.write_all(ciphertext)?;
    
    Ok(())
}

/// Load and decrypt .ms file
pub fn load_encrypted_bytecode(path: &str, secret: Option<&str>) -> MintasResult<BytecodeProgram> {
    let mut file = fs::File::open(path)
        .map_err(|e| MintasError::RuntimeError {
            message: format!("Failed to open file: {}", e),
            location: SourceLocation::new(0, 0),
        })?;
    
    // Read header
    let mut magic = [0u8; 8];
    file.read_exact(&mut magic)?;
    if &magic != MAGIC {
        return Err(MintasError::RuntimeError {
            message: "Invalid .ms file: bad magic number".to_string(),
            location: SourceLocation::new(0, 0),
        });
    }
    
    let mut version_bytes = [0u8; 4];
    file.read_exact(&mut version_bytes)?;
    let version = u32::from_le_bytes(version_bytes);
    if version != VERSION {
        return Err(MintasError::RuntimeError {
            message: format!("Unsupported .ms version: {}", version),
            location: SourceLocation::new(0, 0),
        });
    }
    
    let mut _flags = [0u8; 4];
    file.read_exact(&mut _flags)?;
    
    // Read IV
    let mut iv = [0u8; 16];
    file.read_exact(&mut iv)?;
    
    // Read ciphertext
    let mut ciphertext = Vec::new();
    file.read_to_end(&mut ciphertext)?;
    
    // Derive key
    let key = derive_key(secret);

    // Decrypt
    let cipher = Aes256CbcDec::new(&key.into(), &iv.into());
    let plaintext = cipher.decrypt_padded_mut::<cbc::cipher::block_padding::Pkcs7>(&mut ciphertext)
        .map_err(|e| MintasError::RuntimeError {
            message: format!("Decryption failed: (Invalid Key?) {}", e),
            location: SourceLocation::new(0, 0),
        })?;
    
    // Deserialize
    let json = String::from_utf8(plaintext.to_vec())
        .map_err(|e| MintasError::RuntimeError {
            message: format!("Invalid UTF-8 in decrypted data: {}", e),
            location: SourceLocation::new(0, 0),
        })?;
    
    let program: BytecodeProgram = serde_json::from_str(&json)
        .map_err(|e| MintasError::RuntimeError {
            message: format!("Failed to deserialize bytecode: {}", e),
            location: SourceLocation::new(0, 0),
        })?;
    
    Ok(program)
}
