# import packages
import pandas as pd
import syre

# initialize project
db = syre.Database()

# get noise data from asset
noise_data = db.find_asset(type='noise-data')

# import noise data into a pandas data frame
df = pd.read_csv(noise_data.file, header = 0, index_col = 0, names = ('trial', 'volume'))

# compute statistics
stats = df.describe()

# create a new asset for the statistics
stats_path = db.add_asset(
	'noise-stats.csv',
	name='Noise Statistics',
	type='noise-stats'
)

# save the statistics data
stats.to_csv(stats_path) 
