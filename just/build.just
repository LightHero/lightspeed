
# Builds all artifacts
[group('build')]
build:
  cargo build
  just for_each_feature "cargo build --features "
  cargo build --all-features

# Publishes all artifacts on crates.io
[group('build')]
publish:
  #!/usr/bin/env bash

  ## exit if something fails
  set -e

  declare -a publish_list=(
      "logger"
      "test_utils"
      "scheduler"
      "cache"
      "core"
      "auth"
      "cms"
      "email"
      "file_store"
      "hash"
      "lightspeed"
  )

  echo 'Execute before publishing'
  just test

  for i in "${publish_list[@]}"
  do
      LINE_SEPARATOR='--------------------------------------------------------'

      cd $i
      echo $LINE_SEPARATOR
      echo 'Run Cargo publish for [' $i ']'
      echo $LINE_SEPARATOR

      cargo publish

      cd ..
      rc=$?
      if [[ $rc -ne 0 ]] ; then
          echo "Failure publishing $i";
      fi

  done


[private]
for_each_feature command:
  #!/usr/bin/env bash
  
  ## exit if something fails
  set -e

  declare -a features=(
    "actix_web"
    "axum"
    "poem"
    "poem_openapi"
    "postgres"
    "sqlx_postgres"
    "sqlx_mysql"
    "sqlx_sqlite"
  )

  for i in "${features[@]}"
  do
      LINE_SEPARATOR='--------------------------------------------------------'

      echo $LINE_SEPARATOR
      {{command}} $i
      echo $LINE_SEPARATOR

  done