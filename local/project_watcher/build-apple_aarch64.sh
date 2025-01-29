#!/bin/bash
program_basename=syre-local-database
root=../..
releases=${root}/target/release
crate_release_dir=target/release
target=aarch64-apple-darwin
target_file=${program_basename}-${target}
target_out=${releases}/${target_file}

mkdir -p ${releases}
cargo build --release -F server --target ${target}
mv ${crate_release_dir}/${program_basename} ${target_out}

# copy to other packages
lang=${root}/lang
python_path=${lang}/python/src/syre/bin
r_path=${lang}/r/inst
mkdir -p ${python_path}
mkdir -p ${r_path}

cp ${target_out} ${python_path}
cp ${target_out} ${r_path}

chmod a+x ${python_path}/${target_file} ${r_path}/${target_file}
