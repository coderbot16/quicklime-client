use std::fs::File;
use std::path::{PathBuf, Path, Component};
use std::ffi::OsStr;
use std::io::{Read, Write, BufReader};

pub trait Asset: Sized {
	type Err;
	type WErr;
	
	fn load<R: Read>(input: &mut R, name: &str) -> Result<Self, Self::Err>;
	fn save<W: Write>(&self, output: &mut W) -> Result<(), Self::WErr>;
}

pub trait AssetSource<A: Asset> {
	type Ticket;
	
	fn lookup(&self, asset: &str) -> Option<Self::Ticket>;
	fn load(&self, ticket: &mut Self::Ticket, name: &str) -> Result<A, A::Err>;
}

pub struct DirAssetSource {
	path: PathBuf
}

impl<A: Asset> AssetSource<A> for DirAssetSource {
	type Ticket = File;
	
	fn lookup(&self, asset: &str) -> Option<Self::Ticket> {
		let mut path = self.path.clone();
		path.push(OsStr::new(asset));
		
		File::open(&path).ok()
	}
	
	fn load(&self, ticket: &mut Self::Ticket, name: &str) -> Result<A, A::Err> {
		A::load(ticket, name)
	}
}