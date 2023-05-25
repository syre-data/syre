setClassUnion("OptionChar", c("character", "NULL"))

setClass(
  'Database',
  slots = list(
    root = "character",
    root_path = "character",
    socket = "externalptr"
  )
)

setClass(
  'Container',
  slots = list(
    name = "OptionChar",
    type = "OptionChar",
    tags = "list",
    metadata = "list",
    assets = "list"
  )
)

setClass(
  'Asset',
  slots = list(
    name = "OptionChar",
    type = "OptionChar",
    tags = "list",
    metadata = "list",
    file = "OptionChar"
  )
)

#' Create a new Asset.
#'
#' @param file File path.
#' @param name Name.
#' @param type Type.
#' @param tags List of tags.
#' @param metadata List of metadata.
Asset <- function(file, name = NULL, type = NULL, tags = list(), metadata = list()) {
  new("Asset", file = file, name = name, type = type, tags = tags, metadata = metadata)
}

#' Create a new Asset.
#'
#' @param name Name.
#' @param type Type.
#' @param tags List of tags.
#' @param metadata List of metadata.
#' @param assets List of Assets.
Container <- function(name = NULL, type = NULL, tags = list(), metadata = list(), assets=list()) {
  new("Container", name = name, type = type, tags = tags, metadata = metadata, assets = assets)
}

#' Converts a list of properties to a Container.
#' The list should mirror the structure of the JSON representation of a Container.
#'
#' @param container List of properties.
#'
#' @returns Container.
container_from_json <- function(container) {
  # convert assets
  assets <- container$assets |> map(asset_from_json)

  # convert container
  properties <- container$properties
  Container(
    name = properties$name,
    type = properties$type,
    tags = properties$tag,
    metadata = properties$metadata,
    assets = assets
  )
}

#' Converts a list of properties to an Asset.
#' The list should mirror the structure of the JSON representation of an Asset.
#'
#' @param asset List of properties.
#'
#' @returns Asset.
asset_from_json <- function(asset) {
  properties <- asset$properties
  Asset(
    file = asset$path[[1]],
    name = properties$name,
    type = properties$kind,
    tags = properties$tags,
    metadata = properties$metadata
  )
}
