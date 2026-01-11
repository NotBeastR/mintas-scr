use crate::errors::{MintasError, MintasResult, SourceLocation};
use crate::evaluator::Value;
pub struct HashModule;
impl HashModule {
    pub fn call_function(name: &str, args: &[Value]) -> MintasResult<Value> {
        match name {
            "md5" => Self::md5(args),
            "sha1" => Self::sha1(args),
            "sha256" => Self::sha256(args),
            "sha512" => Self::sha512(args),
            "crc32" => Self::crc32(args),
            "fnv" | "fnv1a" => Self::fnv1a(args),
            _ => Err(MintasError::RuntimeError {
                message: format!("Unknown hash function: {}", name),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    fn md5(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(s)) = args.get(0) {
            Ok(Value::String(Self::compute_md5(s.as_bytes())))
        } else {
            Ok(Value::String(String::new()))
        }
    }
    fn sha1(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(s)) = args.get(0) {
            Ok(Value::String(Self::compute_sha1(s.as_bytes())))
        } else {
            Ok(Value::String(String::new()))
        }
    }
    fn sha256(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(s)) = args.get(0) {
            Ok(Value::String(Self::compute_sha256(s.as_bytes())))
        } else {
            Ok(Value::String(String::new()))
        }
    }
    fn sha512(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(s)) = args.get(0) {
            let first = Self::compute_sha256(s.as_bytes());
            let second = Self::compute_sha256(first.as_bytes());
            Ok(Value::String(format!("{}{}", first, second)))
        } else {
            Ok(Value::String(String::new()))
        }
    }
    fn crc32(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(s)) = args.get(0) {
            let crc = Self::compute_crc32(s.as_bytes());
            Ok(Value::Number(crc as f64))
        } else {
            Ok(Value::Number(0.0))
        }
    }
    fn fnv1a(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(s)) = args.get(0) {
            let mut hash: u64 = 0xcbf29ce484222325;
            for byte in s.bytes() {
                hash ^= byte as u64;
                hash = hash.wrapping_mul(0x100000001b3);
            }
            Ok(Value::String(format!("{:016x}", hash)))
        } else {
            Ok(Value::String(String::new()))
        }
    }
    fn compute_md5(data: &[u8]) -> String {
        let mut state = [0x67452301u32, 0xefcdab89, 0x98badcfe, 0x10325476];
        let s: [u32; 64] = [
            7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22,
            5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20,
            4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23,
            6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
        ];
        let k: [u32; 64] = [
            0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a, 0xa8304613, 0xfd469501,
            0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be, 0x6b901122, 0xfd987193, 0xa679438e, 0x49b40821,
            0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa, 0xd62f105d, 0x02441453, 0xd8a1e681, 0xe7d3fbc8,
            0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed, 0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a,
            0xfffa3942, 0x8771f681, 0x6d9d6122, 0xfde5380c, 0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70,
            0x289b7ec6, 0xeaa127fa, 0xd4ef3085, 0x04881d05, 0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665,
            0xf4292244, 0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
            0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1, 0xf7537e82, 0xbd3af235, 0x2ad7d2bb, 0xeb86d391,
        ];
        let orig_len = data.len();
        let mut msg = data.to_vec();
        msg.push(0x80);
        while (msg.len() % 64) != 56 { msg.push(0); }
        msg.extend_from_slice(&((orig_len * 8) as u64).to_le_bytes());
        for chunk in msg.chunks(64) {
            let mut m = [0u32; 16];
            for (i, c) in chunk.chunks(4).enumerate() {
                m[i] = u32::from_le_bytes([c[0], c[1], c[2], c[3]]);
            }
            let (mut a, mut b, mut c, mut d) = (state[0], state[1], state[2], state[3]);
            for i in 0..64 {
                let (f, g) = match i {
                    0..=15 => ((b & c) | ((!b) & d), i),
                    16..=31 => ((d & b) | ((!d) & c), (5 * i + 1) % 16),
                    32..=47 => (b ^ c ^ d, (3 * i + 5) % 16),
                    _ => (c ^ (b | (!d)), (7 * i) % 16),
                };
                let temp = d;
                d = c;
                c = b;
                b = b.wrapping_add(
                    (a.wrapping_add(f).wrapping_add(k[i]).wrapping_add(m[g]))
                        .rotate_left(s[i])
                );
                a = temp;
            }
            state[0] = state[0].wrapping_add(a);
            state[1] = state[1].wrapping_add(b);
            state[2] = state[2].wrapping_add(c);
            state[3] = state[3].wrapping_add(d);
        }
        state.iter().flat_map(|x| x.to_le_bytes()).map(|b| format!("{:02x}", b)).collect()
    }
    fn compute_sha1(data: &[u8]) -> String {
        let mut h = [0x67452301u32, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0];
        let orig_len = data.len();
        let mut msg = data.to_vec();
        msg.push(0x80);
        while (msg.len() % 64) != 56 { msg.push(0); }
        msg.extend_from_slice(&((orig_len * 8) as u64).to_be_bytes());
        for chunk in msg.chunks(64) {
            let mut w = [0u32; 80];
            for (i, c) in chunk.chunks(4).enumerate() {
                w[i] = u32::from_be_bytes([c[0], c[1], c[2], c[3]]);
            }
            for i in 16..80 {
                w[i] = (w[i-3] ^ w[i-8] ^ w[i-14] ^ w[i-16]).rotate_left(1);
            }
            let (mut a, mut b, mut c, mut d, mut e) = (h[0], h[1], h[2], h[3], h[4]);
            for i in 0..80 {
                let (f, k) = match i {
                    0..=19 => ((b & c) | ((!b) & d), 0x5A827999u32),
                    20..=39 => (b ^ c ^ d, 0x6ED9EBA1),
                    40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1BBCDC),
                    _ => (b ^ c ^ d, 0xCA62C1D6),
                };
                let temp = a.rotate_left(5).wrapping_add(f).wrapping_add(e).wrapping_add(k).wrapping_add(w[i]);
                e = d; d = c; c = b.rotate_left(30); b = a; a = temp;
            }
            h[0] = h[0].wrapping_add(a);
            h[1] = h[1].wrapping_add(b);
            h[2] = h[2].wrapping_add(c);
            h[3] = h[3].wrapping_add(d);
            h[4] = h[4].wrapping_add(e);
        }
        h.iter().flat_map(|x| x.to_be_bytes()).map(|b| format!("{:02x}", b)).collect()
    }
    fn compute_sha256(data: &[u8]) -> String {
        let mut h = [
            0x6a09e667u32, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
            0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
        ];
        let k: [u32; 64] = [
            0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
            0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
            0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
            0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
            0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
            0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
            0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
            0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
        ];
        let orig_len = data.len();
        let mut msg = data.to_vec();
        msg.push(0x80);
        while (msg.len() % 64) != 56 { msg.push(0); }
        msg.extend_from_slice(&((orig_len * 8) as u64).to_be_bytes());
        for chunk in msg.chunks(64) {
            let mut w = [0u32; 64];
            for (i, c) in chunk.chunks(4).enumerate() {
                w[i] = u32::from_be_bytes([c[0], c[1], c[2], c[3]]);
            }
            for i in 16..64 {
                let s0 = w[i-15].rotate_right(7) ^ w[i-15].rotate_right(18) ^ (w[i-15] >> 3);
                let s1 = w[i-2].rotate_right(17) ^ w[i-2].rotate_right(19) ^ (w[i-2] >> 10);
                w[i] = w[i-16].wrapping_add(s0).wrapping_add(w[i-7]).wrapping_add(s1);
            }
            let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) = 
                (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);
            for i in 0..64 {
                let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
                let ch = (e & f) ^ ((!e) & g);
                let t1 = hh.wrapping_add(s1).wrapping_add(ch).wrapping_add(k[i]).wrapping_add(w[i]);
                let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
                let maj = (a & b) ^ (a & c) ^ (b & c);
                let t2 = s0.wrapping_add(maj);
                hh = g; g = f; f = e; e = d.wrapping_add(t1);
                d = c; c = b; b = a; a = t1.wrapping_add(t2);
            }
            h[0] = h[0].wrapping_add(a); h[1] = h[1].wrapping_add(b);
            h[2] = h[2].wrapping_add(c); h[3] = h[3].wrapping_add(d);
            h[4] = h[4].wrapping_add(e); h[5] = h[5].wrapping_add(f);
            h[6] = h[6].wrapping_add(g); h[7] = h[7].wrapping_add(hh);
        }
        h.iter().flat_map(|x| x.to_be_bytes()).map(|b| format!("{:02x}", b)).collect()
    }
    fn compute_crc32(data: &[u8]) -> u32 {
        let mut crc = 0xFFFFFFFFu32;
        for byte in data {
            crc ^= *byte as u32;
            for _ in 0..8 {
                crc = if crc & 1 != 0 { (crc >> 1) ^ 0xEDB88320 } else { crc >> 1 };
            }
        }
        !crc
    }
}