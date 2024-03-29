use std::collections::BTreeMap;

use cid::Cid;
use quick_protobuf::{BytesReader, MessageRead, MessageWrite, Writer};

use crate::{
    codec::Encoder,
    error::CarError,
    pb::unixfs::Data,
    unixfs::{FileType, Link, UnixFs},
    Decoder, Ipld,
};

impl Decoder<UnixFs> for Ipld {
    fn decode(&self) -> Result<UnixFs, CarError> {
        match self {
            ipld::Ipld::Map(ref m) => {
                let mut unix_fs: UnixFs = if let Some(ipld::Ipld::Bytes(data)) = m.get("Data") {
                    let mut reader = BytesReader::from_bytes(data);
                    Data::from_reader(&mut reader, data)
                        .map(|d| d.into())
                        .map_err(|e| CarError::Parsing(e.to_string()))?
                } else {
                    return Err(CarError::Parsing("ipld format error".into()));
                };
                if let Some(ipld::Ipld::List(links)) = m.get("Links") {
                    links.iter().for_each(|l| {
                        if let ipld::Ipld::Map(ref m) = l {
                            let cid = if let Some(ipld::Ipld::Link(cid)) = m.get("Hash") {
                                *cid
                            } else {
                                return;
                            };
                            let name = if let Some(ipld::Ipld::String(name)) = m.get("Name") {
                                name.clone()
                            } else {
                                String::new()
                            };
                            let size = if let Some(ipld::Ipld::Integer(size)) = m.get("Tsize") {
                                *size as u64
                            } else {
                                0
                            };
                            unix_fs.add_link(Link {
                                hash: cid,
                                file_type: FileType::Raw,
                                name,
                                tsize: size,
                            });
                        }
                    });
                }
                Ok(unix_fs)
            }
            _ => Err(CarError::Parsing("Not unixfs format".into())),
        }
    }
}

impl TryFrom<Ipld> for UnixFs {
    type Error = CarError;

    fn try_from(value: Ipld) -> Result<Self, Self::Error> {
        value.decode()
    }
}

impl TryFrom<(Cid, Ipld)> for UnixFs {
    type Error = CarError;

    fn try_from(value: (Cid, Ipld)) -> Result<Self, Self::Error> {
        value.1.decode().map(|mut v| {
            v.cid = Some(value.0);
            v
        })
    }
}

fn convert_to_ipld(value: &Link) -> Result<Ipld, CarError> {
    let mut map: BTreeMap<String, Ipld> = BTreeMap::new();
    map.insert("Hash".to_string(), Ipld::Link(value.hash));
    let file_name: Ipld = Ipld::String(value.name.to_owned());
    let tsize = Ipld::Integer(value.tsize as i128);
    map.insert("Name".to_string(), file_name);
    map.insert("Tsize".to_string(), tsize);
    Ok(Ipld::Map(map))
}

impl Encoder<Ipld> for UnixFs {
    fn encode(&self) -> Result<Ipld, CarError> {
        match self.file_type {
            FileType::Directory | FileType::File => {
                let mut map = BTreeMap::new();
                let data = Data {
                    mode: self.mode,
                    fanout: self.fanout,
                    hashType: self.hash_type,
                    filesize: self.file_size,
                    Type: self.file_type.into(),
                    blocksizes: self.block_sizes.clone(),
                    mtime: self.mtime().map(|s| s.clone().into()),
                    ..Default::default()
                };
                let mut buf: Vec<u8> = Vec::new();
                let mut bw = Writer::new(&mut buf);
                data.write_message(&mut bw)
                    .map_err(|e| CarError::Parsing(e.to_string()))?;
                map.insert("Data".into(), Ipld::Bytes(buf));
                let mut children_ipld: Vec<Ipld> = Vec::new();
                for child in self.links.iter() {
                    children_ipld.push(convert_to_ipld(child)?);
                }
                map.insert("Links".to_string(), Ipld::List(children_ipld));
                Ok(Ipld::Map(map))
            }
            _ => Err(CarError::Parsing("Not support unixfs format".into())),
        }
    }
}

impl TryFrom<UnixFs> for Ipld {
    type Error = CarError;

    fn try_from(value: UnixFs) -> Result<Self, Self::Error> {
        value.encode()
    }
}
