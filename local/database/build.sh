#!/bin/bash

root=../..
releases=${root}/target/release
target=$(rustc -Vv | grep host | cut -f2 -d' ')

mkdir -p ${dir}
cargo build --release -F server
mv ${releases}/thot-local-database ${releases}/thot-local-database-${target}

# copy to other packages
lang=${root}/lang
cp ${releases}/thot-local-database-${target} ${lang}/python/python/thot/package_data
cp ${releases}/thot-local-database-${target} ${lang}/r/inst

