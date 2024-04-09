use cron_job::{CronJob, Result};
use std::process::Command;

fn create_cron_job(schedule: &str, name: &str, command: &str) -> Result<CronJob> {
    Ok(CronJob::new(schedule, name, || {
        let output = Command::new("/usr/bin/php")
            .arg("/var/www/html/protected/yii")
            .arg(command)
            .output()?;
        println!("{}", String::from_utf8_lossy(&output.stdout));
        Ok(())
    })?)
}

fn cron() -> Result<()> {
    // Define cron jobs
    let cron_jobs = vec![
        create_cron_job("* * * * *", "humhub_queue_run", "queue/run")?,
        create_cron_job("* * * * *", "humhub_cron_run", "cron/run")?,
    ];

    // Set up cron jobs
    for job in cron_jobs {
        match job.schedule() {
            Ok(_) => println!("Cron job '{}' is scheduled successfully.", job.name()),
            Err(e) => eprintln!("Error scheduling cron job '{}': {}", job.name(), e),
        }
    }

    Ok(())
}

fn main() {
    if let Err(err) = cron() {
        eprintln!("Error: {}", err);
    }
}
