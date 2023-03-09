#! /bin/bash
# Builds a release of the project.
# Copies the binaries to a `thot-desktop-releases` folder,
# Renaming the files as `<arch>-<vendor>-<system>-<subsystem>--<major_version>_<minor_version>_<patch_version>.<ext>`.
#
# Accepts a relative path to the folder containing the signing files as the first argument.

# collect info
base_path=$1
key_path=$(realpath $(pwd)/$base_path/tauri.key)
key_pwd_path=$base_path/tauri.key.pwd
key_pwd=$(cat $key_pwd_path)
target=$(rustc -Vv | grep host | cut -f2 -d' ')

# build app
export TAURI_PRIVATE_KEY=$key_path
export TAURI_KEY_PASSWORD=$key_pwd

res=$(cargo tauri build 2>&1)
echo $res > ~/Downloads/tauri_build.log

# rename
pub_dir=thot-desktop-releases
file_delimeter="_"
bundle_key="Finished * bundles at:"
signature_key="(updater) Info 1 updater archive at: Info "
bundles=${res#*$bundle_key}
bundles=( ${bundles//$signature_key/} )

echo $bundles
for file_path in "${bundles[@]}"
do
    echo "parsing $file_path"

    # parse file path
    base_path=$(dirname $file_path)
    filename=$(basename $file_path)
    filename_components=( ${filename//$file_delimeter/ } )

    # create new path
    version=${filename_components[1]//./_}
    ext=${filename_components[2]#*.}
    new_base_path=$(realpath $base_path/$pub_dir)
    new_path=$new_base_path/$target--$version.$ext

    # copy file
    mkdir -p $new_base_path
    cp $file_path $new_path
done
