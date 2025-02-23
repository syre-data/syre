#! /bin/bash
# Builds a release of the project.
# Copies the binaries to a `bundles` folder,
# Renaming the files as 
# `syre_desktop--<arch>-<vendor>-<system>-<subsystem>--<major_version>_<minor_version>_<patch_version>[--debug]<ext>`.
#
# Accepts a relative path to the folder containing the signing files as the first argument.


# TODO: See `build.ps1` for updated functionality to match. 
exit 1

# APP_NAME="syre_desktop"
# FILE_DELIMETER="_"
# PUB_DIR=bundles

# # collect info
# env_path=$1
# target=$(rustc -Vv | grep host | cut -f2 -d' ')

# # build app
# export $(cat $env_path | xargs -L 1)
# res=$(cargo tauri build 2>&1)
# echo $res > ~/Downloads/tauri_build.log

# # rename
# BUNDLE_KEY="Finished * bundles at:"
# SIGNATURE_KEY="(updater) Info 1 updater archive at: Info "
# bundles=${res#*$BUNDLE_KEY}
# bundles=( ${bundles//$SIGNATURE_KEY/} )

# echo $bundles
# for file_path in "${bundles[@]}"
# do
#     # parse file path
#     base_path=$(dirname $file_path)
#     filename=$(basename $file_path)
#     filename_components=( ${filename//$FILE_DELIMETER/ } )

#     # create new path
#     version=${filename_components[1]//./_}
#     ext=${filename_components[2]#*.}
#     new_base_path=$(realpath $base_path/$PUB_DIR)
#     new_path=$new_base_path/$APP_NAME--$target--$version.$ext

#     # copy file
#     mkdir -p $new_base_path
#     cp $file_path $new_path
#     echo "Copied $file_path"

# done
