//! Helpers to ensure ABI compatibility for use when testing

pub use crate::UInt;
use crate::{
    ed25519::consts::{PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH, SIGNATURE_LENGTH},
    ed25519::{self, PublicKey, Scalar, SecretKey, Signature},
    ffi,
};

/// Driver ABI for dalek or donna impls
#[allow(unused)]
pub struct Driver {
    pub ed25519_publickey: unsafe extern "C" fn(*mut SecretKey, *mut PublicKey),
    pub ed25519_sign:
        unsafe extern "C" fn(*const u8, UInt, *mut SecretKey, *mut PublicKey, *mut Signature),
    pub ed25519_sign_open:
        unsafe extern "C" fn(*const u8, UInt, *mut PublicKey, *mut Signature) -> i32,
    pub ed25519_sign_open_batch: unsafe extern "C" fn(
        *mut *const u8,
        *mut UInt,
        *mut *const u8,
        *mut *const u8,
        UInt,
        *mut i32,
    ) -> i32,

    pub curved25519_scalarmult_basepoint: unsafe extern "C" fn(*mut Scalar, *mut Scalar),
    pub curve25519_scalarmult: unsafe extern "C" fn(*mut Scalar, *mut SecretKey, *mut Scalar),

    pub ed25519_publickey_ext:
        unsafe extern "C" fn(sk: *mut SecretKey, sk_ext: *mut SecretKey, pk: *mut PublicKey),

    pub ed25519_sign_ext: unsafe extern "C" fn(
        m: *const u8,
        mlen: UInt,
        sk: *mut SecretKey,
        sk_ext: *mut SecretKey,
        pk: *mut PublicKey,
        sig: *mut Signature,
    ),
}

/// Donna driver implementation (via FFI)
#[cfg(feature = "build_donna")]
pub const DONNA: Driver = Driver {
    ed25519_publickey: ffi::ed25519_publickey,
    ed25519_sign_open: ffi::ed25519_sign_open,
    ed25519_sign: ffi::ed25519_sign,
    ed25519_sign_open_batch: ffi::ed25519_sign_open_batch,
    curved25519_scalarmult_basepoint: ffi::curved25519_scalarmult_basepoint,
    curve25519_scalarmult: ffi::curve25519_scalarmult,
    ed25519_publickey_ext: ffi::ed25519_publickey_ext,
    ed25519_sign_ext: ffi::ed25519_sign_ext,
};

/// Dalek driver implementation (native rust)
pub const DALEK: Driver = Driver {
    ed25519_publickey: ed25519::dalek_ed25519_publickey,
    ed25519_sign_open: ed25519::dalek_ed25519_sign_open,
    ed25519_sign: ed25519::dalek_ed25519_sign,
    ed25519_sign_open_batch: ed25519::dalek_ed25519_sign_open_batch,
    curved25519_scalarmult_basepoint: ed25519::dalek_curved25519_scalarmult_basepoint,
    curve25519_scalarmult: ed25519::dalek_curve25519_scalarmult,
    ed25519_publickey_ext: ed25519::dalek_ed25519_publickey_ext,
    ed25519_sign_ext: ed25519::dalek_ed25519_sign_ext,
};

pub struct Batch<const N: usize, const M: usize = 128> {
    pub secret_keys: [SecretKey; N],
    pub public_keys: [PublicKey; N],
    pub messages: [[u8; M]; N],
    pub lengths: [UInt; N],
    pub signatures: [Signature; N],
}

impl<const N: usize, const M: usize> Batch<N, M> {
    /// Generate a collection for batch verification
    pub fn new(signer: &Driver) -> Self {
        let mut secret_keys = [[0u8; SECRET_KEY_LENGTH]; N];
        let mut public_keys = [[0u8; PUBLIC_KEY_LENGTH]; N];

        let mut messages = [[0u8; M]; N];
        let mut signatures = [[0u8; SIGNATURE_LENGTH]; N];

        for i in 0..N {
            // Generate random secret key
            getrandom::getrandom(&mut secret_keys[i]).unwrap();

            // Generate matching public key
            unsafe {
                (signer.ed25519_publickey)(
                    secret_keys[i].as_mut_ptr() as *mut SecretKey,
                    public_keys[i].as_mut_ptr() as *mut PublicKey,
                )
            };

            // Generate message
            getrandom::getrandom(&mut messages[i]).unwrap();

            // Generate signature
            unsafe {
                (signer.ed25519_sign)(
                    messages[i].as_mut_ptr(),
                    M as UInt,
                    secret_keys[i].as_mut_ptr() as *mut SecretKey,
                    public_keys[i].as_mut_ptr() as *mut PublicKey,
                    signatures[i].as_mut_ptr() as *mut Signature,
                )
            };
        }

        Self {
            secret_keys,
            public_keys,
            messages,
            lengths: [M as UInt; N],
            signatures,
        }
    }
}
