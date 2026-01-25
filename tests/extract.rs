use assert_cmd::cargo;
use assert_cmd::prelude::*;
use std::process::Command;

macro_rules! mt {
    ($name: ident, $in: expr, $out: expr) => {
        #[test]
        fn $name() -> Result<(), Box<dyn std::error::Error>> {
            let mut cmd = Command::new(cargo::cargo_bin!("pgextract"));
            let out = cmd
                .args([
                    "extract",
                    "-u",
                    "postgres://postgres:example@localhost",
                    $in,
                ])
                .unwrap();

            assert_eq!(String::from_utf8(out.stdout).unwrap(), $out);
            Ok(())
        }
    };
}

mt!(
    two_lines,
    "select * from (values ('a'),('b')) as t(col)",
    r#"{"col":"a"}
{"col":"b"}
"#
);

mt!(
    text,
    "select 'a'::text col",
    r#"{"col":"a"}
"#
);

mt!(
    int,
    "select 1234::bigint col",
    r#"{"col":1234}
"#
);

mt!(
    timestamptz,
    "select timestamptz '2023-01-01T12:23:45Z' col",
    r#"{"col":"2023-01-01T12:23:45Z"}
"#
);

mt!(
    timestamp,
    "select timestamp '2023-01-01T12:23:45' col",
    r#"{"col":"2023-01-01T12:23:45"}
"#
);

mt!(
    date,
    "select date '2023-01-01' col",
    r#"{"col":"2023-01-01"}
"#
);

mt!(
    jsonb,
    r#"select '{"str":"","num":2}'::jsonb col"#,
    r#"{"col":{"num":2,"str":""}}
"#
);

// mt!(
//     boolarray,
//     r#"select array[true,false] col"#,
//     r#"{"col":[true,false]}
// "#
// );

mt!(
    preserve_order,
    r#"select 1 a, 2 z, 3 c"#,
    r#"{"a":1,"z":2,"c":3}
"#
);
