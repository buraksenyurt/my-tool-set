//! Turning the run form (parameter values + extra arguments) into argv.
//!
//! argv = `run.args` ++ each parameter in declared order (`<value>` or
//! `<flag> <value>`; empty optional ones skipped) ++ extra arguments.

use std::collections::BTreeMap;

use crate::manifest::{ParamKind, ParamValue, RunSpec};

/// Builds the argument list, or explains in Turkish what is missing/wrong.
pub fn build_args(
    run: &RunSpec,
    values: &BTreeMap<String, String>,
    extra: &str,
) -> Result<Vec<String>, String> {
    let mut args = run.args.clone();
    for param in &run.params {
        let given = values
            .get(&param.name)
            .map(|v| v.trim())
            .filter(|v| !v.is_empty())
            .map(str::to_string);
        let value = match (given, &param.default) {
            (Some(v), _) => v,
            (None, Some(ParamValue::Number(n))) => n.to_string(),
            (None, Some(ParamValue::Text(s))) => s.clone(),
            (None, None) => return Err(format!("\"{}\" gerekli", param.label)),
        };
        if param.kind == ParamKind::Number && value.parse::<i64>().is_err() {
            return Err(format!("\"{}\" bir tam sayı olmalı", param.label));
        }
        if let Some(flag) = &param.flag {
            args.push(flag.clone());
        }
        args.push(value);
    }
    args.extend(split_args(extra)?);
    Ok(args)
}

/// Splits a line like a shell would: whitespace separates, `"..."` and
/// `'...'` group, `\` escapes the next character outside single quotes.
pub fn split_args(line: &str) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut in_word = false;
    let mut chars = line.chars();
    while let Some(c) = chars.next() {
        match c {
            c if c.is_whitespace() => {
                if in_word {
                    out.push(std::mem::take(&mut current));
                    in_word = false;
                }
            }
            '"' => {
                in_word = true;
                loop {
                    match chars.next() {
                        Some('"') => break,
                        Some('\\') => match chars.next() {
                            Some(e @ ('"' | '\\')) => current.push(e),
                            Some(e) => {
                                current.push('\\');
                                current.push(e);
                            }
                            None => return Err("Kapanmamış tırnak (\")".into()),
                        },
                        Some(ch) => current.push(ch),
                        None => return Err("Kapanmamış tırnak (\")".into()),
                    }
                }
            }
            '\'' => {
                in_word = true;
                loop {
                    match chars.next() {
                        Some('\'') => break,
                        Some(ch) => current.push(ch),
                        None => return Err("Kapanmamış tırnak (')".into()),
                    }
                }
            }
            '\\' => {
                in_word = true;
                // A lone backslash before whitespace/end is kept (Windows paths).
                match chars.next() {
                    Some(e) if e == '"' || e == '\'' || e == '\\' || e == ' ' => current.push(e),
                    Some(e) => {
                        current.push('\\');
                        current.push(e);
                    }
                    None => current.push('\\'),
                }
            }
            c => {
                in_word = true;
                current.push(c);
            }
        }
    }
    if in_word {
        out.push(current);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{HostMode, RunParam};

    fn run(params: Vec<RunParam>, args: &[&str]) -> RunSpec {
        RunSpec {
            host: HostMode::Terminal,
            exe: BTreeMap::new(),
            dev: None,
            args: args.iter().map(|s| s.to_string()).collect(),
            params,
        }
    }

    fn param(
        name: &str,
        kind: ParamKind,
        flag: Option<&str>,
        default: Option<ParamValue>,
    ) -> RunParam {
        RunParam {
            name: name.into(),
            kind,
            label: name.to_uppercase(),
            flag: flag.map(str::to_string),
            default,
        }
    }

    fn values(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn gitdash_style_positional_params_with_default() {
        let spec = run(
            vec![
                param("repo", ParamKind::Folder, None, None),
                param(
                    "days",
                    ParamKind::Number,
                    None,
                    Some(ParamValue::Number(180)),
                ),
            ],
            &[],
        );
        assert_eq!(
            build_args(&spec, &values(&[("repo", "C:\\My Repos\\proje")]), "").unwrap(),
            ["C:\\My Repos\\proje", "180"]
        );
        assert_eq!(
            build_args(&spec, &values(&[("repo", "/r"), ("days", " 60 ")]), "").unwrap(),
            ["/r", "60"]
        );
    }

    #[test]
    fn required_and_number_checks() {
        let spec = run(
            vec![
                param("repo", ParamKind::Folder, None, None),
                param(
                    "days",
                    ParamKind::Number,
                    None,
                    Some(ParamValue::Number(180)),
                ),
            ],
            &[],
        );
        assert_eq!(
            build_args(&spec, &values(&[]), "").unwrap_err(),
            "\"REPO\" gerekli"
        );
        assert_eq!(
            build_args(&spec, &values(&[("repo", "/r"), ("days", "altmış")]), "").unwrap_err(),
            "\"DAYS\" bir tam sayı olmalı"
        );
    }

    #[test]
    fn docky_style_fixed_args_flags_and_extra() {
        let spec = run(
            vec![param(
                "model",
                ParamKind::Text,
                Some("--model"),
                Some(ParamValue::Text("base".into())),
            )],
            &["generate"],
        );
        assert_eq!(
            build_args(&spec, &values(&[]), "--output \"docker compose.yml\" -v").unwrap(),
            [
                "generate",
                "--model",
                "base",
                "--output",
                "docker compose.yml",
                "-v"
            ]
        );
    }

    #[test]
    fn splitting_like_a_shell() {
        assert_eq!(split_args("").unwrap(), Vec::<String>::new());
        assert_eq!(split_args("  a  b ").unwrap(), ["a", "b"]);
        assert_eq!(
            split_args(r#""a b" 'c d' e\ f"#).unwrap(),
            ["a b", "c d", "e f"]
        );
        assert_eq!(
            split_args(r#"say "it's" 'x"y'"#).unwrap(),
            ["say", "it's", "x\"y"]
        );
        assert_eq!(split_args(r"C:\Users\burak").unwrap(), [r"C:\Users\burak"]);
        assert_eq!(split_args(r#""" ''"#).unwrap(), ["", ""]);
        assert!(split_args("\"açık").is_err());
        assert!(split_args("'açık").is_err());
    }
}
