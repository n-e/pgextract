# pgextract

`pgextract` is a command-line utility that loads or extracts large or small amounts of data from PostgreSQL in various formats.

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

`pgextract` can do 200MB/s+, and its performance is almost always limited by how fast the PostgreSQL server can stream the data.

## Supported formats

Right now `pgextract` only supports extracting data to NDJSON, but I plan to add relevant formats shortly.

| Format                 | Load |  Extract |
| ---------------------- | ---- | -------- |
| Newline-delimited JSON | ❌   | ✅       |
