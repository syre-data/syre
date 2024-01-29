suppressPackageStartupMessages(library(tidyverse))
library(syre)

# initialize project
db <- database()

# get noise data
noise_data <- db |> find_asset(type = "noise-data")

# import data into tibble
df <- noise_data@file |> read_csv(
  col_types = cols(
    Trial = col_integer(),
    "Volume [dB]" = col_double()
  )
)

df <- df |> rename("trial" = "Trial", "volume" = "Volume [dB]")

# compute summary statistics
sdf <- df |> summarise(
  count = n(),
  mean = mean(volume),
  std = sd(volume),
  min = min(volume),
  max = max(volume)
)

# create new asset for the statistics
stats_path <- db |> add_asset(
  "noise-stats.csv",
  name = "Noise Statistics", type = "noise-stats"
)

# save the statistics to the new asset
sdf |> write.csv(stats_path, row.names = FALSE)
