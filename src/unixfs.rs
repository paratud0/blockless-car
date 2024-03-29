use std::fmt::Display;

use cid::Cid;

use crate::pb::{
    self,
    unixfs::{mod_Data::DataType, Data},
};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum FileType {
    #[default]
    Raw = 0,
    Directory = 1,
    File = 2,
    Metadata = 3,
    Symlink = 4,
    HAMTShard = 5,
}

impl Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let file_type = match self {
            FileType::Raw => "raw",
            FileType::Directory => "directory",
            FileType::File => "file",
            FileType::Metadata => "metadata",
            FileType::Symlink => "symlink",
            FileType::HAMTShard => "hasmtshard",
        };
        write!(f, "{file_type}")
    }
}

impl From<DataType> for FileType {
    fn from(value: DataType) -> Self {
        match value {
            DataType::Raw => FileType::Raw,
            DataType::Directory => FileType::Directory,
            DataType::File => FileType::File,
            DataType::Metadata => FileType::Metadata,
            DataType::Symlink => FileType::Symlink,
            DataType::HAMTShard => FileType::HAMTShard,
        }
    }
}

impl From<FileType> for DataType {
    fn from(value: FileType) -> Self {
        match value {
            FileType::Raw => DataType::Raw,
            FileType::Directory => DataType::Directory,
            FileType::File => DataType::File,
            FileType::Metadata => DataType::Metadata,
            FileType::Symlink => DataType::Symlink,
            FileType::HAMTShard => DataType::HAMTShard,
        }
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct UnixTime {
    pub seconds: i64,
    pub fractional_nanoseconds: Option<u32>,
}

impl From<pb::unixfs::UnixTime> for UnixTime {
    fn from(value: pb::unixfs::UnixTime) -> Self {
        Self {
            seconds: value.Seconds,
            fractional_nanoseconds: value.FractionalNanoseconds,
        }
    }
}

impl From<UnixTime> for pb::unixfs::UnixTime {
    fn from(value: UnixTime) -> Self {
        Self {
            Seconds: value.seconds,
            FractionalNanoseconds: value.fractional_nanoseconds,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct UnixFs {
    pub cid: Option<Cid>,
    pub mode: Option<u32>,
    pub file_type: FileType,
    pub fanout: Option<u64>,
    pub block_sizes: Vec<u64>,
    pub file_size: Option<u64>,
    pub hash_type: Option<u64>,
    pub links: Vec<Link>,
    pub mtime: Option<UnixTime>,
    pub file_name: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Link {
    pub hash: Cid,
    pub file_type: FileType,
    pub name: String,
    pub tsize: u64,
}

impl<'a> From<Data<'a>> for UnixFs {
    fn from(value: Data<'a>) -> Self {
        Self {
            cid: None,
            file_name: None,
            file_type: value.Type.into(),
            file_size: value.filesize,
            block_sizes: value.blocksizes,
            hash_type: value.hashType,
            fanout: value.fanout,
            mode: value.mode,
            mtime: value.mtime.map(|t| t.into()),
            links: Default::default(),
        }
    }
}

impl UnixFs {
    pub fn new(cid: Cid) -> Self {
        Self {
            cid: Some(cid),
            ..Default::default()
        }
    }

    pub fn new_directory() -> Self {
        Self {
            file_type: FileType::Directory,
            ..Default::default()
        }
    }

    #[inline(always)]
    pub fn add_link(&mut self, child: Link) -> usize {
        let idx = self.links.len();
        self.links.push(child);
        idx
    }

    #[inline(always)]
    pub fn links(&self) -> Vec<&Link> {
        self.links.iter().collect()
    }

    #[inline(always)]
    pub fn mtime(&self) -> Option<&UnixTime> {
        self.mtime.as_ref()
    }

    #[inline(always)]
    pub fn mode(&self) -> Option<u32> {
        self.mode
    }

    #[inline(always)]
    pub fn fanout(&self) -> Option<u64> {
        self.fanout
    }

    #[inline(always)]
    pub fn file_name(&self) -> Option<&str> {
        self.file_name.as_deref()
    }

    #[inline(always)]
    pub fn hash_type(&self) -> Option<u64> {
        self.hash_type
    }

    #[inline(always)]
    pub fn block_sizes(&self) -> Vec<u64> {
        self.block_sizes.clone()
    }

    #[inline(always)]
    pub fn file_size(&self) -> Option<u64> {
        self.file_size
    }

    #[inline(always)]
    pub fn file_type(&self) -> FileType {
        self.file_type
    }

    #[inline(always)]
    pub fn cid(&self) -> Option<Cid> {
        self.cid
    }
}
