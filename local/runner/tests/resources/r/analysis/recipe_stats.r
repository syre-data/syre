suppressPackageStartupMessages(library(tidyverse))
library(syre)

# initialize project
db <- database()

# get recipe name
recipe <- root(db)

# get noise-stats data
noise_stats <- db |> find_assets(type = "noise-stats")

# read data
df <- list()
for (stat in noise_stats) {
  tdf <- stat@file |> read_csv()
  df[[length(df) + 1]] <- tdf
}

# combine all the data frames into one
df <- do.call(rbind, df)

# calculate the mean of the 'mean' column
mean_of_mean <- mean(df$mean)

# calculate the mean of the 'std' column
mean_of_std <- mean(df$std)

# create a new data frame with mean_of_mean and mean_of_std
stats_df <- data.frame(
  mean = mean_of_mean,
  std = mean_of_std
)

name <- paste(recipe@name, "Statistics")

# create new asset for the statistics
stats_path <- db |> add_asset(
  "recipe-stats.csv",
  name = name,
  type = "recipe-stats"
)

# save the statistics to the new asset
stats_df |> write.csv(stats_path, row.names = FALSE)
