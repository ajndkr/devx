use crate::{CliResult, Command};
use clap::Subcommand;
use colored::Colorize;
use inquire::Select;
use std::process;
use which::which;

use crate::CliError;

// define subcommands for 'git' command
#[derive(Subcommand)]
pub enum GitCommands {
    #[command(about = "sync latest changes from remote")]
    Sync {},
    #[command(about = "switch local branch")]
    Switch {},
    #[command(about = "delete a local branch")]
    Delete {},
}

// map 'git' subcommands to functions
impl Command for GitCommands {
    fn execute(&self) -> CliResult<()> {
        match self {
            GitCommands::Sync {} => sync(),
            GitCommands::Switch {} => switch_branch(),
            GitCommands::Delete {} => delete_branch(),
        }
    }
}

// run git command with arguments
//
// errors:
// - CliError::Command: if the git command fails
fn git_exec(
    args: &[&str],
    error_msg: &str,
    capture_output: bool,
) -> Result<process::Output, CliError> {
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
            .map(|status| process::Output {
                status,
                stdout: Vec::new(),
                stderr: Vec::new(),
            })
    }
}

// sync latest changes from remote branch
//
// panics: if git is not installed
//
// errors:
// - CliError::Command: if any git command fails
pub fn sync() -> CliResult<()> {
    which("git").expect("git not found. install git and try again.");

    let git_check = git_exec(
        &["rev-parse", "--git-dir"],
        "failed to execute git command",
        true,
    )?;
    if !git_check.status.success() {
        println!("current directory is not a git repository. nothing to sync.");
        return Ok(());
    }

    let remote_status = git_exec(
        &["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"],
        "failed to get upstream branch",
        true,
    )?;
    if !remote_status.status.success() {
        println!("no upstream branch found. nothing to sync");
        return Ok(());
    }

    println!("{}", "checking local branch status".bold());
    let mut local_changes_stashed = false;
    let git_status = git_exec(&["status", "--porcelain"], "failed to get git status", true)?;
    if !git_status.stdout.is_empty() {
        println!("- local changes found. stashing local changes");
        git_exec(&["add", "."], "failed to stage local changes", false)?;
        git_exec(&["stash"], "failed to stash local changes", false)?;
        local_changes_stashed = true;
    }

    println!("{}", "syncing changes with upstream branch".bold());
    git_exec(&["fetch", "-p"], "failed to fetch remote changes", false)?;
    git_exec(
        &["pull", "--rebase"],
        "failed to pull remote changes",
        false,
    )?;

    let git_log_output = git_exec(
        &["log", "-1", "--oneline"],
        "failed to get latest commit",
        true,
    )?;
    let latest_commit = String::from_utf8_lossy(&git_log_output.stdout)
        .trim()
        .to_string();

    println!("- latest commit: {}", latest_commit.dimmed());

    if local_changes_stashed {
        println!("{}", "restoring stashed changes".bold());
        git_exec(&["stash", "pop"], "failed to restore local changes", false)?;
        git_exec(&["stash", "clear"], "failed to clear stash", false)?;

        println!("{}", "unstaging local changes.".bold());
        git_exec(&["reset"], "failed to unstage local changes", false)?;
    }

    println!("{}", "git sync complete ^.^".bold());

    Ok(())
}

// switch local branch
pub fn switch_branch() -> CliResult<()> {
    which("git").expect("git not found. install git and try again.");

    let proc_output = git_exec(
        &["--no-pager", "branch", "--no-color"],
        "failed to get branch list",
        true,
    )?;

    let proc_output_str = String::from_utf8_lossy(&proc_output.stdout);
    let all_branches = proc_output_str
        .lines()
        .map(|line| line.trim())
        .collect::<Vec<&str>>();

    // finds current branch from the above git command output
    // if no branch is found, defaults to 'main'
    let current_branch = all_branches
        .iter()
        .find(|branch| branch.starts_with('*'))
        .map(|branch| branch.trim_start_matches('*').trim())
        .unwrap_or("main");

    println!("{}: {}", "current branch".bold(), current_branch);

    let other_branches = all_branches
        .iter()
        .filter(|branch| !branch.starts_with('*'))
        .map(|branch| branch.trim())
        .collect::<Vec<&str>>();

    let new_branch = Select::new("select new branch:", other_branches).prompt()?;

    println!("{}", "checking local branch status".bold());
    let mut local_changes_stashed = false;
    let git_status = git_exec(&["status", "--porcelain"], "failed to get git status", true)?;
    if !git_status.stdout.is_empty() {
        println!("- local changes found. stashing local changes");
        git_exec(&["add", "."], "failed to stage local changes", false)?;
        git_exec(&["stash"], "failed to stash local changes", false)?;
        local_changes_stashed = true;
    }

    git_exec(&["checkout", new_branch], "failed to switch branch", false)?;

    if local_changes_stashed {
        println!("{}", "restoring stashed changes".bold());
        git_exec(&["stash", "pop"], "failed to restore local changes", false)?;
        git_exec(&["stash", "clear"], "failed to clear stash", false)?;

        println!("{}", "unstaging local changes.".bold());
        git_exec(&["reset"], "failed to unstage local changes", false)?;
    }

    println!("{}", "branch switch complete ^.^".bold());

    Ok(())
}

// delete a local branch
pub fn delete_branch() -> CliResult<()> {
    which("git").expect("git not found. install git and try again.");

    Ok(())
}
