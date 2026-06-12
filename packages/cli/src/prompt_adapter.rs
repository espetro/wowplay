//! Terminal prompt adapter using `dialoguer`.

use std::path::{Path, PathBuf};

use dialoguer::{Input, MultiSelect, Select};
use wow_silicon_core::adapters::errors::LaunchError;
use wow_silicon_core::{PromptItem, PromptPort};

pub struct DialoguerPrompt;

impl PromptPort for DialoguerPrompt {
    fn select_one(
        &self,
        prompt: &str,
        items: &[PromptItem],
        default: Option<usize>,
    ) -> Result<usize, LaunchError> {
        let labels: Vec<String> = items.iter().map(format_item).collect();
        let mut select = Select::new();
        select = select.with_prompt(prompt).items(&labels);
        if let Some(d) = default {
            select = select.default(d);
        }
        select
            .interact()
            .map_err(|e| LaunchError::SetupFailed(format!("prompt failed: {e}")))
    }

    fn select_many(
        &self,
        prompt: &str,
        items: &[PromptItem],
        defaults: &[bool],
    ) -> Result<Vec<usize>, LaunchError> {
        let labels: Vec<String> = items.iter().map(format_item).collect();
        MultiSelect::new()
            .with_prompt(prompt)
            .items(&labels)
            .defaults(defaults)
            .interact()
            .map_err(|e| LaunchError::SetupFailed(format!("prompt failed: {e}")))
    }

    fn input_path(
        &self,
        prompt: &str,
        must_exist: bool,
        default: Option<&Path>,
    ) -> Result<PathBuf, LaunchError> {
        let mut input: Input<String> = Input::new();
        input = input.with_prompt(prompt);
        if let Some(d) = default {
            input = input.default(d.display().to_string());
        }
        if must_exist {
            input = input.validate_with(|s: &String| -> Result<(), &str> {
                if Path::new(s).exists() {
                    Ok(())
                } else {
                    Err("path does not exist")
                }
            });
        }
        let value = input
            .interact_text()
            .map_err(|e| LaunchError::SetupFailed(format!("prompt failed: {e}")))?;
        Ok(PathBuf::from(value))
    }
}

pub struct HeadlessPrompt {
    pub runner: Option<String>,
    pub wow_dir: Option<PathBuf>,
    pub enable_lib_silicon: bool,
}

impl PromptPort for HeadlessPrompt {
    fn select_one(
        &self,
        _prompt: &str,
        items: &[PromptItem],
        default: Option<usize>,
    ) -> Result<usize, LaunchError> {
        if let Some(ref name) = self.runner {
            return items
                .iter()
                .position(|i| i.label.eq_ignore_ascii_case(name))
                .ok_or_else(|| {
                    LaunchError::SetupFailed(format!(
                        "unknown runner '{}'; valid: crossover, whisky, moonshine",
                        name
                    ))
                });
        }
        default.ok_or_else(|| {
            LaunchError::SetupFailed("setup requires --runner in non-interactive mode".into())
        })
    }

    fn select_many(
        &self,
        _prompt: &str,
        items: &[PromptItem],
        _defaults: &[bool],
    ) -> Result<Vec<usize>, LaunchError> {
        if self.enable_lib_silicon {
            Ok(items
                .iter()
                .enumerate()
                .filter(|(_, i)| i.label.contains("libSiliconPatch"))
                .map(|(idx, _)| idx)
                .collect())
        } else {
            Ok(vec![])
        }
    }

    fn input_path(
        &self,
        _prompt: &str,
        must_exist: bool,
        default: Option<&Path>,
    ) -> Result<PathBuf, LaunchError> {
        let path = self
            .wow_dir
            .clone()
            .or_else(|| default.map(|p| p.to_path_buf()))
            .ok_or_else(|| {
                LaunchError::SetupFailed("setup requires --wow-dir in non-interactive mode".into())
            })?;
        if must_exist && !path.exists() {
            return Err(LaunchError::SetupFailed(format!(
                "wow-dir does not exist: {}",
                path.display()
            )));
        }
        Ok(path)
    }
}

fn format_item(item: &PromptItem) -> String {
    match &item.detail {
        Some(d) => format!("{} — {}", item.label, d),
        None => item.label.clone(),
    }
}
