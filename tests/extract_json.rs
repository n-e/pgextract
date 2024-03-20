use assert_cmd::prelude::*;
use std::process::Command;

macro_rules! test_extract_json {
    ($name: ident, $in: expr, $out: expr) => {
        #[test]
        fn $name() -> Result<(), Box<dyn std::error::Error>> {
            let mut cmd = Command::cargo_bin("pgextract")?;
            let out = cmd
                .args([
                    "extract",
                    "-u",
                    "postgres://postgres:example@localhost",
                    "-f",
                    "ndjson",
                    $in,
                ])
                .unwrap();

            assert_eq!(String::from_utf8(out.stdout).unwrap(), $out);
            Ok(())
        }
    };
}

test_extract_json!(
    two_lines,
    "select * from (values ('a'),('b')) as t(col)",
    r#"{"col":"a"}
{"col":"b"}
"#
);

test_extract_json!(
    text,
    "select 'a'::text col",
    r#"{"col":"a"}
"#
);

test_extract_json!(
    int,
    "select 1234::bigint col",
    r#"{"col":1234}
"#
);

test_extract_json!(
    date,
    "select timestamptz '2023-01-01T12:23:45Z' col",
    r#"{"col":"2023-01-01T12:23:45+00:00"}
"#
);
