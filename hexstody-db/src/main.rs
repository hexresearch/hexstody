use clap::Parser;
use std::error::Error;
use sqlx::postgres::PgPoolOptions;
use log::*;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    /// PostgreSQL connection string
    #[clap(
        long,
        short,
        default_value = "postgres://hexstody:hexstody@localhost/hexstody",
        env = "HEXSTODY_POSTGRES"
    )]
    dbconnect: String,
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser, Debug)]
enum SubCommand {
    /// Apply migrations to the given database
    Migrate,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    env_logger::init();

    match args.subcmd {
        SubCommand::Migrate => {
            info!("Connecting to database");
            let pool = PgPoolOptions::new()
                .max_connections(1)
                .connect(&args.dbconnect)
                .await?;

            info!("Applying migrations");
            sqlx::migrate!("./migrations").run(&pool).await?;
            info!("Done");
        }
    }
    Ok(())
}
