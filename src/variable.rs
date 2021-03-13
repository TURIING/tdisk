use once_cell::sync::OnceCell;
use indicatif::MultiProgress;

use crate::util::{Info, MulBar};

pub static INFO: OnceCell<Info> = OnceCell::new();
pub static MULBAR: MulBar = MulBar { mulbar: OnceCell::new()};
// 配置文件路径
pub static CFG_PATH: &str = "./tdisk.yml";