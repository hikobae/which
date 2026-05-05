pub struct Args {
    /// -h または --help オプション指定の有無
    pub has_help_option: bool,

    /// 位置引数
    pub positional: Vec<String>,
}

pub fn parse_args() -> Args {
    let mut has_help_option = false;
    let mut positional: Vec<String> = Vec::new();
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "-h" | "--help" => {
                has_help_option = true;
            }
            _ => {
                positional.push(arg);
            }
        }
    }
    Args {
        has_help_option,
        positional,
    }
}
