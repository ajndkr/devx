use crate::{CliResult, Command};
use clap::Subcommand;
use colored::Colorize;
use std::process;
use which::which;

use crate::CliError;

// define subcommands for 'git' command
#[derive(Subcommand)]
pub enum GitCommands {
    #[command(about = "sync latest changes from remote branch")]
    Sync {},
}

// map 'git' subcommands to functions
impl Command for GitCommands {
    fn execute(&self) -> CliResult<()> {
        match self {
            GitCommands::Sync {} => sync(),
        }
    }
}

// run git command with arguments
//
// errors:
// - CliError::Command: if the git command fails
fn run_git_command(
    args: &[&str],
    error_msg: &str,
    capture_output: bool,
) -> Result<std::process::Output, CliError> {
    let mut cmd = process::Command::new("git");
    cmd.args(args);

    if capture_output {
        cmd.output()
            .map_err(|e| CliError::Command(format!("{}: {}", error_msg, e)))
    } else {
        cmd.stdout(process::Stdio::inherit())
            .stderr(process::Stdio::inherit())
            .status()
            .map_err(|e| CliError::Command(format!("{}: {}", error_msg, e)))
            .and_then(|_| {
                cmd.output()
                    .map_err(|e| CliError::Command(format!("{}: {}", error_msg, e)))
            })
    }
}

// sync latest changes from remote branch.
//
// workflow:
//  stage local -> fetch remote -> stash local -> pull changes
//  -> restore (and clear) stash -> unstage local
//
// errors:
// - CliError::Command: if the binary file cannot be found
// - CliError::IOError: if the binary file cannot be removed
pub fn sync() -> CliResult<()> {
    which("git").expect("git not found. install git and try again.");

    println!("{}", "running git sync workflow.".bold());

    let git_check = run_git_command(
        &["rev-parse", "--git-dir"],
        "failed to execute git command",
        true,
    )?;
    if !git_check.status.success() {
        println!("current directory is not a git repository. nothing to sync.");
        return Ok(());
    }

    let remote_status = run_git_command(
        &["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"],
        "failed to get upstream branch",
        true,
    )?;
    if !remote_status.status.success() {
        println!("no upstream branch found. nothing to sync.");
        return Ok(());
    }

    println!("{}", "staging local changes.".bold());
    run_git_command(&["add", "."], "failed to stage local changes", false)?;

    println!("{}", "fetching remote changes.".bold());
    run_git_command(&["fetch", "-p"], "failed to fetch remote changes", false)?;

    println!("{}", "stashing local changes.".bold());
    run_git_command(&["stash"], "failed to stash local changes", false)?;

    println!("{}", "pulling remote changes.".bold());
    run_git_command(
        &["pull", "--rebase"],
        "failed to pull remote changes",
        false,
    )?;

    println!("{}", "restoring local changes.".bold());
    run_git_command(&["stash", "pop"], "failed to restore local changes", false)?;
    run_git_command(&["stash", "clear"], "failed to clear stash", false)?;

    println!("{}", "unstaging local changes.".bold());
    run_git_command(&["reset"], "failed to unstage local changes", false)?;

    println!("{}", "git sync complete!".bold());

    let git_log_output = run_git_command(
        &["log", "-1", "--oneline"],
        "failed to get latest commit",
        true,
    )?;
    let latest_commit = String::from_utf8_lossy(&git_log_output.stdout)
        .trim()
        .to_string();

    println!("latest commit: {}", latest_commit.dimmed());

    Ok(())
}
