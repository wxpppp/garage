use serde::{Deserialize, Serialize};

use garage_util::data::*;

pub trait PartitionKey {
	fn hash(&self) -> Hash;
}

impl PartitionKey for String {
	fn hash(&self) -> Hash {
		sha256sum(self.as_bytes())
	}
}

impl PartitionKey for Hash {
	fn hash(&self) -> Hash {
		self.clone()
	}
}

pub trait SortKey {
	fn sort_key(&self) -> &[u8];
}

impl SortKey for String {
	fn sort_key(&self) -> &[u8] {
		self.as_bytes()
	}
}

impl SortKey for Hash {
	fn sort_key(&self) -> &[u8] {
		self.as_slice()
	}
}

pub trait Entry<P: PartitionKey, S: SortKey>:
	PartialEq + Clone + Serialize + for<'de> Deserialize<'de> + Send + Sync
{
	fn partition_key(&self) -> &P;
	fn sort_key(&self) -> &S;

	fn merge(&mut self, other: &Self);
}

pub trait TableSchema: Send + Sync {
	type P: PartitionKey + Clone + PartialEq + Serialize + for<'de> Deserialize<'de> + Send + Sync;
	type S: SortKey + Clone + Serialize + for<'de> Deserialize<'de> + Send + Sync;
	type E: Entry<Self::P, Self::S>;
	type Filter: Clone + Serialize + for<'de> Deserialize<'de> + Send + Sync;

	// Action to take if not able to decode current version:
	// try loading from an older version
	fn try_migrate(_bytes: &[u8]) -> Option<Self::E> {
		None
	}

	// Updated triggers some stuff downstream, but it is not supposed to block or fail,
	// as the update itself is an unchangeable fact that will never go back
	// due to CRDT logic. Typically errors in propagation of info should be logged
	// to stderr.
	fn updated(&self, _old: Option<Self::E>, _new: Option<Self::E>) {}

	fn matches_filter(_entry: &Self::E, _filter: &Self::Filter) -> bool {
		true
	}
}
