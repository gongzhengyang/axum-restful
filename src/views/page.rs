use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct PageQuery {
    #[serde(default = "default_page_num")]
    pub page_num: usize,

    #[serde(default = "default_page_size")]
    pub page_size: usize,
}

fn default_page_num() -> usize {
    1
}

fn default_page_size() -> usize {
    20
}
