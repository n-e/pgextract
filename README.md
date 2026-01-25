# pgextract - Load & Extract newline-delimited JSON from PostgreSQL

## Download

Get the binary for your platform on the releases page

## Run

```bash
pgextract extract --url postgres://postgres:example@localhost 'select * from t' > out.ndjson
```

## Why pgextract?

- A well-defined file-format. No more CSVs
- Export or Load huge tables
- Great Performance (>200MB/s in many cases). You will be limited by the I/O speed.

## Recipes

Coming Soon
