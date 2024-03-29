% Generated by roxygen2: do not edit by hand
% Please edit documentation in R/functions.R
\name{new_asset}
\alias{new_asset}
\title{Creates a new core Asset.}
\usage{
new_asset(file, name = NULL, type = NULL, tags = list(), metadata = list())
}
\arguments{
\item{file}{File name of the associated data. Use relative paths to place the Asset in a bucket.}

\item{name}{Name of the Asset to match.}

\item{type}{Type of the Asset to match.}

\item{tags}{List of tags the Asset has to match.}

\item{metadata}{Named list of metadata the Asset has to match.}
}
\value{
NewAsset as a list.
}
\description{
Creates a new core Asset.
}
