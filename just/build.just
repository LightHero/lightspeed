
# Builds all artifacts
[group('build')]
build:
  cargo build
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
      "validator"
      "cache"
      "core"
      "auth"
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
