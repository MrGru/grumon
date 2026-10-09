//! Player-facing strings and the interpolation mini-language
//! (`{playerName}`, `{g:nam|nữ|trung tính}`, `{param}`), see content-schema §2.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Locale that ships and is used as the fallback for every other locale.
pub const SOURCE_LOCALE: &str = "vi-VN";

/// How NPCs address the protagonist (chosen at character creation).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Addressing {
    #[default]
    Male,
    Female,
    Neutral,
}

impl Addressing {
    pub const ALL: [Addressing; 3] = [Addressing::Male, Addressing::Female, Addressing::Neutral];

    fn index(self) -> usize {
        match self {
            Addressing::Male => 0,
            Addressing::Female => 1,
            Addressing::Neutral => 2,
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Addressing::Male => "ui.create.addr.male",
            Addressing::Female => "ui.create.addr.female",
            Addressing::Neutral => "ui.create.addr.neutral",
        }
    }
}

/// Values that every template can reference.
#[derive(Debug, Clone, Default)]
pub struct TextContext {
    pub player_name: String,
    pub addressing: Addressing,
}

/// All strings of one locale, merged from every `.locale.ron` file.
#[derive(Debug, Clone, Default)]
pub struct Locale {
    strings: HashMap<String, String>,
}

impl Locale {
    pub fn new(strings: HashMap<String, String>) -> Self {
        Self { strings }
    }

    /// Merges a file; returns keys that were already defined.
    pub fn merge(&mut self, file: HashMap<String, String>) -> Vec<String> {
        let mut duplicates = Vec::new();
        for (key, value) in file {
            if self.strings.insert(key.clone(), value).is_some() {
                duplicates.push(key);
            }
        }
        duplicates
    }

    pub fn has(&self, key: &str) -> bool {
        self.strings.contains_key(key)
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.strings.keys()
    }

    pub fn entries(&self) -> impl Iterator<Item = (&String, &String)> {
        self.strings.iter()
    }

    /// Raw template, or a visible marker for missing keys (the validator forbids those).
    pub fn raw(&self, key: &str) -> String {
        self.strings
            .get(key)
            .cloned()
            .unwrap_or_else(|| format!("«{key}»"))
    }

    /// Formats `key` with no extra parameters.
    pub fn text(&self, key: &str, ctx: &TextContext) -> String {
        self.format(key, ctx, &[])
    }

    /// Formats `key` with named parameters.
    pub fn format(&self, key: &str, ctx: &TextContext, params: &[(&str, String)]) -> String {
        match self.strings.get(key) {
            Some(template) => render(template, ctx, params),
            None => format!("«{key}»"),
        }
    }
}

/// Errors found while parsing a template.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateError {
    Unbalanced,
    EmptyToken,
    GenderOptions(usize),
}

/// A parsed piece of a template.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Piece<'a> {
    Text(&'a str),
    Brace(char),
    PlayerName,
    Gender([&'a str; 3]),
    Param(&'a str),
}

fn parse(template: &str) -> Result<Vec<Piece<'_>>, TemplateError> {
    let mut pieces = Vec::new();
    let mut rest = template;
    while !rest.is_empty() {
        if let Some(stripped) = rest.strip_prefix("{{") {
            pieces.push(Piece::Brace('{'));
            rest = stripped;
            continue;
        }
        if let Some(stripped) = rest.strip_prefix("}}") {
            pieces.push(Piece::Brace('}'));
            rest = stripped;
            continue;
        }
        if rest.starts_with('}') {
            return Err(TemplateError::Unbalanced);
        }
        if let Some(inner_start) = rest.strip_prefix('{') {
            let end = inner_start.find('}').ok_or(TemplateError::Unbalanced)?;
            let token = &inner_start[..end];
            if token.contains('{') {
                return Err(TemplateError::Unbalanced);
            }
            rest = &inner_start[end + 1..];
            if token.is_empty() {
                return Err(TemplateError::EmptyToken);
            }
            if token == "playerName" {
                pieces.push(Piece::PlayerName);
            } else if let Some(options) = token.strip_prefix("g:") {
                let parts: Vec<&str> = options.split('|').collect();
                if parts.len() != 3 {
                    return Err(TemplateError::GenderOptions(parts.len()));
                }
                pieces.push(Piece::Gender([parts[0], parts[1], parts[2]]));
            } else {
                pieces.push(Piece::Param(token));
            }
            continue;
        }
        let next = rest.find(['{', '}']).unwrap_or(rest.len());
        pieces.push(Piece::Text(&rest[..next]));
        rest = &rest[next..];
    }
    Ok(pieces)
}

/// Checks a template; returns the named parameters it uses.
pub fn check_template(template: &str) -> Result<Vec<String>, TemplateError> {
    Ok(parse(template)?
        .into_iter()
        .filter_map(|p| match p {
            Piece::Param(name) => Some(name.to_string()),
            _ => None,
        })
        .collect())
}

/// Renders a template. Malformed templates are returned unchanged.
pub fn render(template: &str, ctx: &TextContext, params: &[(&str, String)]) -> String {
    let Ok(pieces) = parse(template) else {
        return template.to_string();
    };
    let mut out = String::with_capacity(template.len() + 16);
    for piece in pieces {
        match piece {
            Piece::Text(text) => out.push_str(text),
            Piece::Brace(c) => out.push(c),
            Piece::PlayerName => out.push_str(&ctx.player_name),
            Piece::Gender(options) => out.push_str(options[ctx.addressing.index()]),
            Piece::Param(name) => match params.iter().find(|(n, _)| *n == name) {
                Some((_, value)) => out.push_str(value),
                None => {
                    out.push('{');
                    out.push_str(name);
                    out.push('}');
                }
            },
        }
    }
    out
}

/// Visible text with gender tokens and the player name resolved for *every*
/// addressing option. Used by validators to inspect all variants.
pub fn all_variants(template: &str) -> Vec<String> {
    Addressing::ALL
        .iter()
        .map(|&addressing| {
            render(
                template,
                &TextContext {
                    player_name: String::new(),
                    addressing,
                },
                &[],
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(addressing: Addressing) -> TextContext {
        TextContext {
            player_name: "Lâm Vô Trần".into(),
            addressing,
        }
    }

    #[test]
    fn substitutes_player_name_and_gender() {
        let t = "{g:Thằng nhóc|Con bé|Nhóc con} {playerName} này!";
        assert_eq!(
            render(t, &ctx(Addressing::Male), &[]),
            "Thằng nhóc Lâm Vô Trần này!"
        );
        assert_eq!(
            render(t, &ctx(Addressing::Female), &[]),
            "Con bé Lâm Vô Trần này!"
        );
        assert_eq!(
            render(t, &ctx(Addressing::Neutral), &[]),
            "Nhóc con Lâm Vô Trần này!"
        );
    }

    #[test]
    fn substitutes_params_and_escapes() {
        let out = render(
            "Nhận được {item} ×{count}. {{ok}}",
            &ctx(Addressing::Male),
            &[("item", "Thanh Tâm Thảo".into()), ("count", "3".into())],
        );
        assert_eq!(out, "Nhận được Thanh Tâm Thảo ×3. {ok}");
    }

    #[test]
    fn detects_broken_templates() {
        assert_eq!(check_template("abc {x"), Err(TemplateError::Unbalanced));
        assert_eq!(check_template("abc }"), Err(TemplateError::Unbalanced));
        assert_eq!(check_template("{}"), Err(TemplateError::EmptyToken));
        assert_eq!(
            check_template("{g:a|b}"),
            Err(TemplateError::GenderOptions(2))
        );
        assert_eq!(
            check_template("{a} và {b} {playerName}"),
            Ok(vec!["a".into(), "b".into()])
        );
    }

    #[test]
    fn missing_key_is_visible() {
        let locale = Locale::default();
        assert_eq!(locale.text("x.y", &ctx(Addressing::Male)), "«x.y»");
    }
}
