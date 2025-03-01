# Set the version in `Cargo.toml`, `src-lib/Cargo.toml`, `src-tauri/Cargo.toml`, and `src-tauri/Cargo.toml`.
#
# # Arguments
# The version of all the resources before building.

param (
  [Parameter(Mandatory=$true)]
  [string]$Version,

  [Parameter(ValueFromRemainingArguments)]
  [string]$args
)

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

function Read-File-Lines {
  param (
    [string]$filePath
  )

  $content = Get-Content $filePath -Raw
  return $content -split "`n"
}

function Get-Version-Cargo {
  param (
    [string]$filePath
  )
  $VERSION_PATTERN = "version = *"

  $lines = Read-File-Lines -filePath $filePath
  for ($i = 0; $i -lt $lines.Length; $i++) {
    $line = $lines[$i].Trim()
    if ($line -like $VERSION_PATTERN) {
      $version_line_no = $i
      break
    }
  }
  if ($null -eq $version_line_no) {
    Write-Error "Could not find version line in $filePath."
    exit
  }

  $version_line = $lines[$version_line_no]
  $version = $version_line.Substring($VERSION_PATTERN.Length).Trim().Trim('"')
  return $version, $version_line_no
}

function Get-Version-Node {
  param (
    [string]$filePath
  )
  $VERSION_PATTERN = "*""version"": ""*"","

  $lines = Read-File-Lines -filePath $filePath
  for ($i = 0; $i -lt $lines.Length; $i++) {
    $line = $lines[$i].Trim()
    if ($line -like $VERSION_PATTERN) {
      $version_line_no = $i
      break
    }
  }
  if ($null -eq $version_line_no) {
    Write-Error "Could not find version line in $filePath."
    exit
  }

  $version_line = $lines[$version_line_no]
  $version = $version_line -replace ".*""version"": """, ""
  $version = $version.Trim().Trim('",')
  return $version, $version_line_no
}

function Set-Version-Cargo {
  param (
    [string]$filePath,
    [string]$version
  )
  
  $_, $version_line_no = Get-Version-Cargo -filePath $filePath
  $lines = Read-File-Lines -filePath $filePath
  $lines[$version_line_no] = "version = ""$version"""
  Set-Content $filePath -Value ($lines -join "`n")
}

function Set-Version-Node {
  param (
    [string]$filePath,
    [string]$version
  )
  
  $_, $version_line_no = Get-Version-Node -filePath $filePath
  $lines = Read-File-Lines -filePath $filePath
  $lines[$version_line_no] = "  ""version"": ""$version"","
  Set-Content $filePath -Value ($lines -join "`n")
}


#--- main ---

if ($null -eq $Version) {
    Write-Error "Version not set."
}

Write-Output "Setting versions to ``$Version``"
Set-Version-Cargo -filePath $DESKTOP_TOML_PATH -version $Version
Set-Version-Cargo -filePath $TAURI_TOML_PATH -version $Version
Set-Version-Cargo -filePath $LIB_TOML_PATH -version $Version
Set-Version-Node -filePath $TAURI_CONF_PATH -version $Version
Write-Output "Versions successfully updated"
