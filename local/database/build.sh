#!/bin/bash

root=../..
releases=${root}/target/release
target=$(rustc -Vv | grep host | cut -f2 -d' ')
target_file=thot-local-database-${target}
target_out=${releases}/${target_file}

mkdir -p ${dir}
cargo build --release -F server
mv ${releases}/thot-local-database ${target_out}

# copy to other packages
lang=${root}/lang
python_path=${lang}/python/python/thot/package_data
r_path=${lang}/r/inst

cp ${target_out} ${python_path}
cp ${target_out} ${r_path}

chmod a+x ${python_path}/${target_file} ${r_path}/${target_file}
