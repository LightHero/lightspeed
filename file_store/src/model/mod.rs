use c3p0::{C3p0Error, JsonCodec, Model};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{borrow::Cow, sync::Arc};
use strum::{AsRefStr, Display};

pub type FileStoreDataModel = Model<u64, FileStoreDataData>;

#[derive(Clone)]
pub enum BinaryContent<'a> {
    InMemory { content: Cow<'a, [u8]> },
    OpenDal { operator: Arc<opendal::Operator>, path: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileStoreDataData {
    pub filename: String,
    pub repository: RepositoryFile,
    pub content_type: String,
    pub created_date_epoch_seconds: i64,
}

#[derive(Debug, Clone, PartialEq, AsRefStr, Display)]
pub enum Repository {
    DB { repository_name: String },
    FS { repository_name: String },
}

impl From<&RepositoryFile> for Repository {
    fn from(repo: &RepositoryFile) -> Self {
        match repo {
            RepositoryFile::DB { repository_name, .. } => {
                Repository::DB { repository_name: repository_name.to_owned() }
            }
            RepositoryFile::FS { repository_name, .. } => {
                Repository::FS { repository_name: repository_name.to_owned() }
            }
        }
    }
}

impl From<&SaveRepository> for Repository {
    fn from(repo: &SaveRepository) -> Self {
        match repo {
            SaveRepository::DB { repository_name, .. } => {
                Repository::DB { repository_name: repository_name.to_owned() }
            }
            SaveRepository::FS { repository_name, .. } => {
                Repository::FS { repository_name: repository_name.to_owned() }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, AsRefStr, Display)]
#[serde(tag = "_json_tag")]
pub enum RepositoryFile {
    DB { file_path: String, repository_name: String },
    FS { file_path: String, repository_name: String },
}

impl RepositoryFile {
    pub fn from(repo: &SaveRepository, file_name: &str) -> Self {
        match repo {
            SaveRepository::DB { repository_name, subfolder } => RepositoryFile::DB {
                repository_name: repository_name.to_owned(),
                file_path: to_file_path(subfolder.as_deref(), file_name),
            },
            SaveRepository::FS { repository_name, subfolder } => RepositoryFile::FS {
                repository_name: repository_name.to_owned(),
                file_path: to_file_path(subfolder.as_deref(), file_name),
            },
        }
    }

    pub fn file_path(&self) -> &str {
        match self {
            RepositoryFile::DB { file_path, .. } => file_path,
            RepositoryFile::FS { file_path, .. } => file_path,
        }
    }
}

#[derive(Debug, Clone)]
pub enum SaveRepository {
    DB { subfolder: Option<String>, repository_name: String },
    FS { subfolder: Option<String>, repository_name: String },
}

pub fn to_file_path(subfolder: Option<&str>, filename: &str) -> String {
    match subfolder {
        Some(path) => format!("{path}/{filename}"),
        None => filename.to_owned(),
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "_json_tag")]
enum FileStoreDataVersioning<'a> {
    V1(Cow<'a, FileStoreDataData>),
}

#[derive(Clone)]
pub struct FileStoreDataDataCodec {}

impl JsonCodec<FileStoreDataData> for FileStoreDataDataCodec {
    fn data_from_value(&self, value: Value) -> Result<FileStoreDataData, C3p0Error> {
        let versioning = serde_json::from_value(value)?;
        let data = match versioning {
            FileStoreDataVersioning::V1(data_v1) => data_v1.into_owned(),
        };
        Ok(data)
    }

    fn data_to_value(&self, data: &FileStoreDataData) -> Result<Value, C3p0Error> {
        serde_json::to_value(FileStoreDataVersioning::V1(Cow::Borrowed(data))).map_err(C3p0Error::from)
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use lightspeed_core::utils::new_hyphenated_uuid;

    #[test]
    fn should_convert_repository_file_to_repository() {
        let main_repository_name = new_hyphenated_uuid();
        let main_file_path = new_hyphenated_uuid();

        match Repository::from(&RepositoryFile::DB {
            repository_name: main_repository_name.clone(),
            file_path: main_file_path.clone(),
        }) {
            Repository::DB { repository_name } => assert_eq!(main_repository_name, repository_name),
            _ => panic!(),
        };

        match Repository::from(&RepositoryFile::FS {
            repository_name: main_repository_name.clone(),
            file_path: main_file_path,
        }) {
            Repository::FS { repository_name } => assert_eq!(main_repository_name, repository_name),
            _ => panic!(),
        };
    }

    #[test]
    fn should_convert_save_repository_to_repository() {
        let main_repository_name = new_hyphenated_uuid();

        match Repository::from(&SaveRepository::DB { repository_name: main_repository_name.clone(), subfolder: None }) {
            Repository::DB { repository_name } => {
                assert_eq!(main_repository_name, repository_name);
            }
            _ => panic!(),
        };

        match Repository::from(&SaveRepository::FS { repository_name: main_repository_name.clone(), subfolder: None }) {
            Repository::FS { repository_name } => {
                assert_eq!(main_repository_name, repository_name);
            }
            _ => panic!(),
        };
    }

    #[test]
    fn should_convert_save_repository_to_repository_file() {
        let main_repository_name = new_hyphenated_uuid();
        let main_subfolder = new_hyphenated_uuid();
        let main_file_name = new_hyphenated_uuid();

        match RepositoryFile::from(
            &SaveRepository::DB { repository_name: main_repository_name.clone(), subfolder: None },
            &main_file_name,
        ) {
            RepositoryFile::DB { repository_name, file_path } => {
                assert_eq!(main_repository_name, repository_name);
                assert_eq!(to_file_path(None, &main_file_name), file_path);
            }
            _ => panic!(),
        };

        match RepositoryFile::from(
            &SaveRepository::FS { repository_name: main_repository_name.clone(), subfolder: None },
            &main_file_name,
        ) {
            RepositoryFile::FS { repository_name, file_path } => {
                assert_eq!(main_repository_name, repository_name);
                assert_eq!(to_file_path(None, &main_file_name), file_path);
            }
            _ => panic!(),
        };

        match RepositoryFile::from(
            &SaveRepository::DB {
                repository_name: main_repository_name.clone(),
                subfolder: Some(main_subfolder.clone()),
            },
            &main_file_name,
        ) {
            RepositoryFile::DB { repository_name, file_path } => {
                assert_eq!(main_repository_name, repository_name);
                assert_eq!(to_file_path(Some(&main_subfolder), &main_file_name), file_path);
            }
            _ => panic!(),
        };

        match RepositoryFile::from(
            &SaveRepository::FS {
                repository_name: main_repository_name.clone(),
                subfolder: Some(main_subfolder.clone()),
            },
            &main_file_name,
        ) {
            RepositoryFile::FS { repository_name, file_path } => {
                assert_eq!(main_repository_name, repository_name);
                assert_eq!(to_file_path(Some(&main_subfolder), &main_file_name), file_path);
            }
            _ => panic!(),
        };
    }
}
