use std::{io, io::Write};

use blake2::{
    digest::consts::{U32, U64},
    Blake2b,
};
use digest::Digest;
use serde::Serialize;
use tari_bor::encode_into;
use tari_crypto::{hash_domain, hashing::DomainSeparation};
use tari_template_lib::Hash;

hash_domain!(TariEngineHashDomain, "com.tari.dan.engine", 0);

pub fn hasher64(label: EngineHashDomainLabel) -> TariHasher64 {
    TariHasher64::new_with_label::<TariEngineHashDomain>(label.as_label())
}

pub fn template_hasher64() -> TariHasher64 {
    hasher64(EngineHashDomainLabel::Template)
}

pub fn hasher32(label: EngineHashDomainLabel) -> TariHasher32 {
    TariHasher32::new_with_label::<TariEngineHashDomain>(label.as_label())
}

pub fn template_hasher32() -> TariHasher32 {
    hasher32(EngineHashDomainLabel::Template)
}

hash_domain!(ConfidentialOutputHashDomain, "com.tari.dan.confidential_output", 1);

#[derive(Debug, Clone)]
pub struct TariHasher32 {
    hasher: Blake2b<U32>,
}
impl TariHasher32 {
    pub fn new_with_label<TDomain: DomainSeparation>(label: &'static str) -> Self {
        let mut hasher = Blake2b::<U32>::new();
        TDomain::add_domain_separation_tag(&mut hasher, label);
        Self { hasher }
    }

    pub fn update<T: Serialize + ?Sized>(&mut self, data: &T) {
        // CBOR encoding does not make any contract to say that if the writer is infallible (as it is here) then
        // encoding in infallible. However this should be the case. Since it is very unergonomic to return an
        // error in hash chain functions, and therefore all usages of the hasher, we assume all types implement
        // infallible encoding.
        encode_into(data, &mut self.hash_writer()).expect("encoding failed")
    }

    pub fn chain<T: Serialize + ?Sized>(mut self, data: &T) -> Self {
        self.update(data);
        self
    }

    pub fn digest<T: Serialize + ?Sized>(self, data: &T) -> Hash {
        self.chain(data).result()
    }

    pub fn result(self) -> Hash {
        let hash: [u8; 32] = self.hasher.finalize().into();
        hash.into()
    }

    pub fn finalize_into(self, output: &mut digest::Output<Blake2b<U32>>) {
        digest::FixedOutput::finalize_into(self.hasher, output)
    }

    fn hash_writer(&mut self) -> impl Write + '_ {
        struct HashWriter<'a>(&'a mut Blake2b<U32>);
        impl Write for HashWriter<'_> {
            fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
                self.0.update(buf);
                Ok(buf.len())
            }

            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        HashWriter(&mut self.hasher)
    }
}

#[derive(Debug, Clone)]
pub struct TariHasher64 {
    hasher: Blake2b<U64>,
}

impl TariHasher64 {
    pub fn new_with_label<TDomain: DomainSeparation>(label: &'static str) -> Self {
        let mut hasher = Blake2b::<U64>::new();
        TDomain::add_domain_separation_tag(&mut hasher, label);
        Self { hasher }
    }

    pub fn update<T: Serialize + ?Sized>(&mut self, data: &T) {
        // CBOR encoding does not make any contract to say that if the writer is infallible (as it is here) then
        // encoding in infallible. However this should be the case. Since it is very unergonomic to return an
        // error in hash chain functions, and therefore all usages of the hasher, we assume all types implement
        // infallible encoding.
        encode_into(data, &mut self.hash_writer()).expect("encoding failed")
    }

    pub fn chain<T: Serialize + ?Sized>(mut self, data: &T) -> Self {
        self.update(data);
        self
    }

    pub fn digest<T: Serialize + ?Sized>(self, data: &T) -> [u8; 64] {
        self.chain(data).result()
    }

    pub fn result(self) -> [u8; 64] {
        self.hasher.finalize().into()
    }

    pub fn finalize_into(self, output: &mut digest::Output<Blake2b<U64>>) {
        digest::FixedOutput::finalize_into(self.hasher, output)
    }

    fn hash_writer(&mut self) -> impl Write + '_ {
        struct HashWriter<'a>(&'a mut Blake2b<U64>);
        impl Write for HashWriter<'_> {
            fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
                self.0.update(buf);
                Ok(buf.len())
            }

            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        HashWriter(&mut self.hasher)
    }
}

#[derive(Debug)]
pub enum EngineHashDomainLabel {
    Template,
    ShardId,
    ConfidentialProof,
    ConfidentialTransfer,
    ShardPledgeCollection,
    HotStuffTreeNode,
    Transaction,
    NonFungibleId,
    NonFungibleIndex,
    UuidOutput,
    Output,
    TransactionSignature,
    ResourceAddress,
    ComponentAddress,
    RandomBytes,
    TransactionReceipt,
    FeeClaimAddress,
    QuorumCertificate,
}

impl EngineHashDomainLabel {
    pub fn as_label(&self) -> &'static str {
        match self {
            Self::Template => "Template",
            Self::ShardId => "ShardId",
            Self::ConfidentialProof => "ConfidentialProof",
            Self::ConfidentialTransfer => "ConfidentialTransfer",
            Self::ShardPledgeCollection => "ShardPledgeCollection",
            Self::HotStuffTreeNode => "HotStuffTreeNode",
            Self::Transaction => "Transaction",
            Self::NonFungibleId => "NonFungibleId",
            Self::NonFungibleIndex => "NonFungibleIndex",
            Self::UuidOutput => "UuidOutput",
            Self::Output => "Output",
            Self::TransactionSignature => "TransactionSignature",
            Self::ResourceAddress => "ResourceAddress",
            Self::ComponentAddress => "ComponentAddress",
            Self::RandomBytes => "RandomBytes",
            Self::TransactionReceipt => "TransactionReceipt",
            Self::FeeClaimAddress => "FeeClaimAddress",
            Self::QuorumCertificate => "QuorumCertificate",
        }
    }
}