#!/usr/bin/env just --justfile

database_name := "archive_witness"

clean-db:
  #!/usr/bin/env bash
  set -e

  pg_command="SELECT pg_terminate_backend(pg_stat_activity.pid) \
    FROM pg_stat_activity \
    WHERE pg_stat_activity.datname = '{{database_name}}' \
      AND pid <> pg_backend_pid();"
  psql -U postgres -h localhost -d postgres -c "$pg_command"
  (
    cd db
    sqlx database drop --force
    sqlx database create
    sqlx migrate run
  )

dev-database:
  #!/usr/bin/env bash
  set -e

  cargo run -- nist import videos --path resources/dev_test_data/videos.csv
  cargo run -- nist import tapes --path resources/dev_test_data/tapes.csv
  cargo run -- news networks add --path resources/dev_test_data/abc_news_network
  cargo run -- news affiliates add --path resources/dev_test_data/wabc_affiliate
  cargo run -- news broadcasts add --path resources/dev_test_data/wabc_broadcast
  cargo run -- releases init --torrent-path resources/torrents
