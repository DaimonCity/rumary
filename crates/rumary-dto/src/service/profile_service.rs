use crate::domain::launcher::{CheckDirs, FileInfo, Files, Profile};
use crate::dto::api::response::{CheckDirsDto,  FileInfoDto, FilesDto, ProfileDto};
use std::error::Error;

pub(crate) type ServiceResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

pub struct ProfileMapperService;

pub(crate) fn string_to_hash(string: &str) -> ServiceResult<Vec<u8>> {
    match hex::decode(string) {
        Ok(bytes) => Ok(bytes),
        Err(e) => Err(Box::new(e)),
    }
}

impl ProfileMapperService {
    pub fn _build(&self, _dto: ProfileDto) -> ServiceResult<Profile> {
        todo!();
    }

    fn _map_check_dirs(_dto: CheckDirsDto) -> ServiceResult<CheckDirs> {
        todo!()
    }
    fn _map_files(_dto: FilesDto) -> ServiceResult<Files> {
        todo!()
    }

    fn _map_file_info(_dto: FileInfoDto) -> ServiceResult<FileInfo> {
        todo!()
    }
}