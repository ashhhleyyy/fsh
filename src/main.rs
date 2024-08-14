use ansi_term::{Colour as AnsiColour, Style};
use gethostname::gethostname;
use git2::{Repository, Status as GitStatus};
use std::{fmt::Display, path::PathBuf};

struct PromptComponent {
    text: String,
    style: Style,
    space_after: bool,
}

enum Colour {
    Red,
    Green,
    Blue,
    Yellow,
    Pink,
    Purple,
}

fn from_hex(hex: u32) -> AnsiColour {
    AnsiColour::RGB(
        (hex >> 16 & 0xff) as u8,
        (hex >> 8 & 0xff) as u8,
        (hex & 0xff) as u8,
    )
}

impl Colour {
    fn to_ansi(&self) -> AnsiColour {
        match self {
            Colour::Red => from_hex(0xff5555),
            Colour::Green => from_hex(0x50fa7b),
            Colour::Blue => from_hex(0x8be9fd),
            Colour::Yellow => from_hex(0xf1fa8c),
            Colour::Pink => from_hex(0xff79c6),
            Colour::Purple => from_hex(0xbd93f9),
        }
    }
}

impl PromptComponent {
    fn new(text: &str, style: Style) -> Self {
        Self {
            text: text.to_string(),
            style,
            space_after: true,
        }
    }

    fn bold(text: &str, colour: Colour) -> Self {
        Self::new(text, colour.to_ansi().bold())
    }

    fn unstyled(text: &str) -> Self {
        Self {
            text: text.to_string(),
            style: Style::default(),
            space_after: true,
        }
    }

    fn no_space(self) -> Self {
        Self {
            text: self.text,
            style: self.style,
            space_after: false,
        }
    }
}

impl Display for PromptComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.style.paint(&self.text)))
    }
}

fn main() {
    let last_status = if let Some(status) = std::env::args().into_iter().nth(1) {
        status.parse().unwrap()
    } else {
        0
    };

    let mut components = Vec::new();

    let user = users::get_current_username()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "unknown".to_string());
    if std::env::var("FSH_NO_HOSTNAME").is_ok() {
        components.push(PromptComponent::bold(&user, Colour::Purple));
    } else {
        components.push(PromptComponent::bold(&user, Colour::Purple).no_space());
        components.push(PromptComponent::unstyled("@").no_space());
        let hostname = gethostname()
            .into_string()
            .unwrap_or_else(|e| format!("{:?}", e));
        components.push(PromptComponent::bold(&hostname, Colour::Pink));
    }

    let pwd = std::env::current_dir().unwrap();
    let dir = format!("C:{}", pwd.to_string_lossy()).replace('/', "\\");

    let repo = Repository::discover(".").ok();

    components.push(PromptComponent::unstyled("in"));
    components.push(PromptComponent::bold(&dir, Colour::Green));

    get_git_info(&mut components, &repo);

    if last_status == 0 {
        components.push(PromptComponent::bold("\u{f061}", Colour::Yellow));
    } else {
        components.push(PromptComponent::bold(
            &format!("{} \u{f061}", last_status),
            Colour::Red,
        ));
    }

    for component in components {
        if component.space_after {
            print!("{} ", component);
        } else {
            print!("{}", component);
        }
    }
}

fn get_git_info(components: &mut Vec<PromptComponent>, repo: &Option<Repository>) {
    match repo {
        Some(repo) => {
            let head = match repo.head() {
                Ok(head) => Some(head.shorthand().unwrap().to_string()),
                Err(ref e) if e.code() == git2::ErrorCode::NotFound => None,
                Err(ref e) if e.code() == git2::ErrorCode::UnbornBranch => {
                    // https://github.com/starship/starship/commit/489838e6a24ea1c08be6abe56d066724a1d59abd#diff-d6346fd7d17270b1282142aeeda9c4bc2b7d8fd0f37b24a1c871a9257f0ed0aaR324-R336
                    let mut head_path = repo.path().to_path_buf();
                    head_path.push("HEAD");

                    std::fs::read_to_string(&head_path)
                        .ok()
                        .unwrap()
                        .lines()
                        .next()
                        .unwrap()
                        .trim()
                        .split('/')
                        .last()
                        .map(|r| r.to_owned())
                }
                Err(e) => Err(e).unwrap(),
            };

            let head = head.unwrap_or_else(|| "(no HEAD)".to_string());

            components.push(PromptComponent::bold(
                &format!("\u{e725} {}", head),
                Colour::Blue,
            ));

            let mut unstaged = false;
            let mut staged = false;
            for status in repo.statuses(None).unwrap().iter() {
                let status = status.status();
                if status.intersects(
                    GitStatus::WT_DELETED
                        | GitStatus::WT_MODIFIED
                        | GitStatus::WT_NEW
                        | GitStatus::WT_RENAMED
                        | GitStatus::WT_TYPECHANGE,
                ) {
                    unstaged = true;
                }
                if status.intersects(
                    GitStatus::INDEX_DELETED
                        | GitStatus::INDEX_MODIFIED
                        | GitStatus::INDEX_NEW
                        | GitStatus::INDEX_RENAMED
                        | GitStatus::INDEX_TYPECHANGE,
                ) {
                    staged = true;
                }
            }

            let action = match repo.state() {
                git2::RepositoryState::Merge => Some(PromptComponent::bold("merge", Colour::Pink)),
                git2::RepositoryState::Revert | git2::RepositoryState::RevertSequence => {
                    Some(PromptComponent::bold("revert", Colour::Pink))
                }
                git2::RepositoryState::CherryPick | git2::RepositoryState::CherryPickSequence => {
                    Some(PromptComponent::bold("cherry pick", Colour::Pink))
                }
                git2::RepositoryState::Rebase
                | git2::RepositoryState::RebaseInteractive
                | git2::RepositoryState::RebaseMerge => {
                    Some(PromptComponent::bold("rebase", Colour::Pink))
                }
                _ => None,
            };

            if let Some(action) = action {
                components.push(PromptComponent::unstyled("performing a"));
                components.push(action);
            }

            if staged {
                components.push(PromptComponent::bold("+", Colour::Green));
            }

            if unstaged {
                components.push(PromptComponent::bold("●", Colour::Red));
            }
        }
        None => {}
    }
}
