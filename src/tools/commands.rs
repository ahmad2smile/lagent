pub(crate) enum Commands<'a> {
    Exit,
    Help,
    New,
    None,
    Run(&'a str),
}

impl<'a> From<&'a str> for Commands<'a> {
    fn from(value: &'a str) -> Self {
        match value {
            "/exit" | "/quit" | "/q" => Self::Exit,
            "/help" | "/?" => Self::Help,
            "/clear" | "/new" => Self::New,
            _ if value.starts_with('!') => Self::Run(&value[1..]),
            _ => Self::None,
        }
    }
}
