mod copy_out_stream;

use crate::copy_out_stream::BCOStream;
use chrono::Utc;
use clap::{Parser, Subcommand};
use native_tls::TlsConnector;
use postgres_native_tls::MakeTlsConnector;
use postgres_protocol::types::{
    bool_from_sql, char_from_sql, int2_from_sql, int4_from_sql, int8_from_sql, oid_from_sql,
    text_from_sql,
};
use regex::Regex;
use serde_json::json;
use std::{
    error::Error,
    io::{stdout, BufWriter, Write},
};
use tokio_postgres::types::{FromSql, Type};
use tokio_stream::StreamExt;

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

        query: String,
    },
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    match args.command {
        Commands::Extract { url, query } => {
            let builder = TlsConnector::builder().build()?;
            let connector = MakeTlsConnector::new(builder);
            let (client, connection) = tokio_postgres::connect(&url, connector).await?;

            tokio::spawn(async move {
                if let Err(e) = connection.await {
                    eprintln!("connection error: {}", e);
                }
            });

            // On check que la query est compilable
            let s = client.prepare(&query).await?;

            let re = Regex::new(r";\s*$").unwrap();
            let q = re.replace(&query, "");
            let qq = format!("copy ({}) to stdout (format binary)", q);

            let copy_stream = client.copy_out(&qq).await?;

            let bco_stream = BCOStream::new(copy_stream);

            let mut out = BufWriter::new(stdout().lock());

            tokio::pin!(bco_stream);
            while let Some(_msg) = bco_stream.next().await {
                let row = _msg.unwrap();

                let json_row_elems = s.columns().iter().zip(row.fields).map(|(col, val)| {
                    let json_value = match val {
                        None => Ok(serde_json::Value::Null),
                        Some(val) => match col.type_() {
                            &Type::TEXT | &Type::NAME | &Type::VARCHAR => {
                                text_from_sql(&val).map(|v| json!(v))
                            }
                            &Type::OID | &Type::REGPROC | &Type::REGTYPE | &Type::REGCLASS => {
                                oid_from_sql(&val).map(|v| json!(v))
                            }
                            &Type::BOOL => bool_from_sql(&val).map(|v| json!(v)),
                            &Type::CHAR => char_from_sql(&val).map(|v| json!(v)),
                            &Type::INT2 => int2_from_sql(&val).map(|v| json!(v)),
                            &Type::INT4 => int4_from_sql(&val).map(|v| json!(v)),
                            &Type::INT8 => int8_from_sql(&val).map(|v| json!(v)),
                            &Type::JSONB => serde_json::Value::from_sql(&Type::JSONB, &val),
                            &Type::TIMESTAMPTZ => chrono::DateTime::from_sql(
                                &Type::TIMESTAMPTZ,
                                &val,
                            )
                            .map(|v: chrono::DateTime<Utc>| {
                                json!(v.to_rfc3339_opts(chrono::SecondsFormat::AutoSi, true))
                            }),
                            &Type::TIMESTAMP => {
                                chrono::NaiveDateTime::from_sql(&Type::TIMESTAMP, &val)
                                    .map(|v| json!(v.format("%Y-%m-%dT%H:%M:%S%.f").to_string()))
                            }
                            &Type::DATE => chrono::NaiveDate::from_sql(&Type::DATE, &val)
                                .map(|v| json!(v.format("%Y-%m-%d").to_string())),
                            &Type::UUID => uuid::Uuid::from_sql(&Type::UUID, &val)
                                .map(|v| json!(v.to_string())),
                            _ => Err(format!("Unsupported type {}", col.type_().name()).into()),
                        },
                    };

                    (col.name().to_string(), json_value)
                });

                let mut out_map = serde_json::Map::new();

                let mut err_count = 0;
                for (k, v) in json_row_elems {
                    match v {
                        Ok(v) => {
                            out_map.insert(k, v);
                        }
                        Err(e) => {
                            err_count += 1;
                            eprintln!("Can't extract field {}: {}", k, e)
                        }
                    };
                }
                if err_count > 0 {
                    return Err("Invalid fields (see above). You may want to file a bug report at https://github.com/n-e/pgextract".into());
                }

                serde_json::to_writer(&mut out, &out_map)?;
                out.write_all(b"\n").unwrap();
            }

            Ok(())
        }
    }
}
