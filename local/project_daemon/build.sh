#!/bin/bash
program_basename=syre-project-daemon
bin_name=${program_basename}-server
root=../..
release_dir=${root}/target/release
bin=${root}/desktop/src-tauri/bin
lang=${root}/lang

target=$(rustc --print host-tuple)
target_file=${bin_name}-${target}
target_out=${bin}/${target_file}

mkdir -p ${bin}
cargo build --release -F server
cp ${release_dir}/${program_basename} ${target_out}

# copy to other packages
python_path=${lang}/python/src/syre/bin
r_path=${lang}/r/inst
mkdir -p ${python_path}
mkdir -p ${r_path}

cp ${target_out} ${python_path}
cp ${target_out} ${r_path}

chmod a+x ${target_out} ${python_path}/${target_file} ${r_path}/${target_file}
