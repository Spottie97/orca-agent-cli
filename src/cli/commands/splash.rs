use std::thread;
use std::time::Duration;

use anyhow::Result;
use clap::Args;

const CYAN: &str = "\x1b[96m";
const WHITE: &str = "\x1b[97m";
const DIM: &str = "\x1b[2m";
const BOLD: &str = "\x1b[1m";
const GREEN: &str = "\x1b[92m";
const RESET: &str = "\x1b[0m";
const CLEAR: &str = "\x1b[2J\x1b[H";

const SPLASH_ART: &str = r#"
                         ___
                       /     \
                      /  >_   \
                     /    --    \
                    /   ______   \
                   /___/      \___\
                      \      /
                       \    /
                        \  /
                         \/
"#;

const BOOT_STEPS: &[(&str, &str)] = &[
    ("SYSTEM ", "Initializing agent runtime"),
    ("MEMORY ", "Loading context store"),
    ("TOOLS  ", "Registering tool registry"),
    ("PLANNER", "Bootstrapping task planner"),
    ("AGENT  ", "Orca is ready"),
];

#[derive(Args)]
pub struct SplashArgs {
    #[arg(long, help = "Skip animation delays")]
    pub no_wait: bool,
}

pub fn run(args: SplashArgs) -> Result<()> {
    print!("{}", CLEAR);

    // Banner
    println!(
        "{}{}  ╔══════════════════════════════════════════════════════════════════════╗{}",
        CYAN, BOLD, RESET
    );
    println!(
        "{}{}  ║                                                                      ║{}",
        CYAN, BOLD, RESET
    );
    for line in SPLASH_ART.lines() {
        if line.trim().is_empty() {
            continue;
        }
        println!("{}{}  ║  {}{:<68}║{}", CYAN, BOLD, WHITE, line, RESET);
    }
    println!(
        "{}{}  ║           A G E N T   C L I   v1.0.0                                 ║{}",
        CYAN, BOLD, RESET
    );
    println!(
        "{}{}  ╚══════════════════════════════════════════════════════════════════════╝{}",
        CYAN, BOLD, RESET
    );

    // Boot steps
    for (label, msg) in BOOT_STEPS {
        print!(
            "  {}[ {}{:<7}{} {}]{}  {}...",
            DIM, CYAN, label, RESET, DIM, RESET, msg
        );
        std::io::Write::flush(&mut std::io::stdout())?;

        if !args.no_wait {
            thread::sleep(Duration::from_millis(400));
        }

        println!("  {}[ ✓ ]{}", GREEN, RESET);
    }

    println!(
        "\n  {}Type {}help{}{} for commands.  Ctrl+C to exit.{}\n",
        DIM, CYAN, RESET, DIM, RESET
    );
    println!("  {}{}>_{} ", CYAN, BOLD, RESET);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_splash_art_non_empty() {
        assert!(!SPLASH_ART.is_empty());
        assert!(!SPLASH_ART.contains("\x1b["));
    }

    #[test]
    fn test_boot_steps_present() {
        assert_eq!(BOOT_STEPS.len(), 5);
        assert_eq!(BOOT_STEPS[0].0, "SYSTEM ");
        assert_eq!(BOOT_STEPS[4].0, "AGENT  ");
    }

    #[test]
    fn test_splash_runs_no_wait() {
        let args = SplashArgs { no_wait: true };
        assert!(run(args).is_ok());
    }
}
