//! `winget search` → JSON. Parse conservatively; keep raw text on failure.

use std::process::Command;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Hit {
    pub id: String,
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResultDoc {
    pub query: String,
    pub hits: Vec<Hit>,
    pub raw: Option<String>,
}

pub fn search(query: &str) -> ResultDoc {
    let out = Command::new("winget")
        .args([
            "search",
            query,
            "--disable-interactivity",
            "--accept-source-agreements",
        ])
        .output();
    let text = match out {
        Ok(o) => format!(
            "{}{}",
            String::from_utf8_lossy(&o.stdout),
            String::from_utf8_lossy(&o.stderr)
        ),
        Err(e) => {
            return ResultDoc {
                query: query.into(),
                hits: Vec::new(),
                raw: Some(format!("winget missing: {e}")),
            };
        }
    };
    let hits = parse_table(&text);
    ResultDoc {
        query: query.into(),
        hits,
        raw: if parse_table(&text).is_empty() {
            Some(text.chars().take(4000).collect())
        } else {
            None
        },
    }
}

fn parse_table(text: &str) -> Vec<Hit> {
    let mut hits = Vec::new();
    for line in text.lines() {
        let line = line.trim_end();
        if line.is_empty() || line.starts_with('-') || line.starts_with("Name") {
            continue;
        }
        // Id looks like Vendor.Product — take the first dotted token.
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }
        let mut id = None;
        let mut id_idx = 0usize;
        for (i, p) in parts.iter().enumerate() {
            if p.contains('.') && p.chars().next().unwrap_or(' ').is_ascii_alphabetic() {
                id = Some((*p).to_string());
                id_idx = i;
                break;
            }
        }
        let Some(id) = id else {
            continue;
        };
        let name = parts[..id_idx].join(" ");
        let version = parts.get(id_idx + 1).unwrap_or(&"").to_string();
        if name.is_empty() {
            continue;
        }
        hits.push(Hit { id, name, version });
    }
    hits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_sample_row() {
        let text = "\
Name                 Id                       Version
-----------------------------------------------------
Terraform            Hashicorp.Terraform      1.9.8
Azure CLI            Microsoft.AzureCLI       2.64.0
";
        let hits = parse_table(text);
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].id, "Hashicorp.Terraform");
        assert_eq!(hits[1].id, "Microsoft.AzureCLI");
    }
}
