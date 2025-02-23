# Builds a release of the project.
# Bumps the patch version in `Cargo.toml`, `src-lib/Cargo.toml`, `src-tauri/Cargo.toml`, and `src-tauri/Cargo.toml`.
# Seaches for `.tauri/syre.key` to set the `TAURI_SIGNING_PRIVATE_KEY` environment variable if not already set.
# Copies the binaries to a `bundles` folder renaming the files as 
# `syre_desktop--<arch>-<vendor>-<system>-<subsystem>--<major_version>_<minor_version>_<patch_version>[--debug]<ext>`.
#
# Use the `--debug` flag to build a debug version.
# Use the `--keep-version` flag to keep the same version.

$APP_NAME = "syre_desktop"
$FILE_DELIMETER = "_"
$PUB_DIR = "bundles"
$DEBUG_FLAG = "--debug"
$KEEP_VERSION_FLAG = "--keep-version"
$SIGNATURE_EXT = ".sig"
$PRIVATE_KEY_PATH = ".tauri/syre.key"
$PRIVATE_KEY_KEY = "TAURI_SIGNING_PRIVATE_KEY"
$PRIVATE_KEY_PASSWORD_KEY = "TAURI_SIGNING_PRIVATE_KEY_PASSWORD"
$DESKTOP_TOML_PATH = "Cargo.toml"
$TAURI_TOML_PATH = "src-tauri/Cargo.toml"
$TAURI_CONF_PATH = "src-tauri/tauri.conf.json"
$LIB_TOML_PATH = "src-lib/Cargo.toml"
$BUILD_FILE = "build.out"

function Get-Version-Cargo {
  param (
    [string]$filePath
  )
  $VERSION_PATTERN = "version = *"

  $toml = Get-Content $filePath -Raw
  $toml_lines = $toml -split "`n"
  for ($i = 0; $i -lt $toml_lines.Length; $i++) {
    $line = $toml_lines[$i].Trim()
    if ($line -like $VERSION_PATTERN) {
      $version_line_no = $i
      break
    }
  }
  if ($null -eq $version_line_no) {
    Write-Error "Could not find version line in $filePath."
    exit
  }

  $version_line = $toml_lines[$version_line_no]
  $version = $version_line.Substring($VERSION_PATTERN.Length).Trim().Trim('"')
  return $version
}
function Get-Version-Node {
  param (
    [string]$filePath
  )
  $VERSION_PATTERN = "*""version"": ""*"","

  $json = Get-Content $filePath -Raw
  $json_lines = $json -split "`n"
  for ($i = 0; $i -lt $json_lines.Length; $i++) {
    $line = $json_lines[$i].Trim()
    if ($line -like $VERSION_PATTERN) {
      $version_line_no = $i
      break
    }
  }
  if ($null -eq $version_line_no) {
    Write-Error "Could not find version line in $filePath."
    exit
  }

  $version_line = $json_lines[$version_line_no]
  $version = $version_line -replace ".*""version"": """, ""
  $version = $version.Trim().Trim('",')
  return $version
}

function Update-Version-Cargo {
  param (
    [string]$filePath
  )
  
  $version = Get-Version-Cargo -filePath $filePath
  $version_parts = $version -split "\."
  $major = $version_parts[0]
  $minor = $version_parts[1]
  $patch = [int]$version_parts[2] + 1

  $new_version = "$major.$minor.$patch"
  $toml_lines[$version_line_no] = "version = ""$new_version"""
  Set-Content $filePath -Value ($toml_lines -join "`n")
}
function Update-Version-Node {
  param (
    [string]$filePath
  )
  
  $version = Get-Version-Node -filePath $filePath
  $version_parts = $version -split "\."
  $major = $version_parts[0]
  $minor = $version_parts[1]
  $patch = [int]$version_parts[2] + 1

  $new_version = "$major.$minor.$patch"
  $json_lines[$version_line_no] = "  ""version"": ""$new_version"","
  Set-Content $filePath -Value ($json_lines -join "`n")

  return $new_version
}

# get target
$target_output = rustc -Vv
foreach ($line in $target_output) {
  if ($line -like "host: *") {
    $target = $line.Substring(6)
    break
  }
}

if ($null -eq (Get-Item -Path Env:$PRIVATE_KEY_KEY -ErrorAction SilentlyContinue)) {
  if (Test-Path $PRIVATE_KEY_PATH) {
    $private_key = Get-Content -Path $PRIVATE_KEY_PATH -Raw
    Set-Item -Path "Env:$PRIVATE_KEY_KEY" -Value $private_key
  }
  else {
    Write-Error "Could not find Tauri private signing key file or environment variable '$PRIVATE_KEY_KEY' was not set."
    exit
  }
}

if ($null -eq (Get-Item -Path Env:$PRIVATE_KEY_PASSWORD_KEY -ErrorAction SilentlyContinue)) {
  Write-Error "Environment variable '$PRIVATE_KEY_PASSWORD_KEY' not set."
  exit
}

$desktop_version = Get-Version-Cargo -filePath $DESKTOP_TOML_PATH
$tauri_version = Get-Version-Cargo -filePath $TAURI_TOML_PATH
$lib_version = Get-Version-Cargo -filePath $LIB_TOML_PATH
$node_version = Get-Version-Node -filePath $TAURI_CONF_PATH
if ($desktop_version -ne $tauri_version -or $desktop_version -ne $lib_version -or $desktop_version -ne $node_version) {
  Write-Error "Versions do not match."
  exit
}

if (-not ($args -contains $KEEP_VERSION_FLAG)) { 
  Write-Output "Bumping patch versions."
  Update-Version-Cargo -filePath $DESKTOP_TOML_PATH
  Update-Version-Cargo -filePath $TAURI_TOML_PATH
  Update-Version-Cargo -filePath $LIB_TOML_PATH
  Update-Version-Node -filePath $TAURI_CONF_PATH
  Write-Output "Versions bumped."
}

$debug = $args -contains $DEBUG_FLAG
if ($debug) {
  cargo tauri build --debug 2>&1 | Tee-Object -FilePath $BUILD_FILE
}
else {
  cargo tauri build 2>&1 | Tee-Object -FilePath $BUILD_FILE
}

$BUNDLE_PATTERN = "Finished * bundles at:"
$output = Get-Content -Path build.out -Raw
$output_lines = $output -split "`n"
for ($i = 0; $i -lt $output_lines.Length; $i++) {
  $line = $output_lines[$i].Trim()
  if ($line -like $BUNDLE_PATTERN) {
    $bundle_line = $i
  }
}

if (-not (Test-Path $PUB_DIR)) {
  New-Item -ItemType Directory -Path $PUB_DIR
}

$i = $bundle_line + 1
while ($output_lines[$i].Trim() -ne "") {
  $path = $output_lines[$i].Trim()
  $i += 1
  
  $filename = Split-Path -Path $path -Leaf
  $ext = [System.IO.Path]::GetExtension($filename)
  $parts = $filename -split $FILE_DELIMETER
  $version = $parts[1]
  $version = $version -replace "\.", "_"

  $new_filename = "$APP_NAME--$target--$version"
  if ($debug) {
    $new_filename = "$new_filename--debug"
  }
  $new_filename = "$new_filename$ext"
  $signature_filename = "$new_filename$SIGNATURE_EXT"

  Copy-Item -Path $path -Destination "$PUB_DIR/$new_filename"
  Copy-Item -Path "$path$SIGNATURE_EXT" -Destination "$PUB_DIR/$signature_filename"
  Write-Output "Bundle $new_filename created."
}