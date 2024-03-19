use std::{
    error::Error,
    io::{stdout, BufWriter, Write},
};

use clap::{Parser, Subcommand, ValueEnum};
use openssl::ssl::{SslConnector, SslMethod};
use postgres::{
    binary_copy::BinaryCopyOutIter, fallible_iterator::FallibleIterator, types::Type, Client,
};
use postgres_openssl::MakeTlsConnector;
use regex::Regex;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Extract {
        #[arg(short, long)]
        url: String,

        #[arg(short, long)]
        format: Format,

        query: String,
    },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Format {
    NDJSON,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    match args.command {
        Commands::Extract { url, format, query } => {
            let builder = SslConnector::builder(SslMethod::tls())?;
            let connector = MakeTlsConnector::new(builder.build());
            let mut client = Client::connect(&url, connector)?;

            // On check que la query est compilable
            let _s = client.prepare(&query)?;
            let re = Regex::new(r";\s+$").unwrap();
            let q = re.replace(&query, "");
            let qq = format!(
                "copy (with q as ({}) select row_to_json(q) from q) to stdout (format binary)",
                q
            );

            let reader = client.copy_out(&qq)?;
            let iter = BinaryCopyOutIter::new(reader, &[Type::TEXT]);

            let mut out = BufWriter::new(stdout().lock());
            iter.for_each(|row| {
                let data: String = row.get(0);
                out.write_all(data.as_bytes()).unwrap();
                Ok(())
            })?;

            Ok(())
        }
    }
}
