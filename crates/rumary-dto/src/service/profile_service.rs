use crate::domain::download_url::DownloadUrl;
use crate::domain::file_path::FilePath;
use crate::domain::launcher::{CheckDirs, CheckType, FileInfo, Files, Profile};
use crate::dto::api::response::{CheckDirsDto, CheckTypeDto, FileInfoDto, FilesDto, ProfileDto};
use crate::service::util::string_to_hash;
use std::collections::HashMap;
use std::error::Error;

pub(crate) type ServiceResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

pub struct ProfileMapperService;

impl ProfileMapperService {
    pub fn build(&self, dto: ProfileDto) -> ServiceResult<Profile> {
        let icon = DownloadUrl::from_string(dto.icon)?;

        let hard_check = Self::map_check_dirs(dto.hard_check)?;
        let soft_check = Self::map_check_dirs(dto.soft_check)?;

        Ok(Profile::new(dto.id, dto.name, icon, hard_check, soft_check))
    }

    fn map_check_dirs(dto: CheckDirsDto) -> ServiceResult<CheckDirs> {
        let dirs_map = dto
            .dirs
            .into_iter()
            .map(|(dirname, files_dto)| {
                let files = Self::map_files(files_dto)?;
                Ok((dirname, files))
            })
            .collect::<ServiceResult<HashMap<_, _>>>()?;

        Ok(CheckDirs {
            dirs: dirs_map,
        })
    }
    fn map_files(dto: FilesDto) -> ServiceResult<Files> {
        let files_map = dto
            .files
            .into_iter()
            .map(|(filename, file_info_dto)| {
                let file_info = Self::map_file_info(file_info_dto)?;
                Ok((filename, file_info))
            })
            .collect::<ServiceResult<HashMap<_, _>>>()?;

        Ok(Files::new(files_map))
    }

    fn map_file_info(dto: FileInfoDto) -> ServiceResult<FileInfo> {
        let sha1 = string_to_hash(&dto.sha1)?;

        let check_type = match dto._type {
            CheckTypeDto::Required => CheckType::Required,
            CheckTypeDto::Optional => CheckType::Optional,
        };

        let path = FilePath::new(dto.path.into());
        let url = DownloadUrl::from_string(dto.url)?;

        Ok(FileInfo::new(sha1, check_type, path, url))
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use tokio::time::Instant;
    use uuid::Uuid;

    use crate::dto::api::response::{
        CheckDirsDto,
        CheckTypeDto,
        FileInfoDto,
        FilesDto,
        ProfileDto,
    };

   #[test]
   fn test() {
       let b = big_profile_dto();

       assert_eq!(b.hard_check.dirs.len(), 50);
   }

    #[test]
    fn benchmark_profile_build() {
        let start = Instant::now();

        let profile = big_profile_dto();

        let elapsed = start.elapsed();

        println!("Elapsed: {:?}", elapsed);
        println!("Dirs: {}", profile.hard_check.dirs.len());
    }

    fn big_profile_dto() -> ProfileDto {
        let mut hard_dirs = HashMap::new();
        let mut soft_dirs = HashMap::new();

        // 50 директорий
        for dir_index in 0..50 {
            let dir_name = format!("mods-pack-{}", dir_index);

            let mut files = HashMap::new();

            // 500 файлов в каждой директории
            for file_index in 0..500 {
                let file_name = format!("mod-{}-{}.jar", dir_index, file_index);

                files.insert(
                    file_name,
                    FileInfoDto {
                        sha1: "5d41402abc4b2a76b9719d911017c592".to_string(),
                        _type: if file_index % 2 == 0 {
                            CheckTypeDto::Required
                        } else {
                            CheckTypeDto::Optional
                        },
                        path: format!(
                            ".rumary/mods/category-{}/subdir-{}",
                            dir_index,
                            file_index % 10
                        ),
                        url: format!(
                            "https://cdn.example.com/mods/{}/{}.jar",
                            dir_index,
                            file_index
                        ),
                    },
                );
            }

            hard_dirs.insert(
                dir_name.clone(),
                FilesDto {
                    files: files.clone(),
                },
            );

            soft_dirs.insert(
                format!("optional-{}", dir_name),
                FilesDto {
                    files,
                },
            );
        }

        ProfileDto {
            id: Uuid::new_v4(),
            name: "Ultra Large Benchmark Profile".to_string(),
            icon: "https://cdn.example.com/icons/profile.png".to_string(),
            hard_check: CheckDirsDto {
                dirs: hard_dirs,
            },
            soft_check: CheckDirsDto {
                dirs: soft_dirs,
            },
        }
    }

}