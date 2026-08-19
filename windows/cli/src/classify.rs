//! Category for a winget id: catalog first, then prefix rules.

pub const WINDOWS_CATEGORIES: &[&str] = &[
    "environment",
    "sdks",
    "devops",
    "cloud",
    "integrations",
    "agents",
    "browsers",
    "editors",
    "games",
    "media",
    "utils",
    "other",
];

pub const LINUX_CATEGORIES: &[&str] = &[
    "environment",
    "sdks",
    "devops",
    "cloud",
    "integrations",
    "agents",
    "content",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Class {
    pub category: &'static str,
    pub prefer_linux: bool,
}

pub fn windows_id(id: &str) -> Class {
    let id = id.trim();
    if let Some(c) = prefix_class(id) {
        return c;
    }
    Class {
        category: "other",
        prefer_linux: false,
    }
}

fn starts(id: &str, p: &str) -> bool {
    id.len() >= p.len() && id[..p.len()].eq_ignore_ascii_case(p)
}

fn prefix_class(id: &str) -> Option<Class> {
    const GAMES: &[&str] = &[
        "Valve.",
        "EpicGames.",
        "Ubisoft.",
        "Blizzard.",
        "ElectronicArts.",
        "GOG.",
        "RockstarGames.",
        "Nvidia.GeForceNow",
        "CloudImperiumGames.",
        "NexusMods.",
        "Battle.net",
        "Activision.",
    ];
    for p in GAMES {
        if starts(id, p) {
            return Some(Class {
                category: "games",
                prefer_linux: false,
            });
        }
    }
    const MEDIA: &[&str] = &[
        "Spotify.",
        "VideoLAN.",
        "Plex.",
        "OBSProject.",
        "Ytmdesktop.",
        "Apple.AppleMusic",
    ];
    for p in MEDIA {
        if starts(id, p) {
            return Some(Class {
                category: "media",
                prefer_linux: false,
            });
        }
    }
    const BROWSERS: &[&str] = &["Brave.", "Google.Chrome", "Mozilla.Firefox", "Vivaldi."];
    for p in BROWSERS {
        if starts(id, p) {
            return Some(Class {
                category: "browsers",
                prefer_linux: false,
            });
        }
    }
    const SDKS: &[&str] = &[
        "Microsoft.DotNet",
        "OpenJS.NodeJS",
        "Python.",
        "Rustlang.",
        "GoLang.",
        "Microsoft.OpenJDK",
        "Oracle.Java",
        "astral-sh.uv",
        "Oven-sh.Bun",
        "Kitware.CMake",
    ];
    for p in SDKS {
        if starts(id, p) {
            return Some(Class {
                category: "sdks",
                prefer_linux: true,
            });
        }
    }
    const CLOUD: &[&str] = &[
        "Microsoft.Azure",
        "Microsoft.Azd",
        "Amazon.AWS",
        "Google.CloudSDK",
        "Cloudflare.",
    ];
    for p in CLOUD {
        if starts(id, p) {
            return Some(Class {
                category: "cloud",
                prefer_linux: true,
            });
        }
    }
    const AGENTS: &[&str] = &[
        "xAI.",
        "Anthropic.",
        "GitHub.Copilot",
        "Anysphere.Cursor",
    ];
    for p in AGENTS {
        if starts(id, p) {
            return Some(Class {
                category: "agents",
                prefer_linux: false,
            });
        }
    }
    if starts(id, "JanDeDobbeleer.OhMyPosh")
        || starts(id, "Microsoft.WindowsTerminal")
        || starts(id, "Microsoft.PowerToys")
        || starts(id, "Starship.")
        || starts(id, "Atuinsh.")
    {
        return Some(Class {
            category: "environment",
            prefer_linux: starts(id, "Atuinsh.") || starts(id, "Starship."),
        });
    }
    if starts(id, "Microsoft.VisualStudioCode")
        || starts(id, "Microsoft.Office")
        || starts(id, "Microsoft.Outlook")
        || starts(id, "Microsoft.VisualStudio.202")
    {
        return Some(Class {
            category: "editors",
            prefer_linux: false,
        });
    }
    if starts(id, "Docker.")
        || starts(id, "GitHub.cli")
        || starts(id, "Kubernetes.")
        || starts(id, "Helm.")
    {
        let prefer = starts(id, "GitHub.cli") || starts(id, "Kubernetes.") || starts(id, "Helm.");
        return Some(Class {
            category: "devops",
            prefer_linux: prefer,
        });
    }
    if starts(id, "AgileBits.1Password")
        || starts(id, "Piriform.")
        || starts(id, "SteelSeries.")
        || starts(id, "Corsair.")
        || starts(id, "7zip.")
        || starts(id, "Discord.")
        || starts(id, "Git.Git")
    {
        return Some(Class {
            category: "utils",
            prefer_linux: starts(id, "AgileBits.1Password.CLI"),
        });
    }
    if starts(id, "Stripe.")
        || starts(id, "Tailscale.")
        || starts(id, "NordSecurity.")
    {
        return Some(Class {
            category: "integrations",
            prefer_linux: starts(id, "Stripe."),
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steam_is_games() {
        assert_eq!(windows_id("Valve.Steam").category, "games");
    }

    #[test]
    fn node_prefers_linux() {
        let c = windows_id("OpenJS.NodeJS.LTS");
        assert_eq!(c.category, "sdks");
        assert!(c.prefer_linux);
    }

    #[test]
    fn azure_prefers_linux() {
        let c = windows_id("Microsoft.AzureCLI");
        assert_eq!(c.category, "cloud");
        assert!(c.prefer_linux);
    }

    #[test]
    fn brave_is_browser() {
        assert_eq!(windows_id("Brave.Brave").category, "browsers");
    }

    #[test]
    fn unknown_is_other() {
        assert_eq!(windows_id("Contoso.FizzBuzz").category, "other");
        assert!(!windows_id("Contoso.FizzBuzz").prefer_linux);
    }
}
