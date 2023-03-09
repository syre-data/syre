#!/bin/bash

dir=../../target/release
target=$(rustc -Vv | grep host | cut -f2 -d' ')
mkdir -p ${dir}
cargo build --release -F server
mv ${dir}/thot-local-database ${dir}/thot-local-database-${target}
