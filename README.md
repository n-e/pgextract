# pgextract

`pgextract` is a command-line utility to load or extract large or small amounts data from PostgreSQL in various formats.

## Download

Get the binary for your platform from the releases page

## Run

```bash
pgextract extract --url postgres://postgres:example@localhost  --format ndjson 'select * from t'   > out.ndjson
```

## Rationale

Unlike many databases, the `psql` CLI only supports text, csv and proprietary binary input and input formats via the `COPY` command.

`pgextract` aims to fill this gap, by supporting input and output in common big data formats.

## Performance

Same as psql with quick testing

## Supported formats

| Format                 | Load |  Extract |
| ---------------------- | ---- | -------- |
| Newline-delimited JSON | ❌   | ✅       |
