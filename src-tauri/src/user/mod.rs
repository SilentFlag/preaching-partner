use rand::{rng, Rng};
use serde::{Deserialize, Serialize};
// use std::borrow::Borrow;
use std::collections::HashSet;
// use std::fs::File;
// use std::io::{BufReader, BufWriter};
// use std::path::PathBuf;
use std::{cell::RefCell, collections::HashMap, str};

use openmls::prelude::{tls_codec::*, *};
use openmls_rust_crypto::RustCrypto;
use openmls_traits::OpenMlsProvider;

use openmls_basic_credential::SignatureKeyPair;

use serde::{Deserializer, Serializer};

use std::fs::File;

// from cli::backend

// from memory_storage::lib

/// A reader-writer lock
///
/// This type of lock allows a number of readers or at most one writer at any
/// point in time. The write portion of this lock typically allows modification
/// of the underlying data (exclusive access) and the read portion of this lock
/// typically allows for read-only access (shared access).
///
/// In comparison, a [`Mutex`] does not distinguish between readers or writers
/// that acquire the lock, therefore blocking any threads waiting for the lock to
/// become available. An `RwLock` will allow any number of readers to acquire the
/// lock as long as a writer is not holding the lock.
///
/// The priority policy of the lock is dependent on the underlying operating
/// system's implementation, and this type does not guarantee that any
/// particular policy will be used. In particular, a writer which is waiting to
/// acquire the lock in `write` might or might not block concurrent calls to
/// `read`, e.g.:
///
/// <details><summary>Potential deadlock example</summary>
///
/// ```text
/// // Thread 1              |  // Thread 2
/// let _rg1 = lock.read();  |
///                          |  // will block
///                          |  let _wg = lock.write();
/// // may deadlock          |
/// let _rg2 = lock.read();  |
/// ```
///
/// </details>
///
/// The type parameter `T` represents the data that this lock protects. It is
/// required that `T` satisfies [`Send`] to be shared across threads and
/// [`Sync`] to allow concurrent access through readers. The RAII guards
/// returned from the locking methods implement [`Deref`] (and [`DerefMut`]
/// for the `write` methods) to allow access to the content of the lock.
///
/// # Poisoning
///
/// An `RwLock`, like [`Mutex`], will [usually] become poisoned on a panic. Note,
/// however, that an `RwLock` may only be poisoned if a panic occurs while it is
/// locked exclusively (write mode). If a panic occurs in any reader, then the
/// lock will not be poisoned.
///
/// [usually]: super::Mutex#poisoning
///
/// # Examples
///
/// ```
/// use std::sync::RwLock;
///
/// let lock = RwLock::new(5);
///
/// // many reader locks can be held at once
/// {
///     let r1 = lock.read().unwrap();
///     let r2 = lock.read().unwrap();
///     assert_eq!(*r1, 5);
///     assert_eq!(*r2, 5);
/// } // read locks are dropped at this point
///
/// // only one write lock may be held, however
/// {
///     let mut w = lock.write().unwrap();
///     *w += 1;
///     assert_eq!(*w, 6);
/// } // write lock is dropped here
/// ```
///
/// [`Mutex`]: super::Mutex
#[cfg_attr(not(test))]
pub struct RwLock<T: ?Sized> {
    /// The inner [`sys::RwLock`] that synchronizes thread access to the protected data.
    inner: sys::RwLock,
    /// A flag denoting if this `RwLock` has been poisoned.
    poison: poison::Flag,
    /// The lock-protected data.
    data: UnsafeCell<T>,
}

pub struct MemoryStorage {
    pub values: RwLock<HashMap<Vec<u8>, Vec<u8>>>,
}

// from memory_storage::persistence

impl MemoryStorage {
    pub fn save(&self, user_name: String) -> Result<(), String> {
        let ks_output_path = Self::get_file_path(&user_name);

        match File::create(ks_output_path) {
            Ok(output_file) => self.save_to_file(&output_file),
            Err(e) => Err(e.to_string()),
        }
    }
}

// from cli::serialize_any_hashmap

pub(crate) fn serialize_hashmap<'a, T, U, V, S>(v: &'a V, serializer: S) -> Result<S::Ok, S::Error>
where
    T: Serialize,
    U: Serialize,
    &'a V: IntoIterator<Item = (T, U)> + 'a,
    S: Serializer,
{
    let vec = v.into_iter().collect::<Vec<_>>();
    vec.serialize(serializer)
}

pub(crate) fn deserialize_hashmap<'de, T, U, D>(deserializer: D) -> Result<HashMap<T, U>, D::Error>
where
    T: Eq + std::hash::Hash + Deserialize<'de>,
    U: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Ok(Vec::<(T, U)>::deserialize(deserializer)?
        .into_iter()
        .collect::<HashMap<T, U>>())
}

// from cli::openmls_rust_persistent_crypto

#[derive(Default, Debug)]
pub struct OpenMlsRustPersistentCrypto {
    crypto: RustCrypto,
    storage: MemoryStorage,
}

impl OpenMlsProvider for OpenMlsRustPersistentCrypto {
    type CryptoProvider = RustCrypto;
    type RandProvider = RustCrypto;
    type StorageProvider = MemoryStorage;

    fn crypto(&self) -> &Self::CryptoProvider {
        &self.crypto
    }

    fn rand(&self) -> &Self::RandProvider {
        &self.crypto
    }

    fn storage(&self) -> &Self::StorageProvider {
        &self.storage
    }
}

impl OpenMlsRustPersistentCrypto {
    pub fn save_keystore(&self, user_name: String) -> Result<(), String> {
        self.storage.save(user_name)
    }

    pub fn load_keystore(&mut self, user_name: String) -> Result<(), String> {
        self.storage.load(user_name)
    }
}

// from cli::identity

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Identity {
    #[serde(
        serialize_with = "serialize_hashmap",
        deserialize_with = "deserialize_hashmap"
    )]
    pub(crate) kp: HashMap<Vec<u8>, KeyPackage>,
    pub(crate) credential_with_key: CredentialWithKey,
    pub(crate) signer: SignatureKeyPair,
}

impl Identity {
    pub(crate) fn new(
        ciphersuite: Ciphersuite,
        crypto: &OpenMlsRustPersistentCrypto,
        username: &[u8],
    ) -> Self {
        let credential = BasicCredential::new(username.to_vec());
        let signature_keys = SignatureKeyPair::new(ciphersuite.signature_algorithm()).unwrap();
        let credential_with_key = CredentialWithKey {
            credential: credential.into(),
            signature_key: signature_keys.to_public_vec().into(),
        };
        signature_keys.store(crypto.storage()).unwrap();

        let key_package = KeyPackage::builder()
            .build(
                ciphersuite,
                crypto,
                &signature_keys,
                credential_with_key.clone(),
            )
            .unwrap();

        Self {
            kp: HashMap::from([(
                key_package
                    .key_package()
                    .hash_ref(crypto.crypto())
                    .unwrap()
                    .as_slice()
                    .to_vec(),
                key_package.key_package().clone(),
            )]),
            credential_with_key,
            signer: signature_keys,
        }
    }

    /// Create an additional key package using the credential_with_key/signer bound to this identity
    pub fn add_key_package(
        &mut self,
        ciphersuite: Ciphersuite,
        crypto: &OpenMlsRustPersistentCrypto,
    ) -> KeyPackage {
        let key_package = KeyPackage::builder()
            .build(
                ciphersuite,
                crypto,
                &self.signer,
                self.credential_with_key.clone(),
            )
            .unwrap();

        self.kp.insert(
            key_package
                .key_package()
                .hash_ref(crypto.crypto())
                .unwrap()
                .as_slice()
                .to_vec(),
            key_package.key_package().clone(),
        );
        key_package.key_package().clone()
    }

    /// Get the plain identity as byte vector.
    pub fn identity(&self) -> &[u8] {
        self.credential_with_key.credential.serialized_content()
    }

    /// Get the plain identity as byte vector.
    pub fn identity_as_string(&self) -> String {
        std::str::from_utf8(self.credential_with_key.credential.serialized_content())
            .unwrap()
            .to_string()
    }
}

// From cli::conversation

/// A conversation is a list of messages (strings).
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Conversation {
    messages: Vec<ConversationMessage>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ConversationMessage {
    pub author: String,
    pub message: String,
}

impl Conversation {
    /// Add a message string to the conversation list.
    pub fn add(&mut self, conversation_message: ConversationMessage) {
        self.messages.push(conversation_message)
    }

    /// Get a list of messages in the conversation.
    /// The function returns the `last_n` messages.
    #[allow(dead_code)]
    pub fn get(&self, last_n: usize) -> Option<&[ConversationMessage]> {
        let num_messages = self.messages.len();
        let start = num_messages.saturating_sub(last_n);
        self.messages.get(start..num_messages)
    }
}

impl ConversationMessage {
    pub fn new(message: String, author: String) -> Self {
        Self { author, message }
    }
}

// From ds-lib::lib

#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    TlsSerialize,
    TlsDeserialize,
    TlsDeserializeBytes,
    TlsSize,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct ClientKeyPackages(pub TlsVecU32<(TlsByteVecU8, KeyPackageIn)>);

/// A core group message.
/// This is an `MLSMessage` plus the list of recipients as a vector of client
/// names.
#[derive(Debug)]
pub struct GroupMessage {
    pub msg: MlsMessageIn,
    pub recipients: TlsVecU32<TlsByteVecU32>,
}

impl GroupMessage {
    /// Create a new `GroupMessage` taking an `MlsMessageIn` and slice of
    /// recipient names.
    pub fn new(msg: MlsMessageIn, recipients: &[Vec<u8>]) -> Self {
        Self {
            msg,
            recipients: recipients
                .iter()
                .map(|r| r.clone().into())
                .collect::<Vec<TlsByteVecU32>>()
                .into(),
        }
    }
}

impl tls_codec::Size for GroupMessage {
    fn tls_serialized_len(&self) -> usize {
        self.msg.tls_serialized_len() + self.recipients.tls_serialized_len()
    }
}

impl tls_codec::Serialize for GroupMessage {
    fn tls_serialize<W: std::io::Write>(&self, writer: &mut W) -> Result<usize, tls_codec::Error> {
        let written = self.msg.tls_serialize(writer)?;
        self.recipients.tls_serialize(writer).map(|l| l + written)
    }
}

impl tls_codec::Deserialize for GroupMessage {
    fn tls_deserialize<R: std::io::Read>(bytes: &mut R) -> Result<Self, tls_codec::Error> {
        let msg = MlsMessageIn::tls_deserialize(bytes)?;
        let recipients = TlsVecU32::<TlsByteVecU32>::tls_deserialize(bytes)?;
        Ok(Self { msg, recipients })
    }
}

// from ds_lib::messages
#[derive(
    Debug, Clone, TlsSize, TlsSerialize, TlsDeserialize, PartialEq, Serialize, Deserialize,
)]
pub struct AuthToken {
    token: Vec<u8>,
}

impl Default for AuthToken {
    fn default() -> Self {
        Self::random()
    }
}

impl AuthToken {
    pub(super) fn random() -> Self {
        let token = rng().random::<[u8; 32]>().to_vec();
        Self { token }
    }
}

// From cli::user
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Contact {
    id: Vec<u8>,
}

impl Contact {
    fn username(&self) -> String {
        String::from_utf8(self.id.clone()).unwrap()
    }
}

pub struct Group {
    group_name: String,
    conversation: Conversation,
    mls_group: RefCell<MlsGroup>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct User {
    #[serde(
        serialize_with = "serialize_hashmap",
        deserialize_with = "deserialize_hashmap"
    )]
    pub(crate) contacts: HashMap<Vec<u8>, Contact>,
    #[serde(skip)]
    pub(crate) groups: RefCell<HashMap<String, Group>>,
    group_list: HashSet<String>,
    pub(crate) identity: RefCell<Identity>,
    // #[serde(skip)]
    // backend: Backend,
    #[serde(skip)]
    provider: OpenMlsRustPersistentCrypto,
    autosave_enabled: bool,
    auth_token: Option<AuthToken>,
}
