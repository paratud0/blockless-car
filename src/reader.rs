use cid::Cid;
use ipld::raw::RawCodec;

mod reader_v1;
use crate::{error::CarError, header::CarHeader, section::Section, unixfs::UnixFs, Ipld};
use integer_encoding::VarIntReader;
use std::{
    collections::VecDeque,
    io::{self, Read, Seek},
};

pub(crate) use reader_v1::CarReaderV1;

const MAX_ALLOWED_SECTION_SIZE: usize = 32 << 20;

pub fn read_block<R>(mut reader: R) -> Result<Option<Vec<u8>>, CarError>
where
    R: std::io::Read,
{
    let l: usize = match reader.read_varint() {
        Ok(i) => i,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                return Ok(None);
            }
            return Err(CarError::IO(e));
        }
    };
    if l > MAX_ALLOWED_SECTION_SIZE {
        return Err(CarError::TooLargeSection(l));
    }
    let mut data = vec![0u8; l];
    reader.read_exact(&mut data[..])?;
    Ok(Some(data))
}

pub(crate) fn read_section<R>(mut reader: R) -> Result<Option<Section>, CarError>
where
    R: io::Read + io::Seek,
{
    let len: usize = match reader.read_varint() {
        Ok(i) => i,
        Err(e) => {
            if e.kind() == io::ErrorKind::UnexpectedEof {
                return Ok(None);
            }
            return Err(CarError::IO(e));
        }
    };
    let start = reader.stream_position()?;
    if len > MAX_ALLOWED_SECTION_SIZE {
        return Err(CarError::TooLargeSection(len));
    }
    let cid = Cid::read_bytes(&mut reader).map_err(|e| CarError::Parsing(e.to_string()))?;
    let pos = reader.stream_position()?;
    let l = len - ((pos - start) as usize);
    reader.seek(io::SeekFrom::Current(l as _))?;
    Ok(Some(Section::new(cid, pos, l)))
}

pub trait CarReader {
    fn header(&self) -> &CarHeader;

    fn sections(&self) -> Vec<Section>;

    fn read_section_data(&mut self, cid: &Cid) -> Result<Vec<u8>, CarError>;

    fn ipld(&mut self, cid: &Cid) -> Result<Ipld, CarError>;

    #[inline(always)]
    fn unixfs(&mut self, cid: &Cid) -> Result<UnixFs, CarError> {
        let fs_ipld = self.ipld(cid)?;
        (*cid, fs_ipld).try_into()
    }

    fn search_file_cid_inner(
        &mut self,
        searchq: &mut VecDeque<Cid>,
        f: &str,
    ) -> Result<Cid, CarError> {
        let raw_code: u64 = RawCodec.into();
        while let Some(root_cid) = searchq.pop_front() {
            let codec = root_cid.codec();
            if codec == raw_code {
                continue;
            }
            let fs_ipld = self.ipld(&root_cid)?;
            if matches!(fs_ipld, Ipld::Map(_)) {
                let unixfs: UnixFs = (root_cid, fs_ipld).try_into()?;
                for ufs in unixfs.links() {
                    if ufs.name == f {
                        return Ok(ufs.hash);
                    }
                    searchq.push_back(ufs.hash);
                }
            }
        }
        Err(CarError::NotFound(format!("search {f} fail.")))
    }

    #[inline]
    fn search_file_cid(&mut self, f: &str) -> Result<Cid, CarError> {
        let roots = self.header().roots();
        let mut searchq = VecDeque::new();
        for root in roots.into_iter() {
            searchq.push_back(root);
            match self.search_file_cid_inner(&mut searchq, f) {
                Ok(o) => return Ok(o),
                Err(CarError::NotFound(_)) => continue,
                Err(e) => return Err(e),
            }
        }
        Err(CarError::NotFound(format!("search {f} fail.")))
    }
}

#[inline(always)]
pub fn new_v1<R>(inner: R) -> Result<impl CarReader, CarError>
where
    R: Read + Seek,
{
    CarReaderV1::new(inner)
}
