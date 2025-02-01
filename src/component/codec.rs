use serde::Deserialize;

use super::INTEGER;

#[derive(Debug, Deserialize)]
pub struct CodecConfig<InnerType> {
    pub inner: InnerType,
}

// trait CodecConfigOps {
//     fn get_tag(&self) -> String;
//     fn get_dir_rawdata(&self) -> Option<String>;
//     fn get_dir_l1_data(&self) -> Option<String>;
//     fn get_dir_sqlite3(&self) -> Option<String>;
//     fn get_regex_rawdata_filename(&self) -> Option<String>;
//     fn get_regex_l1_data_filename(&self) -> Option<String>;
//     fn get_regex_sqlite3_filename(&self) -> Option<String>;
// }

#[allow(unused)]
#[derive(Debug, Deserialize, Clone)]
pub struct CodecConfigDataTypeSpec {
    pub name: String,
    pub description: String,
    pub dtype: String,
    pub unit: Option<String>,
    pub float_number: Option<INTEGER>,
}

#[allow(unused)]
#[derive(Debug, Deserialize, Clone)]
pub struct CodecConfigDataTypeRust {
    pub name: String,
    pub dtype: String,
    pub unit: Option<String>,
    pub major_datetime: Option<bool>,
}

#[allow(unused)]
#[derive(Debug, Deserialize, Clone)]
pub struct CodecConfigDataTypeSqlite3 {
    pub name: String,
    pub dtype: String,
    pub unit: Option<String>,
}

#[allow(unused)]
#[derive(Debug, Deserialize, Clone)]
pub struct CodecConfigDataType {
    pub spec: CodecConfigDataTypeSpec,
    pub rust: CodecConfigDataTypeRust,
    pub sqlite3: CodecConfigDataTypeSqlite3,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct CodecConfigMetadata {
    pub name: String,
    pub dkind: Vec<String>,
    pub raw_save: Option<bool>,
    pub stx_etx: Option<bool>,
    pub formation: Vec<CodecConfigDataType>,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct CodecConfigDB {
    pub directory: String,
    pub regex: Option<String>,
    pub seperate_by: Option<String>,
    pub pattern: Option<String>,
    pub suffix: Option<String>,
}

/* The CodecConfig.InnerType basic format */
#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct CodecConfigBase {
    pub tag: String,
    pub rawdata: Option<CodecConfigDB>,
    pub l1_data: Option<CodecConfigDB>,
    pub sqlite3: Option<CodecConfigDB>,
    pub metadatas: Vec<CodecConfigMetadata>,
}

impl CodecConfigMetadata {
    pub fn get_datetime_info(&self) -> Option<(usize, CodecConfigDataType)> {
        for (idx, val) in self.formation.iter().enumerate() {
            if val.rust.major_datetime == Some(true) {
                return Some((idx, val.clone()));
            }
        }
        None
    }
}
