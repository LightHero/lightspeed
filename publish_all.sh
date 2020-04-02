#!/usr/bin/env bash

## exit if something fails
set -e

declare -a publish_list=(
    "logger"
    "scheduler"
    "cache"
    "core"
    "auth"
    "cms"
    "email"
)

for i in "${publish_list[@]}"
do
    LINE_SEPARATOR='--------------------------------------------------------'

    cd $i
    echo $LINE_SEPARATOR
    echo 'C3p0 - Run Cargo publish for [' $i ']'
    echo $LINE_SEPARATOR

    cargo publish
    sleep 20
    cd ..
    rc=$?
    if [[ $rc -ne 0 ]] ; then
        echo "Failure publishing $i";
    fi

done
