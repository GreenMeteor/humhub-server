use cron_job::{CronJob, Result};
use std::process::{Command, Output};
use std::io::{self, Write};
use log::{info, error, LevelFilter};
use simplelog::{TermLogger, Config, TerminalMode};

fn create_cron_job(schedule: &str, name: &str, command: &str) -> Result<CronJob> {
    CronJob::new(schedule, name, || {
        match run_command(command) {
            Ok(output) => {
                info!("{}", String::from_utf8_lossy(&output.stdout));
                Ok(())
            }
            Err(err) => {
                error!("Failed to run cron job '{}': {}", name, err);
                Err(err)
            }
        }
    })
}

fn run_command(command: &str) -> Result<Output> {
    let output = Command::new("/usr/bin/php")
        .arg("/var/www/html/protected/yii")
        .arg(command)
        .output();

    match output {
        Ok(out) => {
            if !out.status.success() {
                let err_msg = format!(
                    "Command '{}' failed with status: {}",
                    command, out.status
                );
                error!("{}", err_msg);
                Err(io::Error::new(io::ErrorKind::Other, err_msg).into())
            } else {
                Ok(out)
            }
        }
        Err(err) => {
            let err_msg = format!("Error running command '{}': {}", command, err);
            error!("{}", err_msg);
            Err(io::Error::new(io::ErrorKind::Other, err_msg).into())
        }
    }
}

fn cron() -> Result<()> {
    // Initialize logging
    TermLogger::init(LevelFilter::Info, Config::default(), TerminalMode::Mixed)?;

    // Define cron jobs
    let cron_jobs = vec![
        create_cron_job("* * * * *", "humhub_queue_run", "queue/run")?,
        create_cron_job("* * * * *", "humhub_cron_run", "cron/run")?,
    ];

    // Set up cron jobs
    for job in cron_jobs {
        match job.schedule() {
            Ok(_) => info!("Cron job '{}' is scheduled successfully.", job.name()),
            Err(e) => error!("Error scheduling cron job '{}': {}", job.name(), e),
        }
    }

    Ok(())
}

fn main() {
    if let Err(err) = cron() {
        error!("Error: {}", err);
    }
}
