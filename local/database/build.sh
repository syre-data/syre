#!/bin/bash

root=../..
releases=${root}/target/release
target=$(rustc -Vv | grep host | cut -f2 -d' ')
target_out=${releases}/thot-local-database-${target}

mkdir -p ${dir}
cargo build --release -F server
mv ${releases}/thot-local-database ${target_out}

# copy to other packages
lang=${root}/lang
cp ${target_out} ${lang}/python/python/thot/package_data
cp ${target_out} ${lang}/r/inst

