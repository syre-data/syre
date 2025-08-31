#!/bin/bash
program_basename=syre-project-daemon
root=../..
releases=${root}/target/release
crate_release_dir=${root}/target/release
target=$(rustc -Vv | grep host | cut -f2 -d' ')
target_file=${program_basename}-${target}
target_out=${releases}/${target_file}

mkdir -p ${releases}
cargo build --release -F server

# copy to other packages
lang=${root}/lang
python_path=${lang}/python/src/syre/bin
r_path=${lang}/r/inst
mkdir -p ${python_path}
mkdir -p ${r_path}

cp ${target_out} ${python_path}
cp ${target_out} ${r_path}

chmod a+x ${python_path}/${target_file} ${r_path}/${target_file}
