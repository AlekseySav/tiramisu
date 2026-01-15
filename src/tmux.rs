pub struct Tmux {
    args: Vec<String>,
}

impl Tmux {
    pub fn attached() -> bool {
        std::env::var("TMUX").is_ok()
    }

    pub fn new() -> Self {
        Self { args: vec![] }
    }

    pub fn opened_sessions(mut self) -> Self {
        self.args.push("ls".into());
        self.args.push("-F".into());
        self.args.push("#{session_name} #{session_attached}".into());
        self.args.push(";".into());
        self
    }
}

pub fn list_sessions() -> (Vec<String>, Vec<String>) {
    let res = std::process::Command::new("tmux")
        .args(["ls", "-F", "#{session_name} #{session_attached}"])
        .output()
        .unwrap();
    assert!(res.status.success());
    assert_eq!(res.stderr, Vec::<u8>::new());

    let mut r: (Vec<String>, Vec<String>) = (vec![], vec![]);
    for (name, attached) in String::from_utf8(res.stdout)
        .unwrap()
        .lines()
        .map(|s| s.split(' ').collect())
        .map(|s: Vec<&str>| (String::from(s[0]), s[1].parse::<usize>().unwrap() == 1))
    {
        if attached {
            r.0.push(name);
        } else {
            r.1.push(name);
        }
    }

    r
}
