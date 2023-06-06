#!/bin/bash

root=../..
releases=${root}/target/release
target=x86_64-apple-darwin
target_file=thot-local-database-${target}
target_out=${releases}/${target_file}

mkdir -p ${dir}
cargo build --release -F server --target ${target}
build_path=${root}/target/${target}/release/thot-local-database
mv ${build_path} ${target_out}
# mv ${releases}/thot-local-database ${target_out}

# copy to other packages
lang=${root}/lang
python_path=${lang}/python/python/thot/package_data
r_path=${lang}/r/inst

cp ${target_out} ${python_path}
cp ${target_out} ${r_path}

chmod a+x ${python_path}/${target_file} ${r_path}/${target_file}
