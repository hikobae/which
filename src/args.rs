use std::ffi::OsString;

pub struct Args {
    /// -h または --help オプション指定の有無
    pub has_help_option: bool,

    /// -v または --version オプション指定の有無
    pub has_version_option: bool,

    /// 位置引数
    pub positional: Vec<String>,
}

/// コマンドライン引数の解析エラー.
pub enum ArgsError {
    /// 非 UTF-8 の引数が含まれている.
    NotUnicode(OsString),
}

pub fn parse_args() -> Result<Args, ArgsError> {
    let mut has_help_option = false;
    let mut has_version_option = false;
    let mut positional: Vec<String> = Vec::new();
    for arg in std::env::args_os().skip(1) {
        let arg = arg.into_string().map_err(ArgsError::NotUnicode)?;
        match arg.as_str() {
            "-h" | "--help" => {
                has_help_option = true;
            }
            "-v" | "--version" => {
                has_version_option = true;
            }
            _ => {
                positional.push(arg);
            }
        }
    }
    Ok(Args {
        has_help_option,
        has_version_option,
        positional,
    })
}
