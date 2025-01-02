
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
  set -x

  declare -a publish_list=(
      "logger"
      "scheduler"
      "cache"
      "core"
      "auth"
      # "cms"
      "email"
      # "file_store"
      "hash"
      "lightspeed"
  )

  echo 'Execute before publishing'
  ./scripts/test.sh

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
    "axum"
    "mysql"
    "poem"
    "poem_openapi"
    "postgres"
    "sqlite"
  )

  for i in "${features[@]}"
  do
      LINE_SEPARATOR='--------------------------------------------------------'

      echo $LINE_SEPARATOR
      {{command}} $i
      echo $LINE_SEPARATOR

  done