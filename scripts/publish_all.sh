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
    sleep 20
    cd ..
    rc=$?
    if [[ $rc -ne 0 ]] ; then
        echo "Failure publishing $i";
    fi

done
